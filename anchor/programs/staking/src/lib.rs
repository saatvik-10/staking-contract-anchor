#![allow(clippy::result_large_err)]

use anchor_lang::{prelude::*, solana_program::stake::instruction::StakeError};

declare_id!("FqzkXZdwYjurnUKetJCAvaUw5WAqbwzU6gZEwydeEfqS");

pub const ANCHOR_DISCRIMINATOR_SPACE: usize = 8;

const POINTS_PER_SOL_PER_DAY: u64 = 1_000_000;
const LAMPORTS_PER_SOL: u64 = 1000_000_000;
const SECONDS_PER_DAY: u64 = 86400;

#[program]
pub mod staking {
    use super::*;

    pub fn create_pda_account(ctx: Context<CreatePdaAccount>) -> Result<()> {
        let pda_account = &mut ctx.accounts.pda_account;
        let clock = Clock::get()?;

        pda_account.owner = ctx.accounts.payer.key();
        pda_account.staked_account = 0;
        pda_account.total_points = 0;
        pda_account.last_updated_time = clock.unix_timestamp;
        pda_account.bump = ctx.bumps.pda_account;

        Ok(())
    }

    pub fn stake_fn(ctx: Context<Stake>, amount: u64) -> Result<()> {
        require!(amount > 0, Error::InvalidAmount);

        let pda_account = &mut ctx.accounts.pda_account;
        let clock = Clock::get()?;

        Ok(())
    }

    pub fn unstake_fn(ctx: Context<CreatePdaAccount>, amount: u64) -> Result<()> {
        Ok(())
    }

    pub fn claim_points_fn(ctx: Context<CreatePdaAccount>) -> Result<()> {
        Ok(())
    }

    pub fn get_points_fn(ctx: Context<CreatePdaAccount>) -> Result<()> {
        Ok(())
    }
}

//accounts
#[derive(Accounts)]
pub struct CreatePdaAccount<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        init,
        payer = payer,
        space = ANCHOR_DISCRIMINATOR_SPACE + StakeAccount::INIT_SPACE,
        seeds = [b"client", payer.key().as_ref()],
        bump
    )]
    pub pda_account: Account<'info, StakeAccount>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Stake<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        seeds = [b"client", user.key().as_ref()],
        bump,
        constraint = pda_account.owner == user.key() @ Error::Unauthorized
    )]
    pub pda_account: Account<'info, StakeAccount>,

    pub system_program: Program<'info, System>,
}

//structs
#[account]
#[derive(InitSpace)]
pub struct StakeAccount {
    pub owner: Pubkey,
    pub staked_account: u64,
    pub total_points: u64,
    pub last_updated_time: i64,
    pub bump: u8,
}

#[error_code]
pub enum Error {
    #[msg("Amount must be greater than 0")]
    InvalidAmount,
    #[msg("Insufficient staked amount")]
    InsufficientStake,
    #[msg("Unauthorized access")]
    Unauthorized,
    #[msg("Arithmetic overflow")]
    Overflow,
    #[msg("Arithmetic underflow")]
    Underflow,
    #[msg("Invalid timestamp")]
    InvalidTimestamp,
}
