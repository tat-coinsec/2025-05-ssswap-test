use anchor_lang::prelude::*;
use anchor_spl::
{
    token_interface::{mint_to, MintTo, Mint, TokenAccount, TokenInterface, Burn, TransferChecked, burn, transfer_checked},
    associated_token::AssociatedToken,
};

use crate::{LiquidityPool, ANCHOR_DISCRIMINATOR, LP_TOKEN_DECIMALS, transfer_tokens};
use crate::error::AmmError;
use num_integer::Roots;

/// InitializePool and ModifyLiquidity could have been merged into 1 struct for the liquidity provision
/// however, since the target is to NOT have duplicate pools I wanted an additional layer of safety
/// using init for the creation and not init_if_needed
#[derive(Accounts)]
#[instruction(id: u64)]
pub struct InitializePool<'info> {
    #[account(mut)]
    pub creator: Signer<'info>,

    #[account(
        mut,
        associated_token::mint = token_mint_a,
        associated_token::authority = creator
    )]
    pub creator_token_account_a: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = token_mint_b,
        associated_token::authority = creator
    )]
    pub creator_token_account_b: InterfaceAccount<'info, TokenAccount>,

    #[account(
        init,
        payer = creator,
        seeds = [b"lp_mint", liquidity_pool.key().as_ref()],
        bump,
        mint::decimals = LP_TOKEN_DECIMALS,
        mint::authority = liquidity_pool,
        mint::token_program = token_program // Specify the token program
    )]
    pub lp_mint: InterfaceAccount<'info, Mint>,

    #[account(
        init_if_needed,
        payer = creator,
        associated_token::mint = lp_mint,
        associated_token::authority = creator,
    )]
    pub creator_lp_token_account: InterfaceAccount<'info, TokenAccount>,

    #[account(mint::token_program = token_program)]
    pub token_mint_a: InterfaceAccount<'info, Mint>,

    #[account(mint::token_program = token_program)]
    pub token_mint_b: InterfaceAccount<'info, Mint>,

    #[account(
        init,
        payer = creator,
        space = ANCHOR_DISCRIMINATOR + LiquidityPool::INIT_SPACE,
        seeds = [b"pool", token_mint_a.key().as_ref(), token_mint_b.key().as_ref()],
        bump
    )]
    pub liquidity_pool: Account<'info, LiquidityPool>,

    #[account(
        init,
        payer = creator,
        associated_token::mint = token_mint_a,
        associated_token::authority = liquidity_pool,
        associated_token::token_program = token_program
    )]
    pub vault_a: InterfaceAccount<'info, TokenAccount>,

    #[account(
        init,
        payer = creator,
        associated_token::mint = token_mint_b,
        associated_token::authority = liquidity_pool,
        associated_token::token_program = token_program
    )]
    pub vault_b: InterfaceAccount<'info, TokenAccount>,

    pub system_program: Program<'info, System>,
    pub token_program: Interface<'info, TokenInterface>, 
    pub associated_token_program: Program<'info, AssociatedToken>,
}

#[derive(Accounts)]
pub struct ModifyLiquidity<'info> {
    #[account(mut)]
    pub liquidity_provider: Signer<'info>,

    #[account(mint::token_program = token_program)]
    token_a_mint: InterfaceAccount<'info, Mint>,

    #[account(mint::token_program = token_program)]
    token_b_mint: InterfaceAccount<'info, Mint>,

    #[account(
        mut,
        associated_token::mint = token_a_mint,
        associated_token::authority = liquidity_provider,
    )]
    pub liquidity_provider_token_a: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = token_b_mint,
        associated_token::authority = liquidity_provider,
    )]
    pub liquidity_provider_token_b: InterfaceAccount<'info, TokenAccount>,

    #[account(
        seeds = [b"pool", token_a_mint.key().as_ref(), token_b_mint.key().as_ref()],
        bump
    )]
    pub liquidity_pool: Account<'info, LiquidityPool>,

    #[account(
        mut,
        seeds = [b"lp_mint", liquidity_pool.key().as_ref()],
        bump,
        mint::decimals = LP_TOKEN_DECIMALS,
        mint::authority = liquidity_pool,
        mint::token_program = token_program
    )]
    lp_mint: InterfaceAccount<'info, Mint>,

    #[account(
        init_if_needed,
        payer = liquidity_provider,
        associated_token::mint = lp_mint,
        associated_token::authority = liquidity_provider,
    )]
    pub liquidity_provider_lp_account: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = token_a_mint,
        associated_token::authority = liquidity_pool,
        associated_token::token_program = token_program
    )]
    pub vault_a: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = token_b_mint,
        associated_token::authority = liquidity_pool,
        associated_token::token_program = token_program
    )]
    pub vault_b: InterfaceAccount<'info, TokenAccount>,

    pub system_program: Program<'info, System>,
    pub token_program: Interface<'info, TokenInterface>, 
    pub associated_token_program: Program<'info, AssociatedToken>,
}

