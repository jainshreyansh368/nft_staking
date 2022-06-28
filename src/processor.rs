use crate::validation::Validator;
use crate::{
    error::NFTStakingContractError,
    instruction::{NFTStakingContractInstruction, PlatformData},
    state::{PlatformState, UserBaseState, UserNFTState},
};
use metaplex_token_metadata::state::Metadata;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program::{invoke, invoke_signed},
    program_pack::Pack,
    pubkey::Pubkey,
    rent::Rent,
    system_instruction,
    sysvar::{clock::Clock, Sysvar},
};
use spl_associated_token_account::instruction::create_associated_token_account;
use spl_token;

pub struct Processor;

impl Processor {
    pub fn unpack_and_process_instruction(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        instruction_data: &[u8],
    ) -> ProgramResult {
        match NFTStakingContractInstruction::unpack_instruction_data(instruction_data)? {
            NFTStakingContractInstruction::InitializePlatform(platform_data) => {
                msg!("Instruction: InitializePlatform");
                Self::process_initialize_paltform(program_id, accounts, platform_data)?;
            }

            NFTStakingContractInstruction::StakeNFT => {
                msg!("Instruction: StakeNFT");
                Self::process_stake_nft(program_id, accounts)?;
            }

            NFTStakingContractInstruction::UnstakeNFT => {
                msg!("Instruction: UnstakeNFT");
                Self::process_unstake_nft(program_id, accounts)?;
            }

            NFTStakingContractInstruction::ClaimReward => {
                msg!("Instruction: ClaimReward");
                Self::process_claim_reward(program_id, accounts)?;
            }
        }

        Ok(())
    }

    fn process_initialize_paltform(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        platform_data: PlatformData,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();

        let admin_account = next_account_info(account_info_iter)?;

        let platform_data_account = next_account_info(account_info_iter)?;

        let reward_mint = next_account_info(account_info_iter)?;

        let admin_reward_token_ata = next_account_info(account_info_iter)?;

        let pda_reward_token_ata = next_account_info(account_info_iter)?;

        let pda_account = next_account_info(account_info_iter)?;

        let token_program_account = next_account_info(account_info_iter)?;

        let system_program_account = next_account_info(account_info_iter)?;

        let (pda, _bump_seeds) = Pubkey::find_program_address(
            &[
                "nft_staking_contract".as_bytes(),
                platform_data_account.key.as_ref(),
            ],
            program_id,
        );

        Validator::validate_admin(admin_account)?;
        Validator::validate_token_ata(admin_reward_token_ata, reward_mint)?;
        Validator::validate_token_owner(admin_reward_token_ata, admin_account)?;
        Validator::validate_token_ata(pda_reward_token_ata, reward_mint)?;
        Validator::validate_token_owner(pda_reward_token_ata, pda_account)?;
        Validator::validate_equality(*pda_account.key, pda)?;

        let create_program_data_state_ix = system_instruction::create_account_with_seed(
            admin_account.key,
            platform_data_account.key,
            admin_account.key,
            "NFT Staking Main",
            Rent::default().minimum_balance(PlatformState::LEN),
            PlatformState::LEN as u64,
            program_id,
        );

        invoke(
            &create_program_data_state_ix,
            &[
                admin_account.clone(),
                platform_data_account.clone(),
                system_program_account.clone(),
            ],
        )?;

        let transfer_reward_token_to_pda_ata = spl_token::instruction::transfer(
            &spl_token::id(),
            admin_reward_token_ata.key,
            pda_reward_token_ata.key,
            admin_account.key,
            &[],
            100000000000000000,
        )?;

        invoke(
            &transfer_reward_token_to_pda_ata,
            &[
                admin_reward_token_ata.clone(),
                pda_reward_token_ata.clone(),
                admin_account.clone(),
                token_program_account.clone(),
            ],
        )?;

        let mut unpacked_platform_data_account =
            PlatformState::unpack_unchecked(&platform_data_account.try_borrow_data()?)?;

        unpacked_platform_data_account.is_initialized = true;
        unpacked_platform_data_account.coin_emission_percentage = platform_data.percent;
        unpacked_platform_data_account.coin_emission_distribution_in_sec =
            platform_data.distribution;
        unpacked_platform_data_account.reward_accumulation_in_sec = platform_data.accumulation;
        unpacked_platform_data_account.total_coin_emission = 1000000000u64
            .checked_mul(platform_data.percent.into())
            .ok_or(NFTStakingContractError::MathError)?
            .checked_div(100)
            .ok_or(NFTStakingContractError::MathError)?;
        unpacked_platform_data_account.reward_mint = *reward_mint.key;
        unpacked_platform_data_account.reward_token_ata = *pda_reward_token_ata.key;
        unpacked_platform_data_account.pda_account = *pda_account.key;

        PlatformState::pack(
            unpacked_platform_data_account,
            &mut platform_data_account.try_borrow_mut_data()?,
        )?;

        msg!("Platform Data: {:?}", unpacked_platform_data_account);

        Ok(())
    }

