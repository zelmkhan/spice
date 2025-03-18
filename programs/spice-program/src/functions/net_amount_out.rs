use crate::{constants::FEE_SCALE, errors::SpiceError};

pub fn net_amount_out(
    raw_amount_out: u128,
    protocol_fee: u64,
    partner_fee: u64
) -> Result<(u64, u64, u64), SpiceError> {
    let protocol_fee = raw_amount_out
        .checked_mul(protocol_fee as u128)
        .and_then(|num| num.checked_div(FEE_SCALE))
        .ok_or(SpiceError::Overflow)? as u64;

    let partner_fee_amount = raw_amount_out
        .checked_mul(partner_fee as u128)
        .and_then(|num| num.checked_div(FEE_SCALE))
        .ok_or(SpiceError::Overflow)? as u64;

    let total_fee = protocol_fee
        .checked_add(partner_fee_amount)
        .ok_or(SpiceError::Overflow)?;

    let amount_out = raw_amount_out as u64;

    let amount_out = amount_out
        .checked_sub(total_fee)
        .ok_or(SpiceError::Overflow)?;

    let protocol_fee_unscaled = protocol_fee;
    let partner_fee_unscaled = partner_fee_amount;

    Ok((
        amount_out,
        protocol_fee_unscaled,
        partner_fee_unscaled,
    ))
}
