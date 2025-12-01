use anchor_lang::prelude::*;
use crate::constants::*;
use crate::errors::AppError;
use crate::state::*;

#[derive(Accounts)]
pub struct SetFeeCollector<'info> {
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
pub struct SetFeeCollectorArgs {
    pub new_fee_collector: Pubkey,
}

pub fn handler(ctx: Context<SetFeeCollector>, args: SetFeeCollectorArgs) -> Result<()> {
    ctx.accounts.config.fee_collector = args.new_fee_collector;
    Ok(())
}
