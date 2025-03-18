use crate::errors::SpiceError;

pub fn raw_amount_out(
    amount_in: u64,
    price_a: u64,
    price_b: u64,
    decimals_a: u8,
    decimals_b: u8
) -> Result<u128, SpiceError> {
    if price_b == 0 {
        return Err(SpiceError::DivideByZero);
    }
    
    let mut amount_out = u128::from(amount_in)
    .checked_mul(u128::from(price_a))
    .and_then(|num| num.checked_div(u128::from(price_b)))
    .ok_or(SpiceError::Overflow)?;

    if decimals_a > decimals_b {
        let shift = decimals_a as u32 - decimals_b as u32;
        amount_out /= 10u128.pow(shift);
    }

    else if decimals_b > decimals_a {
        let shift = decimals_b as u32 - decimals_a as u32;
        amount_out *= 10u128.pow(shift);
    }

    Ok(amount_out)
}