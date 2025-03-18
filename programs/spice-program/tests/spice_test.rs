
pub const FEE_SCALE: u64 = 100_000;

#[derive(Debug, PartialEq)]
pub enum SpiceError {
    DivideByZero,
    Overflow,
    LiquidityZero,
}

pub fn calculate_output(
    input_amount: u64,
    price_a: u64,
    price_b: u64,
    decimals_a: u8,
    decimals_b: u8,
    pool_a_initial_liquidity: u64,
    pool_a_current_liquidity: u64,
    pool_b_initial_liquidity: u64,
    pool_b_current_liquidity: u64,
    pool_b_base_fee: u64,
    partner_fee: u64,
) -> Result<(u64, u64, u64, u64), SpiceError> {
    if price_b == 0 {
        return Err(SpiceError::DivideByZero);
    }

    let mut output_amount_scaled: u128 = (u128::from(price_a) * u128::from(input_amount)) / u128::from(price_b);


    // Если decimals_a больше decimals_b, уменьшаем точность
    if decimals_a > decimals_b {
        let shift = decimals_a as u32 - decimals_b as u32;
        output_amount_scaled /= 10u128.pow(shift);
    }
    // Если decimals_b больше decimals_a, увеличиваем точность
    else if decimals_b > decimals_a {
        let shift = decimals_b as u32 - decimals_a as u32;
        output_amount_scaled *= 10u128.pow(shift);
    }

    let output_amount = output_amount_scaled as u64;

    let protocol_fee_percentage = dynamic_fee(
        pool_b_base_fee,
        output_amount,
        pool_a_initial_liquidity,
        pool_a_current_liquidity,
        pool_b_initial_liquidity,
        pool_b_current_liquidity,
    )?;

    let protocol_fee = output_amount
        .checked_mul(protocol_fee_percentage)
        .and_then(|num| num.checked_div(FEE_SCALE))
        .ok_or(SpiceError::Overflow)?;

    let partner_fee_amount = output_amount
        .checked_mul(partner_fee)
        .and_then(|num| num.checked_div(FEE_SCALE))
        .ok_or(SpiceError::Overflow)?;

    let total_fee = protocol_fee.checked_add(partner_fee_amount).ok_or(SpiceError::Overflow)?;
    let net_output_amount = output_amount.checked_sub(total_fee).ok_or(SpiceError::Overflow)?;

    let output_amount_unscaled = output_amount_scaled;
    let net_output_amount_unscaled = net_output_amount;

    let protocol_fee_unscaled = protocol_fee;
    let partner_fee_unscaled = partner_fee_amount;

    Ok((output_amount_unscaled as u64, net_output_amount_unscaled, protocol_fee_unscaled, partner_fee_unscaled))
}

fn dynamic_fee(
    output_base_fee: u64,
    output_amount: u64,
    pool_a_initial_liquidity: u64,
    pool_a_current_liquidity: u64,
    pool_b_initial_liquidity: u64,
    pool_b_current_liquidity: u64,
) -> Result<u64, SpiceError> {
    if pool_b_current_liquidity == 0 {
        return Err(SpiceError::LiquidityZero);
    }

    let pool_a_delta = pool_a_current_liquidity as i128 - pool_a_initial_liquidity as i128;
    let pool_b_delta = pool_b_current_liquidity as i128 - pool_b_initial_liquidity as i128;

    if pool_a_delta > pool_b_delta && pool_b_delta < 0 {
        let liquidity_usage_ratio = output_amount as f64 / pool_b_current_liquidity as f64;
        let scaled_fee = (20_000.0 * (1.0 - (-5.0 * liquidity_usage_ratio).exp())).round() as u64;
        return Ok(scaled_fee.min(20_000));
    }
    
    Ok(output_base_fee)
}

pub fn income_calculation(
    cumulative_yield: u64,
    total_lp: u64,
    provider_lp_balance: u64,
    last_cumulative_yield: u64,
) -> Result<u64, SpiceError> {
    
    let cumulative_yield_unscale = cumulative_yield - last_cumulative_yield;
    let cumulative_yield_per_token = cumulative_yield_unscale as f64 / total_lp as f64;
    let provider_income = cumulative_yield_per_token as f64 * provider_lp_balance as f64;

    Ok(provider_income as u64)
}

fn convert_tokens(amount_a: u128, price_a: u128, price_b: u128, decimals_a: u32, decimals_b: u32) -> Result<u128, SpiceError> {
    Ok((amount_a * price_a * 10u128.pow(decimals_b)) / (price_b * 10u128.pow(decimals_a)))
}



#[cfg(test)]
mod spice {

    use super::*;

    #[test]
    fn converting() {
        let a =
        convert_tokens(10000000000000000000000, 17100000000, 100000000, 9, 6).unwrap();

        assert_eq!(a, 1710000000000000000000);
    }

    #[test]
    fn calculating_output() {
        let (a, b, c, d) =
            calculate_output(1_000_000_000_000_000_000, 17100000000, 100000000, 9, 6, 50000000000, 50000000000, 10000000000, 10000000000, 10, 0).unwrap();

        assert_eq!(a, 171000000000000000);
        assert_eq!(b, 170982900000000000);
        assert_eq!(c, 17100000000000);
        assert_eq!(d, 0);
    }

    #[test]
    fn calculating_yield() {
        let a =
        income_calculation(0, 50000000000, 0, 0).unwrap();

        assert_eq!(a, 0);
    }

    #[test]
    fn testing_dynamic_fee() {
        let fee = dynamic_fee(10, 1000000000, 50000000000, 50000000000, 10000000000, 11000000000).unwrap();
        assert_eq!(fee, 10);
    }
    
}
