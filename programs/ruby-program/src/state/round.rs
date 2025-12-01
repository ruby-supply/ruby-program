use anchor_lang::prelude::*;

use crate::constants::TOTAL_BOARD;

#[account]
#[derive(Default)]
pub struct Round {
    /// The round number.
    pub id: u64,

    /// The amount of SOL deployed in each square.
    pub deployed: [u64; TOTAL_BOARD],

    /// The hash of the end slot, provided by solana, used for random number generation.
    pub slot_hash: [u8; 32],

    /// The count of miners on each square.
    pub count: [u64; TOTAL_BOARD],

    /// The slot at which claims for this round account end.
    pub expires_at: u64,

    /// The amount of TOKEN in the motherlode.
    pub motherlode: u64,

    /// The account to which rent should be returned when this account is closed.
    pub rent_payer: Pubkey,

    /// The top miner of the round.
    pub top_miner: Pubkey,

    /// The amount of TOKEN to distribute to the top miner.
    pub top_miner_reward: u64,

    /// The total amount of SOL deployed in the round.
    pub total_deployed: u64,

    /// The total amount of SOL put in the TOKEN vault.
    pub total_vaulted: u64,

    /// The total amount of SOL to staking rewards.
    pub total_stake_rewards: u64,

    /// The total amount of SOL won by miners for the round.
    pub total_winnings: u64,
}

impl Round {
    pub const LEN: usize = 8 + // discriminator
        8 + // id
        (8 * TOTAL_BOARD) + // deployed
        32 + // slot_hash
        (8 * TOTAL_BOARD) + // count
        8 + // expires_at
        8 + // motherlode
        32 + // rent_payer
        32 + // top_miner
        8 + // top_miner_reward
        8 + // total_deployed
        8 + // total_vaulted
        8 + // total_stake_rewards
        8; // total_winnings

    pub fn rng(&self) -> Option<u64> {
        if self.slot_hash == [0; 32] || self.slot_hash == [u8::MAX; 32] {
            return None;
        }
        let r1 = u64::from_le_bytes(self.slot_hash[0..8].try_into().unwrap());
        let r2 = u64::from_le_bytes(self.slot_hash[8..16].try_into().unwrap());
        let r3 = u64::from_le_bytes(self.slot_hash[16..24].try_into().unwrap());
        let r4 = u64::from_le_bytes(self.slot_hash[24..32].try_into().unwrap());
        let r = r1 ^ r2 ^ r3 ^ r4;
        Some(r)
    }

    pub fn winning_square(&self, rng: u64) -> usize {
        (rng % 25) as usize
    }

    pub fn top_miner_sample(&self, rng: u64, winning_square: usize) -> u64 {
        if self.deployed[winning_square] == 0 {
            return 0;
        }
        rng.reverse_bits() % self.deployed[winning_square]
    }

    pub fn calculate_total_winnings(&self, winning_square: usize) -> u64 {
        let mut total_winnings = 0;
        for (i, &deployed) in self.deployed.iter().enumerate() {
            if i != winning_square {
                total_winnings += deployed;
            }
        }
        total_winnings
    }

    pub fn is_split_reward(&self, rng: u64) -> bool {
        // One out of four rounds get split rewards.
        let rng = rng.reverse_bits().to_le_bytes();
        let r1 = u16::from_le_bytes(rng[0..2].try_into().unwrap());
        let r2 = u16::from_le_bytes(rng[2..4].try_into().unwrap());
        let r3 = u16::from_le_bytes(rng[4..6].try_into().unwrap());
        let r4 = u16::from_le_bytes(rng[6..8].try_into().unwrap());
        let r = r1 ^ r2 ^ r3 ^ r4;
        r % 2 == 0
    }

    pub fn did_hit_motherlode(&self, rng: u64) -> bool {
        rng.reverse_bits() % 625 == 0
    }
}
