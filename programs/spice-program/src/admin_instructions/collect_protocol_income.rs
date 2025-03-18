use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount};

use crate::{constants::{POOL_SEED, SOL, SPICE_SEED, TREASURY_SEED}, errors::SpiceError, functions::check_admin, states::{Pool, Settings}};


pub fn collect_protocol_income(ctx: Context<CollectProtocolIncomeInstructionAccounts>) -> Result<()> {
    check_admin(&ctx.accounts.signer, &ctx.accounts.treasury)?;

    let protocol_income = ctx.accounts.pool_pda.protocol_income;
    
    // Transfer from Treasury to Admin
    let seeds = &[SPICE_SEED.as_bytes(), TREASURY_SEED.as_bytes(), &[ctx.bumps.treasury]];
    let signer = [&seeds[..]];

    if ctx.accounts.mint.key().to_string() == SOL {
        **ctx.accounts.treasury.to_account_info().try_borrow_mut_lamports()? -= protocol_income;
        **ctx.accounts.signer.to_account_info().try_borrow_mut_lamports()? += protocol_income;
    } else {
        let from_token_account = ctx.accounts.treasury_ata.as_ref().ok_or(SpiceError::MissingSPLAccount)?;
        let to_token_account = ctx.accounts.signer_ata.as_ref().ok_or(SpiceError::MissingSPLAccount)?;

        let cpi_accounts = token::Transfer {
            from: from_token_account.to_account_info().clone(),
            to: to_token_account.to_account_info().clone(),
            authority: ctx.accounts.treasury.to_account_info().clone(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        token::transfer(CpiContext::new_with_signer(cpi_program, cpi_accounts, &signer), protocol_income)?;
    }

    Ok(())
}

#[derive(Accounts)]
pub struct CollectProtocolIncomeInstructionAccounts<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    /// CHECK:
    pub mint: AccountInfo<'info>,

    #[account(mut, token::authority = signer, token::mint = mint)]
    pub signer_ata: Option<Account<'info, TokenAccount>>,

    #[account(mut, seeds = [POOL_SEED.as_bytes(), mint.key().as_ref()], bump)]
    pub pool_pda: Account<'info, Pool>,

    #[account(mut, seeds = [SPICE_SEED.as_bytes(), TREASURY_SEED.as_bytes()], bump)]
    pub treasury: Account<'info, Settings>,

    #[account(mut, token::authority = treasury, token::mint = mint)]
    pub treasury_ata: Option<Account<'info, TokenAccount>>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}
