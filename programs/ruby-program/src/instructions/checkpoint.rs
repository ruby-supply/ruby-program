use crate::constants::*;
use crate::errors::AppError;
use crate::events::{CheckpointEvent, ReferralRewardAccruedEvent};
use crate::state::*;
use crate::utils::transfer_lamports;
use anchor_lang::prelude::*;
use anchor_lang::solana_program::sysvar::rent::Rent;

#[derive(Accounts)]
pub struct Checkpoint<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        seeds = [BOARD],
        bump,
    )]
    pub board: Account<'info, Board>,

    #[account(
        mut,
        seeds = [MINER, miner.authority.as_ref()],
        bump,
    )]
    pub miner: Account<'info, Miner>,

    #[account(
        mut,
        seeds = [ROUND, &miner.round_id.to_le_bytes()],
        bump,
    )]
    pub round: Account<'info, Round>,

    #[account(
        mut,
        seeds = [TREASURY],
        bump,
    )]
    pub treasury: Account<'info, Treasury>,

    /// Referral account - auto-resolved from miner.authority
    #[account(
        mut,
        seeds = [REFERRAL, miner.authority.as_ref()],
        bump,
    )]
    pub referral: Account<'info, Referral>,

    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<Checkpoint>) -> Result<()> {
    let clock = Clock::get()?;

    // Clone account infos FIRST before any mutable borrows
    let miner_info = ctx.accounts.miner.to_account_info();
    let round_info = ctx.accounts.round.to_account_info();

    let miner = &mut ctx.accounts.miner;
    let round = &mut ctx.accounts.round;
    let treasury = &mut ctx.accounts.treasury;
    let board = &ctx.accounts.board;

    msg!("=== Checkpoint Handler ===");
    msg!("Miner: {}, Round: {}, Slot: {}", miner.authority, round.id, clock.slot);
    msg!("Miner checkpoint_id: {}, round_id: {}", miner.checkpoint_id, miner.round_id);

    // If miner has already checkpointed this round, return
    if miner.checkpoint_id == miner.round_id {
        msg!("Checkpoint skipped: Miner {} already checkpointed round {} (checkpoint_id: {})",
            miner.authority, miner.round_id, miner.checkpoint_id);
        return Ok(());
    }

    // If round is current round, return
    if round.id == board.round_id {
        msg!("Checkpoint skipped: Round {} is still active (current board round: {})",
            round.id, board.round_id);
        return Ok(());
    }

    // If miner round ID does not match, return
    if round.id != miner.round_id {
        msg!("Checkpoint skipped: Round mismatch - round.id: {}, miner.round_id: {}",
            round.id, miner.round_id);
        return Ok(());
    }

    // If round not finalized (slot_hash = 0), just update checkpoint_id and return
    if round.slot_hash == [0; 32] {
        miner.checkpoint_id = miner.round_id;
        msg!("Checkpoint partial: Round {} not finalized (no slot_hash), updating checkpoint_id only",
            round.id);
        return Ok(());
    }

    // Ensure round is not expired
    if clock.slot >= round.expires_at {
        miner.checkpoint_id = miner.round_id;
        msg!("Checkpoint expired: Round {} expired at slot {} (current: {}). Miner {} forfeited rewards.",
            round.id, round.expires_at, clock.slot, miner.authority);
        return Ok(());
    }

    // Calculate bot fee
    let mut bot_fee = 0;
    if clock.slot >= round.expires_at - TWELVE_HOURS_SLOTS {
        bot_fee = miner.checkpoint_fee;
        miner.checkpoint_fee = 0;
    }

    // Calculate miner rewards
    let mut rewards_sol = 0;
    let mut rewards_token = 0;

    // Get the RNG
    if let Some(r) = round.rng() {
        let winning_square = round.winning_square(r);
        msg!("Round {} winning square: {}", round.id, winning_square);

        // Sanity check: validate deployment is consistent
        require!(
            round.deployed[winning_square] >= miner.deployed[winning_square],
            AppError::InvalidDeployment
        );

        // If the miner deployed to the winning square
        if miner.deployed[winning_square] > 0 {
            // Calculate SOL rewards
            let original_deployment = miner.deployed[winning_square];
            // Full refund (no fee on winning square)
            rewards_sol = original_deployment;
            // Add proportional share of losing squares (90% of total losing)
            rewards_sol += ((round.total_winnings as u128 * miner.deployed[winning_square] as u128)
                / round.deployed[winning_square] as u128) as u64;

            if round.top_miner == SPLIT_ADDRESS {
                // Calculate TOKEN rewards - always split proportionally
                rewards_token =
                    ((round.top_miner_reward as u128 * miner.deployed[winning_square] as u128)
                        / round.deployed[winning_square] as u128) as u64;
                msg!("Split mode: Miner {} receives {} TOKEN (proportional to {} lamports on square {})",
                    miner.authority, rewards_token, miner.deployed[winning_square], winning_square);
            } else {
                let top_miner_sample = round.top_miner_sample(r, winning_square);
                if top_miner_sample >= miner.cumulative[winning_square]
                    && top_miner_sample
                        < miner.cumulative[winning_square] + miner.deployed[winning_square]
                {
                    rewards_token = round.top_miner_reward;
                    round.top_miner = miner.authority;
                    msg!("Top miner: Miner {} wins {} TOKEN! (sample: {}, range: [{}, {}), square: {})",
                        miner.authority, rewards_token, top_miner_sample,
                        miner.cumulative[winning_square],
                        miner.cumulative[winning_square] + miner.deployed[winning_square],
                        winning_square);
                }
            }

            // Calculate motherlode rewards
            if round.motherlode > 0 {
                let motherlode_rewards =
                    ((round.motherlode as u128 * miner.deployed[winning_square] as u128)
                        / round.deployed[winning_square] as u128) as u64;
                rewards_sol += motherlode_rewards;
                msg!("Motherlode hit! Miner {} receives {} SOL (proportional share of {} total, ratio: {}/{})",
                    miner.authority, motherlode_rewards, round.motherlode,
                    miner.deployed[winning_square], round.deployed[winning_square]);
            }
        }
    } else {
        // Round has no slot hash, refund all SOL
        let refund_amount = miner.deployed.iter().sum::<u64>();
        rewards_sol = refund_amount;
        msg!("No RNG: Refunding {} SOL to miner {} (no slot_hash available)",
            refund_amount, miner.authority);
    }

    // Update rewards
    miner.update_rewards(treasury);

    // Calculate referral fee (1% of TOKEN rewards)
    let referral = &mut ctx.accounts.referral;
    let mut referral_fee = 0u64;
    if rewards_token > 0 {
        referral_fee = rewards_token / 100; // 1%
        referral.pending_rewards = referral.pending_rewards
            .checked_add(referral_fee)
            .ok_or(AppError::Overflow)?;
        rewards_token = rewards_token.saturating_sub(referral_fee);
        msg!("Referral fee: {} TOKEN (1%) -> referrer: {}", referral_fee, referral.referrer);

        // Emit referral accrued event
        emit!(ReferralRewardAccruedEvent {
            referee: miner.authority,
            referrer: referral.referrer,
            amount: referral_fee,
            timestamp: clock.unix_timestamp,
        });
    }

    // Checkpoint miner
    miner.checkpoint_id = round.id;
    miner.rewards_token += rewards_token;
    miner.lifetime_rewards_token += rewards_token;
    miner.rewards_sol += rewards_sol;
    miner.lifetime_rewards_sol += rewards_sol;

    // Update treasury (total_unclaimed includes full amount before referral fee)
    treasury.total_unclaimed += rewards_token + referral_fee;

    // Rent safety check: ensure miner account maintains rent exemption
    let account_size = 8 + std::mem::size_of::<Miner>();
    let rent = Rent::get()?;
    let required_rent = rent.minimum_balance(account_size);
    let final_checkpoint_fee = miner.checkpoint_fee;

    // Transfer SOL rewards (with safety check to prevent underflow)
    let mut actual_rewards_transferred = 0u64;
    if rewards_sol > 0 {
        let round_balance = round_info.lamports();
        let round_account_size = 8 + std::mem::size_of::<Round>();
        let rent = Rent::get()?;
        let rent_reserve = rent.minimum_balance(round_account_size);
        let available_balance = round_balance.saturating_sub(rent_reserve);
        let actual_rewards = rewards_sol.min(available_balance);

        if actual_rewards < rewards_sol {
            msg!("⚠️  Round balance insufficient: has {}, needs {}, paying {} (rent reserve: {})",
                round_balance, rewards_sol, actual_rewards, rent_reserve);
        }

        if actual_rewards > 0 {
            transfer_lamports(&round_info, &miner_info, actual_rewards)?;
            actual_rewards_transferred = actual_rewards;

            // Update miner's actual rewards (not full calculated amount)
            miner.rewards_sol = miner.rewards_sol.saturating_sub(rewards_sol).saturating_add(actual_rewards);
            miner.lifetime_rewards_sol = miner.lifetime_rewards_sol.saturating_sub(rewards_sol).saturating_add(actual_rewards);
        }
    }

    // Transfer bot fee
    if bot_fee > 0 {
        transfer_lamports(&miner_info, &ctx.accounts.signer.to_account_info(), bot_fee)?;
    }

    // Verify rent exemption after all transfers (use actual transferred amount)
    require!(
        miner_info.lamports() >= required_rent + final_checkpoint_fee + actual_rewards_transferred,
        AppError::InsufficientRent
    );

    msg!("✓ Checkpoint complete: Miner {} earned {} SOL + {} TOKEN (referral_fee: {}, bot_fee: {})",
        miner.authority, rewards_sol, rewards_token, referral_fee, bot_fee);

    // Emit checkpoint event
    emit!(CheckpointEvent {
        miner: miner.authority,
        round_id: round.id,
        winning_square: if let Some(r) = round.rng() {
            round.winning_square(r) as u64
        } else {
            u64::MAX
        },
        rewards_sol,
        rewards_token,
        bot_fee,
        is_top_miner: round.top_miner == miner.authority,
        timestamp: clock.unix_timestamp,
    });

    Ok(())
}
