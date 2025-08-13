use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Intent {
    pub id: u64,
    pub user: Pubkey,
    pub input_token: Pubkey,
    pub output_token: Pubkey,
    pub input_amount: u64, // input amount in input token in smallest units
    pub min_receive: u64,  // min receive in output token in smallest units
    pub active: bool,
    pub bump: u8,
}