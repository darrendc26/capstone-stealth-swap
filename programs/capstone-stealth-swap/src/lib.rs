#![allow(deprecated)]
#![allow(unexpected_cfgs)]
use anchor_lang::prelude::*;

declare_id!("AgpxpxaZ7cu744XZd1URa62jW9okzvXNMpF6nmED4Bnt");

pub mod create_intent;
pub mod fill_intent;
pub mod intent;
pub mod errors;
pub mod auction_account;
pub mod create_auction;
pub mod claim_auction;
use create_intent::*;
use fill_intent::*;
use create_auction::*;
use claim_auction::*;

#[program]
pub mod capstone_stealth_swap {
    use super::*;
    pub fn create_intent(ctx: Context<CreateIntent>, args: CreateIntentArgs, user_id: u64) -> Result<()> {
        create_intent::create_intent_handler(ctx, args, user_id)
    }
    pub fn create_auction(ctx: Context<CreateAuction>) -> Result<()> {
        create_auction::create_auction_handler(ctx)
    }
    pub fn fill_intent(ctx: Context<FillIntent>, order: OrderConfig) -> Result<()> {
        fill_intent::fill_intent_handler(ctx, order)
    }

    pub fn claim_auction(ctx: Context<ClaimAuction>) -> Result<()> {
        claim_auction::handler(ctx)
    }

}