use anchor_lang::prelude::*;
use crate::constants::*;
use crate::errors::AppError;
use crate::state::*;

#[derive(Accounts)]
pub struct SetVarAddress<'info> {
    pub admin: Signer<'info>,

    #[account(
        mut,
        seeds = [CONFIG],
        bump,
        has_one = admin @ AppError::NotAuthorized,
    )]
    pub config: Account<'info, Config>,

    /// CHECK: Entropy var account to be set
    pub var_account: AccountInfo<'info>,
}

pub fn handler(ctx: Context<SetVarAddress>) -> Result<()> {
    ctx.accounts.config.var_address = ctx.accounts.var_account.key();
    Ok(())
}
