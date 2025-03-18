use anchor_lang::prelude::*;
use anchor_spl::token::Token;
use crate::{functions::check_admin, states::{Pool, Settings}};

#[inline(never)]
pub fn set_pool_settings(ctx: Context<SetPoolSettingsInstructionAccounts>, is_active: bool, base_fee: u64) -> Result<()> {
    check_admin(&ctx.accounts.signer, &ctx.accounts.treasury)?;
    
    msg!("Base fee: {}", base_fee);
    msg!("State: {}", is_active);
    ctx.accounts.pool_pda.is_active = is_active;
    ctx.accounts.pool_pda.base_fee = base_fee;

    Ok(())
}

#[derive(Accounts)]
pub struct SetPoolSettingsInstructionAccounts<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(mut)]
    pub pool_pda: Account<'info, Pool>,

    pub treasury: Account<'info, Settings>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}
