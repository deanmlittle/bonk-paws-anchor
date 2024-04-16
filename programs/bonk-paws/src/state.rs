use anchor_lang::prelude::*;

#[account]
pub struct DonationState {
    pub bonk_burned: u64,
    pub sol_donated: u64,
    pub sol_matched: u64,
}

impl Space for DonationState {
    const INIT_SPACE: usize = 8 + 8 + 8 + 8;
}

#[account]
pub struct MatchDonationState {
    pub id: u64,
    pub donation_amount: u64,
    pub match_key: Pubkey,
    pub seed: u64,
}

impl Space for MatchDonationState {
    const INIT_SPACE: usize = 8 + 8 + 8 + 32 + 8;
}

#[account]
pub struct DonationHistory {
    pub donor: Pubkey,
    pub id: u64,
    pub donation_amount: u64,
    pub timestamp: i64
}

impl Space for DonationHistory {
    const INIT_SPACE: usize = 8 + 32 + 8 + 8 + 8;
}