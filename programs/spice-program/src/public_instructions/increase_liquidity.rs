use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{mint_to, Mint, MintTo, Token, TokenAccount},
};

use crate::{
    constants::{MINT_SEED, POOL_SEED, PROVIDER_SEED, SPICE_SEED, TREASURY_SEED}, functions::{check_stoptap, calculate_yield, transfer_to_treasury}, states::{Pool, Provider, Settings}
};

pub fn increase_liquidity(
    ctx: Context<IncreaseLiquidityInstructionAccounts>,
    amount: u64,
) -> Result<()> {
    check_stoptap(&ctx.accounts.treasury)?;

    transfer_to_treasury(
        ctx.accounts.signer.clone(), 
        ctx.accounts.signer_ata.clone(), 
        ctx.accounts.treasury.clone(), 
        ctx.accounts.treasury_ata.clone(), 
        ctx.accounts.mint.key(), 
        amount, 
        ctx.accounts.token_program.clone(), 
        ctx.accounts.system_program.clone())?;


    let pool_pda_key = ctx.accounts.pool_pda.key();
    let seeds = &[MINT_SEED.as_bytes(), pool_pda_key.as_ref(), &[ctx.bumps.lp_token_mint_pda]];
    let signer = [&seeds[..]];
    
    mint_to(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            MintTo {
                authority: ctx.accounts.lp_token_mint_pda.to_account_info(),
                to: ctx.accounts.signer_lp_ata.to_account_info(),
                mint: ctx.accounts.lp_token_mint_pda.to_account_info(),
            },
            &signer,
        ),
        amount,
    )?;


    let total_lp = ctx.accounts.pool_pda.initial_liquidity;
    let provider_lp_balance = ctx.accounts.provider.lp_balance;
    let provider_last_cumulative_yield = ctx.accounts.provider.last_cumulative_yield;
    let cumulative_yield = ctx.accounts.pool_pda.cumulative_yield;

    let provider_income: u64 = calculate_yield(cumulative_yield, total_lp, provider_lp_balance, provider_last_cumulative_yield)?;

    // Update provider
    ctx.accounts.provider.address = ctx.accounts.signer.key();
    ctx.accounts.provider.last_cumulative_yield = ctx.accounts.pool_pda.cumulative_yield;
    ctx.accounts.provider.lp_balance += amount;
    ctx.accounts.provider.pending_claim += provider_income;

    // Update pool
    ctx.accounts.pool_pda.initial_liquidity += amount;
    ctx.accounts.pool_pda.current_liquidity += amount;

    Ok(())
}

#[derive(Accounts)]
pub struct IncreaseLiquidityInstructionAccounts<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    /// CHECK:
    pub mint: AccountInfo<'info>,

    #[account(mut, token::mint = mint)]
    pub signer_ata: Option<Account<'info, TokenAccount>>,

    #[account(
        init_if_needed,
        payer = signer,
        associated_token::mint = lp_token_mint_pda,
        associated_token::authority = signer,
    )]
    pub signer_lp_ata: Account<'info, TokenAccount>,

    #[account(mut, seeds = [POOL_SEED.as_bytes(), mint.key().as_ref()], bump)]
    pub pool_pda: Account<'info, Pool>,

    #[account(mut, seeds = [MINT_SEED.as_bytes(), pool_pda.key().as_ref()], bump)]
    pub lp_token_mint_pda: Account<'info, Mint>,

    #[account(
        init_if_needed,
        payer = signer,
        seeds = [PROVIDER_SEED.as_bytes(), pool_pda.key().as_ref(), signer.key().as_ref()],
        bump,
        space = 8 + 32 + 8 + 8 + 8,
    )]
    pub provider: Account<'info, Provider>,

    #[account(seeds = [SPICE_SEED.as_bytes(), TREASURY_SEED.as_bytes()], bump)]
    pub treasury: Account<'info, Settings>,

    #[account(mut, token::authority = treasury, token::mint = mint)]
    pub treasury_ata: Option<Account<'info, TokenAccount>>,

    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}
