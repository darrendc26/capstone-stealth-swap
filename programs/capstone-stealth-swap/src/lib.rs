#![allow(deprecated)]
#![allow(unexpected_cfgs)]
use anchor_lang::prelude::*;

declare_id!("AgpxpxaZ7cu744XZd1URa62jW9okzvXNMpF6nmED4Bnt");

pub mod create_intent;
pub mod fill_intent;
pub mod intent;
pub mod errors;
// use inten    t::*;
use create_intent::*;

#[program]
pub mod capstone_stealth_swap {
    use super::*;
    pub fn create_intent(ctx: Context<CreateIntent>, args: CreateIntentArgs, user_id: u64) -> Result<()> {
        create_intent::handler(ctx, args, user_id)
    }
}