use anchor_lang::prelude::*;
use crate::{constants::{SPICE_SEED, TREASURY_SEED}, functions::check_admin, states::Settings};


pub fn update_settings(ctx: Context<UpdateSettingsInstructionAccounts>, income_distribution: u64, stoptap: bool) -> Result<()> {
    check_admin(&ctx.accounts.signer, &ctx.accounts.treasury)?;

    ctx.accounts.treasury.admin = ctx.accounts.new_admin.key();
    ctx.accounts.treasury.income_distribution = income_distribution;
    ctx.accounts.treasury.stoptap = stoptap;

    Ok(())
}

#[derive(Accounts)]
pub struct UpdateSettingsInstructionAccounts<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    /// CHECK:
    pub new_admin: AccountInfo<'info>,
    
    #[account(mut, seeds = [SPICE_SEED.as_bytes(), TREASURY_SEED.as_bytes()], bump)]
    pub treasury: Account<'info, Settings>,
    
    pub system_program: Program<'info, System>,
}
