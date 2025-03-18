use anchor_lang::{prelude::*, solana_program::system_instruction};
use anchor_spl::token::{self, Token, TokenAccount};
use crate::{constants::SOL, errors::SpiceError, states::Settings};

pub fn transfer_to_treasury<'info>(
    signer: Signer<'info>,
    signer_ata: Option<Account<'info, TokenAccount>>, 
    treasury: Account<'info, Settings>, 
    treasury_ata: Option<Account<'info, TokenAccount>>,
    token_mint: Pubkey, 
    amount: u64,
    token_program: Program<'info, Token>,
    system_program: Program<'info, System>
) -> Result<()> {

    if token_mint.key().to_string() == SOL {
        let transfer_instruction = system_instruction::transfer(
            signer.key,
            &treasury.key(),
            amount,
        );
        anchor_lang::solana_program::program::invoke(
            &transfer_instruction,
            &[
                signer.to_account_info(),
                treasury.to_account_info(),
                system_program.to_account_info(),
            ],
        )?;
    } else {
        let from_token_account = signer_ata.as_ref().ok_or(SpiceError::MissingSPLAccount)?;
        let to_token_account = treasury_ata.as_ref().ok_or(SpiceError::MissingSPLAccount)?;

        let cpi_accounts = token::Transfer {
            from: from_token_account.to_account_info(),
            to: to_token_account.to_account_info(),
            authority: signer.to_account_info(),
        };
        token::transfer(CpiContext::new(token_program.to_account_info(), cpi_accounts), amount)?;
    }

    Ok(())
}


pub fn transfer_from_treasury<'info>(
    signer: Signer,
    signer_ata: Option<Account<'info, TokenAccount>>, 
    treasury: Account<'info, Settings>, 
    treasury_ata: Option<Account<'info, TokenAccount>>,
    token_mint: Pubkey, 
    amount: u64,
    token_program: Program<'info, Token>,
    signer_seeds: &[&[&[u8]]],
) -> Result<()> {

    if token_mint.key().to_string() == SOL {
        **treasury.to_account_info().try_borrow_mut_lamports()? -= amount;
        **signer.to_account_info().try_borrow_mut_lamports()? += amount;
    } else {
        let from_token_account = treasury_ata.as_ref().ok_or(SpiceError::MissingSPLAccount)?;
        let to_token_account = signer_ata.as_ref().ok_or(SpiceError::MissingSPLAccount)?;

        let cpi_accounts = token::Transfer {
            from: from_token_account.to_account_info(),
            to: to_token_account.to_account_info(),
            authority: treasury.to_account_info(),
        };
        token::transfer(
            CpiContext::new_with_signer(token_program.to_account_info(), cpi_accounts, signer_seeds),
            amount,
        )?;
    }

    Ok(())
}

pub fn transfer_partner_fee<'info>(
    signer: AccountInfo,
    signer_ata: Option<AccountInfo<'info>>, 
    treasury: Account<'info, Settings>, 
    treasury_ata: Option<Account<'info, TokenAccount>>,
    token_mint: Pubkey, 
    amount: u64,
    token_program: Program<'info, Token>,
    signer_seeds: &[&[&[u8]]],
) -> Result<()> {

    if token_mint.key().to_string() == SOL {
        **treasury.to_account_info().try_borrow_mut_lamports()? -= amount;
        **signer.to_account_info().try_borrow_mut_lamports()? += amount;
    } else {
        let from_token_account = treasury_ata.as_ref().ok_or(SpiceError::MissingSPLAccount)?;
        let to_token_account = signer_ata.as_ref().ok_or(SpiceError::MissingSPLAccount)?;

        let cpi_accounts = token::Transfer {
            from: from_token_account.to_account_info(),
            to: to_token_account.to_account_info(),
            authority: treasury.to_account_info(),
        };
        token::transfer(
            CpiContext::new_with_signer(token_program.to_account_info(), cpi_accounts, signer_seeds),
            amount,
        )?;
    }

    Ok(())
}