use anchor_lang::prelude::*;
use crate::{constants::{ADMINISTRATOR, SPICE_SEED, TREASURY_SEED}, errors::SpiceError, states::Settings};


pub fn treasury_settings(ctx: Context<TreasurySettingsInstructionAccounts>, income_distribution: u64, stoptap: bool) -> Result<()> {
    if ctx.accounts.signer.key.to_string() != ADMINISTRATOR {
        return Err(SpiceError::InvalidAdmin.into());
    }

    ctx.accounts.treasury.admin = ctx.accounts.signer.key();
    ctx.accounts.treasury.income_distribution = income_distribution;
    ctx.accounts.treasury.stoptap = stoptap;

    Ok(())
}

#[derive(Accounts)]
pub struct TreasurySettingsInstructionAccounts<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    
    #[account(
        init,
        payer = signer,
        seeds = [SPICE_SEED.as_bytes(), TREASURY_SEED.as_bytes()],
        bump,
        space = 8 + 32 + 8 + 1,
    )]
    pub treasury: Account<'info, Settings>,
    
    pub system_program: Program<'info, System>,
}