pub fn initialize_pool(context: Context<InitializePool>, amount_token_a: u64, amount_token_b: u64) -> Result<()> {
    // Enforce canonical order of mints, to ensure every pair will get initialized only once, inspired by Uniswap
    require!(
        context.accounts.token_mint_a.key() != context.accounts.token_mint_b.key(),
        AmmError::IdenticalTokens 
    );
    require!(
        context.accounts.token_mint_a.key() < context.accounts.token_mint_b.key(),
        AmmError::InvalidTokenOrder 
    );

    // Since the initial liquidity makes the initial price, we enforce that neither of the amounts is 0
    // so an actual price can be derived. If the price is "unrealistic" I expect arbitrage to take care 
    // of corrections relatively quick, so an effect for following users after the initial provision is negligable.
    require!(
        amount_token_a > 0 && amount_token_b > 0,  AmmError::NoZeroProvision
    );
    

    context.accounts.liquidity_pool.set_inner( LiquidityPool { 
        token_a: context.accounts.token_mint_a.key(), 
        token_b: context.accounts.token_mint_b.key(), 
        lp_mint: context.accounts.lp_mint.key(), 
        bump: context.bumps.liquidity_pool,
    });

    transfer_tokens(
        &context.accounts.creator_token_account_a,
        &context.accounts.vault_a,
        &amount_token_a,
        &context.accounts.token_mint_a,
        &context.accounts.creator,
        &context.accounts.token_program,
    )?;

    transfer_tokens(
        &context.accounts.creator_token_account_b,
        &context.accounts.vault_b,
        &amount_token_b,
        &context.accounts.token_mint_b,
        &context.accounts.creator,
        &context.accounts.token_program,
    )?;

    // --- Calculate and Mint Initial LP Tokens --- //

    // Call the calculation function (assuming it returns Result<u64>)
    let lp_amount_to_mint: u64 = liquidity_calculation(amount_token_a, amount_token_b)?;

    // A pool opened with a dust-sized LP supply lets the first depositor
    // inflate the share price: require a minimum initial mint, as Uniswap v2
    // does with MINIMUM_LIQUIDITY.
    require!(lp_amount_to_mint >= 1000, AmmError::LpAmountCalculation);

    // Create the bump slice variable first
    let bump_seed = [context.bumps.liquidity_pool]; // Store the bump in a separate variable
    let token_a_key = context.accounts.token_mint_a.key();
    let token_b_key = context.accounts.token_mint_b.key();

    // Define PDA signer seeds using the variable
    let signer_seeds: &[&[&[u8]]] = &[&[
        b"pool",
        token_a_key.as_ref(),
        token_b_key.as_ref(),
        &bump_seed, // Use the reference to the variable
    ]];

    // CPI Context for minting LP tokens
    let cpi_accounts = MintTo {
        mint: context.accounts.lp_mint.to_account_info(),
        to: context.accounts.creator_lp_token_account.to_account_info(),
        authority: context.accounts.liquidity_pool.to_account_info(), // Pool PDA is mint authority
    };
    let cpi_context = CpiContext::new_with_signer(
        context.accounts.token_program.to_account_info(),
        cpi_accounts,
        signer_seeds, // Pass the signer seeds
    );

    // Execute the mint_to CPI
    mint_to(cpi_context, lp_amount_to_mint)?;

    Ok(())
}

