use arrayref::{array_mut_ref, array_ref, array_refs, mut_array_refs};
use solana_program::{
    program_error::ProgramError,
    program_pack::{IsInitialized, Pack, Sealed},
    pubkey::Pubkey,
};

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct PlatformState {
    pub is_initialized: bool,
    pub coin_emission_percentage: u8,
    pub coin_emission_distribution_in_sec: u64,
    pub reward_accumulation_in_sec: u64,
    pub total_coin_emission: u64,
    pub total_staked_nfts: u64,
    pub reward_per_share: u64,
    pub last_updated: u64,
    pub reward_mint: Pubkey,
    pub reward_token_ata: Pubkey,
    pub pda_account: Pubkey,
}

impl Sealed for PlatformState {}
impl IsInitialized for PlatformState {
    fn is_initialized(&self) -> bool {
        self.is_initialized
    }
}

impl Pack for PlatformState {
    const LEN: usize = 146;

    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        let src = array_ref![src, 0, PlatformState::LEN];

        let (
            is_initialized,
            coin_emission_percentage,
            coin_emission_distribution_in_sec,
            reward_accumulation_in_sec,
            total_coin_emission,
            total_staked_nfts,
            reward_per_share,
            last_updated,
            reward_mint,
            reward_token_ata,
            pda_account,
        ) = array_refs![src, 1, 1, 8, 8, 8, 8, 8, 8, 32, 32, 32];

        let is_initialized = match is_initialized {
            [0] => false,
            [1] => true,
            _ => return Err(ProgramError::InvalidAccountData),
        };

