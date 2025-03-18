use anchor_lang::prelude::*;

#[account]
pub struct Pool {
    pub is_active: bool,
    pub base_fee: u64, // scale 1:FEE_SCALE
    pub mint: Pubkey,
    pub pyth_price_feed_account: Pubkey,
    pub lp_token_mint: Pubkey,
    pub initial_liquidity: u64,
    pub cumulative_yield: u64, // scale 1:SPICE_SCALE
    pub current_liquidity: u64,
    pub protocol_income: u64
}