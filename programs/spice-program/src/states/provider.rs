use anchor_lang::prelude::*;

#[account]
pub struct Provider {
    pub address: Pubkey,
    pub lp_balance: u64,
    pub last_cumulative_yield: u64,
    pub pending_claim: u64
}