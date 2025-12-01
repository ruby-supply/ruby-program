use anchor_lang::prelude::*;
use crate::constants::*;
use crate::errors::AppError;
use crate::state::*;

#[derive(Accounts)]
pub struct CancelAutomate<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        mut,
        seeds = [AUTOMATION, authority.key().as_ref()],
        bump,
        close = authority, // Auto-close account and send rent to authority
    )]
    pub automation: Account<'info, Automation>,

    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<CancelAutomate>) -> Result<()> {
    let automation = &ctx.accounts.automation;

    // Verify authority
    require!(
        automation.authority == ctx.accounts.authority.key(),
        AppError::NotAuthorized
    );

    // Get current balance before close
    let balance = automation.balance;
    let rent_lamports = ctx.accounts.automation.to_account_info().lamports();

    msg!("=== Cancelling Automation ===");
    msg!("Authority: {}", ctx.accounts.authority.key());
    msg!("Automation balance: {} lamports", balance);
    msg!("Account rent: {} lamports", rent_lamports);
    msg!("Total returned: {} lamports", rent_lamports);

    // Transfer automation balance back to authority
    // (This is the tracked balance, separate from account rent)
    if balance > 0 {
        // The balance is just a state field, actual lamports are already in the account
        // When we close the account, all lamports (including balance) go back to authority
        msg!("Balance {} will be returned to authority", balance);
    }

    msg!("✓ Automation account closed successfully");
    msg!("✓ All lamports returned to authority");

    // Account will be auto-closed by Anchor (close = authority constraint)
    Ok(())
}
