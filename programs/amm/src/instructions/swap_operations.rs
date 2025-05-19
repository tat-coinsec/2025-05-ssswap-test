use anchor_lang::prelude::*;
use anchor_spl::
{
    token_interface::{Mint, TokenAccount, TokenInterface, transfer_checked, TransferChecked},
    associated_token::AssociatedToken,
};

use crate::LiquidityPool;
use crate::transfer_tokens;
use crate::error::AmmError;
use num_integer::Integer;

#[derive(Accounts)]
pub struct SwapContext<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mint::token_program = token_program,
    )]
    pub token_a: InterfaceAccount<'info, Mint>,

    #[account(
        mint::token_program = token_program,
    )]
    pub token_b: InterfaceAccount<'info, Mint>,

    #[account(
        init_if_needed,
        payer = user,
        associated_token::mint = token_a,
        associated_token::authority = user,
        associated_token::token_program = token_program,
    )]
    pub user_token_a: InterfaceAccount<'info, TokenAccount>,

    #[account(
        init_if_needed,
        payer = user,
        associated_token::mint = token_b,
        associated_token::authority = user,
        associated_token::token_program = token_program,
    )]
    pub user_token_b: InterfaceAccount<'info, TokenAccount>,

    #[account(
        seeds = [b"pool", token_a.key().as_ref(), token_b.key().as_ref()],
        bump
    )]
    pub liquidity_pool: Account<'info, LiquidityPool>,

    #[account(
        mut,
        associated_token::mint = token_a,
        associated_token::authority = liquidity_pool,
        associated_token::token_program = token_program
    )]
    pub token_vault_a: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut, 
        associated_token::mint = token_b,
        associated_token::authority = liquidity_pool,
        associated_token::token_program = token_program,
    )]
    pub token_vault_b: InterfaceAccount<'info, TokenAccount>,

    pub system_program: Program<'info, System>,
    pub token_program: Interface<'info, TokenInterface>, 
    pub associated_token_program: Program<'info, AssociatedToken>,
}

