use anchor_lang::prelude::*;

/// Referral account - stores referral relationship and pending rewards
#[account]
#[derive(Default)]
pub struct Referral {
    /// The authority (referee) who registered this referral
    pub authority: Pubkey,

    /// The referrer who referred this user
    pub referrer: Pubkey,

    /// Pending TOKEN rewards for referrer to claim
    pub pending_rewards: u64,

    /// Total TOKEN rewards already claimed by referrer
    pub claimed_rewards: u64,

    /// Timestamp when the referral was created
    pub created_at: i64,
}

impl Referral {
    pub const LEN: usize = 8 + // discriminator
        32 + // authority
        32 + // referrer
        8 +  // pending_rewards
        8 +  // claimed_rewards
        8;   // created_at
    // Total: 96 bytes
}
