use anchor_lang::prelude::*;
use crate::{errors::SpiceError, states::Settings};

pub fn check_stoptap(settings: &Account<Settings>) -> Result<()> {
    if settings.stoptap {
        return Err(SpiceError::StoptapActivated.into());
    }
    Ok(())
}

pub fn check_admin(signer: &Signer, settings: &Account<Settings>) -> Result<()> {
    if signer.key() != settings.admin {
        return Err(SpiceError::InvalidAdmin.into());
    }
    
    Ok(())
}