use anchor_lang::prelude::*;
use anchor_spl::token::{self, Burn, Mint, Token, TokenAccount};
use crate::constants::*;
use crate::errors::AppError;
use crate::events::BuryEvent;
use crate::state::*;

/// Accounts for the bury instruction
///
/// This instruction swaps treasury SOL for TOKEN tokens via an external swap program,
/// then burns all acquired TOKEN to create deflationary pressure.
#[derive(Accounts)]
pub struct Bury<'info> {
    /// Authority that can execute bury (must match config.bury_authority)
    #[account(mut)]
    pub signer: Signer<'info>,

    /// Config account containing bury_authority and swap_program
    #[account(
        seeds = [CONFIG],
        bump,
    )]
    pub config: Account<'info, Config>,

    /// TOKEN mint address
    #[account(
        mut,
        address = MINT_ADDRESS
    )]
    pub mint: Account<'info, Mint>,

    /// Treasury account holding SOL balance
    #[account(
        mut,
        seeds = [TREASURY],
        bump,
    )]
    pub treasury: Account<'info, Treasury>,

    /// Treasury's TOKEN token account (will receive TOKEN from swap, then burn)
    #[account(mut)]
    pub treasury_ore_tokens: Account<'info, TokenAccount>,

    /// Treasury's wrapped SOL token account (for swap)
    #[account(mut)]
    pub treasury_sol_tokens: Account<'info, TokenAccount>,

    /// Token program
    pub token_program: Program<'info, Token>,

    /// System program (for potential SOL transfers)
    pub system_program: Program<'info, System>,
}

