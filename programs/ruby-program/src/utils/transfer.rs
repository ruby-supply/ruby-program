use anchor_lang::prelude::*;
use anchor_lang::system_program::{self, Transfer};
use crate::errors::AppError;

/// Transfer SOL between accounts using direct lamport manipulation.
/// Use when source or destination is a PDA with state data.
pub fn transfer_lamports<'info>(
    from: &AccountInfo<'info>,
    to: &AccountInfo<'info>,
    amount: u64,
) -> Result<()> {
    **from.try_borrow_mut_lamports()? -= amount;
    **to.try_borrow_mut_lamports()? += amount;
    Ok(())
}

/// Transfer SOL with rent-exempt validation.
/// Ensures the source account maintains minimum rent-exempt balance after transfer.
pub fn transfer_lamports_safe<'info>(
    from: &AccountInfo<'info>,
    to: &AccountInfo<'info>,
    amount: u64,
) -> Result<()> {
    let rent = Rent::get()?;
    let min_balance = rent.minimum_balance(from.data_len());

    let required = amount
        .checked_add(min_balance)
        .ok_or(AppError::Overflow)?;

    require!(
        from.lamports() >= required,
        AppError::InsufficientBalance
    );

    transfer_lamports(from, to, amount)
}

/// Transfer SOL from a signer using Anchor System Program CPI.
/// Use when the source is a regular signer account (not a PDA with data).
pub fn transfer_sol_cpi<'info>(
    from: AccountInfo<'info>,
    to: AccountInfo<'info>,
    system_program: AccountInfo<'info>,
    amount: u64,
) -> Result<()> {
    let cpi_ctx = CpiContext::new(system_program, Transfer { from, to });
    system_program::transfer(cpi_ctx, amount)
}

/// Transfer SOL from a PDA using Anchor System Program CPI with signer seeds.
/// Use when the source is a PDA without data that needs to sign the transfer.
pub fn transfer_sol_cpi_signed<'info>(
    from: AccountInfo<'info>,
    to: AccountInfo<'info>,
    system_program: AccountInfo<'info>,
    amount: u64,
    signer_seeds: &[&[&[u8]]],
) -> Result<()> {
    let cpi_ctx = CpiContext::new_with_signer(system_program, Transfer { from, to }, signer_seeds);
    system_program::transfer(cpi_ctx, amount)
}
