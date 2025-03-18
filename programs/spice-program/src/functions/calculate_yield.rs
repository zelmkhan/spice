use crate::{constants::SPICE_SCALE, errors::SpiceError};

pub fn calculate_yield(
    cumulative_yield: u64,
    total_lp: u64,
    provider_lp_balance: u64,
    last_cumulative_yield: u64,
) -> Result<u64, SpiceError> {
    
    let cumulative_yield_unscale = (cumulative_yield - last_cumulative_yield) / SPICE_SCALE as u64;
    let cumulative_yield_per_token = cumulative_yield_unscale as f64 / total_lp as f64;
    let provider_income = cumulative_yield_per_token as f64 * provider_lp_balance as f64;

    Ok(provider_income as u64)
}