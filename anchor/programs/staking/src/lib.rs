#![allow(clippy::result_large_err)]

use anchor_lang::prelude::*;

declare_id!("FqzkXZdwYjurnUKetJCAvaUw5WAqbwzU6gZEwydeEfqS");

pub const ANCHOR_DISCRIMINATOR_SPACE: usize = 8;

const POINTS_PER_SOL_PER_DAY: u64 = 1_000_000;
const LAMPORTS_PER_SOL: u64 = 1000_000_000;
const SECONDS_PER_DAY: u64 = 86400;

#[program]
pub mod staking {
    use anchor_lang::system_program;

    use super::*;

    pub fn create_pda_account(ctx: Context<CreatePdaAccount>) -> Result<()> {
        let pda_account = &mut ctx.accounts.pda_account;
        let clock = Clock::get()?;

        pda_account.owner = ctx.accounts.payer.key();
        pda_account.staked_amount = 0;
        pda_account.total_points = 0;
        pda_account.last_updated_time = clock.unix_timestamp;
        pda_account.bump = ctx.bumps.pda_account;

        Ok(())
    }

    pub fn stake_fn(ctx: Context<Stake>, amount: u64) -> Result<()> {
        require!(amount > 0, Error::InvalidAmount);

        let pda_account = &mut ctx.accounts.pda_account;
        let clock = Clock::get()?;

        update_points(pda_account, clock.unix_timestamp as u64);

        //user => PDA transfer
        let cpi_context = CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            system_program::Transfer {
                from: ctx.accounts.user.to_account_info(),
                to: ctx.accounts.pda_account.to_account_info(),
            },
        );
        system_program::transfer(cpi_context, amount)?;

        pda_account.staked_amount = pda_account
            .staked_amount
            .checked_add(amount)
            .ok_or(Error::Overflow)?;

        msg!(
            "Staked Lamports are: {}, Total Staked Lamports are: {}, Total earned points are: {}",
            amount,
            pda_account.staked_amount,
            pda_account.total_points / POINTS_PER_SOL_PER_DAY
        );

        Ok(())
    }

    pub fn unstake_fn(ctx: Context<Unstake>, amount: u64) -> Result<()> {
        require!(amount > 0, Error::InvalidAmount);

        let pda_account = &mut ctx.accounts.pda_account;
        let clock = Clock::get()?;

        require!(
            pda_account.staked_amount >= amount,
            Error::InsufficientStake
        );

        update_points(pda_account, clock.unix_timestamp as u64)?;

        //PDA => user
        let seeds = &[
            b"client",
            ctx.accounts.user.key().as_ref(),
            &[pda_account.bump],
        ];

        let signer = &[&seeds[..]];

        let cpi_context = CpiContext::new_with_signer(
            ctx.accounts.system_program.to_account_info(),
            system_program::Transfer {
                from: ctx.accounts.pda_account.to_account_info(),
                to: ctx.accounts.user.to_account_info(),
            },
            signer,
        );
        system_program::transfer(cpi_context, amount)?;

        pda_account.staked_amount = pda_account
            .staked_amount
            .checked_sub(amount)
            .ok_or(Error::Underflow)?;

        msg!(
            "Unstaked Lamports are: {}, Remaining Staked Lamports are: {}, Total earned points are: {}",
            amount,
            pda_account.staked_amount,
            pda_account.total_points / POINTS_PER_SOL_PER_DAY
        );

        Ok(())
    }

    pub fn claim_points_fn(ctx: Context<CreatePdaAccount>) -> Result<()> {
        Ok(())
    }

    pub fn get_points_fn(ctx: Context<CreatePdaAccount>) -> Result<()> {
        Ok(())
    }
}

//functions
fn update_points(pda_account: &mut StakeAccount, current_time: u64) -> Result<()> {
    let start_time = current_time
        .checked_sub(pda_account.last_updated_time as u64)
        .ok_or(Error::InvalidTimestamp)?;

    if start_time > 0 && pda_account.staked_amount > 0 {
        let new_points = calc_earned_points(pda_account.staked_amount, start_time)?;
        pda_account.total_points = pda_account
            .total_points
            .checked_add(new_points)
            .ok_or(Error::Overflow)?;
    }
    pda_account.last_updated_time = current_time as i64;

    Ok(())
}

fn calc_earned_points(staked_amount: u64, start_time: u64) -> Result<u64> {
    let points = (staked_amount as u128)
        .checked_mul(start_time as u128)
        .ok_or(Error::Overflow)?
        .checked_mul(POINTS_PER_SOL_PER_DAY as u128)
        .ok_or(Error::Overflow)?
        .checked_div(LAMPORTS_PER_SOL as u128)
        .ok_or(Error::Overflow)?
        .checked_div(SECONDS_PER_DAY as u128)
        .ok_or(Error::Overflow)?;

    Ok(points as u64)
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

#[derive(Accounts)]
pub struct Unstake<'info> {
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
    pub staked_amount: u64,
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
