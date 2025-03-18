use anchor_lang::prelude::*;

#[account]
pub struct Settings {
    pub admin: Pubkey,
    pub income_distribution: u64,
    pub stoptap: bool
}