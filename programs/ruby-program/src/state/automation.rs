use anchor_lang::prelude::*;

#[account]
#[derive(Default)]
pub struct Automation {
    /// The amount of SOL to deploy on each territory per round.
    pub amount: u64,

    /// The authority of this automation account.
    pub authority: Pubkey,

    /// The amount of SOL this automation has left.
    pub balance: u64,

    /// The executor of this automation account.
    pub executor: Pubkey,

    /// The amount of SOL the executor should receive in fees.
    pub fee: u64,

    /// The strategy this automation uses.
    pub strategy: u64,

    /// The mask of squares this automation should deploy to if preferred strategy.
    /// If strategy is Random, first bit is used to determine how many squares to deploy to.
    pub mask: u64,
}

impl Automation {
    pub const LEN: usize = 8 + // discriminator
        8 + // amount
        32 + // authority
        8 + // balance
        32 + // executor
        8 + // fee
        8 + // strategy
        8; // mask
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq)]
pub enum AutomationStrategy {
    Random,
    Preferred,
}

impl AutomationStrategy {
    pub fn from_u64(value: u64) -> Self {
        match value {
            0 => AutomationStrategy::Random,
            1 => AutomationStrategy::Preferred,
            _ => AutomationStrategy::Random,
        }
    }
}
