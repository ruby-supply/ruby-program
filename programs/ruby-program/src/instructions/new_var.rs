use anchor_lang::prelude::*;
use solana_program::program::invoke_signed;
use crate::constants::*;
use crate::errors::AppError;
use crate::state::*;

#[derive(Accounts)]
#[instruction(args: NewVarArgs)]
pub struct NewVar<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        mut,
        seeds = [CONFIG],
        bump,
    )]
    pub config: Account<'info, Config>,

    #[account(
        seeds = [BOARD],
        bump,
    )]
    pub board: Account<'info, Board>,

    /// CHECK: Provider account that commits the hash
    pub provider: AccountInfo<'info>,

    /// CHECK: Entropy var account (PDA of entropy program)
    #[account(mut)]
    pub var_account: AccountInfo<'info>,

    /// CHECK: Entropy program
    pub entropy_program: AccountInfo<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct NewVarArgs {
    pub id: u64,
    pub commit: [u8; 32],
    pub samples: u64,
    pub end_at: u64,
}

pub fn handler(ctx: Context<NewVar>, args: NewVarArgs) -> Result<()> {
    let clock = Clock::get()?;

    // Validate admin authorization
    require!(
        ctx.accounts.config.admin == ctx.accounts.signer.key(),
        AppError::NotAuthorized
    );

    // Default end_at to board.end_slot if 0, otherwise use provided value
    let end_at = if args.end_at == 0 {
        ctx.accounts.board.end_slot
    } else {
        args.end_at
    };

    // Validate end_at is in the future
    require!(
        end_at > clock.slot,
        AppError::InvalidEndSlot
    );

    // Build Open instruction data for Entropy program
    // Format: [discriminator: u8, id: u64, commit: [u8;32], is_auto: u64, samples: u64, end_at: u64]
    let mut ix_data = Vec::with_capacity(1 + 8 + 32 + 8 + 8 + 8);
    ix_data.push(0u8); // Open instruction discriminator
    ix_data.extend_from_slice(&args.id.to_le_bytes());
    ix_data.extend_from_slice(&args.commit);
    ix_data.extend_from_slice(&0u64.to_le_bytes()); // is_auto = 0 (manual)
    ix_data.extend_from_slice(&args.samples.to_le_bytes());
    ix_data.extend_from_slice(&end_at.to_le_bytes());

    // Create Entropy Open instruction
    let open_ix = solana_program::instruction::Instruction {
        program_id: ctx.accounts.entropy_program.key(),
        accounts: vec![
            solana_program::instruction::AccountMeta::new_readonly(ctx.accounts.board.key(), true), // authority (Board PDA signer)
            solana_program::instruction::AccountMeta::new(ctx.accounts.signer.key(), true), // payer
            solana_program::instruction::AccountMeta::new_readonly(ctx.accounts.provider.key(), false), // provider
            solana_program::instruction::AccountMeta::new(ctx.accounts.var_account.key(), false), // var
            solana_program::instruction::AccountMeta::new_readonly(ctx.accounts.system_program.key(), false), // system_program
        ],
        data: ix_data,
    };

    // Get Board PDA seeds for signing (Board is the authority for entropy var)
    let board_bump = ctx.bumps.board;
    let signer_seeds: &[&[&[u8]]] = &[&[
        BOARD,
        &[board_bump],
    ]];

    // Invoke Entropy Open instruction
    invoke_signed(
        &open_ix,
        &[
            ctx.accounts.board.to_account_info(),
            ctx.accounts.signer.to_account_info(),
            ctx.accounts.provider.to_account_info(),
            ctx.accounts.var_account.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
        ],
        signer_seeds,
    )?;

    // Update config with the new var address
    ctx.accounts.config.var_address = ctx.accounts.var_account.key();

    Ok(())
}
