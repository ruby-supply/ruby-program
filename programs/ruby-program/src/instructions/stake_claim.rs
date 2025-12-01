use anchor_lang::prelude::*;
use crate::constants::*;
use crate::errors::AppError;
use crate::events::StakeClaimEvent;
use crate::state::*;
use crate::utils::transfer_lamports_safe;

#[derive(Accounts)]
pub struct StakeClaim<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        mut,
        seeds = [STAKE, signer.key().as_ref()],
        bump,
    )]
    pub stake: Account<'info, Stake>,

    #[account(mut, seeds = [TREASURY], bump)]
    pub treasury: Account<'info, Treasury>,

    /// CHECK: Recipient to receive SOL rewards
    #[account(mut)]
    pub recipient: UncheckedAccount<'info>,

    pub clock: Sysvar<'info, Clock>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<StakeClaim>) -> Result<()> {
    let stake = &mut ctx.accounts.stake;
    let treasury = &mut ctx.accounts.treasury;

    // Validate authority
    require!(
        stake.authority == ctx.accounts.signer.key(),
        AppError::NotAuthorized
    );

    // Update rewards to ensure we have latest balance
    stake.update_rewards(treasury);

    // Claim all available rewards
    let amount = stake.claim(
        u64::MAX, // Claim maximum available (will be capped by stake.rewards)
        &ctx.accounts.clock,
        treasury
    );

    if amount > 0 {
        // Update treasury balance BEFORE transfer to prevent reentrancy
        treasury.stake_rw_bl = treasury.stake_rw_bl.checked_sub(amount)
            .ok_or(AppError::Underflow)?;

        // Transfer SOL from treasury to recipient (with rent-exempt check)
        transfer_lamports_safe(
            &treasury.to_account_info(),
            &ctx.accounts.recipient.to_account_info(),
            amount,
        )?;

        // Emit event
        emit!(StakeClaimEvent {
            authority: ctx.accounts.signer.key(),
            amount,
            rewards_remaining: stake.rewards,
            lifetime_rewards: stake.lifetime_rewards,
        });

        msg!("Claimed {} lamports ({:.4} SOL) in staking rewards", amount, amount as f64 / 1_000_000_000.0);
    } else {
        msg!("No staking rewards available to claim");
    }

    Ok(())
}
