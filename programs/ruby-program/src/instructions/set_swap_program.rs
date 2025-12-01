use anchor_lang::prelude::*;
use crate::constants::*;
use crate::errors::AppError;
use crate::state::*;

#[derive(Accounts)]
pub struct SetSwapProgram<'info> {
    pub admin: Signer<'info>,

    #[account(
        mut,
        seeds = [CONFIG],
        bump,
        has_one = admin @ AppError::NotAuthorized,
    )]
    pub config: Account<'info, Config>,

    /// CHECK: Swap program to be set
    pub swap_program: AccountInfo<'info>,
}

pub fn handler(ctx: Context<SetSwapProgram>) -> Result<()> {
    ctx.accounts.config.swap_program = ctx.accounts.swap_program.key();
    Ok(())
}