pub fn swap_exact_in(context: Context<SwapContext>, amount_in: u64, min_out: u64, zero_for_one: bool) -> Result<()> {
    context.accounts.token_vault_a.reload()?;
    context.accounts.token_vault_b.reload()?;
    let reserve_a: u64 = context.accounts.token_vault_a.amount;
    let reserve_b: u64 = context.accounts.token_vault_b.amount;

    if reserve_a == 0 || reserve_b == 0 {
        return err!(AmmError::PoolIsEmpty);
    }
    if amount_in == 0 {
        return err!(AmmError::NoZeroAmount);
    }

    if zero_for_one {
        let numerator: u128 = (reserve_b as u128).checked_mul(amount_in as u128).ok_or(AmmError::Overflow)?;
        let denominator: u128 = (reserve_a as u128).checked_add(amount_in as u128).ok_or(AmmError::Overflow)?;
        if denominator == 0 {
            return err!(AmmError::DivisionByZero);
        }

        let mut amount_out: u64 = numerator.div_floor(&denominator) as u64;

        let lp_fees = (amount_out as u128 * 3).div_floor(&1000) as u64;

        amount_out = amount_out - lp_fees;

        if amount_out == 0 || amount_out < min_out {
            return err!(AmmError::Slippage);
        }

        transfer_tokens(
            &context.accounts.user_token_a,
            &mut context.accounts.token_vault_a,
            &amount_in,
            &context.accounts.token_a,
            &context.accounts.user,
            &context.accounts.token_program,
        )?;

        let token_a_key = context.accounts.token_a.key();
        let token_b_key = context.accounts.token_b.key();
        let bump_seed = [context.bumps.liquidity_pool];

        let signer_seeds: &[&[&[u8]]] = &[&[
            b"pool",
            token_a_key.as_ref(),
            token_b_key.as_ref(),
            &bump_seed,
        ]];

        let cpi_accounts_transfer_out = TransferChecked {
            from: context.accounts.token_vault_b.to_account_info(),
            mint: context.accounts.token_b.to_account_info(),
            to: context.accounts.user_token_b.to_account_info(),
            authority: context.accounts.liquidity_pool.to_account_info(),
        };
        let cpi_context_transfer_out = CpiContext::new_with_signer(
            context.accounts.token_program.to_account_info(),
            cpi_accounts_transfer_out,
            signer_seeds,
        );
        transfer_checked(cpi_context_transfer_out, amount_out, context.accounts.token_b.decimals)?
    } else {
        let numerator: u128 = (reserve_a as u128).checked_mul(amount_in as u128).ok_or(AmmError::Overflow)?;
        let denominator: u128 = (reserve_b as u128).checked_add(amount_in as u128).ok_or(AmmError::Overflow)?;
        if denominator == 0 {
            return err!(AmmError::DivisionByZero);
        }
        let mut amount_out: u64 = numerator.div_floor(&denominator) as u64;

        let lp_fees = (amount_out as u128 * 3).div_floor(&1000) as u64;

        amount_out = amount_out - lp_fees;

        if amount_out == 0 || amount_out < min_out {
            return err!(AmmError::Slippage);
        }

        transfer_tokens(
            &context.accounts.user_token_b,
            &mut context.accounts.token_vault_b,
            &amount_in,
            &context.accounts.token_b,
            &context.accounts.user,
            &context.accounts.token_program,
        )?;

        let token_a_key = context.accounts.token_a.key();
        let token_b_key = context.accounts.token_b.key();
        let bump_seed = [context.bumps.liquidity_pool];

        let signer_seeds: &[&[&[u8]]] = &[&[
            b"pool",
            token_a_key.as_ref(),
            token_b_key.as_ref(),
            &bump_seed,
        ]];

        let cpi_accounts_transfer_out = TransferChecked {
            from: context.accounts.token_vault_a.to_account_info(),
            mint: context.accounts.token_a.to_account_info(),
            to: context.accounts.user_token_a.to_account_info(),
            authority: context.accounts.liquidity_pool.to_account_info(),
        };
        let cpi_context_transfer_out = CpiContext::new_with_signer(
            context.accounts.token_program.to_account_info(),
            cpi_accounts_transfer_out,
            signer_seeds,
        );
        transfer_checked(cpi_context_transfer_out, amount_out, context.accounts.token_a.decimals)?
    }

    Ok(())
}

