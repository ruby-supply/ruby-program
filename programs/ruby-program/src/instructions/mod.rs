// Miner instructions
pub mod deploy;
pub mod claim_token;
pub mod claim_sol;
pub mod checkpoint;
pub mod close;

// Staking instructions
pub mod stake_deposit;
pub mod stake_withdraw;
pub mod stake_claim;

// Referral instructions
pub mod register_referral;
pub mod claim_referral_rewards;

// Automation
pub mod automate;
pub mod cancel_automate;

// Admin instructions
pub mod initialize;
pub mod reset;
pub mod bury;
pub mod wrap;
pub mod set_admin;
pub mod set_fee_collector;
pub mod set_swap_program;
pub mod set_var_address;
pub mod new_var;
pub mod set_buffer;

// Glob re-exports are needed for Anchor macro to generate client accounts
// The ambiguous `handler` name is intentional - each module has its own handler
#[allow(ambiguous_glob_reexports)]
pub use deploy::*;
#[allow(ambiguous_glob_reexports)]
pub use claim_token::*;
#[allow(ambiguous_glob_reexports)]
pub use claim_sol::*;
#[allow(ambiguous_glob_reexports)]
pub use checkpoint::*;
#[allow(ambiguous_glob_reexports)]
pub use close::*;
#[allow(ambiguous_glob_reexports)]
pub use stake_deposit::*;
#[allow(ambiguous_glob_reexports)]
pub use stake_withdraw::*;
#[allow(ambiguous_glob_reexports)]
pub use stake_claim::*;
#[allow(ambiguous_glob_reexports)]
pub use register_referral::*;
#[allow(ambiguous_glob_reexports)]
pub use claim_referral_rewards::*;
#[allow(ambiguous_glob_reexports)]
pub use automate::*;
#[allow(ambiguous_glob_reexports)]
pub use cancel_automate::*;
#[allow(ambiguous_glob_reexports)]
pub use initialize::*;
#[allow(ambiguous_glob_reexports)]
pub use reset::*;
#[allow(ambiguous_glob_reexports)]
pub use bury::*;
#[allow(ambiguous_glob_reexports)]
pub use wrap::*;
#[allow(ambiguous_glob_reexports)]
pub use set_admin::*;
#[allow(ambiguous_glob_reexports)]
pub use set_fee_collector::*;
#[allow(ambiguous_glob_reexports)]
pub use set_swap_program::*;
#[allow(ambiguous_glob_reexports)]
pub use set_var_address::*;
#[allow(ambiguous_glob_reexports)]
pub use new_var::*;
#[allow(ambiguous_glob_reexports)]
pub use set_buffer::*;
