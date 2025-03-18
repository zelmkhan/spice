use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};
use pyth_solana_receiver_sdk::price_update::PriceUpdateV2;

use crate::{
    constants::{POOL_SEED, SPICE_SCALE, SPICE_SEED, TREASURY_SEED},
    errors::SpiceError,
    functions::{check_stoptap, dynamic_fee, net_amount_out, raw_amount_out, transfer_from_treasury, transfer_partner_fee, transfer_to_treasury},
    states::{Pool, Settings},
};

pub fn swap(
    ctx: Context<SwapInstructionAccounts>,
    amount_in: u64,
    min_amount_out: u64,
    partner_fee: u64,
) -> Result<()> {
    check_stoptap(&ctx.accounts.treasury)?;

    let pool_a = &mut ctx.accounts.a_pool_pda;
    let pool_b = &mut ctx.accounts.b_pool_pda;

    if pool_a.is_active == false {
        return Err(SpiceError::PoolANotActive.into());
    }
    if pool_b.is_active == false {
        return Err(SpiceError::PoolBNotActive.into());
    }
    if ctx.accounts.token_a_pyth_price_feed_account.key() != pool_a.pyth_price_feed_account {
        return Err(SpiceError::InvalidPythAccount.into());
    }
    if ctx.accounts.token_b_pyth_price_feed_account.key() != pool_b.pyth_price_feed_account {
        return Err(SpiceError::InvalidPythAccount.into());
    }

    let price_a = ctx.accounts.token_a_pyth_price_feed_account.price_message.price as u64;
    let price_b = ctx.accounts.token_b_pyth_price_feed_account.price_message.price as u64;

    msg!("Input price: {}", price_a);
    msg!("Output price: {}", price_b);

    let token_a_decimals = ctx.accounts.token_a_mint.decimals;
    let token_b_decimals = ctx.accounts.token_b_mint.decimals;
 
    let raw_amount_out = raw_amount_out(
        amount_in,
        price_a,
        price_b,
        token_a_decimals,
        token_b_decimals
    )?;

    let fee = dynamic_fee(
        pool_b.base_fee, 
        amount_in,
        raw_amount_out as u64, 
        pool_a.initial_liquidity, 
        pool_a.current_liquidity, 
        pool_b.initial_liquidity, 
        pool_b.current_liquidity, 
    )?;

    let amount_out = net_amount_out(raw_amount_out, fee, partner_fee)?;

    msg!("Raw output: {}", raw_amount_out);
    msg!("Net output: {}", amount_out.0);
    msg!("Protocol fee: {}", amount_out.1);
    msg!("Partner fee: {}", amount_out.2);

    if amount_out.0 < min_amount_out {
        return Err(SpiceError::HighSlippage.into());
    }
    
    if amount_out.0 > pool_b.current_liquidity {
        return Err(SpiceError::InsufficientLiquidity.into());
    }

    let output_amount_scale = amount_out.1 * SPICE_SCALE;
    let protocol_income = output_amount_scale / ctx.accounts.treasury.income_distribution;
    pool_b.protocol_income += protocol_income;

    let protocol_income_unscale = protocol_income / SPICE_SCALE;

    pool_a.current_liquidity += amount_in;
    pool_b.current_liquidity -= raw_amount_out as u64 - protocol_income_unscale;

    let provider_income = output_amount_scale - protocol_income;
    pool_b.cumulative_yield += provider_income;

    transfer_to_treasury(
        ctx.accounts.signer.clone(), 
        ctx.accounts.signer_a_ata.clone(), 
        ctx.accounts.treasury.clone(), 
        ctx.accounts.treasury_a_ata.clone(), 
        ctx.accounts.token_a_mint.key(), 
        amount_in, 
        ctx.accounts.token_program.clone(), 
        ctx.accounts.system_program.clone())?;
    
    let seeds = &[SPICE_SEED.as_bytes(), TREASURY_SEED.as_bytes(), &[ctx.bumps.treasury]];
    let signer = &[&seeds[..]];
    
    transfer_from_treasury(
        ctx.accounts.signer.clone(), 
        ctx.accounts.signer_b_ata.clone(), 
        ctx.accounts.treasury.clone(), 
        ctx.accounts.treasury_b_ata.clone(), 
        ctx.accounts.token_b_mint.key(), 
        amount_out.0, 
        ctx.accounts.token_program.clone(), 
        signer)?;
    
    if partner_fee > 0 {
        let partner_fee_account = ctx.accounts.fee_account.as_ref().ok_or(SpiceError::MissingSPLAccount)?;
        let partner_fee_output = amount_out.2;

        transfer_partner_fee(
            partner_fee_account.clone(), 
            Some(partner_fee_account.clone()), 
            ctx.accounts.treasury.clone(), 
            ctx.accounts.treasury_b_ata.clone(), 
            ctx.accounts.token_b_mint.key(), 
            partner_fee_output, 
            ctx.accounts.token_program.clone(), 
            signer)?;
    }

    Ok(())
}

#[derive(Accounts)]
pub struct SwapInstructionAccounts<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    /// CHECK:
    pub token_a_mint: Account<'info, Mint>,

    /// CHECK:
    pub token_b_mint: Account<'info, Mint>,

    /// CHECK: pool_pda.pyth
    pub token_a_pyth_price_feed_account: Account<'info, PriceUpdateV2>,

    /// CHECK: pool_pda.pyth
    pub token_b_pyth_price_feed_account: Account<'info, PriceUpdateV2>,

    #[account(mut, seeds = [POOL_SEED.as_bytes(), token_a_mint.key().as_ref()], bump)]
    pub a_pool_pda: Account<'info, Pool>,

    #[account(mut, seeds = [POOL_SEED.as_bytes(), token_b_mint.key().as_ref()], bump)]
    pub b_pool_pda: Account<'info, Pool>,

    #[account(mut, token::authority = signer, token::mint = token_a_mint)]
    pub signer_a_ata: Option<Account<'info, TokenAccount>>,

    #[account(mut, token::authority = signer, token::mint = token_b_mint)]
    pub signer_b_ata: Option<Account<'info, TokenAccount>>,

    /// CHECK:
    #[account(mut, seeds = [SPICE_SEED.as_bytes(), TREASURY_SEED.as_bytes()], bump)]
    pub treasury: Account<'info, Settings>,

    #[account(mut, token::authority = treasury, token::mint = token_a_mint)]
    pub treasury_a_ata: Option<Account<'info, TokenAccount>>,

    #[account(mut, token::authority = treasury, token::mint = token_b_mint)]
    pub treasury_b_ata: Option<Account<'info, TokenAccount>>,

    #[account(mut)]
    pub fee_account: Option<AccountInfo<'info>>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}
