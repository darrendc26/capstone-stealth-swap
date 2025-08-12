use anchor_lang::prelude::*;
use crate::auction_account::*;
use crate::errors::ErrorCode;
use crate::intent::Intent;
use anchor_lang::system_program;

#[derive(Accounts)]
pub struct ClaimAuction<'info> {
    #[account(mut)]
    pub solver: Signer<'info>,

    #[account(
        constraint = intent.active @ ErrorCode::IntentInactive,
    )]
    pub intent: Account<'info, Intent>,

    #[account(
        mut,
        seeds = [
            b"auction".as_ref(),
            intent.key().as_ref(),
        ],
       bump,
        constraint = auction.intent == intent.key() @ ErrorCode::IntentMismatch,
        constraint = auction.status == AuctionStatus::Started @ ErrorCode::AuctionNotStarted,
        constraint = auction.claimed_by.is_none() @ ErrorCode::AuctionAlreadyClaimed,
    )]
    pub auction: Account<'info, AuctionAccount>,

    /// CHECK: Bond vault PDA for holding lamports
    #[account(
        mut,
        seeds = [b"bond_vault"],
        bump,
    )]
    pub bond_vault: AccountInfo<'info>,

    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<ClaimAuction>) -> Result<()> {
    let auction = &mut ctx.accounts.auction;
    let now = Clock::get()?.unix_timestamp;

    require!(now >= auction.start_time, ErrorCode::AuctionNotStarted);
    require!(now <= auction.end_time, ErrorCode::AuctionExpired);

    let price_at_claim = current_price(auction, now);
    require!(price_at_claim >= auction.min_quote, ErrorCode::PriceBelowMinimum);

    let bond_transfer = system_program::Transfer {
        from: ctx.accounts.solver.to_account_info(),
        to: ctx.accounts.bond_vault.to_account_info(),
    };
    
    system_program::transfer(
        CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            bond_transfer,
        ),
        auction.bond_amount,
    )?;

    auction.claimed_by = Some(ctx.accounts.solver.key());
    auction.claim_expiry = Some(now + auction.exclusive_window_secs);
    auction.claim_price = Some(price_at_claim);
    auction.status = AuctionStatus::Awarded; 
    emit!(AuctionClaimed {
        intent: ctx.accounts.intent.key(),
        auction: auction.key(),
        solver: ctx.accounts.solver.key(),
        price_at_claim,
        claim_expiry: auction.claim_expiry.unwrap(),
    });

    Ok(())
}
pub fn current_price(auction: &AuctionAccount, current_time: i64) -> u64 {
    if current_time <= auction.start_time {
        return auction.start_quote;
    }
    
    if current_time >= auction.end_time {
        return auction.min_quote;
    }
    
    let elapsed = current_time - auction.start_time;
    let total_duration = auction.end_time - auction.start_time;
    let price_range = auction.start_quote - auction.min_quote;
    
    let decay = (price_range as u128 * elapsed as u128) / total_duration as u128;
    let current = auction.start_quote - decay as u64;
    
    current.max(auction.min_quote)
}

#[event]
pub struct AuctionClaimed {
    pub intent: Pubkey,
    pub auction: Pubkey,
    pub solver: Pubkey,
    pub price_at_claim: u64,
    pub claim_expiry: i64,
}