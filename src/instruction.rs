use crate::error::NFTStakingContractError;
use solana_program::program_error::ProgramError;

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct PlatformData {
    pub percent: u8,
    pub distribution: u64,
    pub accumulation: u64,
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum NFTStakingContractInstruction {
    InitializePlatform(PlatformData),
    StakeNFT,
    UnstakeNFT,
    ClaimReward,
}

impl NFTStakingContractInstruction {
    pub fn unpack_instruction_data(ins_data: &[u8]) -> Result<Self, ProgramError> {
        let (ins_no, data) = ins_data
            .split_first()
            .ok_or(NFTStakingContractError::InvalidInstruction)?;

        Ok(match ins_no {
            0 => Self::InitializePlatform(Self::get_platform_data(data)?),
            1 => Self::StakeNFT,
            2 => Self::UnstakeNFT,
            3 => Self::ClaimReward,
            _ => return Err(NFTStakingContractError::InvalidInstruction.into()),
        })
    }

    fn get_platform_data(data: &[u8]) -> Result<PlatformData, ProgramError> {
        let percent = data
            .get(0..1)
            .and_then(|slice| slice.try_into().ok())
            .map(u8::from_le_bytes)
            .ok_or(NFTStakingContractError::InvalidArgs)?;
        let distribution = data
            .get(1..9)
            .and_then(|slice| slice.try_into().ok())
            .map(u64::from_le_bytes)
            .ok_or(NFTStakingContractError::InvalidArgs)?;
        let accumulation = data
            .get(9..17)
            .and_then(|slice| slice.try_into().ok())
            .map(u64::from_le_bytes)
            .ok_or(NFTStakingContractError::InvalidArgs)?;

        Ok(PlatformData {
            percent,
            distribution,
            accumulation,
        })
    }
}
