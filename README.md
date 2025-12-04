# RUBY

**A fair launch digital store of value on Solana.**

RUBY is a fair launch cryptocurrency with a capped maximum supply of 6,000,000 tokens. There are no private sales, no team allocations, and no venture capital investors. All tokens are distributed through presale and mining, ensuring equal opportunity for all participants.

## Overview

The protocol operates entirely onchain through smart contracts on Solana, providing transparency, security, and decentralization. RUBY combines competitive mining mechanics with modern DeFi features like referrals, automated buybacks, and a dual motherlode system.

## Tokenomics

### Fixed Supply
- **Maximum supply**: 6,000,000 RUBY
- **No team allocation**, no private sale, premine if public presale goals are not reached
- All remaining RUBY must be mined through the emission schedule
- The supply is permanently capped

### Presale Details
- Maximum 5 SOL contribution per person
- Total presale cap: 200 SOL
- Rate: 1 SOL = 200 Unrefined RUBY
- Presale funds are used entirely for initial liquidity

### Mining Emission Schedule
- **Days 0–3**: 20 RUBY per block
- **After Day 3**: emissions decrease by 2 RUBY per day until reaching 6 RUBY per block
- **2 months after hitting 6 RUBY/block**: block rewards reduce by 1 RUBY every month
- **Minimum reward**: 1 RUBY per block, until all tokens are mined

## Fee Distribution

From losing squares each round:
- **88%** → Winners (miners on winning square)
- **8%** → Automated buybacks and burn
- **2%** → Strategic reserve (leaderboard rewards + mini-motherlode)
- **2%** → Motherlode pool

## Key Features

- **Mining**: Deploy SOL on blocks in a 5x5 grid and compete for RUBY rewards every round.
- **Buybacks**: Protocol automatically buys back RUBY using 8% of SOL mining revenue.
- **Motherlode (ML)**: Progressive jackpot that increases by +3 RUBY per round until hit.
- **Mini-Motherlode**: SOL reload rewards to help miners stay active.
- **Leaderboard**: Weekly SOL rewards for top deployers.

## API

### Register Referral

Registers a referral relationship. Must be called before deploying SOL for the first time.

```rust
pub fn register_referral(ctx: Context<RegisterReferral>) -> Result<()>
```

#### Accounts

| Name | Type | Description |
| ---- | ---- | ----------- |
| authority | Signer | The user registering the referral |
| referrer | AccountInfo | The wallet that referred this user |
| referral | Account\<Referral\> | PDA to store the referral relationship |
| system_program | Program | Solana system program |

---

### Deploy

Deploys SOL to squares on the 5x5 game board to participate in mining.

```rust
pub fn deploy(ctx: Context<Deploy>, amount: u64, squares: u32) -> Result<()>
```

#### Accounts

| Name | Type | Description |
| ---- | ---- | ----------- |
| signer | Signer | The user deploying SOL |
| authority | AccountInfo | The authority for the miner account |
| config | Account\<Config\> | Global configuration |
| automation | Option\<Account\<Automation\>\> | Optional automation settings |
| board | Account\<Board\> | Current game board state |
| miner | Account\<Miner\> | User's miner account |
| round | Account\<Round\> | Current round state |
| entropy_var | AccountInfo | Entropy VRF account |
| entropy_program | AccountInfo | Entropy program |
| referral | Account\<Referral\> | User's referral account |
| system_program | Program | Solana system program |

#### Arguments

| Name | Type | Description |
| ---- | ---- | ----------- |
| amount | u64 | Amount of SOL to deploy per square (in lamports) |
| squares | u32 | Bitmask of squares to deploy to (25 bits for 5x5 grid) |

---

### Checkpoint

Calculates and records mining rewards after a round ends. Must be called before claiming rewards.

```rust
pub fn checkpoint(ctx: Context<Checkpoint>) -> Result<()>
```

#### Accounts

| Name | Type | Description |
| ---- | ---- | ----------- |
| signer | Signer | Anyone can checkpoint (bots incentivized with fees) |
| board | Account\<Board\> | Current game board state |
| miner | Account\<Miner\> | Miner account to checkpoint |
| round | Account\<Round\> | Round to checkpoint against |
| treasury | Account\<Treasury\> | Treasury for reward tracking |
| referral | Account\<Referral\> | Miner's referral account |
| system_program | Program | Solana system program |

---

### Claim SOL

Claims accumulated SOL rewards from winning rounds.

```rust
pub fn claim_sol(ctx: Context<ClaimSol>) -> Result<()>
```

#### Accounts

| Name | Type | Description |
| ---- | ---- | ----------- |
| authority | Signer | Owner of the miner account |
| miner | Account\<Miner\> | Miner account with rewards |
| system_program | Program | Solana system program |

---

### Claim Token

Claims accumulated RUBY token rewards from mining.

```rust
pub fn claim_token(ctx: Context<ClaimToken>) -> Result<()>
```

#### Accounts

| Name | Type | Description |
| ---- | ---- | ----------- |
| authority | Signer | Owner of the miner account |
| miner | Account\<Miner\> | Miner account with rewards |
| treasury | Account\<Treasury\> | Treasury holding tokens |
| mint | Account\<Mint\> | RUBY token mint |
| treasury_tokens | Account\<TokenAccount\> | Treasury's token account |
| recipient | Account\<TokenAccount\> | User's token account |
| token_program | Program | SPL Token program |
| associated_token_program | Program | Associated Token program |
| system_program | Program | Solana system program |

---

### Stake Deposit