        Ok(PlatformState {
            is_initialized,
            coin_emission_percentage: u8::from_le_bytes(*coin_emission_percentage),
            coin_emission_distribution_in_sec: u64::from_le_bytes(
                *coin_emission_distribution_in_sec,
            ),
            reward_accumulation_in_sec: u64::from_le_bytes(*reward_accumulation_in_sec),
            total_coin_emission: u64::from_le_bytes(*total_coin_emission),
            total_staked_nfts: u64::from_le_bytes(*total_staked_nfts),
            reward_per_share: u64::from_le_bytes(*reward_per_share),
            last_updated: u64::from_le_bytes(*last_updated),
            reward_mint: Pubkey::new_from_array(*reward_mint),
            reward_token_ata: Pubkey::new_from_array(*reward_token_ata),
            pda_account: Pubkey::new_from_array(*pda_account),
        })
    }

    fn pack_into_slice(&self, dst: &mut [u8]) {
        let dst = array_mut_ref![dst, 0, PlatformState::LEN];

        let (
            is_initialized_dst,
            coin_emission_percentage_dst,
            coin_emission_distribution_in_sec_dst,
            reward_accumulation_in_sec_dst,
            total_coin_emission_dst,
            total_staked_nfts_dst,
            reward_per_share_dst,
            last_updated_dst,
            reward_mint_dst,
            reward_token_ata_dst,
            pda_account_dst,
        ) = mut_array_refs![dst, 1, 1, 8, 8, 8, 8, 8, 8, 32, 32, 32];

        let PlatformState {
            is_initialized,
            coin_emission_percentage,
            coin_emission_distribution_in_sec,
            reward_accumulation_in_sec,
            total_coin_emission,
            total_staked_nfts,
            reward_per_share,
            last_updated,
            reward_mint,
            reward_token_ata,
            pda_account,
        } = self;

        is_initialized_dst[0] = *is_initialized as u8;
        *coin_emission_percentage_dst = coin_emission_percentage.to_le_bytes();
        *coin_emission_distribution_in_sec_dst = coin_emission_distribution_in_sec.to_le_bytes();
        *reward_accumulation_in_sec_dst = reward_accumulation_in_sec.to_le_bytes();
        *total_coin_emission_dst = total_coin_emission.to_le_bytes();
        *total_staked_nfts_dst = total_staked_nfts.to_le_bytes();
        *reward_per_share_dst = reward_per_share.to_le_bytes();
        *last_updated_dst = last_updated.to_le_bytes();
        reward_mint_dst.copy_from_slice(reward_mint.as_ref());
        reward_token_ata_dst.copy_from_slice(reward_token_ata.as_ref());
        pda_account_dst.copy_from_slice(pda_account.as_ref());
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct UserBaseState {
    pub is_initialized: bool,
    pub user: Pubkey,
    pub user_reward_ata: Pubkey,
    pub total_staked_nfts: u64,
    pub total_nft_points: u64,
    pub total_reward_claimed: u64,
    pub reward_debt: u64,
}

impl Sealed for UserBaseState {}
impl IsInitialized for UserBaseState {
    fn is_initialized(&self) -> bool {
        self.is_initialized
    }
}

impl Pack for UserBaseState {
    const LEN: usize = 97;

    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        let src = array_ref![src, 0, UserBaseState::LEN];
        let (
            is_initialized,
            user,
            user_reward_ata,
            total_staked_nfts,
            total_nft_points,
            total_reward_claimed,
            reward_debt,
        ) = array_refs![src, 1, 32, 32, 8, 8, 8, 8];
        let is_initialized = match is_initialized {
            [0] => false,
            [1] => true,
            _ => return Err(ProgramError::InvalidAccountData),
        };
        Ok(UserBaseState {
            is_initialized,
            user: Pubkey::new_from_array(*user),
            user_reward_ata: Pubkey::new_from_array(*user_reward_ata),
            total_staked_nfts: u64::from_le_bytes(*total_staked_nfts),
            total_nft_points: u64::from_le_bytes(*total_nft_points),
            total_reward_claimed: u64::from_le_bytes(*total_reward_claimed),
            reward_debt: u64::from_le_bytes(*reward_debt),
        })
    }

    fn pack_into_slice(&self, dst: &mut [u8]) {
        let dst = array_mut_ref![dst, 0, UserBaseState::LEN];
        let (
            is_initialized_dst,
            user_dst,
            user_reward_ata_dst,
            total_staked_nft_dst,
            total_nft_points_dst,
            total_reward_claimed_dst,
            reward_debt_dst,
        ) = mut_array_refs![dst, 1, 32, 32, 8, 8, 8, 8];
        let UserBaseState {
            is_initialized,
            user,
            user_reward_ata,
            total_staked_nfts,
            total_nft_points,
            total_reward_claimed,
            reward_debt,
        } = self;

        is_initialized_dst[0] = *is_initialized as u8;
        user_dst.copy_from_slice(user.as_ref());
        user_reward_ata_dst.copy_from_slice(user_reward_ata.as_ref());
        *total_staked_nft_dst = total_staked_nfts.to_le_bytes();
        *total_nft_points_dst = total_nft_points.to_le_bytes();
        *total_reward_claimed_dst = total_reward_claimed.to_le_bytes();
        *reward_debt_dst = reward_debt.to_le_bytes();
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct UserNFTState {
    pub is_initialized: bool,
    pub user: Pubkey,
    pub user_base_state: Pubkey,
    pub nft_ata: Pubkey,
    pub nft_mint: Pubkey,
}

impl Sealed for UserNFTState {}
impl IsInitialized for UserNFTState {
    fn is_initialized(&self) -> bool {
        self.is_initialized
    }
}

impl Pack for UserNFTState {
    const LEN: usize = 129;

    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        let src = array_ref![src, 0, UserNFTState::LEN];
        let (is_initialized, user, user_base_state, nft_ata, nft_mint) =
            array_refs![src, 1, 32, 32, 32, 32];
        let is_initialized = match is_initialized {
            [0] => false,
            [1] => true,
            _ => return Err(ProgramError::InvalidAccountData),
        };
        Ok(UserNFTState {
            is_initialized,
            user: Pubkey::new_from_array(*user),
            user_base_state: Pubkey::new_from_array(*user_base_state),
            nft_ata: Pubkey::new_from_array(*nft_ata),
            nft_mint: Pubkey::new_from_array(*nft_mint),
        })
    }

    fn pack_into_slice(&self, dst: &mut [u8]) {
        let dst = array_mut_ref![dst, 0, UserNFTState::LEN];
        let (is_initialized_dst, user_dst, user_base_state_dst, nft_ata_dst, nft_mint_dst) =
            mut_array_refs![dst, 1, 32, 32, 32, 32];
        let UserNFTState {
            is_initialized,
            user,
            user_base_state,
            nft_ata,
            nft_mint,
        } = self;

        is_initialized_dst[0] = *is_initialized as u8;
        user_dst.copy_from_slice(user.as_ref());
        user_base_state_dst.copy_from_slice(user_base_state.as_ref());
        nft_ata_dst.copy_from_slice(nft_ata.as_ref());
        nft_mint_dst.copy_from_slice(nft_mint.as_ref());
    }
}
