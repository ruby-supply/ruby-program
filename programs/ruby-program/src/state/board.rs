use anchor_lang::prelude::*;
use crate::constants::{ONE_TOKEN, INITIAL_REWARD_PER_ROUND, MIN_REWARD_PER_ROUND, ONE_DAY_SLOTS};

#[account]
#[derive(Default)]
pub struct Board {
    /// The current round number.
    pub round_id: u64,

    /// The slot at which the current round starts mining.
    pub start_slot: u64,

    /// The slot at which the current round ends mining.
    pub end_slot: u64,

    /// The slot at which mining started (for emission schedule).
    pub mining_start_slot: u64,

    /// Current RUBY motherlode pool (increases by +3 RUBY per round).
    pub motherlode_ruby: u64,

    /// Current SOL mini-motherlode pool (for SOL reload rewards).
    pub mini_motherlode_sol: u64,

    /// Total RUBY minted so far.
    pub total_minted: u64,
}

impl Board {
    pub const LEN: usize = 8 + // discriminator
        8 + // round_id
        8 + // start_slot
        8 + // end_slot
        8 + // mining_start_slot
        8 + // motherlode_ruby
        8 + // mini_motherlode_sol
        8; // total_minted

    /// Calculate current reward per round based on emission schedule.
    /// Days 0-3: 20 RUBY
    /// After Day 3: decreases by 2 RUBY per day until reaching 6 RUBY
    /// 2 months after hitting 6 RUBY: decreases by 1 RUBY per month
    /// Minimum: 1 RUBY
    pub fn current_reward(&self, current_slot: u64) -> u64 {
        if self.mining_start_slot == 0 {
            return INITIAL_REWARD_PER_ROUND;
        }

        let slots_elapsed = current_slot.saturating_sub(self.mining_start_slot);
        let days_elapsed = slots_elapsed / ONE_DAY_SLOTS;

        if days_elapsed <= 3 {
            // Days 0-3: 20 RUBY
            INITIAL_REWARD_PER_ROUND
        } else if days_elapsed <= 10 {
            // Days 4-10: decrease by 2 RUBY per day (20 -> 6)
            // Day 4: 18, Day 5: 16, Day 6: 14, Day 7: 12, Day 8: 10, Day 9: 8, Day 10: 6
            let reduction = (days_elapsed - 3) * 2 * ONE_TOKEN;
            INITIAL_REWARD_PER_ROUND.saturating_sub(reduction).max(6 * ONE_TOKEN)
        } else {
            // After reaching 6 RUBY, wait 2 months then decrease by 1 RUBY per month
            let days_at_6 = days_elapsed - 10;
            if days_at_6 <= 60 {
                // First 2 months at 6 RUBY
                6 * ONE_TOKEN
            } else {
                // After 2 months: decrease by 1 RUBY per month (30 days)
                let months_past = (days_at_6 - 60) / 30;
                let reward = (6 * ONE_TOKEN).saturating_sub(months_past * ONE_TOKEN);
                reward.max(MIN_REWARD_PER_ROUND)
            }
        }
    }
}