pub fn provide_liquidity(context: Context<ModifyLiquidity>, 
    amount_a: u64
) -> Result<()> {
    let amount_b = calculate_token_b_provision_with_a_given(
        &mut context.accounts.vault_a,
        &mut context.accounts.vault_b,
        amount_a
    )?;

    transfer_tokens(
        &context.accounts.liquidity_provider_token_a, 
        &mut context.accounts.vault_a,
        &amount_a, 
        &context.accounts.token_a_mint, 
        &context.accounts.liquidity_provider, 
        &context.accounts.token_program
    )?;

    transfer_tokens(
        &context.accounts.liquidity_provider_token_b, 
        &mut context.accounts.vault_b,
        &amount_b, 
        &context.accounts.token_b_mint, 
        &context.accounts.liquidity_provider, 
        &context.accounts.token_program
    )?;

    let lp_to_mint: u64 = liquidity_calculation(amount_a, amount_b)?;

    let bump_seed = [context.bumps.liquidity_pool]; 
    let token_a_key = context.accounts.token_a_mint.key();
    let token_b_key = context.accounts.token_b_mint.key();

    let signer_seeds: &[&[&[u8]]] = &[&[
        b"pool",
        token_a_key.as_ref(),
        token_b_key.as_ref(),
        &bump_seed, 
    ]];

    let cpi_accounts = MintTo {
        mint: context.accounts.lp_mint.to_account_info(),
        to: context.accounts.liquidity_provider_lp_account.to_account_info(),
        authority: context.accounts.liquidity_pool.to_account_info(),
    };
    let cpi_context = CpiContext::new_with_signer(
        context.accounts.token_program.to_account_info(),
        cpi_accounts,
        signer_seeds, 
    );

    mint_to(cpi_context, lp_to_mint)?;

    Ok(())
}

