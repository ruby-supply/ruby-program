use anchor_lang::prelude::*;
use crate::constants::*;
use crate::errors::AppError;
use crate::events::ReferralRegisteredEvent;
use crate::state::*;

#[derive(Accounts)]
pub struct RegisterReferral<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    /// CHECK: The referrer account - can be any wallet
    pub referrer: AccountInfo<'info>,

    #[account(
        init,
        payer = authority,
        space = Referral::LEN,
        seeds = [REFERRAL, authority.key().as_ref()],
        bump,
    )]
    pub referral: Account<'info, Referral>,

    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<RegisterReferral>) -> Result<()> {
    let clock = Clock::get()?;
    let referral = &mut ctx.accounts.referral;
    let authority = &ctx.accounts.authority;
    let referrer = &ctx.accounts.referrer;

    // Validate: cannot refer yourself
    require!(
        authority.key() != referrer.key(),
        AppError::SelfReferral
    );

    // Initialize referral account
    referral.authority = authority.key();
    referral.referrer = referrer.key();
    referral.created_at = clock.unix_timestamp;

    // Emit event
    emit!(ReferralRegisteredEvent {
        authority: authority.key(),
        referrer: referrer.key(),
        timestamp: clock.unix_timestamp,
    });

    msg!("Referral registered: {} referred by {}", authority.key(), referrer.key());

    Ok(())
}
