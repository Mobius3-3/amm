use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("Deposit Failed")]
    DepositToPoolFailed,
    #[msg("Pool Not Found")]
    PoolNotFound,
    #[msg("Pool Locked")]
    PoolLocked,
    #[msg("InvalidDepositAmount")]
    InvalidDepositAmount,    
    #[msg("Invalid mint for the pool")]
    InvalidMint,
    #[msg("Depositing too little liquidity")]
    DepositTooSmall,
    #[msg("Output is below the minimum expected")]
    OutputTooSmall,
    #[msg("SwapInFailed")]
    SwapInFailed,
    #[msg("SwapOutFailed")]
    SwapOutFailed,
}
