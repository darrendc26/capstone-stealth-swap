
use anchor_lang::prelude::*;
use crate::auction_account::*;
use crate::intent::Intent;
use crate::errors::ErrorCode;

#[derive(Accounts)]
pub struct CreateAuction<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        init,
        payer = user,
        space = 8 + AuctionAccount::INIT_SPACE,
        seeds = [
            b"auction".as_ref(),
            intent.key().as_ref(),
        ],
        bump,
    )]
    pub auction: Account<'info, AuctionAccount>,

    #[account(mut,
        constraint = intent.active @ ErrorCode::IntentInactive,
    )]
    pub intent: Account<'info, Intent>,
    pub system_program: Program<'info, System>,
}

pub fn create_auction_handler(ctx: Context<CreateAuction>) -> Result<()> {
    let auction = &mut ctx.accounts.auction;
    let now = Clock::get()?.unix_timestamp; 
    let intent = &ctx.accounts.intent;
    
    // Basic validation
    require!(intent.min_receive > 0, ErrorCode::InvalidMinReceive);
    require!(intent.input_amount > 0, ErrorCode::InvalidInputAmount);
    
    auction.intent = intent.key();
    auction.start_quote = intent.min_receive.checked_mul(110)
            .and_then(|x| x.checked_div(100)).ok_or(ErrorCode::MathOverflow)?;    
    auction.min_quote = intent.min_receive;
    auction.start_time = now;
    auction.end_time = now + 120; // 2 minutes
    auction.exclusive_window_secs = 30;
    auction.bond_amount = 1000000; // 0.001 SOL
    auction.claimed_by = None;
    auction.claim_price = None;
    auction.claim_expiry = None;
    auction.status = AuctionStatus::Started; 
    
    Ok(())
}