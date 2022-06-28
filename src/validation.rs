use solana_program::{
    account_info::AccountInfo, program_error::ProgramError, program_pack::Pack, pubkey::Pubkey,
};
use spl_token;

use crate::error::NFTStakingContractError;

pub mod admin {
    solana_program::declare_id!("J7A8AeFaPNxe3w7jCxnE2xHVWZz2GgjAF9LWky5AG2Jq");
}

pub mod reward_mint {
    solana_program::declare_id!("2umZBxtNPeDLvj7VATBAPruVtqEZoxHYEwdvjhSyRjk7");
}

pub struct Validator;

impl Validator {
    pub fn validate_is_signer(signer: &AccountInfo) -> Result<(), ProgramError> {
        if !signer.is_signer {
            return Err(NFTStakingContractError::UserNotSigner.into());
        }

        Ok(())
    }

    pub fn validate_admin(admin: &AccountInfo) -> Result<(), ProgramError> {
        if !admin.is_signer || *admin.key != admin::id() {
            return Err(NFTStakingContractError::NotAdmin.into());
        }

        Ok(())
    }

    pub fn validate_token_owner(
        token_ata: &AccountInfo,
        user: &AccountInfo,
    ) -> Result<(), ProgramError> {
        let token_ata_unpacked = spl_token::state::Account::unpack(&token_ata.try_borrow_data()?)?;

        if token_ata_unpacked.owner != *user.key {
            return Err(NFTStakingContractError::IncorrectATAOwner.into());
        }

        Ok(())
    }

    pub fn validate_token_ata(
        token_ata: &AccountInfo,
        token_mint: &AccountInfo,
    ) -> Result<(), ProgramError> {
        let token_ata_unpacked = spl_token::state::Account::unpack(&token_ata.try_borrow_data()?)?;

        let token_mint_unpacked = spl_token::state::Mint::unpack(&token_mint.try_borrow_data()?)?;

        if token_mint_unpacked.decimals != 8
            || token_ata_unpacked.mint != reward_mint::id()
            || *token_ata.owner != spl_token::ID
        {
            return Err(NFTStakingContractError::InvalidTokenATA.into());
        }

        Ok(())
    }

    pub fn validate_nft_ata(
        token_ata: &AccountInfo,
        token_mint: &AccountInfo,
    ) -> Result<(), ProgramError> {
        let token_ata_unpacked = spl_token::state::Account::unpack(&token_ata.try_borrow_data()?)?;

        let token_mint_unpacked = spl_token::state::Mint::unpack(&token_mint.try_borrow_data()?)?;

        if token_ata_unpacked.amount != 1
            || token_mint_unpacked.decimals != 0
            || token_ata_unpacked.mint != *token_mint.key
            || *token_ata.owner != spl_token::ID
        {
            return Err(NFTStakingContractError::InvalidTokenATA.into());
        }

        Ok(())
    }

    pub fn validate_equality(lt: Pubkey, rt: Pubkey) -> Result<(), ProgramError> {
        if lt != rt {
            return Err(NFTStakingContractError::EqualityMismatch.into());
        }

        Ok(())
    }

    pub fn validate_bool(lt: bool, rt: bool) -> Result<(), ProgramError> {
        if lt != rt {
            return Err(NFTStakingContractError::BoolEqualityMismatch.into());
        }

        Ok(())
    }

    pub fn validate_state_account(
        state_account: &AccountInfo,
        program_id: Pubkey,
    ) -> Result<(), ProgramError> {
        if *state_account.owner != program_id || state_account.data_is_empty() {
            return Err(NFTStakingContractError::InvalidStateAccount.into());
        }

        Ok(())
    }
}
