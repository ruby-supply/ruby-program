use anchor_lang::prelude::*;

#[account]
#[derive(Default)]
pub struct Config {
    /// The address that can update the config.
    pub admin: Pubkey,

    /// The address with authority to call bury.
    pub bury_authority: Pubkey,

    /// The address that receives admin fees.
    pub fee_collector: Pubkey,

    /// The program id for the protocol swaps.
    pub swap_program: Pubkey,

    /// The address of the entropy var account.
    pub var_address: Pubkey,

    /// Buffer array
    pub buffer: u64,
}

impl Config {
    pub const LEN: usize = 8 + // discriminator
        32 + // admin
        32 + // bury_authority
        32 + // fee_collector
        32 + // swap_program
        32 + // var_address
        8; // buffer
}
