pub const MIN_MATCH_THRESHOLD: u64 = 100_000_000;
pub const MAX_MATCH_THRESHOLD: u64 = 1_000_000_000_000_000;
pub const MIN_BURN_THRESHOLD: u64 = 100_000_000;
pub const MAX_BURN_THRESHOLD: u64 = 1_000_000_000_000_000;
pub const BURN_DENOMINATOR: u16 = 100;

pub mod signing_authority {
    use anchor_lang::declare_id;

    declare_id!("pawsyGgeZrkzVnfNt88BW4pihiUP5LLYAoddVBhjfJN");
}