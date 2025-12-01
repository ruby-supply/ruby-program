use crate::utils::Numeric;
use anchor_lang::prelude::*;

#[account]
#[derive(Default)]
pub struct Treasury {
    /// The amount of SOL collected for buyback and burn operations (8%).
    pub buyback_bl: u64,

    /// The amount of SOL in the strategic reserve (2% - leaderboard + mini-motherlode).
    pub reserve_bl: u64,

    /// The amount of SOL allocated to motherlode pool (2%).
    pub motherlode_sol_bl: u64,

    /// The cumulative RUBY distributed to miners, divided by the total unclaimed RUBY.
    pub miner_rewards_factor: Numeric,

    /// The cumulative rewards distributed to stakers, divided by the total stake.
    pub stake_rewards_factor: Numeric,

    /// The current total amount of RUBY staking deposits.
    pub total_staked: u64,

    /// The current total amount of unclaimed RUBY mining rewards.
    pub total_unclaimed: u64,

    /// The current total amount of refined RUBY mining rewards.
    pub total_refined: u64,

    /// Total SOL collected for weekly leaderboard rewards.
    pub leaderboard_bl: u64,
}

impl Treasury {
    pub const LEN: usize = 8 + // discriminator
        8 + // buyback_bl
        8 + // reserve_bl
        8 + // motherlode_sol_bl
        16 + // miner_rewards_factor (i128)
        16 + // stake_rewards_factor (i128)
        8 + // total_staked
        8 + // total_unclaimed
        8 + // total_refined
        8; // leaderboard_bl
}
