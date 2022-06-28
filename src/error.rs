use solana_program::program_error::ProgramError;
use thiserror::Error;

#[derive(Error, Debug, Copy, Clone)]
pub enum NFTStakingContractError {
    #[error("Invalid Instruction")]
    InvalidInstruction,

    #[error("Invalid Program Data Args")]
    InvalidArgs,

    #[error("Incorrect token ATA Owner")]
    IncorrectATAOwner,

    #[error("Invalid token ata, mint mismatch")]
    InvalidTokenATA,

    #[error("Not Admin")]
    NotAdmin,

    #[error("Equality Mismatch")]
    EqualityMismatch,

    #[error("Math Error")]
    MathError,

    #[error("User Not Signer")]
    UserNotSigner,

    #[error("Invalid State Account")]
    InvalidStateAccount,

    #[error("Bool Equality Mismatch")]
    BoolEqualityMismatch,

    #[error("Invalid NFT Metadata")]
    InvalidNFTMetadata,
}

impl From<NFTStakingContractError> for ProgramError {
    fn from(e: NFTStakingContractError) -> Self {
        ProgramError::Custom(e as u32)
    }
}
