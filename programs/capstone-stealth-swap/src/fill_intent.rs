use anchor_lang::prelude::*;
use anchor_spl::token::{ Mint, Token, TokenAccount, Transfer, transfer};
use anchor_spl::associated_token::{AssociatedToken};

use crate::intent::Intent;
use crate::errors::ErrorCode;

#[derive(Accounts)]
pub struct FillIntent<'info> {
    #[account(mut)]
    pub solver: Signer<'info>,

    #[account(mut)]
    pub user: Signer<'info>,

    #[account(mut, has_one = user)]
    pub intent: Account<'info, Intent>,

    #[account(mut)]
    pub input_token: Account<'info, TokenAccount>,

    #[account(mut)]
    pub output_token: Account<'info, TokenAccount>,
    
#[account(
    init_if_needed,
    payer = user,
    associated_token::mint = swap_token_mint,
    associated_token::authority = user,
)]
pub user_receive_ata: Account<'info, TokenAccount>,

    #[account(
        init_if_needed,
        payer = solver,
        associated_token::mint = input_token_mint,
        associated_token::authority = solver,
    )]
    pub solver_receive_ata: Account<'info, TokenAccount>,
    
    #[account(mut, 
        constraint = user_input_escrow.mint == intent.input_token, 
        constraint = user_input_escrow.owner == intent.key()
    )]
    pub user_input_escrow: Account<'info, TokenAccount>,

    #[account(mut,
        constraint = solver_output_escrow.mint == intent.output_token,
        constraint = solver_output_escrow.owner == intent.key()
    )]
    pub solver_output_escrow: Account<'info, TokenAccount>,

    #[account(mut)]
    pub input_token_mint: Account<'info, Mint>,

    #[account(mut)]
    pub swap_token_mint: Account<'info, Mint>,

    #[account(mut)]
    pub output_token_mint: Account<'info, Mint>,

    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}



#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct OrderConfig {
    pub id: u64,
    pub input_amount: u64,
    pub min_receive: u64,
    pub receive_amount: u64,
    pub input_token: Pubkey,
    pub output_token: Pubkey,
    pub user: Pubkey,
}

pub fn fill_intent_handler(ctx: Context<FillIntent>, order: OrderConfig) -> Result<()> {
    let intent = &mut ctx.accounts.intent;
    require!( intent.active, ErrorCode::IntentInactive);
    require!( intent.user == order.user, ErrorCode::IntentUserMismatch);
    require!(order.receive_amount >= intent.min_receive, ErrorCode::InsufficientOutput);
    require!( intent.input_token == order.input_token, ErrorCode::IntentInputTokenMismatch);
    require!( intent.output_token == order.output_token, ErrorCode::IntentOutputTokenMismatch);
    require!( intent.input_amount == order.input_amount, ErrorCode::IntentInputAmountMismatch);
    require!( intent.min_receive == order.min_receive, ErrorCode::IntentMinReceiveMismatch);

    let id = intent.id.to_le_bytes();

    let signer_seeds = &[
        b"intent".as_ref(),
        intent.user.as_ref(),
        &id,
    ];
    let signer = &[&signer_seeds[..]];

    let transfer_to_escrow = Transfer {
        from: ctx.accounts.solver_output_escrow.to_account_info(),
        to: ctx.accounts.solver_output_escrow.to_account_info(),
        authority: ctx.accounts.solver.to_account_info(),
    };
    
    let cpi_token_program = ctx.accounts.token_program.to_account_info();
    let cpi_context = CpiContext::new_with_signer(cpi_token_program, transfer_to_escrow, signer);

    transfer(cpi_context, order.receive_amount)?;

    let transfer_to_solver = Transfer {
        from: ctx.accounts.user_input_escrow.to_account_info(),
        to: ctx.accounts.solver_receive_ata.to_account_info(),
        authority: intent.to_account_info(),
    };

    let cpi_token_program = ctx.accounts.token_program.to_account_info();
    let cpi_context = CpiContext::new_with_signer(cpi_token_program, transfer_to_solver, signer);
    
    transfer(cpi_context, order.input_amount)?;

    let transfer_to_user = Transfer {
        from: ctx.accounts.solver_output_escrow.to_account_info(),
        to: ctx.accounts.user_receive_ata.to_account_info(),
        authority: intent.to_account_info(),
    };
    
    let cpi_token_program = ctx.accounts.token_program.to_account_info();
    let cpi_context = CpiContext::new_with_signer(cpi_token_program, transfer_to_user, signer);
    
    transfer(cpi_context, order.receive_amount)?;

    intent.active = false;
    
    Ok(())
}