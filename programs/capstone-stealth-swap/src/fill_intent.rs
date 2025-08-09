use anchor_lang::prelude::*;
use anchor_spl::token::{ Mint, Token, TokenAccount, Transfer, transfer};
use anchor_spl::associated_token::AssociatedToken;

use crate::intent::Intent;
use crate::errors::ErrorCode;

#[derive(Accounts)]
pub struct FillIntent<'info> {
    #[account(mut)]
    pub solver: Signer<'info>,

  #[account(mut)]
    pub user: Signer<'info>,

    #[account(mut)]
    pub intent: Account<'info, Intent>,

    #[account(
        init_if_needed,
        payer = user,
        associated_token::mint = output_token_mint,
        associated_token::authority = user
    )]
    pub user_receive_ata: Account<'info, TokenAccount>,

    #[account(
        init_if_needed,
        payer = solver,
        associated_token::mint = input_token_mint,
        associated_token::authority = solver
    )]
    pub solver_receive_ata: Account<'info, TokenAccount>,

    #[account(mut)]
    pub solver_output_ata: Account<'info, TokenAccount>,
    
    #[account(mut,
        constraint = user_input_escrow.mint == intent.input_token @ ErrorCode::InvalidAccountData,
        constraint = user_input_escrow.owner == intent.key() @ ErrorCode::InvalidAccountData,
    )]
    pub user_input_escrow: Account<'info, TokenAccount>,

    #[account(
        init_if_needed,
        payer = solver,
        seeds = [
            b"escrow".as_ref(),
            solver.key().as_ref(),
            output_token_mint.key().as_ref()
        ],
        bump,
        token::mint = output_token_mint,
        token::authority = intent,
    )]
    pub solver_output_escrow: Account<'info, TokenAccount>,


    #[account(mut)]
    pub input_token_mint: Account<'info, Mint>,

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

    require!(intent.active, ErrorCode::IntentInactive);
    require!(intent.user == order.user, ErrorCode::IntentUserMismatch);
    require!(order.receive_amount >= intent.min_receive, ErrorCode::InsufficientOutput);
    require!(ctx.accounts.solver_output_ata.amount >= order.receive_amount, ErrorCode::InsufficientSolverBalance);
    require!(ctx.accounts.user_input_escrow.amount >= order.input_amount, ErrorCode::InsufficientUserEscrow);

    let user_key = intent.user.key();
    let bump = intent.bump;
    let signer_seeds: &[&[u8]] = &[
        b"intent",
        user_key.as_ref(),
        &[bump],
    ];
    let signer = &[&signer_seeds[..]];

    
        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.user_input_escrow.to_account_info(),
                to: ctx.accounts.solver_receive_ata.to_account_info(),
                authority: intent.to_account_info(),
            },
            signer,
        );
        transfer(cpi_ctx, order.input_amount)?;
    
        let cpi_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.solver_output_ata.to_account_info(),
                to: ctx.accounts.user_receive_ata.to_account_info(),
                authority: ctx.accounts.solver.to_account_info(),
            },
        );
        transfer(cpi_ctx, order.receive_amount)?;
    

    intent.active = false;
    Ok(())
}