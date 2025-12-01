use crate::constants::*;
use crate::errors::AppError;
use crate::events::DeployEvent;
use crate::state::*;
use crate::utils::{transfer_lamports, transfer_sol_cpi};
use anchor_lang::prelude::*;
use solana_nostd_keccak::hashv;
use solana_program::program::invoke_signed;

#[derive(Accounts)]
pub struct Deploy<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    /// CHECK: Authority account
    #[account(mut)]
    pub authority: AccountInfo<'info>,

    #[account(
        seeds = [CONFIG],
        bump,
    )]
    pub config: Account<'info, Config>,

    #[account(
        mut,
        seeds = [AUTOMATION, authority.key().as_ref()],
        bump,
    )]
    pub automation: Option<Account<'info, Automation>>,

    #[account(
        mut,
        seeds = [BOARD],
        bump,
    )]
    pub board: Account<'info, Board>,

    #[account(
        init_if_needed,
        payer = signer,
        space = Miner::LEN,
        seeds = [MINER, authority.key().as_ref()],
        bump,
    )]
    pub miner: Account<'info, Miner>,

    #[account(
        init_if_needed,
        payer = signer,
        space = Round::LEN,
        seeds = [ROUND, &board.round_id.to_le_bytes()],
        bump,
    )]
    pub round: Account<'info, Round>,

    /// CHECK: Entropy var account (REQUIRED for VRF)
    #[account(mut)]
    pub entropy_var: AccountInfo<'info>,

    /// CHECK: Entropy program (REQUIRED for VRF)
    pub entropy_program: AccountInfo<'info>,

    /// Referral account - required to deploy
    #[account(
        seeds = [REFERRAL, authority.key().as_ref()],
        bump,
    )]
    pub referral: Account<'info, Referral>,

    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<Deploy>, mut amount: u64, squares: u32) -> Result<()> {
    msg!("=== Deploy Handler Start ===");
    msg!("Amount: {}, Squares: {}", amount, squares);
    msg!("Signer: {}", ctx.accounts.signer.key());
    msg!("Authority: {}", ctx.accounts.authority.key());

    let clock = Clock::get()?;

    // Clone account infos FIRST before any mutable borrows
    let round_info = ctx.accounts.round.to_account_info();
    let miner_info = ctx.accounts.miner.to_account_info();

    let board = &mut ctx.accounts.board;
    let round = &mut ctx.accounts.round;
    let miner = &mut ctx.accounts.miner;

    msg!("Board end_slot: {}", board.end_slot);
    msg!(
        "Round ID: {}, Total deployed: {}",
        round.id,
        round.total_deployed
    );

    // Initialize round on first deploy
    if board.end_slot == u64::MAX {
        msg!("Initializing new round...");
        // Initialize round fields if newly created
        if round.id == 0 && round.total_deployed == 0 {
            round.id = board.round_id;
            round.deployed = [0; TOTAL_BOARD];
            round.slot_hash = [0; 32];
            round.count = [0; TOTAL_BOARD];
            round.expires_at = u64::MAX;
            round.motherlode = 0;
            round.rent_payer = ctx.accounts.signer.key();
            round.top_miner = Pubkey::default();
            round.top_miner_reward = 0;
            round.total_deployed = 0;
            round.total_vaulted = 0;
            round.total_stake_rewards = 0;
            round.total_winnings = 0;
        }

        board.start_slot = clock.slot;
        board.end_slot = board.start_slot + DEPLOYMENT_WINDOW_SLOTS; // 1 minute deployment window
        round.expires_at = board.end_slot + ONE_DAY_SLOTS;

        // Bump var to the next value (only if var is finalized)
        let config = &ctx.accounts.config;
        let entropy_var = &ctx.accounts.entropy_var;
        let entropy_program = &ctx.accounts.entropy_program;

        // Validate entropy var matches config
        require!(
            config.var_address != Pubkey::default(),
            AppError::EntropyNotConfigured
        );
        require!(
            entropy_var.key() == config.var_address,
            AppError::InvalidEntropyVar
        );

        // Check if var is finalized (has been sampled & revealed)
        // Var is considered finalized if slot_hash, seed, and value are all non-zero
        let var_data = entropy_var.try_borrow_data()?;
        if var_data.len() >= 200 {
            // Ensure account has enough data
            let slot_hash = &var_data[144..176]; // 8 disc + 32 auth + 8 id + 32 provider + 32 commit + 32 seed = 144
            let seed = &var_data[112..144];
            let value = &var_data[176..208];

            let is_finalized = !slot_hash.iter().all(|&b| b == 0)
                && !seed.iter().all(|&b| b == 0)
                && !value.iter().all(|&b| b == 0);

            if is_finalized {
                msg!("Var is finalized, calling Next to reuse for this round");

                // Calculate end_at slot for entropy sampling (10 slots after deploy window)
                let entropy_end_at = board.end_slot + 10;

                // Build Next instruction data for Entropy program
                // Format: [discriminator: u8, end_at: u64]
                let mut ix_data = Vec::with_capacity(1 + 8);
                ix_data.push(2u8); // Next instruction discriminator
                ix_data.extend_from_slice(&entropy_end_at.to_le_bytes());

                // Create Entropy Next instruction
                let next_ix: instruction::Instruction = solana_program::instruction::Instruction {
                    program_id: entropy_program.key(),
                    accounts: vec![
                        solana_program::instruction::AccountMeta::new_readonly(board.key(), true), // authority (Board PDA)
                        solana_program::instruction::AccountMeta::new(entropy_var.key(), false), // var
                    ],
                    data: ix_data,
                };

                // Get Board PDA seeds for signing
                let board_bump = ctx.bumps.board;
                let signer_seeds: &[&[&[u8]]] = &[&[BOARD, &[board_bump]]];

                // Invoke Entropy Next instruction
                drop(var_data); // Release borrow before invoke
                invoke_signed(
                    &next_ix,
                    &[board.to_account_info(), entropy_var.to_account_info()],
                    signer_seeds,
                )?;
            } else {
                msg!("Var not finalized yet, skipping Next call");
                msg!("Make sure to sample + reveal var after this round ends");
            }
        }
    }

    // Validate round timing
    require!(
        clock.slot >= board.start_slot && clock.slot < board.end_slot,
        AppError::RoundEnded
    );

    // Check if signer is automation executor
    let is_automation = if let Some(automation) = &ctx.accounts.automation {
        msg!("Automation mode detected");
        // Validate executor
        require!(
            automation.executor == ctx.accounts.signer.key(),
            AppError::NotAuthorized
        );
        require!(
            automation.authority == ctx.accounts.authority.key(),
            AppError::NotAuthorized
        );
        true
    } else {
        msg!("Manual deploy mode");
        false
    };

    // Initialize miner if needed
    if miner.authority == Pubkey::default() {
        msg!("Initializing new miner...");
        miner.authority = if is_automation {
            ctx.accounts.automation.as_ref().unwrap().authority
        } else {
            ctx.accounts.signer.key()
        };
        miner.round_id = 0;
        miner.checkpoint_id = 0;
        miner.deployed = [0; TOTAL_BOARD];
        miner.cumulative = [0; TOTAL_BOARD];
        miner.rewards_sol = 0;
        miner.rewards_token = 0;
        miner.refined_token = 0;
        miner.lifetime_rewards_sol = 0;
        miner.lifetime_rewards_token = 0;
        miner.checkpoint_fee = 0;
        miner.last_claim_token_at = 0;
        miner.last_claim_sol_at = 0;
    }

    // Validate miner authority
    if is_automation {
        require!(
            miner.authority == ctx.accounts.automation.as_ref().unwrap().authority,
            AppError::NotAuthorized
        );
    } else {
        require!(
            miner.authority == ctx.accounts.signer.key(),
            AppError::NotAuthorized
        );
    }

    // Reset miner if joining new round
    if miner.round_id != round.id {
        // Assert miner has checkpointed prior round
        require!(
            miner.checkpoint_id == miner.round_id,
            AppError::MustCheckpoint
        );

        // Reset miner for new round
        miner.deployed = [0; TOTAL_BOARD];
        miner.cumulative = round.deployed;
        miner.round_id = round.id;
    }

    // Determine squares to deploy and amount
    let mut selected_squares = [false; TOTAL_BOARD];
    if is_automation {
        let automation = ctx.accounts.automation.as_ref().unwrap();

        // Use automation amount
        amount = automation.amount;

        // Validate minimum deployment amount
        require!(amount >= MIN_DEPLOYMENT, AppError::AmountTooSmall);

        // Determine squares based on strategy
        match AutomationStrategy::from_u64(automation.strategy) {
            AutomationStrategy::Preferred => {
                // Use automation's preferred mask
                for i in 0..TOTAL_BOARD {
                    selected_squares[i] = (automation.mask & (1 << i)) != 0;
                }
            }
            AutomationStrategy::Random => {
                // Generate random mask using Keccak256 hash (same as Steel version)
                let num_squares = ((automation.mask & 0xFF) as u64).min(TOTAL_BOARD as u64);
                let hash = hashv(&[&automation.authority.to_bytes(), &round.id.to_le_bytes()]);
                selected_squares = generate_random_mask(num_squares, &hash);
            }
        }
    } else {
        // Validate minimum deployment amount
        require!(amount >= MIN_DEPLOYMENT, AppError::AmountTooSmall);

        // Use provided mask
        for i in 0..TOTAL_BOARD {
            selected_squares[i] = (squares & (1 << i)) != 0;
        }
    }

    // Deploy to selected squares
    let mut total_deployed = 0u64;
    let mut _total_squares = 0u64;

    for (square_id, &should_deploy) in selected_squares.iter().enumerate() {
        // Skip if not selected
        if !should_deploy {
            continue;
        }

        // Skip if miner already deployed to this square
        if miner.deployed[square_id] > 0 {
            continue;
        }

        // For automation: exit early if not enough balance
        if is_automation {
            let automation = ctx.accounts.automation.as_ref().unwrap();
            if total_deployed + automation.fee + amount > automation.balance {
                break;
            }
        }

        // Record cumulative amount (snapshot before this deployment)
        miner.cumulative[square_id] = round.deployed[square_id];

        // Update miner deployment
        miner.deployed[square_id] = amount;

        // Update round state
        round.deployed[square_id] += amount;
        round.count[square_id] += 1;

        // Track totals
        total_deployed += amount;
        _total_squares += 1;
    }

    round.total_deployed += total_deployed;

    msg!("Total deployed this tx: {} lamports", total_deployed);
    msg!("Round total deployed: {} lamports", round.total_deployed);

    // Top up checkpoint fee on first deploy
    if miner.checkpoint_fee == 0 {
        msg!("Collecting checkpoint fee: {} lamports", CHECKPOINT_FEE);
        miner.checkpoint_fee = CHECKPOINT_FEE;

        let signer_balance_before = ctx.accounts.signer.lamports();
        msg!(
            "Signer balance before checkpoint fee: {}",
            signer_balance_before
        );

        // Collect checkpoint fee from signer using System Program CPI
        transfer_sol_cpi(
            ctx.accounts.signer.to_account_info(),
            miner_info.clone(),
            ctx.accounts.system_program.to_account_info(),
            CHECKPOINT_FEE,
        )?;

        msg!("Checkpoint fee collected successfully");
    }

    // Transfer SOL
    msg!("Starting SOL transfer, is_automation: {}", is_automation);
    if is_automation {
        let automation = ctx.accounts.automation.as_mut().unwrap();

        // Save fee before mutation
        let executor_fee = automation.fee;
        let deploy_amount = automation.amount;

        // Deduct from automation balance
        automation.balance = automation
            .balance
            .checked_sub(total_deployed + executor_fee)
            .ok_or(AppError::InsufficientBalance)?;

        // Check if should close
        let should_close = automation.balance < deploy_amount + executor_fee;

        let automation_info = ctx.accounts.automation.as_ref().unwrap().to_account_info();

        // Transfer deployment amount to round
        transfer_lamports(&automation_info, &round_info, total_deployed)?;

        // Transfer fee to executor
        transfer_lamports(&automation_info, &ctx.accounts.signer.to_account_info(), executor_fee)?;

        // Close automation if balance too low
        if should_close {
            let remaining_balance = automation_info.lamports();
            transfer_lamports(&automation_info, &ctx.accounts.authority, remaining_balance)?;
        }
    } else {
        msg!("Manual deploy: transferring {} lamports", total_deployed);
        let signer_balance = ctx.accounts.signer.lamports();
        let round_balance = round_info.lamports();

        msg!("Signer balance: {}", signer_balance);
        msg!("Round balance before: {}", round_balance);
        msg!("Signer key: {}", ctx.accounts.signer.key());
        msg!("Round key: {}", round_info.key);

        // Transfer from signer to round using System Program CPI
        transfer_sol_cpi(
            ctx.accounts.signer.to_account_info(),
            round_info.clone(),
            ctx.accounts.system_program.to_account_info(),
            total_deployed,
        )?;

        msg!("Transfer successful!");
    }

    msg!("=== Deploy Handler Complete ===");

    // Emit deploy event (copy values to avoid borrow issues)
    let round_id = round.id;
    let total_deployed_final = round.total_deployed;
    emit!(DeployEvent {
        authority: ctx.accounts.authority.key(),
        round_id,
        amount,
        squares,
        total_deployed: total_deployed_final,
        timestamp: clock.unix_timestamp,
    });

    Ok(())
}

/// Generate random mask for automation
fn generate_random_mask(num_squares: u64, hash: &[u8]) -> [bool; TOTAL_BOARD] {
    let mut mask = [false; TOTAL_BOARD];
    let mut selected = 0;

    for i in 0..TOTAL_BOARD {
        if selected >= num_squares {
            break;
        }

        let rand_byte = hash[i % hash.len()];
        let remaining_needed = num_squares - selected;
        let remaining_positions = TOTAL_BOARD - i;

        if (rand_byte as u64) * (remaining_positions as u64) < (remaining_needed * 256) {
            mask[i] = true;
            selected += 1;
        }
    }

    mask
}
