use anchor_lang::prelude::*;
use public_instructions::*;
use admin_instructions::*;

pub mod states;
pub mod public_instructions;
pub mod constants;
pub mod errors;
pub mod functions;
pub mod admin_instructions;

declare_id!("CcoYv1X4RczUmuGCUw3Vg1XWxspaofsh8dtD26cmN7nm");

#[program]
pub mod spice_program {
    use super::*;

    pub fn treasury_settings(ctx: Context<TreasurySettingsInstructionAccounts>, income_distribution: u64, stoptap: bool) -> Result<()> {
        admin_instructions::treasury_settings(ctx, income_distribution, stoptap)
    }

    pub fn update_settings(ctx: Context<UpdateSettingsInstructionAccounts>, income_distribution: u64, stoptap: bool) -> Result<()> {
        admin_instructions::update_settings(ctx, income_distribution, stoptap)
    }

    pub fn create_pool(ctx: Context<InitPoolInstructionAccounts>, is_active: bool, base_fee: u64) -> Result<()> {
        admin_instructions::create_pool(ctx, is_active, base_fee)
    }

    pub fn set_pool_settings(ctx: Context<SetPoolSettingsInstructionAccounts>, is_active: bool, base_fee: u64) -> Result<()> {
        admin_instructions::set_pool_settings(ctx, is_active, base_fee)
    }
    
    pub fn collect_protocol_income(ctx: Context<CollectProtocolIncomeInstructionAccounts>) -> Result<()> {
        admin_instructions::collect_protocol_income(ctx)
    }

    //---------

    pub fn increase_liquidity(ctx: Context<IncreaseLiquidityInstructionAccounts>, amount: u64) -> Result<()> {
        public_instructions::increase_liquidity(ctx, amount)
    }

    pub fn decrease_liquidity(ctx: Context<DecreaseLiquidityInstructionAccounts>, amount: u64) -> Result<()> {
        public_instructions::decrease_liquidity(ctx, amount)
    }

    pub fn harvest_yield(ctx: Context<HarvestYieldInstructionAccounts>) -> Result<()> {
        public_instructions::harvest_yield(ctx)
    }

    pub fn swap(ctx: Context<SwapInstructionAccounts>, input: u64, output: u64, fee_bps: u64) -> Result<()> {
        public_instructions::swap(ctx, input, output, fee_bps)
    }
    
}
