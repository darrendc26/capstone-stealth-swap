use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("Intent is inactive")]
    IntentInactive,

    #[msg("Intent user mismatch")]
    IntentUserMismatch,

    #[msg("Intent input token mismatch")]
    IntentInputTokenMismatch,

    #[msg("Intent output token mismatch")]
    IntentOutputTokenMismatch,

    #[msg("Intent input amount mismatch")]
    IntentInputAmountMismatch,

    #[msg("Intent min receive mismatch")]
    IntentMinReceiveMismatch,

    #[msg("Insufficient output")]
    InsufficientOutput,

    #[msg("Insufficient balance")]  
    InsufficientBalance,
    #[msg("Insufficient user escrow")]
    InsufficientUserEscrow,

    #[msg("Insufficient solver balance")]
    InsufficientSolverBalance,

    #[msg("Invalid account data")]  
    InvalidAccountData,
    #[msg("Invalid input amount")]
    InvalidInputAmount,
    #[msg("Invalid min receive")]
    InvalidMinReceive,
    #[msg("Math overflow")]  
    MathOverflow,
    #[msg("Auction not started")]
    AuctionNotStarted,
    #[msg("Auction expired")]
    AuctionExpired,
    #[msg("Auction already claimed")]
    AuctionAlreadyClaimed,
    #[msg("Intent mismatch")]
    IntentMismatch,
    #[msg("Price below minimum")]
    PriceBelowMinimum,
    #[msg("Auction not awarded")]
    AuctionNotAwarded,
    #[msg("Auction not solver")]
    AuctionNotSolver,
    #[msg("Time exceeded")]
    TimeExceeded,
}