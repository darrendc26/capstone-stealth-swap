use anchor_lang::prelude::*;
use anchor_spl::token::{ transfer, CloseAccount, Mint, Token, TokenAccount, Transfer, close_account};
use anchor_spl::associated_token::AssociatedToken;
use anchor_lang::system_program;

use crate::auction_account::*;
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

    #[account(mut)]
    pub auction: Account<'info, AuctionAccount>,

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

    /// CHECK: Bond vault PDA that holds lamports from solver bonds. 
    /// Seeds and bump ensure this is the correct program-derived address.
    /// Only used for lamport transfers, not token operations.
    #[account(
        mut,
        seeds = [b"bond_vault"],
        bump,
    )]
    pub bond_vault: AccountInfo<'info>,

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
    let auction = &mut ctx.accounts.auction;
    let now = Clock::get()?.unix_timestamp;

    require!(intent.active, ErrorCode::IntentInactive);
    require!(intent.user == order.user, ErrorCode::IntentUserMismatch);
    require!(order.receive_amount >= intent.min_receive, ErrorCode::InsufficientOutput);
    require!(ctx.accounts.solver_output_ata.amount >= order.receive_amount, ErrorCode::InsufficientSolverBalance);
    require!(ctx.accounts.user_input_escrow.amount >= order.input_amount, ErrorCode::InsufficientUserEscrow);
    require!(auction.status == AuctionStatus::Awarded, ErrorCode::AuctionNotAwarded);
    require!(ctx.accounts.solver.key() == auction.claimed_by.unwrap(), ErrorCode::AuctionNotSolver);
    require!(auction.claim_price.unwrap() >= intent.min_receive, ErrorCode::PriceBelowMinimum);
    require!(now < auction.end_time + auction.exclusive_window_secs, ErrorCode::TimeExceeded);

    let user_key = intent.user.key();
    let bump = intent.bump;
    let signer_seeds: &[&[u8]] = &[
        b"intent",
        user_key.as_ref(),
        &[bump],
    ];

    let cpi_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.solver_output_ata.to_account_info(),
            to: ctx.accounts.user_receive_ata.to_account_info(),
            authority: ctx.accounts.solver.to_account_info(),
        },
    );
    transfer(cpi_ctx, auction.claim_price.unwrap())?;

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
    
    auction.status = AuctionStatus::Ended;

    let bond_vault_bump = ctx.bumps.bond_vault;
    let bond_vault_seeds: &[&[u8]] = &[
        b"bond_vault",
        &[bond_vault_bump],
    ];
    let bond_vault_signer = &[&bond_vault_seeds[..]];

    let bond_refund = system_program::Transfer {
        from: ctx.accounts.bond_vault.to_account_info(),
        to: ctx.accounts.solver.to_account_info(),
    };
    
    system_program::transfer(
        CpiContext::new_with_signer(
            ctx.accounts.system_program.to_account_info(),
            bond_refund,
            bond_vault_signer,
        ),
        auction.bond_amount,
    )?;

    let cpi_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        CloseAccount {
            account: ctx.accounts.user_input_escrow.to_account_info(),
            destination: ctx.accounts.user.to_account_info(),
            authority: intent.to_account_info(),
        },
        signer,
    );
    close_account(cpi_ctx)?;
    
    intent.active = false;
    
    Ok(())
}