    fn process_stake_nft(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();

        let user_account = next_account_info(account_info_iter)?;

        let admin_account = next_account_info(account_info_iter)?;

        let platform_data_account = next_account_info(account_info_iter)?;

        let user_base_state_account = next_account_info(account_info_iter)?;

        let user_nft_state_account = next_account_info(account_info_iter)?;

        let user_nft_ata = next_account_info(account_info_iter)?;

        let user_nft_mint = next_account_info(account_info_iter)?;

        let reward_mint = next_account_info(account_info_iter)?;

        let user_reward_ata = next_account_info(account_info_iter)?;

        let pda_reward_token_ata = next_account_info(account_info_iter)?;

        let pda_account = next_account_info(account_info_iter)?;

        let nft_metadata_account = next_account_info(account_info_iter)?;

        let metadata_program_account = next_account_info(account_info_iter)?;

        let token_program_account = next_account_info(account_info_iter)?;

        let rent_sysvar_account = next_account_info(account_info_iter)?;

        let associated_token_account_program_account = next_account_info(account_info_iter)?;

        let system_program_account = next_account_info(account_info_iter)?;

        let (pda, bump_seeds) = Pubkey::find_program_address(
            &[
                "nft_staking_contract".as_bytes(),
                platform_data_account.key.as_ref(),
            ],
            program_id,
        );

        Validator::validate_admin(admin_account)?;
        Validator::validate_is_signer(user_account)?;
        Validator::validate_equality(*pda_account.key, pda)?;
        Validator::validate_nft_ata(user_nft_ata, user_nft_mint)?;
        Validator::validate_token_owner(user_nft_ata, user_account)?;
        Validator::validate_state_account(platform_data_account, *program_id)?;

        Self::update_pool(platform_data_account)?;

        if user_base_state_account.data_is_empty() {
            let create_user_base_state_ix = system_instruction::create_account_with_seed(
                user_account.key,
                user_base_state_account.key,
                user_account.key,
                "NFT Staking User Base",
                Rent::default().minimum_balance(UserBaseState::LEN),
                UserBaseState::LEN as u64,
                program_id,
            );

            invoke(
                &create_user_base_state_ix,
                &[
                    user_account.clone(),
                    user_base_state_account.clone(),
                    system_program_account.clone(),
                ],
            )?;

            let mut unpacked_user_base_state_account =
                UserBaseState::unpack_unchecked(&user_base_state_account.try_borrow_data()?)?;

            unpacked_user_base_state_account.is_initialized = true;
            unpacked_user_base_state_account.user = *user_account.key;
            unpacked_user_base_state_account.user_reward_ata = *user_reward_ata.key;

            UserBaseState::pack(
                unpacked_user_base_state_account,
                &mut user_base_state_account.try_borrow_mut_data()?,
            )?;
        } else {
            Validator::validate_state_account(user_base_state_account, *program_id)?;
        }

        let mut unpacked_platform_data_account =
            PlatformState::unpack(&platform_data_account.try_borrow_data()?)?;

        let mut unpacked_user_base_state_account =
            UserBaseState::unpack_unchecked(&user_base_state_account.try_borrow_data()?)?;

        let seed = format!("{}", unpacked_platform_data_account.total_staked_nfts);

        let create_user_nft_state_ix = system_instruction::create_account_with_seed(
            user_account.key,
            user_nft_state_account.key,
            user_account.key,
            &seed,
            Rent::default().minimum_balance(UserNFTState::LEN),
            UserNFTState::LEN as u64,
            program_id,
        );

        invoke(
            &create_user_nft_state_ix,
            &[
                user_account.clone(),
                user_nft_state_account.clone(),
                system_program_account.clone(),
            ],
        )?;

        let mut unpacked_user_nft_state_account =
            UserNFTState::unpack_unchecked(&user_nft_state_account.try_borrow_data()?)?;

        if user_reward_ata.data_is_empty() {
            let create_associated_reward_token_account_ix = create_associated_token_account(
                user_account.key,
                user_account.key,
                reward_mint.key,
            );

            invoke(
                &create_associated_reward_token_account_ix,
                &[
                    user_account.clone(),
                    user_reward_ata.clone(),
                    user_account.clone(),
                    reward_mint.clone(),
                    system_program_account.clone(),
                    token_program_account.clone(),
                    rent_sysvar_account.clone(),
                    associated_token_account_program_account.clone(),
                ],
            )?;
        }

        let set_authority_pda_ins = spl_token::instruction::set_authority(
            &spl_token::ID,
            user_nft_ata.key,
            Some(pda_account.key),
            spl_token::instruction::AuthorityType::AccountOwner,
            user_account.key,
            &[user_account.key],
        )?;

        invoke(
            &set_authority_pda_ins,
            &[
                user_nft_ata.clone(),
                user_account.clone(),
                token_program_account.clone(),
            ],
        )?;

        msg!(
            "Reward Debt Before: {}",
            unpacked_user_base_state_account.reward_debt
        );

        let dividend = (100_u64)
            .checked_mul(unpacked_user_base_state_account.total_staked_nfts)
            .ok_or(NFTStakingContractError::MathError)?;

        if unpacked_user_base_state_account.total_staked_nfts > 0 {
            let pending_reward = unpacked_user_base_state_account
                .total_staked_nfts
                .checked_mul(unpacked_platform_data_account.reward_per_share)
                .ok_or(NFTStakingContractError::MathError)?
                .checked_mul(unpacked_user_base_state_account.total_nft_points)
                .ok_or(NFTStakingContractError::MathError)?
                .checked_div(dividend)
                .ok_or(NFTStakingContractError::MathError)?
                .checked_sub(unpacked_user_base_state_account.reward_debt)
                .ok_or(NFTStakingContractError::MathError)?;

            msg!("Pending Reward: {}", pending_reward / 100);

            if pending_reward > 0 {
                let transfer_reward_ix = spl_token::instruction::transfer(
                    &spl_token::id(),
                    pda_reward_token_ata.key,
                    user_reward_ata.key,
                    pda_account.key,
                    &[],
                    pending_reward * 1000000,
                )?;

                invoke_signed(
                    &transfer_reward_ix,
                    &[
                        pda_reward_token_ata.clone(),
                        user_reward_ata.clone(),
                        pda_account.clone(),
                        token_program_account.clone(),
                    ],
                    &[&[
                        "nft_staking_contract".as_bytes(),
                        platform_data_account.key.as_ref(),
                        &[bump_seeds],
                    ]],
                )?;

                unpacked_user_base_state_account.total_reward_claimed =
                    unpacked_user_base_state_account
                        .total_reward_claimed
                        .checked_add(pending_reward)
                        .ok_or(NFTStakingContractError::MathError)?;
            }
        }

        unpacked_platform_data_account.total_staked_nfts = unpacked_platform_data_account
            .total_staked_nfts
            .checked_add(1)
            .ok_or(NFTStakingContractError::MathError)?;

        unpacked_user_base_state_account.total_staked_nfts = unpacked_user_base_state_account
            .total_staked_nfts
            .checked_add(1)
            .ok_or(NFTStakingContractError::MathError)?;

        unpacked_user_nft_state_account.is_initialized = true;
        unpacked_user_nft_state_account.user = *user_account.key;
        unpacked_user_nft_state_account.user_base_state = *user_base_state_account.key;
        unpacked_user_nft_state_account.nft_ata = *user_nft_ata.key;
        unpacked_user_nft_state_account.nft_mint = *user_nft_mint.key;

        const METADATA_PREFIX: &str = "metadata";

        let metadata_seeds = &[
            METADATA_PREFIX.as_bytes(),
            metadata_program_account.key.as_ref(),
            user_nft_mint.key.as_ref(),
        ];

        let (metadata_key, _metadata_bump_seed) =
            Pubkey::find_program_address(metadata_seeds, metadata_program_account.key);

        if metadata_key != *nft_metadata_account.key {
            return Err(NFTStakingContractError::InvalidNFTMetadata.into());
        }

        let nft_metadata = Metadata::from_account_info(nft_metadata_account)?;

        let nft_name_split: Vec<&str> = nft_metadata.data.name.split(' ').collect();

        let mut rarity = nft_name_split[1];

        rarity = rarity.trim_matches(char::from(0));

        match rarity {
            "CO" => {
                unpacked_user_base_state_account.total_nft_points =
                    unpacked_user_base_state_account
                        .total_nft_points
                        .checked_add(10)
                        .ok_or(NFTStakingContractError::MathError)?;
            }

            "RA" => {
                unpacked_user_base_state_account.total_nft_points =
                    unpacked_user_base_state_account
                        .total_nft_points
                        .checked_add(20)
                        .ok_or(NFTStakingContractError::MathError)?;
            }

            "EP" => {
                unpacked_user_base_state_account.total_nft_points =
                    unpacked_user_base_state_account
                        .total_nft_points
                        .checked_add(50)
                        .ok_or(NFTStakingContractError::MathError)?;
            }

            "LE" => {
                unpacked_user_base_state_account.total_nft_points =
                    unpacked_user_base_state_account
                        .total_nft_points
                        .checked_add(100)
                        .ok_or(NFTStakingContractError::MathError)?;
            }

            _ => return Err(NFTStakingContractError::InvalidNFTMetadata.into()),
        }

        msg!("nft points {}", unpacked_user_base_state_account
            .total_nft_points);

        let dividend_after = (100_u64)
            .checked_mul(unpacked_user_base_state_account.total_staked_nfts)
            .ok_or(NFTStakingContractError::MathError)?;
        
        msg!("dividend after {}", dividend_after);

        if dividend_after > 0 {
            unpacked_user_base_state_account.reward_debt = unpacked_user_base_state_account
                .total_staked_nfts
                .checked_mul(unpacked_platform_data_account.reward_per_share)
                .ok_or(NFTStakingContractError::MathError)?
                .checked_mul(unpacked_user_base_state_account.total_nft_points)
                .ok_or(NFTStakingContractError::MathError)?
                .checked_div(dividend_after)
                .ok_or(NFTStakingContractError::MathError)?;
        }

        msg!(
            "Reward Debt After: {}",
            unpacked_user_base_state_account.reward_debt
        );

        PlatformState::pack(
            unpacked_platform_data_account,
            &mut platform_data_account.try_borrow_mut_data()?,
        )?;

        UserBaseState::pack(
            unpacked_user_base_state_account,
            &mut user_base_state_account.try_borrow_mut_data()?,
        )?;

        UserNFTState::pack(
            unpacked_user_nft_state_account,
            &mut user_nft_state_account.try_borrow_mut_data()?,
        )?;

        msg!("Platform Data: {:?}", unpacked_platform_data_account);
        msg!("User Base: {:?}", unpacked_user_base_state_account);
        msg!("User NFT: {:?}", unpacked_user_nft_state_account);

        Ok(())
    }