pub fn remove_liquidity(context: Context<ModifyLiquidity>, lpt_to_redeem: u64) -> Result<()> {
    require!(lpt_to_redeem > 0, AmmError::NoZeroRedemption);

    context.accounts.liquidity_provider_lp_account.reload()?;
    context.accounts.lp_mint.reload()?;
    context.accounts.vault_a.reload()?;
    context.accounts.vault_b.reload()?;

    let user_lp_balance = context.accounts.liquidity_provider_lp_account.amount;
    let total_lp_supply = context.accounts.lp_mint.supply;
    let reserve_a = context.accounts.vault_a.amount;
    let reserve_b = context.accounts.vault_b.amount;
    
    require!(
        user_lp_balance >= lpt_to_redeem,
        AmmError::AskExceedsBalance
    );

    if total_lp_supply == 0 {
        return err!(AmmError::PoolIsEmpty);
    }
    
    let amount_a_to_return_u128 = (lpt_to_redeem as u128)
        .checked_mul(reserve_a as u128)
        .ok_or(AmmError::Overflow)?
        .checked_div(total_lp_supply as u128)
        .ok_or(AmmError::DivisionByZero)?;

    let amount_a_to_return = amount_a_to_return_u128 as u64;

    let amount_b_to_return_u128 = (lpt_to_redeem as u128)
        .checked_mul(reserve_b as u128)
        .ok_or(AmmError::Overflow)?
        .checked_div(total_lp_supply as u128)
        .ok_or(AmmError::DivisionByZero)?;

    let amount_b_to_return = amount_b_to_return_u128 as u64;

    if amount_a_to_return == 0 && amount_b_to_return == 0 && lpt_to_redeem > 0 {
        return err!(AmmError::CalculationFailure);
    }

    let cpi_accounts_burn = Burn {
        mint: context.accounts.lp_mint.to_account_info(),
        from: context.accounts.liquidity_provider_lp_account.to_account_info(),
        authority: context.accounts.liquidity_provider.to_account_info(),
    };
    let cpi_context_burn = CpiContext::new(
        context.accounts.token_program.to_account_info(),
        cpi_accounts_burn,
    );
    burn(cpi_context_burn, lpt_to_redeem)?;

    let token_a_mint_key = context.accounts.token_a_mint.key();
    let token_b_mint_key = context.accounts.token_b_mint.key();
    let pool_bump_seed_slice = [context.accounts.liquidity_pool.bump];

    let pool_signer_seeds: &[&[&[u8]]] = &[&[
        b"pool",
        token_a_mint_key.as_ref(),
        token_b_mint_key.as_ref(),
        &pool_bump_seed_slice,
    ]];

    if amount_a_to_return > 0 {
        let cpi_accounts_transfer_a = TransferChecked {
            from: context.accounts.vault_a.to_account_info(),
            mint: context.accounts.token_a_mint.to_account_info(),
            to: context.accounts.liquidity_provider_token_a.to_account_info(),
            authority: context.accounts.liquidity_pool.to_account_info(),
        };
        let cpi_context_transfer_a = CpiContext::new_with_signer(
            context.accounts.token_program.to_account_info(),
            cpi_accounts_transfer_a,
            pool_signer_seeds,
        );
        transfer_checked(cpi_context_transfer_a, amount_a_to_return, context.accounts.token_a_mint.decimals)?;
    }

    if amount_b_to_return > 0 {
        let cpi_accounts_transfer_b = TransferChecked {
            from: context.accounts.vault_b.to_account_info(),
            mint: context.accounts.token_b_mint.to_account_info(),
            to: context.accounts.liquidity_provider_token_b.to_account_info(),
            authority: context.accounts.liquidity_pool.to_account_info(),
        };
        let cpi_context_transfer_b = CpiContext::new_with_signer(
            context.accounts.token_program.to_account_info(),
            cpi_accounts_transfer_b,
            pool_signer_seeds,
        );
        transfer_checked(cpi_context_transfer_b, amount_b_to_return, context.accounts.token_b_mint.decimals)?;
    }
    
    Ok(())
}

pub fn calculate_token_b_provision_with_a_given<'info>(
    vault_a: &mut InterfaceAccount<'info, TokenAccount>, 
    vault_b: &mut InterfaceAccount<'info, TokenAccount>, 
    amount_a_to_deposit: u64
) -> Result<u64> {
    vault_a.reload()?;
    let reserve_a = vault_a.amount;

    vault_b.reload()?;
    let reserve_b = vault_b.amount;

    if reserve_a == 0 || reserve_b == 0 {
        return err!(AmmError::CannotAddLiquidityToEmptyPool); 
    }

    let amount_b_to_deposit_u128 = (amount_a_to_deposit as u128)
        .checked_mul(reserve_b as u128)
        .ok_or(AmmError::Overflow)?
        .checked_div(reserve_a as u128)
        .ok_or(AmmError::DivisionByZero)?;

    let amount_b_to_deposit = amount_b_to_deposit_u128 as u64;

    if amount_b_to_deposit == 0 {
        return err!(AmmError::CalculationFailure); 
    }

    Ok(amount_b_to_deposit)
}

fn liquidity_calculation(amount_token_a: u64, amount_token_b: u64) -> Result<u64> {
    let amount_a_u128 = amount_token_a as u128;
    let amount_b_u128 = amount_token_b as u128;

    let lp_amount_to_mint_u128 = amount_a_u128
        .checked_mul(amount_b_u128)
        .ok_or(AmmError::Overflow)? 
        .sqrt(); 

    let lp_amount_to_mint = lp_amount_to_mint_u128 as u64; // Cast back to u64

    // Check if the result is zero (could happen with very small initial amounts)
    if lp_amount_to_mint == 0 {
        return err!(AmmError::LpAmountCalculation); 
    }

    Ok(lp_amount_to_mint) 
}

