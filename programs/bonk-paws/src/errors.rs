use anchor_lang::prelude::*;

#[error_code]
pub enum BonkPawsError {
    #[msg("Invalid amount")]
    InvalidAmount,
    #[msg("Swap IX not found")]
    MissingSwapIx,
    #[msg("Finalize IX not found")]
    MissingFinalizeIx, 
    #[msg("Donate IX not found")]
    MissingDonateIx,
    #[msg("Invalid Program ID")]
    ProgramMismatch,
    #[msg("Invalid instruction")]
    InvalidInstruction,
    #[msg("Invalid number of routes")]
    InvalidRoute,
    #[msg("Invalid slippage")]
    InvalidSlippage,
    #[msg("Invalid Solana amount")]
    InvalidSolanaAmount,
    #[msg("Invalid BONK mint address")]
    InvalidBonkMint,
    #[msg("Invalid BONK account")]
    InvalidBonkAccount,
    #[msg("Invalid BONK ATA")]
    InvalidBonkATA,
    #[msg("Invalid wSOL account")]
    InvalidwSolAccount,
    #[msg("Invalid wSOL balance")]
    InvalidwSolBalance,
    #[msg("Invalid charity address")]
    InvalidCharityAddress,
    #[msg("Invalid lamports balance")]
    InvalidLamportsBalance,
    #[msg("Invalid instruction index")]
    InvalidInstructionIndex,
    #[msg("Signature header mismatch")]
    SignatureHeaderMismatch,
    #[msg("Signature authority mismatch")]
    SignatureAuthorityMismatch,
}