    fn process_unstake_nft(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();

        let user_account = next_account_info(account_info_iter)?;

        let platform_data_account = next_account_info(account_info_iter)?;

        let user_base_state_account = next_account_info(account_info_iter)?;

        let user_nft_state_account = next_account_info(account_info_iter)?;

        let user_nft_ata = next_account_info(account_info_iter)?;

        let user_nft_mint = next_account_info(account_info_iter)?;

        let reward_mint = next_account_info(account_info_iter)?;

        let user_reward_ata = next_account_info(account_info_iter)?;

        let pda_reward_token_ata = next_account_info(account_info_iter)?;

        let pda_account = next_account_info(account_info_iter)?;

        let nft_metadata_account = next_account_info(account_info_iter)?;

        let metadata_program_account = next_account_info(account_info_iter)?;

        let token_program_account = next_account_info(account_info_iter)?;

        let (pda, bump_seeds) = Pubkey::find_program_address(
            &[
                "nft_staking_contract".as_bytes(),
                platform_data_account.key.as_ref(),
            ],
            program_id,
        );

        Validator::validate_is_signer(user_account)?;
        Validator::validate_equality(*pda_account.key, pda)?;
        Validator::validate_nft_ata(user_nft_ata, user_nft_mint)?;
        Validator::validate_token_owner(user_nft_ata, pda_account)?;
        Validator::validate_token_ata(user_reward_ata, reward_mint)?;
        Validator::validate_token_owner(user_reward_ata, user_account)?;
        Validator::validate_token_ata(pda_reward_token_ata, reward_mint)?;
        Validator::validate_token_owner(pda_reward_token_ata, pda_account)?;
        Validator::validate_state_account(platform_data_account, *program_id)?;
        Validator::validate_state_account(user_base_state_account, *program_id)?;
        Validator::validate_state_account(user_nft_state_account, *program_id)?;

        Self::update_pool(platform_data_account)?;

        let mut unpacked_user_base_state_account =
            UserBaseState::unpack(&user_base_state_account.try_borrow_data()?)?;

        Validator::validate_equality(unpacked_user_base_state_account.user, *user_account.key)?;
        Validator::validate_equality(
            unpacked_user_base_state_account.user_reward_ata,
            *user_reward_ata.key,
        )?;

        let unpacked_user_nft_state_account =
            UserNFTState::unpack(&user_nft_state_account.try_borrow_data()?)?;

        Validator::validate_equality(unpacked_user_nft_state_account.user, *user_account.key)?;
        Validator::validate_equality(
            unpacked_user_nft_state_account.user_base_state,
            *user_base_state_account.key,
        )?;
        Validator::validate_equality(unpacked_user_nft_state_account.nft_ata, *user_nft_ata.key)?;

        let mut unpacked_platform_data_account =
            PlatformState::unpack(&platform_data_account.try_borrow_data()?)?;

        Validator::validate_equality(unpacked_platform_data_account.reward_mint, *reward_mint.key)?;
        Validator::validate_equality(
            unpacked_platform_data_account.reward_token_ata,
            *pda_reward_token_ata.key,
        )?;

        let set_authority_back_to_user_ix = spl_token::instruction::set_authority(
            &spl_token::id(),
            user_nft_ata.key,
            Some(user_account.key),
            spl_token::instruction::AuthorityType::AccountOwner,
            pda_account.key,
            &[],
        )?;

        invoke_signed(
            &set_authority_back_to_user_ix,
            &[
                user_nft_ata.clone(),
                pda_account.clone(),
                token_program_account.clone(),
            ],
            &[&[
                "nft_staking_contract".as_bytes(),
                platform_data_account.key.as_ref(),
                &[bump_seeds],
            ]],
        )?;

        msg!(
            "User Staked NFT's Count: {}",
            unpacked_user_base_state_account.total_staked_nfts
        );
        msg!(
            "Reward Per Share: {}",
            unpacked_platform_data_account.reward_per_share
        );


        msg!(
            "Reward Debt Before: {}",
            unpacked_user_base_state_account.reward_debt
        );

        let dividend = (100_u64)
            .checked_mul(unpacked_user_base_state_account.total_staked_nfts)
            .ok_or(NFTStakingContractError::MathError)?;

        msg!(
            "dividend: {}",
            dividend
        );
        
        if unpacked_user_base_state_account.total_staked_nfts > 0 {
            let pending_reward = unpacked_user_base_state_account
                .total_staked_nfts
                .checked_mul(unpacked_platform_data_account.reward_per_share)
                .ok_or(NFTStakingContractError::MathError)?
                .checked_mul(unpacked_user_base_state_account.total_nft_points)
                .ok_or(NFTStakingContractError::MathError)?
                .checked_div(dividend)
                .ok_or(NFTStakingContractError::MathError)?
                .checked_sub(unpacked_user_base_state_account.reward_debt)
                .ok_or(NFTStakingContractError::MathError)?;

            msg!("Pending Reward: {}", pending_reward / 100);

            if pending_reward > 0 {
                let transfer_reward_ix = spl_token::instruction::transfer(
                    &spl_token::id(),
                    pda_reward_token_ata.key,
                    user_reward_ata.key,
                    pda_account.key,
                    &[],
                    pending_reward * 1000000,
                )?;

                invoke_signed(
                    &transfer_reward_ix,
                    &[
                        pda_reward_token_ata.clone(),
                        user_reward_ata.clone(),
                        pda_account.clone(),
                        token_program_account.clone(),
                    ],
                    &[&[
                        "nft_staking_contract".as_bytes(),
                        platform_data_account.key.as_ref(),
                        &[bump_seeds],
                    ]],
                )?;

                unpacked_user_base_state_account.total_reward_claimed =
                    unpacked_user_base_state_account
                        .total_reward_claimed
                        .checked_add(pending_reward)
                        .ok_or(NFTStakingContractError::MathError)?;
            }
        }

        unpacked_platform_data_account.total_staked_nfts = unpacked_platform_data_account
            .total_staked_nfts
            .checked_sub(1)
            .ok_or(NFTStakingContractError::MathError)?;

        unpacked_user_base_state_account.total_staked_nfts = unpacked_user_base_state_account
            .total_staked_nfts
            .checked_sub(1)
            .ok_or(NFTStakingContractError::MathError)?;

        const METADATA_PREFIX: &str = "metadata";

        let metadata_seeds = &[
            METADATA_PREFIX.as_bytes(),
            metadata_program_account.key.as_ref(),
            user_nft_mint.key.as_ref(),
        ];

        let (metadata_key, _metadata_bump_seed) =
            Pubkey::find_program_address(metadata_seeds, metadata_program_account.key);

        if metadata_key != *nft_metadata_account.key {
            return Err(NFTStakingContractError::InvalidNFTMetadata.into());
        }

        let nft_metadata = Metadata::from_account_info(nft_metadata_account)?;

        let nft_name_split: Vec<&str> = nft_metadata.data.name.split(' ').collect();

        let mut rarity = nft_name_split[1];

        rarity = rarity.trim_matches(char::from(0));

        match rarity {
            "CO" => {
                unpacked_user_base_state_account.total_nft_points =
                    unpacked_user_base_state_account
                        .total_nft_points
                        .checked_sub(10)
                        .ok_or(NFTStakingContractError::MathError)?;
            }

            "RA" => {
                unpacked_user_base_state_account.total_nft_points =
                    unpacked_user_base_state_account
                        .total_nft_points
                        .checked_sub(20)
                        .ok_or(NFTStakingContractError::MathError)?;
            }

            "EP" => {
                unpacked_user_base_state_account.total_nft_points =
                    unpacked_user_base_state_account
                        .total_nft_points
                        .checked_sub(50)
                        .ok_or(NFTStakingContractError::MathError)?;
            }

            "LE" => {
                unpacked_user_base_state_account.total_nft_points =
                    unpacked_user_base_state_account
                        .total_nft_points
                        .checked_sub(100)
                        .ok_or(NFTStakingContractError::MathError)?;
            }

            _ => return Err(NFTStakingContractError::InvalidNFTMetadata.into()),
        }

        msg!("nft points {}", unpacked_user_base_state_account
            .total_nft_points);

        let dividend_after = (100_u64)
            .checked_mul(unpacked_user_base_state_account.total_staked_nfts)
            .ok_or(NFTStakingContractError::MathError)?;
        
        msg!("dividend after {}", dividend_after);

        if dividend_after > 0 {
            unpacked_user_base_state_account.reward_debt = unpacked_user_base_state_account
                .total_staked_nfts
                .checked_mul(unpacked_platform_data_account.reward_per_share)
                .ok_or(NFTStakingContractError::MathError)?
                .checked_mul(unpacked_user_base_state_account.total_nft_points)
                .ok_or(NFTStakingContractError::MathError)?
                .checked_div(dividend_after)
                .ok_or(NFTStakingContractError::MathError)?;
        }

        msg!(
            "Reward Debt After: {}",
            unpacked_user_base_state_account.reward_debt
        );

        PlatformState::pack(
            unpacked_platform_data_account,
            &mut platform_data_account.try_borrow_mut_data()?,
        )?;

        UserBaseState::pack(
            unpacked_user_base_state_account,
            &mut user_base_state_account.try_borrow_mut_data()?,
        )?;

        Self::close_state_account(user_nft_state_account, user_account)?;

        msg!("Platform Data: {:?}", unpacked_platform_data_account);
        msg!("User Base: {:?}", unpacked_user_base_state_account);

        Ok(())
    }

