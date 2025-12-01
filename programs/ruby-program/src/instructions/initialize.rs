use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token};
use crate::constants::*;
use crate::state::*;

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        init,
        payer = signer,
        space = Config::LEN,
        seeds = [CONFIG],
        bump,
    )]
    pub config: Account<'info, Config>,

    #[account(
        init,
        payer = signer,
        space = Board::LEN,
        seeds = [BOARD],
        bump,
    )]
    pub board: Account<'info, Board>,

    #[account(
        init,
        payer = signer,
        space = Treasury::LEN,
        seeds = [TREASURY],
        bump,
    )]
    pub treasury: Account<'info, Treasury>,

    /// The RUBY token mint
    pub mint: Account<'info, Mint>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}

pub fn handler(ctx: Context<Initialize>) -> Result<()> {
    let clock = Clock::get()?;

    // Initialize config
    let config = &mut ctx.accounts.config;
    config.admin = ctx.accounts.signer.key();
    config.bury_authority = ctx.accounts.signer.key();
    config.fee_collector = ctx.accounts.signer.key();
    config.swap_program = Pubkey::default();
    config.var_address = Pubkey::default();
    config.buffer = 0;

    // Initialize board
    let board = &mut ctx.accounts.board;
    board.round_id = 0;
    board.start_slot = u64::MAX;
    board.end_slot = u64::MAX;
    board.mining_start_slot = clock.slot; // Set mining start for emission schedule
    board.motherlode_ruby = 0;
    board.mini_motherlode_sol = 0;
    board.total_minted = 0;

    // Initialize treasury
    let treasury = &mut ctx.accounts.treasury;
    treasury.buyback_bl = 0;
    treasury.reserve_bl = 0;
    treasury.motherlode_sol_bl = 0;
    treasury.miner_rewards_factor = Default::default();
    treasury.stake_rewards_factor = Default::default();
    treasury.total_staked = 0;
    treasury.total_unclaimed = 0;
    treasury.total_refined = 0;
    treasury.leaderboard_bl = 0;

    Ok(())
}
