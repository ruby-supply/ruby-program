use anchor_lang::prelude::*;

/// Event emitted when reset instruction is executed
#[event]
pub struct ResetEvent {
    /// The round that was reset
    pub round_id: u64,

    /// The start slot of the next round
    pub start_slot: u64,

    /// The end slot of the next round
    pub end_slot: u64,

    /// The winning square (u64::MAX if no RNG)
    pub winning_square: u64,

    /// The top miner of the round
    pub top_miner: Pubkey,

    /// The number of miners on the winning square
    pub num_winners: u64,

    /// The amount of TOKEN payout for the motherlode
    pub motherlode: u64,

    /// The total amount of SOL deployed in the round
    pub total_deployed: u64,

    /// The total amount of SOL put in the treasury
    pub total_vaulted: u64,

    /// The total amount of SOL won by miners for the round
    pub total_winnings: u64,

    /// The total amount of TOKEN minted for the round
    pub total_minted: u64,

    /// Unix timestamp when event occurred
    pub timestamp: i64,
}

/// Event emitted when deploy instruction is executed
#[event]
pub struct DeployEvent {
    /// The authority of the deployer
    pub authority: Pubkey,

    /// The round id
    pub round_id: u64,

    /// The amount of SOL deployed per square
    pub amount: u64,

    /// The number of squares deployed to
    pub squares: u32,

    /// The total cumulative SOL deployed in the round
    pub total_deployed: u64,

    /// Unix timestamp when event occurred
    pub timestamp: i64,
}

/// Event emitted when checkpoint instruction is executed
#[event]
pub struct CheckpointEvent {
    /// The miner authority
    pub miner: Pubkey,

    /// The round id
    pub round_id: u64,

    /// The winning square
    pub winning_square: u64,

    /// SOL rewards earned
    pub rewards_sol: u64,

    /// TOKEN rewards earned
    pub rewards_token: u64,

    /// Bot fee paid (if applicable)
    pub bot_fee: u64,

    /// Whether this miner was the top miner
    pub is_top_miner: bool,

    /// Unix timestamp when event occurred
    pub timestamp: i64,
}

/// Event emitted when bury instruction is executed
#[event]
pub struct BuryEvent {
    /// Amount of TOKEN burned
    pub token_burned: u64,

    /// Amount of TOKEN shared with stakers (0 in Anchor version - no staking)
    pub token_shared: u64,

    /// Amount of SOL swapped
    pub sol_amount: u64,

    /// New circulating supply after burn
    pub new_circulating_supply: u64,

    /// Unix timestamp when event occurred
    pub timestamp: i64,
}

/// Event emitted when stake_deposit instruction is executed
#[event]
pub struct StakeDepositEvent {
    /// The authority of the staker
    pub authority: Pubkey,

    /// The amount of TOKEN deposited
    pub amount: u64,

    /// The new balance after deposit
    pub balance: u64,

    /// The total staked across all users
    pub total_staked: u64,
}

/// Event emitted when stake_withdraw instruction is executed
#[event]
pub struct StakeWithdrawEvent {
    /// The authority of the staker
    pub authority: Pubkey,

    /// The amount of TOKEN withdrawn
    pub amount: u64,

    /// The remaining balance after withdrawal
    pub balance: u64,

    /// The total staked across all users
    pub total_staked: u64,
}

/// Event emitted when stake_claim instruction is executed
#[event]
pub struct StakeClaimEvent {
    /// The authority of the staker
    pub authority: Pubkey,

    /// The amount of SOL claimed
    pub amount: u64,

    /// The remaining claimable rewards
    pub rewards_remaining: u64,

    /// The lifetime rewards earned
    pub lifetime_rewards: u64,
}

/// Event emitted when referral is registered
#[event]
pub struct ReferralRegisteredEvent {
    /// The user who registered the referral
    pub authority: Pubkey,

    /// The referrer who referred this user
    pub referrer: Pubkey,

    /// Unix timestamp when event occurred
    pub timestamp: i64,
}

/// Event emitted when referral reward is accrued (when referee claims TOKEN)
#[event]
pub struct ReferralRewardAccruedEvent {
    /// The referee (user who claimed TOKEN)
    pub referee: Pubkey,

    /// The referrer who will receive the reward
    pub referrer: Pubkey,

    /// Amount of TOKEN accrued as referral reward
    pub amount: u64,

    /// Unix timestamp when event occurred
    pub timestamp: i64,
}

/// Event emitted when referrer claims their referral rewards
#[event]
pub struct ReferralRewardClaimedEvent {
    /// The referrer who claimed rewards
    pub referrer: Pubkey,

    /// Total amount of TOKEN claimed
    pub amount: u64,

    /// Number of referees claimed from
    pub num_referees: u64,

    /// Unix timestamp when event occurred
    pub timestamp: i64,
}
