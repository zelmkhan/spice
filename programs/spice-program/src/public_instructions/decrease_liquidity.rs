use anchor_lang::prelude::*;
use anchor_spl::{associated_token::AssociatedToken, token::{self, Mint, Token, TokenAccount}};

use crate::{constants::{MINT_SEED, POOL_SEED, PROVIDER_SEED, SPICE_SEED, TREASURY_SEED}, errors::SpiceError, functions::{check_stoptap, calculate_yield, transfer_from_treasury}, states::{Pool, Provider, Settings}};

pub fn decrease_liquidity(ctx: Context<DecreaseLiquidityInstructionAccounts>, amount: u64) -> Result<()> {
    check_stoptap(&ctx.accounts.treasury)?;
    
    let provider_lp_balance: u64 = ctx.accounts.provider.lp_balance;
    if provider_lp_balance < amount {
        return err!(SpiceError::InvalidLpAmount);
    }

    let total_lp = ctx.accounts.pool_pda.initial_liquidity;
    let provider_income: u64 = calculate_yield(ctx.accounts.pool_pda.cumulative_yield, total_lp, provider_lp_balance, ctx.accounts.provider.last_cumulative_yield)? + ctx.accounts.provider.pending_claim;
    
    let pool_delta = ctx.accounts.pool_pda.current_liquidity as i64 - ctx.accounts.pool_pda.initial_liquidity as i64;
    let mut total_withdrawal_amount: u64 = amount + provider_income;

    if pool_delta < 0 {
        let fee = total_withdrawal_amount / 100;
        total_withdrawal_amount -= fee;
    }

    // Update Provider
    ctx.accounts.provider.lp_balance -= amount;
    ctx.accounts.provider.last_cumulative_yield = ctx.accounts.pool_pda.cumulative_yield;
    ctx.accounts.provider.pending_claim = 0;
    
    // Update Pool
    ctx.accounts.pool_pda.initial_liquidity -= amount;
    ctx.accounts.pool_pda.current_liquidity -= amount;

    // Burn LP
    token::burn(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            token::Burn {
                mint: ctx.accounts.lp_token_mint_pda.to_account_info(),
                from: ctx.accounts.signer_lp_ata.to_account_info(),
                authority: ctx.accounts.signer.to_account_info(),
            },
        ),
        amount,
    )?;
    
    // Transfer from Treasury to User
    let seeds = &[SPICE_SEED.as_bytes(), TREASURY_SEED.as_bytes(), &[ctx.bumps.treasury]];
    let signer = [&seeds[..]];

    transfer_from_treasury(
        ctx.accounts.signer.clone(), 
        ctx.accounts.signer_ata.clone(), 
        ctx.accounts.treasury.clone(), 
        ctx.accounts.treasury_ata.clone(), 
        ctx.accounts.mint.key(), 
        total_withdrawal_amount, 
        ctx.accounts.token_program.clone(), 
        &signer)?;

    Ok(())
}


#[derive(Accounts)]
pub struct DecreaseLiquidityInstructionAccounts<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    /// CHECK:
    pub mint: AccountInfo<'info>,

    #[account(mut, token::authority = signer, token::mint = mint)]
    pub signer_ata: Option<Account<'info, TokenAccount>>,

    #[account(mut, token::authority = signer, token::mint = lp_token_mint_pda)]
    pub signer_lp_ata: Account<'info, TokenAccount>,

    #[account(mut, seeds = [POOL_SEED.as_bytes(), mint.key().as_ref()], bump)]
    pub pool_pda: Account<'info, Pool>,

    #[account(mut, seeds = [MINT_SEED.as_bytes(), pool_pda.key().as_ref()], bump)]
    pub lp_token_mint_pda: Account<'info, Mint>,

    #[account(mut, seeds = [PROVIDER_SEED.as_bytes(), pool_pda.key().as_ref(), signer.key().as_ref()], bump)]
    pub provider: Account<'info, Provider>,

    /// CHECK:
    #[account(mut, seeds = [SPICE_SEED.as_bytes(), TREASURY_SEED.as_bytes()], bump)]
    pub treasury: Account<'info, Settings>,

    #[account(mut, token::authority = treasury, token::mint = mint)]
    pub treasury_ata: Option<Account<'info, TokenAccount>>,

    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}
