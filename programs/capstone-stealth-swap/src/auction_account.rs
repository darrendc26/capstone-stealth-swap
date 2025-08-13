use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct AuctionAccount {
    pub intent: Pubkey,
    pub start_quote: u64,
    pub min_quote: u64,
    pub start_time: i64,
    pub end_time: i64,
    pub exclusive_window_secs: i64,
    pub bond_amount: u64,
    pub claimed_by: Option<Pubkey>,
    pub claim_price: Option<u64>,
    pub claim_expiry: Option<i64>,
    pub status: AuctionStatus,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, InitSpace, PartialEq)]
#[repr(u8)]
pub enum AuctionStatus {
    Started = 0,
    Cancelled = 1,
    Awarded = 2,
    Ended   = 3,
}
