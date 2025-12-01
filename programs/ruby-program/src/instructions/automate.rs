use anchor_lang::prelude::*;
use crate::constants::*;
use crate::errors::AppError;
use crate::state::*;
use crate::utils::{transfer_lamports, transfer_sol_cpi};

#[derive(Accounts)]
pub struct Automate<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        init_if_needed,
        payer = authority,
        space = Automation::LEN,
        seeds = [AUTOMATION, authority.key().as_ref()],
        bump,
    )]
    pub automation: Account<'info, Automation>,

    /// Executor account - the bot/service that will execute deployments
    /// CHECK: Can be any pubkey, including Pubkey::default() to close automation
    pub executor: AccountInfo<'info>,

    #[account(
        init_if_needed,
        payer = authority,
        space = Miner::LEN,
        seeds = [MINER, authority.key().as_ref()],
        bump,
    )]
    pub miner: Account<'info, Miner>,

    pub system_program: Program<'info, System>,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct AutomateArgs {
    pub amount: u64,
    pub deposit: u64,
    pub fee: u64,
    pub mask: u64,
    pub strategy: u8,
}

pub fn handler(ctx: Context<Automate>, args: AutomateArgs) -> Result<()> {
    // Close automation account if executor is Pubkey::default()
    if ctx.accounts.executor.key() == Pubkey::default() {
        // Verify authority before closing
        require!(
            ctx.accounts.automation.authority == ctx.accounts.authority.key(),
            AppError::NotAuthorized
        );

        // Close automation account properly:
        let automation_info = ctx.accounts.automation.to_account_info();
        let authority_info = ctx.accounts.authority.to_account_info();

        // 1. Transfer all lamports to authority
        let automation_lamports = automation_info.lamports();
        transfer_lamports(&automation_info, &authority_info, automation_lamports)?;

        // 2. Zero out account data (marks account as closed)
        let mut automation_data = automation_info.try_borrow_mut_data()?;
        automation_data.fill(0);

        msg!("Automation account closed successfully");
        msg!("  Returned {} lamports to authority", automation_lamports);
        return Ok(());
    }

    // Initialize automation if needed
    if ctx.accounts.automation.authority == Pubkey::default() {
        ctx.accounts.automation.authority
         = ctx.accounts.authority.key();
        ctx.accounts.automation.balance = 0;
        msg!("Automation account initialized");
    } else {
        // Verify authority if already initialized
        require!(
            ctx.accounts.automation.authority == ctx.accounts.authority.key(),
            AppError::NotAuthorized
        );
    }

    // Initialize miner if needed
    if ctx.accounts.miner.authority == Pubkey::default() {
        ctx.accounts.miner.authority = ctx.accounts.authority.key();
        ctx.accounts.miner.deployed = [0; TOTAL_BOARD];
        ctx.accounts.miner.cumulative = [0; TOTAL_BOARD];
        ctx.accounts.miner.checkpoint_fee = 0;
        ctx.accounts.miner.checkpoint_id = 0;
        ctx.accounts.miner.rewards_sol = 0;
        ctx.accounts.miner.rewards_token = 0;
        ctx.accounts.miner.refined_token = 0;
        ctx.accounts.miner.round_id = 0;
        ctx.accounts.miner.lifetime_rewards_sol = 0;
        ctx.accounts.miner.lifetime_rewards_token = 0;
        ctx.accounts.miner.last_claim_token_at = 0;
        ctx.accounts.miner.last_claim_sol_at = 0;
        msg!("Miner account initialized");
    } else {
        // Verify miner authority
        require!(
            ctx.accounts.miner.authority == ctx.accounts.authority.key(),
            AppError::NotAuthorized
        );
    }

    // Update automation settings
    ctx.accounts.automation.amount = args.amount;
    ctx.accounts.automation.executor = ctx.accounts.executor.key(); // ✅ FIX: Set executor
    ctx.accounts.automation.fee = args.fee;
    ctx.accounts.automation.mask = args.mask;
    ctx.accounts.automation.strategy = args.strategy as u64;

    msg!("Automation settings updated:");
    msg!("  amount: {}", ctx.accounts.automation.amount);
    msg!("  executor: {}", ctx.accounts.automation.executor);
    msg!("  fee: {}", ctx.accounts.automation.fee);
    msg!("  mask: {}", ctx.accounts.automation.mask);
    msg!("  strategy: {}", ctx.accounts.automation.strategy);

    // Add deposit to automation balance
    if args.deposit > 0 {
        ctx.accounts.automation.balance = ctx.accounts.automation.balance
            .checked_add(args.deposit)
            .ok_or(AppError::Overflow)?;

        let new_balance = ctx.accounts.automation.balance;
        msg!("Adding {} lamports to automation balance", args.deposit);

        // Transfer SOL to automation account using CPI
        transfer_sol_cpi(
            ctx.accounts.authority.to_account_info(),
            ctx.accounts.automation.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
            args.deposit,
        )?;

        msg!("Deposit transferred. New balance: {}", new_balance);
    }

    // Top up checkpoint fee if needed
    if ctx.accounts.miner.checkpoint_fee == 0 {
        msg!("Collecting checkpoint fee: {} lamports", CHECKPOINT_FEE);
        ctx.accounts.miner.checkpoint_fee = CHECKPOINT_FEE;

        // Transfer checkpoint fee from authority to miner account using CPI
        transfer_sol_cpi(
            ctx.accounts.authority.to_account_info(),
            ctx.accounts.miner.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
            CHECKPOINT_FEE,
        )?;

        msg!("Checkpoint fee collected successfully");
    }

    msg!("Automation configured successfully");
    Ok(())
}
