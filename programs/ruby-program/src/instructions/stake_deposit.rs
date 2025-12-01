use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};
use crate::constants::*;
use crate::errors::AppError;
use crate::events::StakeDepositEvent;
use crate::state::*;

#[derive(Accounts)]
pub struct StakeDeposit<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        init_if_needed,
        payer = signer,
        space = Stake::LEN,
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
        init_if_needed,
        payer = signer,
        associated_token::mint = mint,
        associated_token::authority = stake
    )]
    pub stake_tokens: Account<'info, TokenAccount>,

    pub clock: Sysvar<'info, Clock>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<StakeDeposit>, amount: u64) -> Result<()> {
    // Validate minimum stake amount
    require!(
        amount >= MIN_STAKE_AMOUNT,
        AppError::AmountTooSmall
    );

    let stake = &mut ctx.accounts.stake;
    let treasury = &mut ctx.accounts.treasury;
    let clock = &ctx.accounts.clock;

    // Initialize stake account if first time
    if stake.authority == Pubkey::default() {
        stake.authority = ctx.accounts.signer.key();
        stake.rewards_factor = treasury.stake_rewards_factor;
    }

    // Update rewards before changing balance
    let amount = stake.deposit(
        amount,
        clock,
        treasury,
        &ctx.accounts.sender_tokens
    );

    // Transfer TOKEN from user to stake account
    token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.sender_tokens.to_account_info(),
                to: ctx.accounts.stake_tokens.to_account_info(),
                authority: ctx.accounts.signer.to_account_info(),
            },
        ),
        amount,
    )?;

    // Reload stake_tokens account to get updated balance after transfer
    ctx.accounts.stake_tokens.reload()?;

    // Safety check
    require!(
        ctx.accounts.stake_tokens.amount >= stake.balance,
        AppError::InsufficientBalance
    );

    // Emit event
    emit!(StakeDepositEvent {
        authority: ctx.accounts.signer.key(),
        amount,
        balance: stake.balance,
        total_staked: treasury.total_staked,
    });

    msg!("Staked {} TOKEN tokens", amount);

    Ok(())
}