Stakes RUBY tokens to earn yield from protocol fees and buybacks.

```rust
pub fn stake_deposit(ctx: Context<StakeDeposit>, amount: u64) -> Result<()>
```

#### Accounts

| Name | Type | Description |
| ---- | ---- | ----------- |
| signer | Signer | The user staking tokens |
| stake | Account\<Stake\> | User's stake account |
| treasury | Account\<Treasury\> | Treasury for stake tracking |
| mint | Account\<Mint\> | RUBY token mint |
| sender_tokens | Account\<TokenAccount\> | User's token account |
| stake_tokens | Account\<TokenAccount\> | Stake escrow token account |
| clock | Sysvar\<Clock\> | Clock sysvar |
| token_program | Program | SPL Token program |
| associated_token_program | Program | Associated Token program |
| system_program | Program | Solana system program |

#### Arguments

| Name | Type | Description |
| ---- | ---- | ----------- |
| amount | u64 | Amount of RUBY tokens to stake |

---

### Stake Withdraw

Withdraws staked RUBY tokens.

```rust
pub fn stake_withdraw(ctx: Context<StakeWithdraw>, amount: u64) -> Result<()>
```

#### Accounts

| Name | Type | Description |
| ---- | ---- | ----------- |
| signer | Signer | Owner of the stake account |
| stake | Account\<Stake\> | User's stake account |
| treasury | Account\<Treasury\> | Treasury for stake tracking |
| mint | Account\<Mint\> | RUBY token mint |
| sender_tokens | Account\<TokenAccount\> | User's token account |
| stake_tokens | Account\<TokenAccount\> | Stake escrow token account |
| clock | Sysvar\<Clock\> | Clock sysvar |
| token_program | Program | SPL Token program |
| associated_token_program | Program | Associated Token program |
| system_program | Program | Solana system program |

#### Arguments

| Name | Type | Description |
| ---- | ---- | ----------- |
| amount | u64 | Amount of RUBY tokens to withdraw |

---

### Stake Claim

Claims accumulated SOL staking rewards.

```rust
pub fn stake_claim(ctx: Context<StakeClaim>) -> Result<()>
```

#### Accounts

| Name | Type | Description |
| ---- | ---- | ----------- |
| signer | Signer | Owner of the stake account |
| stake | Account\<Stake\> | User's stake account |
| treasury | Account\<Treasury\> | Treasury holding rewards |
| recipient | UncheckedAccount | Account to receive SOL rewards |
| clock | Sysvar\<Clock\> | Clock sysvar |
| system_program | Program | Solana system program |

---

### Claim Referral Rewards

Claims accumulated referral rewards from referred users.

```rust
pub fn claim_referral_rewards(ctx: Context<ClaimReferralRewards>) -> Result<()>
```

#### Accounts

| Name | Type | Description |
| ---- | ---- | ----------- |
| authority | Signer | The referrer claiming rewards |
| treasury | Account\<Treasury\> | Treasury holding tokens |
| mint | Account\<Mint\> | RUBY token mint |
| treasury_tokens | Account\<TokenAccount\> | Treasury's token account |
| recipient | Account\<TokenAccount\> | Referrer's token account |
| token_program | Program | SPL Token program |
| associated_token_program | Program | Associated Token program |
| system_program | Program | Solana system program |

#### Remaining Accounts

Pass Referral PDAs as remaining accounts to claim from multiple referees in one transaction.

---

### Automate

Configures automation settings for bot-assisted mining.

```rust
pub fn automate(ctx: Context<Automate>, args: AutomateArgs) -> Result<()>
```

#### Accounts

| Name | Type | Description |
| ---- | ---- | ----------- |
| authority | Signer | Owner of the automation |
| automation | Account\<Automation\> | Automation settings account |
| executor | AccountInfo | Bot/service that will execute |
| miner | Account\<Miner\> | User's miner account |
| system_program | Program | Solana system program |

#### Arguments

| Name | Type | Description |
| ---- | ---- | ----------- |
| amount | u64 | SOL amount to deploy per square |
| deposit | u64 | SOL to deposit into automation balance |
| fee | u64 | Fee to pay executor per deployment |
| mask | u64 | Bitmask for preferred squares |
| strategy | u8 | 0 = Random, 1 = Preferred |

---

### Cancel Automate

Cancels automation and withdraws remaining balance.

```rust
pub fn cancel_automate(ctx: Context<CancelAutomate>) -> Result<()>
```

#### Accounts

| Name | Type | Description |
| ---- | ---- | ----------- |
| authority | Signer | Owner of the automation |
| automation | Account\<Automation\> | Automation account to close |
| system_program | Program | Solana system program |

---

### Close

Closes a miner account and returns rent to owner.

```rust
pub fn close(ctx: Context<Close>) -> Result<()>
```

#### Accounts

| Name | Type | Description |
| ---- | ---- | ----------- |
| authority | Signer | Owner of the miner account |
| miner | Account\<Miner\> | Miner account to close |

---

## State Accounts

### Config
Global configuration storing admin addresses and protocol settings.

### Board
Tracks current round number and timing (start/end slots).

### Treasury
Holds protocol funds including buyback balance, motherlode pool, and staking rewards.

### Round
Per-round state tracking deployed SOL per square, winner counts, and reward distribution.

### Miner
Per-user account tracking deployments, rewards, and checkpoint status.

### Stake
Per-user staking account with balance and reward tracking.

### Referral
Stores referral relationship and pending/claimed rewards.

### Automation
Bot configuration for automated mining deployments.

---


## License

This project is licensed under the MIT License.