pub fn swap_exact_out(context: Context<SwapContext>, amount_out: u64, max_in: u64, zero_for_one: bool) -> Result<()> {
    context.accounts.token_vault_a.reload()?;
    context.accounts.token_vault_b.reload()?;
    let reserve_a: u64 = context.accounts.token_vault_a.amount;
    let reserve_b: u64 = context.accounts.token_vault_b.amount;

    require!(reserve_a > 0 && reserve_b > 0, AmmError::PoolIsEmpty);
    require!(amount_out > 0, AmmError::NoZeroAmount);
    
    if zero_for_one {
        require!(amount_out < reserve_b, AmmError::InsufficientLiquidity);

        let numerator: u128 = (reserve_a as u128)
            .checked_mul(amount_out as u128)
            .ok_or(AmmError::Overflow)?;
        let denominator: u128 = (reserve_b as u128)
            .checked_sub(amount_out as u128)
            .ok_or(AmmError::Underflow)?;

        require!(denominator > 0, AmmError::DivisionByZero); 

        let amount_in_no_fee: u64 = numerator.div_floor(&denominator) as u64;

        let fee_numerator_u128 = (amount_in_no_fee as u128)
            .checked_mul(3)
            .ok_or(AmmError::Overflow)?;
        let lp_fees: u64 = fee_numerator_u128.div_floor(&1000) as u64;
        
        let amount_in_final = (amount_in_no_fee as u128)
            .checked_add(lp_fees as u128)
            .ok_or(AmmError::Overflow)? as u64;

        require!(amount_in_final > 0 || (amount_in_final == 0 && amount_out == 0), AmmError::CalculationFailure);
        require!(amount_in_final <= max_in, AmmError::Slippage);

        transfer_tokens(
            &context.accounts.user_token_a,
            &mut context.accounts.token_vault_a,
            &amount_in_final,
            &context.accounts.token_a,
            &context.accounts.user,
            &context.accounts.token_program,
        )?;

        let token_a_key = context.accounts.token_a.key();
        let token_b_key = context.accounts.token_b.key();
        let bump_seed = [context.bumps.liquidity_pool];

        let signer_seeds: &[&[&[u8]]] = &[&[
            b"pool",
            token_a_key.as_ref(),
            token_b_key.as_ref(),
            &bump_seed,
        ]];

        let cpi_accounts_transfer_out = TransferChecked {
            from: context.accounts.token_vault_b.to_account_info(),
            mint: context.accounts.token_b.to_account_info(),
            to: context.accounts.user_token_b.to_account_info(),
            authority: context.accounts.liquidity_pool.to_account_info(),
        };
        let cpi_context_transfer_out = CpiContext::new_with_signer(
            context.accounts.token_program.to_account_info(),
            cpi_accounts_transfer_out,
            signer_seeds,
        );
        transfer_checked(cpi_context_transfer_out, amount_out, context.accounts.token_b.decimals)?
    } else {
        require!(amount_out < reserve_a, AmmError::InsufficientLiquidity);

        let numerator: u128 = (reserve_b as u128)
            .checked_mul(amount_out as u128)
            .ok_or(AmmError::Overflow)?;
        let denominator: u128 = (reserve_a as u128)
            .checked_sub(amount_out as u128)
            .ok_or(AmmError::Underflow)?;

        require!(denominator > 0, AmmError::DivisionByZero);

        let amount_in_no_fee: u64 = numerator.div_floor(&denominator) as u64;

        let fee_numerator_u128 = (amount_in_no_fee as u128)
            .checked_mul(3)
            .ok_or(AmmError::Overflow)?;
        let lp_fees: u64 = fee_numerator_u128.div_floor(&1000) as u64;
        
        let amount_in_final = (amount_in_no_fee as u128)
            .checked_add(lp_fees as u128)
            .ok_or(AmmError::Overflow)? as u64;

        require!(amount_in_final > 0 || (amount_in_final == 0 && amount_out == 0), AmmError::CalculationFailure);
        require!(amount_in_final <= max_in, AmmError::Slippage);

        transfer_tokens(
            &context.accounts.user_token_b,
            &mut context.accounts.token_vault_b,
            &amount_in_final,
            &context.accounts.token_b,
            &context.accounts.user,
            &context.accounts.token_program,
        )?;

        let token_a_key = context.accounts.token_a.key();
        let token_b_key = context.accounts.token_b.key();
        let bump_seed = [context.bumps.liquidity_pool];

        let signer_seeds: &[&[&[u8]]] = &[&[
            b"pool",
            token_a_key.as_ref(),
            token_b_key.as_ref(),
            &bump_seed,
        ]];

        let cpi_accounts_transfer_out = TransferChecked {
            from: context.accounts.token_vault_a.to_account_info(),
            mint: context.accounts.token_a.to_account_info(),
            to: context.accounts.user_token_a.to_account_info(),
            authority: context.accounts.liquidity_pool.to_account_info(),
        };
        let cpi_context_transfer_out = CpiContext::new_with_signer(
            context.accounts.token_program.to_account_info(),
            cpi_accounts_transfer_out,
            signer_seeds,
        );
        transfer_checked(cpi_context_transfer_out, amount_out, context.accounts.token_a.decimals)?
    }
    Ok(())
}