    fn process_claim_reward(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();

        let user_account = next_account_info(account_info_iter)?;

        let platform_data_account = next_account_info(account_info_iter)?;

        let user_base_state_account = next_account_info(account_info_iter)?;

        let reward_mint = next_account_info(account_info_iter)?;

        let user_reward_ata = next_account_info(account_info_iter)?;

        let pda_reward_token_ata = next_account_info(account_info_iter)?;

        let pda_account = next_account_info(account_info_iter)?;

        let token_program_account = next_account_info(account_info_iter)?;

        let (pda, bump_seeds) = Pubkey::find_program_address(
            &[
                "nft_staking_contract".as_bytes(),
                platform_data_account.key.as_ref(),
            ],
            program_id,
        );

        Validator::validate_is_signer(user_account)?;
        Validator::validate_equality(*pda_account.key, pda)?;
        Validator::validate_token_ata(user_reward_ata, reward_mint)?;
        Validator::validate_token_owner(user_reward_ata, user_account)?;
        Validator::validate_token_ata(pda_reward_token_ata, reward_mint)?;
        Validator::validate_token_owner(pda_reward_token_ata, pda_account)?;
        Validator::validate_state_account(platform_data_account, *program_id)?;
        Validator::validate_state_account(user_base_state_account, *program_id)?;

        Self::update_pool(platform_data_account)?;

        let mut unpacked_user_base_state_account =
            UserBaseState::unpack(&user_base_state_account.try_borrow_data()?)?;

        Validator::validate_equality(unpacked_user_base_state_account.user, *user_account.key)?;
        Validator::validate_equality(
            unpacked_user_base_state_account.user_reward_ata,
            *user_reward_ata.key,
        )?;

        let unpacked_platform_data_account =
            PlatformState::unpack(&platform_data_account.try_borrow_data()?)?;

        Validator::validate_equality(unpacked_platform_data_account.reward_mint, *reward_mint.key)?;
        Validator::validate_equality(
            unpacked_platform_data_account.reward_token_ata,
            *pda_reward_token_ata.key,
        )?;

        msg!(
            "User Staked NFT's Count: {}",
            unpacked_user_base_state_account.total_staked_nfts
        );
        msg!(
            "Reward Per Share: {}",
            unpacked_platform_data_account.reward_per_share
        );
        msg!(
            "Reward Debt: {}",
            unpacked_user_base_state_account.reward_debt
        );

        msg!(
            "Reward Debt Before: {}",
            unpacked_user_base_state_account.reward_debt
        );

        let dividend = (100_u64)
            .checked_mul(unpacked_user_base_state_account.total_staked_nfts)
            .ok_or(NFTStakingContractError::MathError)?;

        if unpacked_user_base_state_account.total_staked_nfts > 0 {
            let pending_reward = unpacked_user_base_state_account
                .total_staked_nfts
                .checked_mul(unpacked_platform_data_account.reward_per_share)
                .ok_or(NFTStakingContractError::MathError)?
                .checked_mul(unpacked_user_base_state_account.total_nft_points)
                .ok_or(NFTStakingContractError::MathError)?
                .checked_div(dividend)
                .ok_or(NFTStakingContractError::MathError)?
                .checked_sub(unpacked_user_base_state_account.reward_debt)
                .ok_or(NFTStakingContractError::MathError)?;

            msg!("Pending Reward: {}", pending_reward / 100);

            // pending_reward = pending_reward
            //     .checked_div(dividend)
            //     .ok_or(NFTStakingContractError::MathError)?;

            // pending_reward = pending_reward
            //     .checked_mul(unpacked_user_base_state_account.total_nft_points)
            //     .ok_or(NFTStakingContractError::MathError)?;

            // msg!("Pending Reward: {}", pending_reward / 100);

            if pending_reward > 0 {
                let transfer_reward_ix = spl_token::instruction::transfer(
                    &spl_token::id(),
                    pda_reward_token_ata.key,
                    user_reward_ata.key,
                    pda_account.key,
                    &[],
                    pending_reward * 1000000,
                )?;

                invoke_signed(
                    &transfer_reward_ix,
                    &[
                        pda_reward_token_ata.clone(),
                        user_reward_ata.clone(),
                        pda_account.clone(),
                        token_program_account.clone(),
                    ],
                    &[&[
                        "nft_staking_contract".as_bytes(),
                        platform_data_account.key.as_ref(),
                        &[bump_seeds],
                    ]],
                )?;

                unpacked_user_base_state_account.total_reward_claimed =
                    unpacked_user_base_state_account
                        .total_reward_claimed
                        .checked_add(pending_reward)
                        .ok_or(NFTStakingContractError::MathError)?;
            }
        }
        
        msg!("nft points {}", unpacked_user_base_state_account
            .total_nft_points);

        let dividend_after = (100_u64)
            .checked_mul(unpacked_user_base_state_account.total_staked_nfts)
            .ok_or(NFTStakingContractError::MathError)?;
        
        msg!("dividend after {}", dividend_after);

        if dividend_after > 0 {
            unpacked_user_base_state_account.reward_debt = unpacked_user_base_state_account
                .total_staked_nfts
                .checked_mul(unpacked_platform_data_account.reward_per_share)
                .ok_or(NFTStakingContractError::MathError)?
                .checked_mul(unpacked_user_base_state_account.total_nft_points)
                .ok_or(NFTStakingContractError::MathError)?
                .checked_div(dividend_after)
                .ok_or(NFTStakingContractError::MathError)?;
        }

        msg!(
            "Reward Debt After: {}",
            unpacked_user_base_state_account.reward_debt
        );

        UserBaseState::pack(
            unpacked_user_base_state_account,
            &mut user_base_state_account.try_borrow_mut_data()?,
        )?;

        msg!("Platform Data: {:?}", unpacked_platform_data_account);
        msg!("User Base: {:?}", unpacked_user_base_state_account);

        Ok(())
    }



    