/// Handler for bury instruction
///
/// Process:
/// 1. Validate bury_authority
/// 2. Record pre-swap state (balances, supply, lamports)
/// 3. Invoke swap program CPI (remaining accounts = swap accounts)
/// 4. Validate post-swap state (no exploits)
/// 5. Burn all acquired TOKEN
/// 6. Log results
pub fn handler<'info>(
    ctx: Context<'_, '_, '_, 'info, Bury<'info>>,
    swap_data: Vec<u8>,
) -> Result<()> {
    // Validate authority
    require!(
        ctx.accounts.signer.key() == ctx.accounts.config.bury_authority,
        AppError::NotAuthorized
    );

    // Verify treasury owns the TOKEN token account
    require!(
        ctx.accounts.treasury_ore_tokens.owner == ctx.accounts.treasury.key(),
        AppError::NotAuthorized
    );

    // Verify treasury owns the SOL token account
    require!(
        ctx.accounts.treasury_sol_tokens.owner == ctx.accounts.treasury.key(),
        AppError::NotAuthorized
    );

    // Sync native SOL balance (important for wrapped SOL)
    let sync_native_ix = spl_token::instruction::sync_native(
        &spl_token::ID,
        &ctx.accounts.treasury_sol_tokens.key(),
    )?;
    solana_program::program::invoke(
        &sync_native_ix,
        &[
            ctx.accounts.treasury_sol_tokens.to_account_info(),
        ],
    )?;

    // Reload SOL token account after sync
    ctx.accounts.treasury_sol_tokens.reload()?;

    // Record pre-swap state
    let pre_swap_ore_balance = ctx.accounts.treasury_ore_tokens.amount;
    let pre_swap_sol_balance = ctx.accounts.treasury_sol_tokens.amount;
    let pre_swap_mint_supply = ctx.accounts.mint.supply;
    let pre_swap_treasury_lamports = ctx.accounts.treasury.to_account_info().lamports();

    // Validate we have SOL to swap
    require!(
        pre_swap_sol_balance > 0,
        AppError::InsufficientBalance
    );

    msg!("Pre-swap state:");
    msg!("  TOKEN balance: {}", pre_swap_ore_balance);
    msg!("  SOL balance: {}", pre_swap_sol_balance);
    msg!("  Mint supply: {}", pre_swap_mint_supply);
    msg!("  Treasury lamports: {}", pre_swap_treasury_lamports);

    // Get remaining accounts for swap CPI
    let remaining_accounts = ctx.remaining_accounts;
    require!(
        !remaining_accounts.is_empty(),
        AppError::InvalidSwapAccounts
    );

    msg!("Invoking swap program: {}", ctx.accounts.config.swap_program);
    msg!("Swap accounts count: {}", remaining_accounts.len());

    // Build swap instruction accounts
    let swap_account_metas: Vec<AccountMeta> = remaining_accounts
        .iter()
        .map(|acc| {
            // Treasury should be signer for the swap
            let is_signer = acc.key == &ctx.accounts.treasury.key();
            AccountMeta {
                pubkey: *acc.key,
                is_signer,
                is_writable: acc.is_writable,
            }
        })
        .collect();

    // Create swap instruction
    let swap_ix = solana_program::instruction::Instruction {
        program_id: ctx.accounts.config.swap_program,
        accounts: swap_account_metas,
        data: swap_data,
    };

    // Get treasury PDA signer seeds
    let treasury_bump = ctx.bumps.treasury;
    let signer_seeds: &[&[&[u8]]] = &[&[TREASURY, &[treasury_bump]]];

    // Invoke swap program with treasury as signer
    solana_program::program::invoke_signed(
        &swap_ix,
        remaining_accounts,
        signer_seeds,
    )?;

    msg!("Swap completed, validating post-swap state...");

    // Reload accounts after swap
    ctx.accounts.treasury.reload()?;
    ctx.accounts.mint.reload()?;
    ctx.accounts.treasury_ore_tokens.reload()?;
    ctx.accounts.treasury_sol_tokens.reload()?;

    // Record post-swap state
    let post_swap_ore_balance = ctx.accounts.treasury_ore_tokens.amount;
    let post_swap_sol_balance = ctx.accounts.treasury_sol_tokens.amount;
    let post_swap_mint_supply = ctx.accounts.mint.supply;
    let post_swap_treasury_lamports = ctx.accounts.treasury.to_account_info().lamports();

    msg!("Post-swap state:");
    msg!("  TOKEN balance: {}", post_swap_ore_balance);
    msg!("  SOL balance: {}", post_swap_sol_balance);
    msg!("  Mint supply: {}", post_swap_mint_supply);
    msg!("  Treasury lamports: {}", post_swap_treasury_lamports);

    // CRITICAL SAFETY CHECKS: Prevent exploits

    // 1. Mint supply must not change during swap (no minting allowed)
    require!(
        post_swap_mint_supply == pre_swap_mint_supply,
        AppError::InvalidSwapState
    );

    // 2. Treasury SOL lamports must not change (swap should use wSOL token account)
    require!(
        post_swap_treasury_lamports == pre_swap_treasury_lamports,
        AppError::InvalidSwapState
    );

    // 3. TOKEN balance must increase (we bought TOKEN)
    require!(
        post_swap_ore_balance > pre_swap_ore_balance,
        AppError::InvalidSwapState
    );

    // 4. All SOL must be used in the swap (balance should be 0)
    require!(
        post_swap_sol_balance == 0,
        AppError::InvalidSwapState
    );

    // Calculate acquired TOKEN
    let total_ore_acquired = post_swap_ore_balance
        .checked_sub(pre_swap_ore_balance)
        .ok_or(AppError::Overflow)?;

    msg!("Total TOKEN acquired from swap: {}", total_ore_acquired);

    // Since staking is removed, burn 100% of acquired TOKEN
    let burn_amount = total_ore_acquired;

    msg!("Burning {} TOKEN tokens...", burn_amount);

    // Burn TOKEN tokens
    let burn_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        Burn {
            mint: ctx.accounts.mint.to_account_info(),
            from: ctx.accounts.treasury_ore_tokens.to_account_info(),
            authority: ctx.accounts.treasury.to_account_info(),
        },
        signer_seeds,
    );
    token::burn(burn_ctx, burn_amount)?;

    // Reload mint to get new supply
    ctx.accounts.mint.reload()?;
    let new_circulating_supply = ctx.accounts.mint.supply;

    msg!("Bury completed successfully!");
    msg!("  Swapped: {} SOL", pre_swap_sol_balance);
    msg!("  Acquired: {} TOKEN", total_ore_acquired);
    msg!("  Burned: {} TOKEN (100%)", burn_amount);
    msg!("  New circulating supply: {}", new_circulating_supply);
    msg!("  Supply reduced by: {}", pre_swap_mint_supply - new_circulating_supply);

    // Emit BuryEvent for indexing
    emit!(BuryEvent {
        token_burned: burn_amount,
        token_shared: 0, // No staking in Anchor version
        sol_amount: pre_swap_sol_balance,
        new_circulating_supply,
        timestamp: Clock::get()?.unix_timestamp,
    });

    // Note: In Steel version, 10% was shared with stakers
    // Since staking is removed, we burn 100% for maximum deflation

    Ok(())
}
