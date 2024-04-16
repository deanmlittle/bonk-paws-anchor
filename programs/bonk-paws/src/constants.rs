pub const MIN_MATCH_THRESHOLD: u64 = 0;
pub const MAX_MATCH_THRESHOLD: u64 = 1_000_000_000_000_000;
pub const MIN_BURN_THRESHOLD: u64 = 100_000_000;
pub const MAX_BURN_THRESHOLD: u64 = 1_000_000_000_000_000;
pub const BURN_DENOMINATOR: u16 = 100;

use anchor_lang::declare_id;

pub mod signing_authority {
    use super::*;
    declare_id!("bfp1sHRTCvq7geo1hkBuaYbiFdEhsfeoidqimJDuSEy");
}

pub mod bonk {
    use super::*;
    declare_id!("DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263");
}

pub mod wsol {
    use super::*;
    declare_id!("So11111111111111111111111111111111111111112");
}

pub mod ed25519program {
    use super::*;
    declare_id!("Ed25519SigVerify111111111111111111111111111");
}