    fn update_pool(platform_data_account: &AccountInfo) -> ProgramResult {
        let mut unpacked_platform_data_account =
            PlatformState::unpack(&platform_data_account.try_borrow_data()?)?;

        let clock = Clock::get()?;

        if unpacked_platform_data_account.total_staked_nfts == 0 {
            unpacked_platform_data_account.last_updated = clock.unix_timestamp as u64;
            unpacked_platform_data_account.reward_per_share = 0;
            PlatformState::pack(
                unpacked_platform_data_account,
                &mut platform_data_account.try_borrow_mut_data()?,
            )?;

            return Ok(());
        }

        let interval = clock.unix_timestamp as u64 - unpacked_platform_data_account.last_updated;

        msg!("Interval: {}", interval);

        let multiplier = interval
            .checked_div(unpacked_platform_data_account.reward_accumulation_in_sec)
            .ok_or(NFTStakingContractError::MathError)?;

        msg!("Multiplier: {}", multiplier);

        let total_coin_emission = unpacked_platform_data_account.total_coin_emission;

        msg!(
            "Total Coin Emission: {} for {} seconds",
            total_coin_emission,
            unpacked_platform_data_account.coin_emission_distribution_in_sec
        );

        let reward_per_multiplier = ((total_coin_emission as f64)
            / unpacked_platform_data_account.coin_emission_distribution_in_sec as f64)
            * unpacked_platform_data_account.reward_accumulation_in_sec as f64
            * 100.00;

        msg!("Reward Per Multiplier: {}", reward_per_multiplier);

        let reward_generated = multiplier
            .checked_mul(reward_per_multiplier as u64)
            .ok_or(NFTStakingContractError::MathError)?;

        msg!("Reward Generated: {}", reward_generated);

        unpacked_platform_data_account.reward_per_share = unpacked_platform_data_account
            .reward_per_share
            .checked_add(
                reward_generated
                    .checked_div(unpacked_platform_data_account.total_staked_nfts)
                    .ok_or(NFTStakingContractError::MathError)?,
            )
            .ok_or(NFTStakingContractError::MathError)?;

        msg!(
            "Reward Per Share: {}",
            unpacked_platform_data_account.reward_per_share
        );

        if multiplier > 0 {
            unpacked_platform_data_account.last_updated = clock.unix_timestamp as u64;
        }

        msg!(
            "Last Updated: {}",
            unpacked_platform_data_account.last_updated
        );

        PlatformState::pack(
            unpacked_platform_data_account,
            &mut platform_data_account.try_borrow_mut_data()?,
        )?;

        Ok(())
    }

    fn close_state_account(
        state_account: &AccountInfo,
        dest_account: &AccountInfo,
    ) -> ProgramResult {
        let state_account_account_initial_lamports = dest_account.lamports();
        **dest_account.lamports.borrow_mut() = state_account_account_initial_lamports
            .checked_add(state_account.lamports())
            .ok_or(NFTStakingContractError::MathError)?;
        **state_account.lamports.borrow_mut() = 0;

        let mut state_account_lamports = state_account.data.borrow_mut();
        state_account_lamports.fill(0);

        Ok(())
    }
}
