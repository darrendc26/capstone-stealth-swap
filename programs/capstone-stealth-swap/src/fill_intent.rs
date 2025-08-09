use anchor_lang::prelude::*;
use anchor_spl::token::{ Mint, Token, TokenAccount, Transfer, transfer};
use anchor_spl::associated_token::AssociatedToken;

use crate::intent::Intent;
use crate::errors::ErrorCode;

#[derive(Accounts)]
pub struct FillIntent<'info> {
    /// Solver (fills the intent) must sign
    #[account(mut)]
    pub solver: Signer<'info>,

  #[account(mut)]
    pub user: Signer<'info>,

    /// Intent account (checked that it belongs to the user)
    #[account(mut)]
    pub intent: Account<'info, Intent>,

    /// User's ATA for receiving output tokens (owned by user; created via Associated Token Program)
    #[account(
        init_if_needed,
        payer = user,
        associated_token::mint = output_token_mint,
        associated_token::authority = user
    )]
    pub user_receive_ata: Account<'info, TokenAccount>,

    /// Solver's ATA for receiving input tokens (owned by solver; created via Associated Token Program)
    #[account(
        init_if_needed,
        payer = solver,
        associated_token::mint = input_token_mint,
        associated_token::authority = solver
    )]
    pub solver_receive_ata: Account<'info, TokenAccount>,

    /// Solver's output ATA (where solver holds output tokens prior to moving to escrow).
    /// This is a normal user-owned ATA (solver owns it) and should exist before fill.
    #[account(mut)]
    pub solver_output_ata: Account<'info, TokenAccount>,
    
    /// User's escrow token account — owned by the intent PDA (created during CreateIntent).
    /// We assert mint + owner match the intent.
    #[account(mut,
        constraint = user_input_escrow.mint == intent.input_token @ ErrorCode::InvalidAccountData,
        constraint = user_input_escrow.owner == intent.key() @ ErrorCode::InvalidAccountData,
    )]
    pub user_input_escrow: Account<'info, TokenAccount>,

    /// Solver's escrow token account — owned by the intent PDA.
    /// Create it as a PDA-owned token account via the Token Program (not the Associated Token Program).
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

    /// Mints
    #[account(mut)]
    pub input_token_mint: Account<'info, Mint>,

    #[account(mut)]
    pub output_token_mint: Account<'info, Mint>,

    /// Programs
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

    // Basic intent validations
    require!(intent.active, ErrorCode::IntentInactive);
    require!(intent.user == order.user, ErrorCode::IntentUserMismatch);
    require!(order.receive_amount >= intent.min_receive, ErrorCode::InsufficientOutput);
    require!(intent.input_token == order.input_token, ErrorCode::IntentInputTokenMismatch);
    require!(intent.output_token == order.output_token, ErrorCode::IntentOutputTokenMismatch);
    require!(intent.input_amount == order.input_amount, ErrorCode::IntentInputAmountMismatch);
    require!(intent.min_receive == order.min_receive, ErrorCode::IntentMinReceiveMismatch);

    // Print balances for debugging
    msg!("solver_output_ata.amount = {}", ctx.accounts.solver_output_ata.amount);
    msg!("user_input_escrow.amount = {}", ctx.accounts.user_input_escrow.amount);
    msg!("solver_output_escrow.amount = {}", ctx.accounts.solver_output_escrow.amount);

    // Check solver actually has enough output tokens to move to escrow
    require!(ctx.accounts.solver_output_ata.amount >= order.receive_amount, ErrorCode::InsufficientSolverBalance);
    // Check user escrow has enough input tokens
    require!(ctx.accounts.user_input_escrow.amount >= order.input_amount, ErrorCode::InsufficientUserEscrow);

    // Build signer seeds for the intent PDA (must match how intent PDA was derived)
    // NOTE: CreateIntent used seeds [b"intent", user_pubkey] with bump
    let bump = intent.bump;
    let signer_seeds: &[&[u8]] = &[
        b"intent",
        intent.user.as_ref(),
        &[bump],
    ];
    let signer = &[&signer_seeds[..]];

    // 1) Transfer solver's output tokens from solver_output_ata -> solver_output_escrow
    //    Authority = solver (a real signer), so use normal CPI (no PDA signer)
    {
        let cpi_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.solver_output_ata.to_account_info(),
                to: ctx.accounts.solver_output_escrow.to_account_info(),
                authority: ctx.accounts.solver.to_account_info(),
            },
        );
        transfer(cpi_ctx, order.receive_amount)?;
    }

    // 2) Transfer user's input tokens from user_input_escrow -> solver_receive_ata
    //    Authority = intent PDA (needs PDA signer)
    {
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
    }

    // 3) Transfer output tokens from solver_output_escrow -> user_receive_ata
    //    Authority = intent PDA (needs PDA signer)
    {
        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.solver_output_escrow.to_account_info(),
                to: ctx.accounts.user_receive_ata.to_account_info(),
                authority: intent.to_account_info(),
            },
            signer,
        );
        transfer(cpi_ctx, order.receive_amount)?;
    }

    // Mark intent inactive
    intent.active = false;

    Ok(())
}
