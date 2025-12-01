use anchor_lang::prelude::*;
use crate::constants::*;
use crate::state::*;
use crate::utils::transfer_lamports_safe;

#[derive(Accounts)]
pub struct ClaimSol<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        mut,
        seeds = [MINER, authority.key().as_ref()],
        bump,
    )]
    pub miner: Account<'info, Miner>,

    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<ClaimSol>) -> Result<()> {
    let clock = Clock::get()?;
    let miner = &mut ctx.accounts.miner;

    let amount = miner.claim_sol(&clock);

    if amount > 0 {
        msg!("Claiming {} lamports ({:.4} SOL)", amount, amount as f64 / 1_000_000_000.0);

        // Transfer SOL from miner account to authority (with rent-exempt check)
        transfer_lamports_safe(
            &miner.to_account_info(),
            &ctx.accounts.authority.to_account_info(),
            amount,
        )?;
    } else {
        msg!("No SOL rewards available to claim");
    }

    Ok(())
}
