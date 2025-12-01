use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};
use anchor_spl::associated_token::AssociatedToken;
use crate::constants::*;
use crate::errors::AppError;
use crate::events::ReferralRewardClaimedEvent;
use crate::state::*;

#[derive(Accounts)]
pub struct ClaimReferralRewards<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        mut,
        seeds = [TREASURY],
        bump,
    )]
    pub treasury: Account<'info, Treasury>,

    #[account(address = MINT_ADDRESS)]
    pub mint: Account<'info, Mint>,

    /// Treasury's token account (source of tokens)
    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = treasury,
    )]
    pub treasury_tokens: Account<'info, TokenAccount>,

    /// Referrer's token account (auto-created if needed)
    #[account(
        init_if_needed,
        payer = authority,
        associated_token::mint = mint,
        associated_token::authority = authority
    )]
    pub recipient: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

/// Handler for claim_referral_rewards instruction
///
/// remaining_accounts:
/// - List of Referral PDAs to claim from (all must have referrer == authority)
///
/// Referrer (authority) claims pending rewards from multiple referees in one transaction
pub fn handler<'info>(ctx: Context<'_, '_, 'info, 'info, ClaimReferralRewards<'info>>) -> Result<()> {
    let clock = Clock::get()?;
    let authority = ctx.accounts.authority.key();

    require!(
        !ctx.remaining_accounts.is_empty(),
        AppError::InvalidAmount
    );

    let mut total_claimed: u64 = 0;

    // Process each referral account
    for referral_info in ctx.remaining_accounts.iter() {
        // Verify account is writable
        require!(
            referral_info.is_writable,
            AppError::InvalidReferral
        );

        // Deserialize referral
        let mut referral_data = referral_info.try_borrow_mut_data()?;
        let mut referral: Referral = Referral::try_deserialize(&mut &referral_data[..])?;

        // Verify this referral belongs to the authority (referrer)
        require!(
            referral.referrer == authority,
            AppError::InvalidReferral
        );

        // Get pending rewards
        let pending = referral.pending_rewards;
        if pending > 0 {
            // Update referral account
            referral.pending_rewards = 0;
            referral.claimed_rewards = referral.claimed_rewards
                .checked_add(pending)
                .ok_or(AppError::Overflow)?;

            // Serialize back
            referral.try_serialize(&mut *referral_data)?;

            total_claimed = total_claimed
                .checked_add(pending)
                .ok_or(AppError::Overflow)?;

            msg!("Claimed {} TOKEN from referee {}", pending, referral.authority);
        }
    }

    require!(
        total_claimed > 0,
        AppError::InvalidAmount
    );

    // Transfer total claimed tokens to referrer
    let treasury_bump = ctx.bumps.treasury;
    let signer_seeds: &[&[&[u8]]] = &[&[TREASURY, &[treasury_bump]]];

    let cpi_accounts = Transfer {
        from: ctx.accounts.treasury_tokens.to_account_info(),
        to: ctx.accounts.recipient.to_account_info(),
        authority: ctx.accounts.treasury.to_account_info(),
    };
    let cpi_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        cpi_accounts,
        signer_seeds,
    );
    token::transfer(cpi_ctx, total_claimed)?;

    // Emit event
    emit!(ReferralRewardClaimedEvent {
        referrer: authority,
        amount: total_claimed,
        num_referees: ctx.remaining_accounts.len() as u64,
        timestamp: clock.unix_timestamp,
    });

    msg!("Total claimed: {} TOKEN from {} referees", total_claimed, ctx.remaining_accounts.len());

    Ok(())
}
