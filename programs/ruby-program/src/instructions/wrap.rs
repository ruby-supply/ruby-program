use anchor_lang::prelude::*;
use crate::constants::*;
use crate::state::*;
use crate::utils::transfer_lamports;

#[derive(Accounts)]
pub struct Wrap<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        mut,
        seeds = [TREASURY],
        bump,
    )]
    pub treasury: Account<'info, Treasury>,

    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<Wrap>) -> Result<()> {
    // Wrap SOL into treasury balance
    // This allows collecting SOL for future buy-and-burn operations

    let amount = ctx.accounts.signer.lamports();

    // Transfer SOL to treasury
    transfer_lamports(
        &ctx.accounts.signer.to_account_info(),
        &ctx.accounts.treasury.to_account_info(),
        amount,
    )?;

    // Update treasury balance tracking
    ctx.accounts.treasury.buyback_bl += amount;

    Ok(())
}
