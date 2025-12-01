use anchor_lang::prelude::*;
use crate::constants::*;
use crate::state::*;

#[derive(Accounts)]
pub struct Close<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        mut,
        close = authority,
        seeds = [MINER, authority.key().as_ref()],
        bump,
    )]
    pub miner: Account<'info, Miner>,
}

pub fn handler(_ctx: Context<Close>) -> Result<()> {
    // Miner account will be closed automatically due to close constraint
    // All lamports will be transferred to authority
    Ok(())
}
