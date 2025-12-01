use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};
use crate::constants::*;
use crate::errors::AppError;
use crate::events::StakeWithdrawEvent;
use crate::state::*;

#[derive(Accounts)]
pub struct StakeWithdraw<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        mut,
        seeds = [STAKE, signer.key().as_ref()],
        bump,
    )]
    pub stake: Account<'info, Stake>,

    #[account(mut, seeds = [TREASURY], bump)]
    pub treasury: Account<'info, Treasury>,

    #[account(address = MINT_ADDRESS)]
    pub mint: Account<'info, Mint>,

    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = signer
    )]
    pub sender_tokens: Account<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = stake
    )]
    pub stake_tokens: Account<'info, TokenAccount>,

    pub clock: Sysvar<'info, Clock>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<StakeWithdraw>, amount: u64) -> Result<()> {
    // Validate authority
    require!(
        ctx.accounts.stake.authority == ctx.accounts.signer.key(),
        AppError::NotAuthorized
    );

    // Get account info before mutable borrows
    let stake_account_info = ctx.accounts.stake.to_account_info();
    let stake_bump = ctx.bumps.stake;
    let authority = ctx.accounts.signer.key();

    // Now get mutable references
    let stake = &mut ctx.accounts.stake;
    let treasury = &mut ctx.accounts.treasury;
    let clock = &ctx.accounts.clock;

    // Update rewards before changing balance
    let amount = stake.withdraw(amount, clock, treasury);

    // Transfer TOKEN from stake to user with PDA signing
    let signer_seeds: &[&[&[u8]]] = &[&[
        STAKE,
        authority.as_ref(),
        &[stake_bump]
    ]];

    token::transfer(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.stake_tokens.to_account_info(),
                to: ctx.accounts.sender_tokens.to_account_info(),
                authority: stake_account_info,
            },
            signer_seeds,
        ),
        amount,
    )?;

    // Reload stake_tokens account after transfer to get updated balance
    ctx.accounts.stake_tokens.reload()?;

    // Safety check
    require!(
        ctx.accounts.stake_tokens.amount >= stake.balance,
        AppError::InsufficientBalance
    );

    // Emit event
    emit!(StakeWithdrawEvent {
        authority: ctx.accounts.signer.key(),
        amount,
        balance: stake.balance,
        total_staked: treasury.total_staked,
    });

    msg!("Withdrew {} TOKEN tokens from stake", amount);

    Ok(())
}
