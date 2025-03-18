use solana_program::msg;

use crate::errors::SpiceError;

pub fn dynamic_fee(
    fee_out: u64,
    amount_in: u64,
    amount_out: u64,
    pool_a_initial_liquidity: u64,
    pool_a_current_liquidity: u64,
    pool_b_initial_liquidity: u64,
    pool_b_current_liquidity: u64,
) -> Result<u64, SpiceError> {
    if pool_b_current_liquidity == 0 {
        return Err(SpiceError::NoLiquidity);
    }

    let pool_a_delta = pool_a_current_liquidity as i64 - pool_a_initial_liquidity as i64;
    let pool_b_delta = pool_b_current_liquidity as i64 - pool_b_initial_liquidity as i64;

    if pool_a_delta + amount_in as i64 == 0 && pool_b_delta >= 0 {
        msg!("Type: Equalizing");
        return Ok(fee_out);
    }

    if pool_a_delta > pool_b_delta && pool_b_delta < 0 {
        msg!("Type: Imbalancing");
        let liquidity_usage_ratio = amount_out as f64 / pool_b_current_liquidity as f64;
        let scaled_fee = (10_000.0 * (1.0 - (-5.0 * liquidity_usage_ratio).exp())).round() as u64;
        return Ok(scaled_fee.clamp(fee_out, 10_000));
    }
    
    msg!("Type: Regular");
    let liquidity_usage_ratio = amount_out as f64 / pool_b_current_liquidity as f64;
    let scaled_fee = (1_000.0 * (1.0 - (-1.0 * liquidity_usage_ratio).exp())).round() as u64;
    Ok(scaled_fee.clamp(fee_out, 1_000))
}