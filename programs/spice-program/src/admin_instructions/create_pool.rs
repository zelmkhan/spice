use anchor_lang::prelude::*;
use anchor_spl::{associated_token::AssociatedToken, token::{Mint, Token, TokenAccount}};
use pyth_solana_receiver_sdk::price_update::PriceUpdateV2;
use crate::{constants::{MINT_SEED, POOL_SEED, SPICE_SEED, TREASURY_SEED}, functions::check_admin, states::{Pool, Settings}};

#[inline(never)]
pub fn create_pool(ctx: Context<InitPoolInstructionAccounts>, is_active: bool, base_fee: u64) -> Result<()> {
    check_admin(&ctx.accounts.signer, &ctx.accounts.treasury)?;

    ctx.accounts.pool_pda.is_active = is_active;
    ctx.accounts.pool_pda.base_fee = base_fee;
    ctx.accounts.pool_pda.mint = ctx.accounts.mint.key();
    ctx.accounts.pool_pda.pyth_price_feed_account = ctx.accounts.pyth_price_feed_account.key();
    ctx.accounts.pool_pda.lp_token_mint = ctx.accounts.lp_token_mint_pda.key();
    ctx.accounts.pool_pda.initial_liquidity = 0;
    ctx.accounts.pool_pda.cumulative_yield = 0;
    ctx.accounts.pool_pda.current_liquidity = 0;
    ctx.accounts.pool_pda.protocol_income = 0;

    Ok(())
}

#[derive(Accounts)]
pub struct InitPoolInstructionAccounts<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    
    /// CHECK:
    pub mint: Account<'info, Mint>,
    pub pyth_price_feed_account: Account<'info, PriceUpdateV2>,

    #[account(
        init,
        payer = signer,
        seeds = [POOL_SEED.as_bytes(), mint.key().as_ref()],
        bump,
        space = 8 + 1 + 8 + 32 + 32 + 32 + 8 + 8 + 8 + 8,
    )]
    pub pool_pda: Account<'info, Pool>,

    #[account(
        init,
        seeds = [MINT_SEED.as_bytes(), pool_pda.key().as_ref()], 
        bump,
        payer = signer,
        mint::decimals = mint.decimals,
        mint::authority = lp_token_mint_pda.key(),
        mint::freeze_authority = lp_token_mint_pda.key(),
    )]
    pub lp_token_mint_pda: Account<'info, Mint>,

    #[account(seeds = [SPICE_SEED.as_bytes(), TREASURY_SEED.as_bytes()], bump)]
    pub treasury: Account<'info, Settings>,
        
    #[account(
        init_if_needed,
        payer = signer,
        associated_token::mint = mint,
        associated_token::authority = treasury,
    )]
    pub treasury_ata: Box<Account<'info, TokenAccount>>,

    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}
