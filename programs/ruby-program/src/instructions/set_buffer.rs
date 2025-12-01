use anchor_lang::prelude::*;
use crate::constants::*;
use crate::errors::AppError;
use crate::state::*;

#[derive(Accounts)]
pub struct SetBuffer<'info> {
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
pub struct SetBufferArgs {
    pub buffer: u64,
}

pub fn handler(ctx: Context<SetBuffer>, args: SetBufferArgs) -> Result<()> {
    ctx.accounts.config.buffer = args.buffer;
    Ok(())
}
