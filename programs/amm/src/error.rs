use anchor_lang::prelude::*;

#[error_code]
pub enum AmmError {
    #[msg("Tokens Identical")] IdenticalTokens,
    #[msg("Invalid Token Order")] InvalidTokenOrder,
    #[msg("You can not provide 0 liquidity")] NoZeroProvision,
    #[msg("Overflow in Liquidity Tokencalculation")] Overflow,
    #[msg("LP Amount Calculation reached 0, can not receive 0 LP Tokens")] LpAmountCalculation,
    CannotAddLiquidityToEmptyPool,
    DivisionByZero,
    CalculationFailure,
    AskExceedsBalance,
    NoZeroRedemption,
    PoolIsEmpty,
    NoZeroAmount,
    Slippage,
    InsufficientLiquidity,
    Underflow
}