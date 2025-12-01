use anchor_lang::prelude::*;

/// One RUBY token, denominated in indivisible units (1 billion units = 1 RUBY with 9 decimals).
pub const ONE_TOKEN: u64 = 1_000_000_000;

/// Initial reward per round: 20 RUBY (Days 0-3)
/// After Day 3: decreases by 2 RUBY per day until reaching 6 RUBY
/// 2 months after hitting 6 RUBY: decreases by 1 RUBY per month
/// Minimum: 1 RUBY per block until all tokens mined
pub const INITIAL_REWARD_PER_ROUND: u64 = ONE_TOKEN * 20;

/// Minimum reward per round: 1 RUBY
pub const MIN_REWARD_PER_ROUND: u64 = ONE_TOKEN;

/// The number of slots for deployment phase (1 minute = 150 slots).
pub const DEPLOYMENT_WINDOW_SLOTS: u64 = 150;

/// The number of slots in 12 hours.
pub const TWELVE_HOURS_SLOTS: u64 = 108_000;

/// The number of slots in one day.
pub const ONE_DAY_SLOTS: u64 = 216_000;

/// The number of slots for breather between rounds.
pub const INTERMISSION_SLOTS: u64 = 10;

/// The maximum token supply: 6,000,000 RUBY
pub const MAX_SUPPLY: u64 = ONE_TOKEN * 6_000_000;

/// Motherlode increment per round: +3 RUBY
pub const MOTHERLODE_INCREMENT: u64 = ONE_TOKEN * 3;

/// The seed of the automation account PDA.
pub const AUTOMATION: &[u8] = b"automation";

/// The seed of the board account PDA.
pub const BOARD: &[u8] = b"board";

/// The seed of the config account PDA.
pub const CONFIG: &[u8] = b"config";

/// The seed of the miner account PDA.
pub const MINER: &[u8] = b"miner";

/// The seed of the round account PDA.
pub const ROUND: &[u8] = b"round";

/// The seed of the referral account PDA.
pub const REFERRAL: &[u8] = b"referral";

/// The seed of the stake account PDA.
pub const STAKE: &[u8] = b"stake";

/// The seed of the treasury account PDA.
pub const TREASURY: &[u8] = b"treasury";

/// The seed of the leaderboard account PDA.
pub const LEADERBOARD: &[u8] = b"leaderboard";

/// Referral fee in basis points (100 = 1%)
pub const REFERRAL_FEE_BPS: u64 = 100;

/// The fee paid to bots if they checkpoint a user (0.00001 SOL).
pub const CHECKPOINT_FEE: u64 = 10_000;

/// The minimum deployment amount per square.
pub const MIN_DEPLOYMENT: u64 = 1_000;

/// The minimum stake amount (1 RUBY).
pub const MIN_STAKE_AMOUNT: u64 = ONE_TOKEN;

/// The address of the mint account (RUBY token - placeholder)
pub const MINT_ADDRESS: Pubkey = pubkey!("So11111111111111111111111111111111111111112");

/// Entropy program ID (deployed on mainnet)
pub const ENTROPY_PROGRAM_ID: Pubkey = pubkey!("So11111111111111111111111111111111111111112");

/// Split address for proportional rewards
pub const SPLIT_ADDRESS: Pubkey = pubkey!("SpLiT11111111111111111111111111111111111112");

/// Total squares on the mining board (5x5 grid)
pub const TOTAL_BOARD: usize = 25;

// ===== RUBY Fee Distribution =====
// Total from losing squares: 100%
// - 88% to winners
// - 8% buyback and burn
// - 2% strategic reserve (leaderboard/mini-motherlode)
// - 2% motherlode pool

/// Winners share: 88%
pub const WINNERS_BPS: u64 = 8800;

/// Buyback share: 8%
pub const BUYBACK_BPS: u64 = 800;

/// Strategic reserve (leaderboard + mini-motherlode): 2%
pub const RESERVE_BPS: u64 = 200;

/// Motherlode pool: 2%
pub const MOTHERLODE_BPS: u64 = 200;

/// Platform fee: 1% of winners amount
pub const PLATFORM_FEE_BPS: u64 = 100;
