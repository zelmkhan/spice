use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount};

use crate::{constants::{POOL_SEED, PROVIDER_SEED, SOL, SPICE_SEED, TREASURY_SEED}, errors::SpiceError, functions::{check_stoptap, calculate_yield}, states::{Pool, Provider, Settings}};


pub fn harvest_yield(ctx: Context<HarvestYieldInstructionAccounts>) -> Result<()> {
    check_stoptap(&ctx.accounts.treasury)?;
    
    // Calculate Income
    let total_lp = ctx.accounts.pool_pda.initial_liquidity;
    let provider_pending_claim = ctx.accounts.provider.pending_claim;
    let provider_lp_balance = ctx.accounts.provider.lp_balance;
    let provider_last_cumulative_yield = ctx.accounts.provider.last_cumulative_yield;
    let cumulative_yield = ctx.accounts.pool_pda.cumulative_yield;

    let provider_income: u64 = calculate_yield(cumulative_yield, total_lp, provider_lp_balance, provider_last_cumulative_yield)? + provider_pending_claim;

    // Update Provider
    ctx.accounts.provider.last_cumulative_yield = ctx.accounts.pool_pda.cumulative_yield;
    ctx.accounts.provider.pending_claim = 0;

    
    // Transfer from Treasury to User
    let seeds = &[SPICE_SEED.as_bytes(), TREASURY_SEED.as_bytes(), &[ctx.bumps.treasury]];
    let signer = [&seeds[..]];

    if ctx.accounts.mint.key().to_string() == SOL {
        **ctx.accounts.treasury.to_account_info().try_borrow_mut_lamports()? -= provider_income;
        **ctx.accounts.signer.to_account_info().try_borrow_mut_lamports()? += provider_income;
    } else {
        let from_token_account = ctx.accounts.treasury_ata.as_ref().ok_or(SpiceError::MissingSPLAccount)?;
        let to_token_account = ctx.accounts.signer_ata.as_ref().ok_or(SpiceError::MissingSPLAccount)?;

        let cpi_accounts = token::Transfer {
            from: from_token_account.to_account_info().clone(),
            to: to_token_account.to_account_info().clone(),
            authority: ctx.accounts.treasury.to_account_info().clone(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        token::transfer(CpiContext::new_with_signer(cpi_program, cpi_accounts, &signer), provider_income)?;
    }

    Ok(())
}

#[derive(Accounts)]
pub struct HarvestYieldInstructionAccounts<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    /// CHECK:
    pub mint: AccountInfo<'info>,

    #[account(mut, token::authority = signer, token::mint = mint)]
    pub signer_ata: Option<Account<'info, TokenAccount>>,

    #[account(mut, seeds = [POOL_SEED.as_bytes(), mint.key().as_ref()], bump)]
    pub pool_pda: Account<'info, Pool>,

    #[account(mut, seeds = [PROVIDER_SEED.as_bytes(), pool_pda.key().as_ref(), signer.key().as_ref()], bump)]
    pub provider: Account<'info, Provider>,

    /// CHECK:
    #[account(mut, seeds = [SPICE_SEED.as_bytes(), TREASURY_SEED.as_bytes()], bump)]
    pub treasury: Account<'info, Settings>,

    #[account(mut, token::authority = treasury, token::mint = mint)]
    pub treasury_ata: Option<Account<'info, TokenAccount>>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}
