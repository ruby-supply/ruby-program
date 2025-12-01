use anchor_lang::prelude::*;

pub mod constants;
pub mod errors;
pub mod events;
pub mod instructions;
pub mod state;
pub mod utils;

use instructions::*;

declare_id!("So11111111111111111111111111111111111111112");

#[program]
pub mod ruby_program {
    use super::*;

    // ===== INITIALIZATION =====

    /// Initialize the program
    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        instructions::initialize::handler(ctx)
    }

    // ===== MINER INSTRUCTIONS =====

    /// Deploy SOL to squares on the game board
    pub fn deploy(ctx: Context<Deploy>, amount: u64, squares: u32) -> Result<()> {
        instructions::deploy::handler(ctx, amount, squares)
    }

    /// Checkpoint miner rewards
    pub fn checkpoint(ctx: Context<Checkpoint>) -> Result<()> {
        instructions::checkpoint::handler(ctx)
    }

    /// Claim RUBY token rewards
    /// Referral fee is calculated at checkpoint instruction
    pub fn claim_token(ctx: Context<ClaimToken>) -> Result<()> {
        instructions::claim_token::handler(ctx)
    }

    /// Claim SOL rewards
    pub fn claim_sol(ctx: Context<ClaimSol>) -> Result<()> {
        instructions::claim_sol::handler(ctx)
    }

    /// Close miner account
    pub fn close(ctx: Context<Close>) -> Result<()> {
        instructions::close::handler(ctx)
    }

    // ===== STAKING INSTRUCTIONS =====

    /// Deposit RUBY tokens into staking
    pub fn stake_deposit(ctx: Context<StakeDeposit>, amount: u64) -> Result<()> {
        instructions::stake_deposit::handler(ctx, amount)
    }

    /// Withdraw RUBY tokens from staking
    pub fn stake_withdraw(ctx: Context<StakeWithdraw>, amount: u64) -> Result<()> {
        instructions::stake_withdraw::handler(ctx, amount)
    }

    /// Claim SOL staking rewards (claims all available rewards)
    pub fn stake_claim(ctx: Context<StakeClaim>) -> Result<()> {
        instructions::stake_claim::handler(ctx)
    }

    // ===== REFERRAL =====

    /// Register a referral relationship
    pub fn register_referral(ctx: Context<RegisterReferral>) -> Result<()> {
        instructions::register_referral::handler(ctx)
    }

    /// Claim referral rewards from multiple referees
    /// Pass Referral PDAs in remaining_accounts
    pub fn claim_referral_rewards<'info>(
        ctx: Context<'_, '_, 'info, 'info, ClaimReferralRewards<'info>>
    ) -> Result<()> {
        instructions::claim_referral_rewards::handler(ctx)
    }

    // ===== AUTOMATION =====

    /// Configure automation
    pub fn automate(ctx: Context<Automate>, args: AutomateArgs) -> Result<()> {
        instructions::automate::handler(ctx, args)
    }

    /// Cancel automation and withdraw all funds
    pub fn cancel_automate(ctx: Context<CancelAutomate>) -> Result<()> {
        instructions::cancel_automate::handler(ctx)
    }

    // ===== ROUND MANAGEMENT =====

    /// Reset round and start new one
    pub fn reset(ctx: Context<Reset>) -> Result<()> {
        instructions::reset::handler(ctx)
    }

    // ===== ADMIN INSTRUCTIONS =====

    /// Buy and burn RUBY tokens
    pub fn bury<'info>(
        ctx: Context<'_, '_, '_, 'info, Bury<'info>>,
        swap_data: Vec<u8>,
    ) -> Result<()> {
        instructions::bury::handler(ctx, swap_data)
    }

    /// Wrap SOL into treasury
    pub fn wrap(ctx: Context<Wrap>) -> Result<()> {
        instructions::wrap::handler(ctx)
    }

    /// Set admin address
    pub fn set_admin(ctx: Context<SetAdmin>, args: SetAdminArgs) -> Result<()> {
        instructions::set_admin::handler(ctx, args)
    }

    /// Set fee collector address
    pub fn set_fee_collector(
        ctx: Context<SetFeeCollector>,
        args: SetFeeCollectorArgs,
    ) -> Result<()> {
        instructions::set_fee_collector::handler(ctx, args)
    }

    /// Set swap program
    pub fn set_swap_program(ctx: Context<SetSwapProgram>) -> Result<()> {
        instructions::set_swap_program::handler(ctx)
    }

    /// Set entropy var address
    pub fn set_var_address(ctx: Context<SetVarAddress>) -> Result<()> {
        instructions::set_var_address::handler(ctx)
    }

    /// Create new entropy var
    pub fn new_var(ctx: Context<NewVar>, args: NewVarArgs) -> Result<()> {
        instructions::new_var::handler(ctx, args)
    }

    /// Set buffer value
    pub fn set_buffer(ctx: Context<SetBuffer>, args: SetBufferArgs) -> Result<()> {
        instructions::set_buffer::handler(ctx, args)
    }
}
