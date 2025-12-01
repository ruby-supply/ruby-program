use crate::constants::*;
use crate::errors::AppError;
use crate::events::ResetEvent;
use crate::state::*;
use crate::utils::transfer_lamports;
use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{self, Mint, Token, TokenAccount};
use bytemuck;
use entropy_api::state::Var;

#[derive(Accounts)]
pub struct Reset<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        mut,
        seeds = [BOARD],
        bump,
    )]
    pub board: Account<'info, Board>,

    #[account(
        seeds = [CONFIG],
        bump,
    )]
    pub config: Account<'info, Config>,

    #[account(
        mut,
        address = MINT_ADDRESS
    )]
    pub mint: Account<'info, Mint>,

    #[account(
        mut,
        seeds = [ROUND, &board.round_id.to_le_bytes()],
        bump,
    )]
    pub current_round: Box<Account<'info, Round>>,

    #[account(
        init,
        payer = signer,
        space = Round::LEN,
        seeds = [ROUND, &(board.round_id + 1).to_le_bytes()],
        bump,
    )]
    pub next_round: Box<Account<'info, Round>>,

    #[account(
        mut,
        seeds = [TREASURY],
        bump,
    )]
    pub treasury: Account<'info, Treasury>,

    /// Treasury token account for RUBY (ATA, auto-created if needed)
    #[account(
        init_if_needed,
        payer = signer,
        associated_token::mint = mint,
        associated_token::authority = treasury,
    )]
    pub treasury_tokens: Account<'info, TokenAccount>,

    /// CHECK: Entropy var account (REQUIRED for VRF)
    #[account(mut)]
    pub entropy_var: AccountInfo<'info>,

    /// CHECK: Fee collector account (receives platform fee)
    #[account(
        mut,
        address = config.fee_collector @ AppError::InvalidFeeCollector
    )]
    pub fee_collector: AccountInfo<'info>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<Reset>) -> Result<()> {
    let clock = Clock::get()?;

    // Clone account infos FIRST before any borrows
    let current_round_info = ctx.accounts.current_round.to_account_info();
    let treasury_info = ctx.accounts.treasury.to_account_info();

    let board = &mut ctx.accounts.board;
    let current_round = &mut ctx.accounts.current_round;
    let next_round = &mut ctx.accounts.next_round;
    let treasury = &mut ctx.accounts.treasury;
    let mint = &ctx.accounts.mint;

    // Validate round can be reset
    require!(
        clock.slot >= board.end_slot + INTERMISSION_SLOTS,
        AppError::RoundNotStarted
    );

    // Sample VRF randomness from Entropy var (REQUIRED)
    // Entropy var must be finalized (sample + reveal completed)
    let entropy_var = &ctx.accounts.entropy_var;

    // Verify entropy_var matches config
    require!(
        entropy_var.key() == ctx.accounts.config.var_address,
        AppError::InvalidEntropyVar
    );

    // Deserialize Var account (Steel programs have no discriminator)
    let var_data = entropy_var.try_borrow_data()?;
    require!(
        var_data.len() >= std::mem::size_of::<Var>(),
        AppError::EntropyNotFinalized
    );

    let var: &Var = bytemuck::try_from_bytes(&var_data[..std::mem::size_of::<Var>()])
        .map_err(|_| AppError::EntropyNotFinalized)?;

    // Validate var is finalized (all three must be non-zero)
    require!(var.slot_hash != [0; 32], AppError::EntropyNotFinalized);
    require!(var.seed != [0; 32], AppError::EntropyNotFinalized);
    require!(var.value != [0; 32], AppError::EntropyNotFinalized);

    // Use entropy var.value as randomness
    current_round.slot_hash = var.value;
    msg!("Using Entropy VRF randomness: {:?}", var.value);

    // Exit early if no slot hash (refund scenario)
    msg!("Current round slot_hash: {:?}", current_round.slot_hash);
    let Some(r) = current_round.rng() else {
        msg!("No randomness - refunding all SOL");
        // No randomness, refund all SOL
        current_round.total_vaulted = 0;
        current_round.total_winnings = 0;
        current_round.total_deployed = 0;

        // Initialize next round
        initialize_next_round(next_round, board, &ctx.accounts.signer.key());

        // Update board
        board.round_id += 1;
        board.start_slot = clock.slot + 1;
        board.end_slot = u64::MAX;

        // Emit reset event (no RNG scenario)
        emit!(ResetEvent {
            round_id: current_round.id,
            start_slot: board.start_slot,
            end_slot: board.end_slot,
            winning_square: u64::MAX, // No winning square
            top_miner: Pubkey::default(),
            num_winners: 0,
            motherlode: 0,
            total_deployed: current_round.total_deployed,
            total_vaulted: 0,
            total_winnings: 0,
            total_minted: 0,
            timestamp: clock.unix_timestamp,
        });

        return Ok(());
    };

    // Get winning square
    let winning_square = current_round.winning_square(r);

    // If no one deployed on winning square, 100% treasury
    if current_round.deployed[winning_square] == 0 {
        let total = current_round.total_deployed;

        current_round.total_vaulted = total;
        treasury.buyback_bl += total;

        // Transfer SOL
        transfer_lamports(&current_round_info, &treasury_info, total)?;

        // Initialize next round
        initialize_next_round(next_round, board, &ctx.accounts.signer.key());

        // Update board
        board.round_id += 1;
        board.start_slot = clock.slot + 1;
        board.end_slot = u64::MAX;

        // Emit reset event (no winner on winning square)
        emit!(ResetEvent {
            round_id: current_round.id,
            start_slot: board.start_slot,
            end_slot: board.end_slot,
            winning_square: winning_square as u64,
            top_miner: Pubkey::default(),
            num_winners: 0,
            motherlode: 0,
            total_deployed: current_round.total_deployed,
            total_vaulted: total,
            total_winnings: 0,
            total_minted: 0,
            timestamp: clock.unix_timestamp,
        });

        return Ok(());
    }

    // Calculate distribution from losing squares
    // RUBY Fee Distribution:
    // - 88% to winners
    // - 8% buyback and burn
    // - 2% strategic reserve (leaderboard + mini-motherlode)
    // - 2% motherlode pool
    let losing_squares_total = current_round.calculate_total_winnings(winning_square);

    let winners_share = losing_squares_total * 88 / 100; // 88%
    let buyback_amount = losing_squares_total * 8 / 100; // 8%
    let reserve_amount = losing_squares_total * 2 / 100; // 2% strategic reserve
    let motherlode_sol_amount = losing_squares_total * 2 / 100; // 2% motherlode pool
    let platform_fee = winners_share / 100; // 1% of winners amount
    let winners_amount = winners_share - platform_fee;

    current_round.total_winnings = winners_amount;
    current_round.total_vaulted = buyback_amount + reserve_amount + motherlode_sol_amount;
    current_round.total_stake_rewards = 0; // RUBY doesn't use staking rewards from mining

    // Update treasury balances
    treasury.buyback_bl += buyback_amount;
    treasury.reserve_bl += reserve_amount;
    treasury.motherlode_sol_bl += motherlode_sol_amount;

    // Split reserve between leaderboard and mini-motherlode
    let leaderboard_share = reserve_amount / 2;
    treasury.leaderboard_bl += leaderboard_share;
    board.mini_motherlode_sol += reserve_amount - leaderboard_share;

    msg!("RUBY distribution: winners={}, buyback={}, reserve={}, motherlode_sol={}",
        winners_amount, buyback_amount, reserve_amount, motherlode_sol_amount);

    // Calculate RUBY reward based on emission schedule
    let current_supply = mint.supply;
    let reward_per_round = board.current_reward(clock.slot);
    msg!("Current mint supply: {}", current_supply);
    msg!("MAX_SUPPLY: {}", MAX_SUPPLY);
    msg!("Current reward per round: {}", reward_per_round);

    let mint_amount = MAX_SUPPLY
        .saturating_sub(current_supply)
        .min(reward_per_round);
    msg!("Calculated mint_amount: {}", mint_amount);

    current_round.top_miner_reward = mint_amount;

    // With 1 in 2 odds, split the RUBY reward proportionally.
    if current_round.is_split_reward(r) {
        current_round.top_miner = SPLIT_ADDRESS;
    }

    // Increment RUBY motherlode by +3 RUBY per round
    board.motherlode_ruby += MOTHERLODE_INCREMENT;
    msg!("Motherlode RUBY pool: {} RUBY", board.motherlode_ruby / ONE_TOKEN);

    // Payout the motherlode if it was activated.
    if current_round.did_hit_motherlode(r) {
        // RUBY motherlode payout (in RUBY tokens)
        let motherlode_ruby = board.motherlode_ruby;
        current_round.motherlode = motherlode_ruby;
        board.motherlode_ruby = 0;

        // Also payout mini-motherlode SOL
        let mini_ml_sol = board.mini_motherlode_sol;
        if mini_ml_sol > 0 {
            transfer_lamports(&treasury_info, &current_round_info, mini_ml_sol)?;
            board.mini_motherlode_sol = 0;
            msg!("Mini-Motherlode hit! Transferred {} SOL to winners", mini_ml_sol);
        }

        msg!("Motherlode hit! {} RUBY to winners", motherlode_ruby / ONE_TOKEN);
    }

    // Perform minting operations (borrow accounts after treasury mutations)
    msg!("Checking if mint_amount > 0: {}", mint_amount > 0);
    if mint_amount > 0 {
        msg!("Minting {} tokens to treasury", mint_amount);
        let treasury_bump = ctx.bumps.treasury;
        let signer_seeds: &[&[&[u8]]] = &[&[TREASURY, &[treasury_bump]]];

        let cpi_accounts = token::MintTo {
            mint: ctx.accounts.mint.to_account_info(),
            to: ctx.accounts.treasury_tokens.to_account_info(),
            authority: ctx.accounts.treasury.to_account_info(),
        };
        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            cpi_accounts,
            signer_seeds,
        );
        token::mint_to(cpi_ctx, mint_amount)?;
    }

    // Initialize next round
    initialize_next_round(next_round, board, &ctx.accounts.signer.key());

    // Update board
    board.round_id += 1;
    board.start_slot = clock.slot + 1;
    board.end_slot = u64::MAX;

    // Transfer SOL to treasury
    let total_to_treasury = current_round.total_vaulted;
    if total_to_treasury > 0 {
        transfer_lamports(&current_round_info, &treasury_info, total_to_treasury)?;
    }

    // Update board total minted
    board.total_minted += mint_amount;

    // Transfer platform fee to fee_collector (1% of winners amount)
    if platform_fee > 0 {
        transfer_lamports(
            &current_round_info,
            &ctx.accounts.fee_collector.to_account_info(),
            platform_fee,
        )?;
    }

    // Emit reset event (normal reset with winners)
    emit!(ResetEvent {
        round_id: current_round.id,
        start_slot: board.start_slot,
        end_slot: board.end_slot,
        winning_square: winning_square as u64,
        top_miner: current_round.top_miner,
        num_winners: current_round.count[winning_square],
        motherlode: current_round.motherlode,
        total_deployed: current_round.total_deployed,
        total_vaulted: current_round.total_vaulted,
        total_winnings: current_round.total_winnings,
        total_minted: mint_amount,
        timestamp: clock.unix_timestamp,
    });

    Ok(())
}

fn initialize_next_round(round: &mut Round, board: &Board, rent_payer: &Pubkey) {
    round.id = board.round_id + 1;
    round.deployed = [0; TOTAL_BOARD];
    round.slot_hash = [0; 32];
    round.count = [0; TOTAL_BOARD];
    round.expires_at = u64::MAX; // Set to MAX, waiting for first deploy
    round.motherlode = 0;
    round.rent_payer = *rent_payer;
    round.top_miner = Pubkey::default();
    round.top_miner_reward = 0;
    round.total_deployed = 0;
    round.total_vaulted = 0;
    round.total_stake_rewards = 0;
    round.total_winnings = 0;
}
