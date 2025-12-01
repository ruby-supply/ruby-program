use anchor_lang::prelude::*;
use crate::constants::*;
use crate::errors::AppError;
use crate::state::*;

#[derive(Accounts)]
pub struct SetAdmin<'info> {
    pub admin: Signer<'info>,

    #[account(
        mut,
        seeds = [CONFIG],
        bump,
        has_one = admin @ AppError::NotAuthorized,
    )]
    pub config: Account<'info, Config>,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct SetAdminArgs {
    pub new_admin: Pubkey,
}

pub fn handler(ctx: Context<SetAdmin>, args: SetAdminArgs) -> Result<()> {
    ctx.accounts.config.admin = args.new_admin;
    Ok(())
}
