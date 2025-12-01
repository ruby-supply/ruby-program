use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};
use anchor_spl::associated_token::AssociatedToken;
use crate::constants::*;
use crate::state::*;

#[derive(Accounts)]
pub struct ClaimToken<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        mut,
        seeds = [MINER, authority.key().as_ref()],
        bump,
    )]
    pub miner: Account<'info, Miner>,

    #[account(
        mut,
        seeds = [TREASURY],
        bump,
    )]
    pub treasury: Account<'info, Treasury>,

    #[account(
        mut,
        address = MINT_ADDRESS
    )]
    pub mint: Account<'info, Mint>,

    /// Treasury's token account (source of tokens)
    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = treasury,
    )]
    pub treasury_tokens: Account<'info, TokenAccount>,

    /// Recipient's token account (auto-created if needed)
    #[account(
        init_if_needed,
        payer = authority,
        associated_token::mint = mint,
        associated_token::authority = authority
    )]
    pub recipient: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

/// Handler for claim_token instruction
///
/// Referral fee is calculated and accrued at checkpoint instruction.
/// Claim token transfers the full rewards_token to user (already minus 1% referral fee).
pub fn handler(ctx: Context<ClaimToken>) -> Result<()> {
    let clock = Clock::get()?;
    let miner = &mut ctx.accounts.miner;
    let treasury = &mut ctx.accounts.treasury;

    let amount = miner.claim_token(&clock, treasury);

    if amount > 0 {
        let treasury_bump = ctx.bumps.treasury;
        let signer_seeds: &[&[&[u8]]] = &[&[TREASURY, &[treasury_bump]]];

        // Transfer full amount to recipient (referral fee already deducted at checkpoint)
        let cpi_accounts = Transfer {
            from: ctx.accounts.treasury_tokens.to_account_info(),
            to: ctx.accounts.recipient.to_account_info(),
            authority: ctx.accounts.treasury.to_account_info(),
        };
        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            cpi_accounts,
            signer_seeds,
        );
        token::transfer(cpi_ctx, amount)?;

        msg!("Claimed {} TOKEN tokens", amount);
    }

    Ok(())
}
