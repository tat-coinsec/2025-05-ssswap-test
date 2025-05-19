pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("3xEd5SCQznznQJvPbVsdMnEZeWLGanQ7EY61Aq8QH5RN");

#[program]
pub mod amm {
    use super::*;

    /* Initializes a Pool 
     * @notice Initializing a Pool has to go through this function since it carries the first LP logic
     * @notice The creator determines an initial price
     * @dev The underlying assumption is that an inaccurate initial price will automatically be arbitraged
     * @dev so if the initial set price by the user is inaccurate, the market will take care of it
     * @Invariant: Tokens need to be provided in order e.g. token_a < token_b so we guarantee uniqueness of the pool
     * @Invariant: A token pair which already exists can not be created a second time
     * @Invariant any deposit of token_a and token_b must be > 0, must return LP-tokens > 0 and proportional to the deposit
     * @param amount_token_a: Amount of token a to provide as initial liquidity
     * @param amount_token_b: Amount of token b to provide as initial liquidity
     */
    pub fn initialize_pool_with_liquidity(context: Context<InitializePool>, amount_token_a: u64, amount_token_b: u64) -> Result<()> {
        liquidity_operations::initialize_pool(context, amount_token_a, amount_token_b)
    }

    /* Provide Liquididty
     * @notice only takes token a as input parameter, token b will be calculated depending on the pool state
     * @dev Pulls in token a and b from the LP and mints a pool specific lp token in return to the user
     * @Invariant any deposit of token_a and token_b must be > 0, must return LP-tokens > 0 and proportional to the deposit
     * @param amount_token_a: Choosen amount of token a to provide
     */
    pub fn provide_liquidity(context: Context<ModifyLiquidity>, amount_token_a: u64) -> Result<()> {
        liquidity_operations::provide_liquidity(context, amount_token_a)
    }

    /* Remove Liquidity
     * @notice burn a choosen amount of the users LP Tokens and returns a proportional amount in token_a and token_b
     * @Invariant All outstanding LP Tokens need to be able to redeem their fair share of liquidity
     * @param lp_tokens_to_remove: amount of lp tokens to burn and redeem
     */
    pub fn remove_liquidity(context: Context<ModifyLiquidity>, lp_tokens_to_remove: u64) -> Result<()> {
        liquidity_operations::remove_liquidity(context, lp_tokens_to_remove)
    }

    /* Swap Exact In
     * @notice User specificies how many token of a or b he wants to sell plus a min_out as slippage protection
     * @dev Slippage protection AFTER fee deduction!
     * @Invariant follows x * y = k, however 0.3% fee are kept within the pool after the swap, which is expected
     * @Invariant 0.3% fee for LP providers will be kept as output token in the pool after swap
     * @param amount_in amount of token_a or token_b the user wants to sell
     * @param min_out slippage protection, minimum amount of tokens out the user is willing to accept
     * @param zero_for_one boolean to determine true: swap token_a -> token_b and false: token_b to token_a
     */
    pub fn swap_exact_in(context: Context<SwapContext>, amount_in: u64, min_out: u64, zero_for_one: bool) -> Result<()> {
        swap_operations::swap_exact_in(context, amount_in, min_out, zero_for_one)
    }

    /* Swap Exact Out
     * @notice User specificies how many token of a or b he wants to buy plus a max_in as slippage protection
     * @Invariant follows x * y = k, however 0.3% are additionally pulled into the pool in token_in and remain within
     * @Invariant 0.3% fee for LP providers will be kept as output token in the pool after swap
     * @param amount_in amount of token_a or token_b the user wants to sell
     * @param min_out slippage protection, minimum amount of tokens out the user is willing to accept
     * @param zero_for_one boolean to determine true: swap token_a -> token_b and false: token_b to token_a
     */
    pub fn swap_exact_out(context: Context<SwapContext>, amount_out: u64, max_in: u64, zero_for_one: bool) -> Result<()> {
        swap_operations::swap_exact_out(context, amount_out, max_in, zero_for_one)
    }
}
