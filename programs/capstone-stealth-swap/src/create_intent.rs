use anchor_lang::prelude::*;
use anchor_spl::token::{transfer, Token, Transfer, TokenAccount, Mint};
use anchor_spl::associated_token::AssociatedToken;
use crate::intent::Intent;
use crate::errors::ErrorCode;

#[derive(Accounts)]
#[instruction(id: u64, args: CreateIntentArgs)]
pub struct CreateIntent<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    
    #[account(
        init,
        payer = user,
        space = 8 + Intent::INIT_SPACE,
        seeds = [
            b"intent".as_ref(),
            user.key().as_ref(),
        ],
        bump,
    )]
    pub intent: Account<'info, Intent>,
    
    #[account(mut)]
    pub user_token_account: Account<'info, TokenAccount>,

    #[account(
        init_if_needed,
        payer = user,
        seeds = [
            b"escrow".as_ref(),
            user.key().as_ref(),
        ],
        bump,
        token::mint = input_token_mint,
        token::authority = intent,
    )]
    pub user_input_escrow: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub input_token_mint: Account<'info, Mint>,
    
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct CreateIntentArgs {
    pub input_token: Pubkey,
    pub output_token: Pubkey,
    pub input_amount: u64,
    pub min_receive: u64,
}

pub fn create_intent_handler(ctx: Context<CreateIntent>, args: CreateIntentArgs, id: u64) -> Result<()> {
    // Validate user has sufficient balance
    require!(
        ctx.accounts.user_token_account.amount >= args.input_amount,
        ErrorCode::InsufficientBalance
    );

    // Initialize the intent
    ctx.accounts.intent.set_inner(Intent {
        user: ctx.accounts.user.key(),
        id,
        input_token: args.input_token,
        output_token: args.output_token,
        input_amount: args.input_amount,
        min_receive: args.min_receive,
        active: true,
        bump: ctx.bumps.intent,
    });

    // Transfer user's input tokens to escrow
    let transfer_ctx = Transfer {
        from: ctx.accounts.user_token_account.to_account_info(),
        to: ctx.accounts.user_input_escrow.to_account_info(),
        authority: ctx.accounts.user.to_account_info(),
    };
    
    let cpi_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        transfer_ctx
    );
    
    transfer(cpi_ctx, args.input_amount)?;

    Ok(())
}