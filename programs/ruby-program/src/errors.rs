use anchor_lang::prelude::*;

#[error_code]
pub enum AppError {
    #[msg("Amount too small")]
    AmountTooSmall,

    #[msg("Not authorized")]
    NotAuthorized,

    #[msg("Invalid square selection")]
    InvalidSquare,

    #[msg("Round has ended")]
    RoundEnded,

    #[msg("Round has not started")]
    RoundNotStarted,

    #[msg("Invalid automation strategy")]
    InvalidAutomationStrategy,

    #[msg("Insufficient balance")]
    InsufficientBalance,

    #[msg("Invalid timestamp")]
    InvalidTimestamp,

    #[msg("Miner must checkpoint previous round before deploying to new round")]
    MustCheckpoint,

    #[msg("Insufficient rent")]
    InsufficientRent,

    #[msg("Invalid deployment")]
    InvalidDeployment,

    #[msg("Invalid end slot for entropy var")]
    InvalidEndSlot,

    #[msg("Entropy var not finalized")]
    EntropyNotFinalized,

    #[msg("Invalid entropy var address")]
    InvalidEntropyVar,

    #[msg("Entropy oracle not configured")]
    EntropyNotConfigured,

    #[msg("Invalid swap accounts provided")]
    InvalidSwapAccounts,

    #[msg("Invalid state after swap (potential exploit detected)")]
    InvalidSwapState,

    #[msg("Arithmetic overflow")]
    Overflow,

    #[msg("Arithmetic underflow")]
    Underflow,

    #[msg("Invalid amount")]
    InvalidAmount,

    #[msg("Invalid fee collector address")]
    InvalidFeeCollector,

    #[msg("Cannot refer yourself")]
    SelfReferral,

    #[msg("Invalid referral account")]
    InvalidReferral,
}
