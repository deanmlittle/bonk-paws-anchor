use anchor_lang::prelude::*;

#[account]
pub struct DonationState {
    pub bonk_burned: u64,
    pub bonk_donated: u64,
    pub bonk_matched: u64,

}

impl Space for DonationState {
    const INIT_SPACE: usize = 8 + 8 + 8 + 8;
}

#[account]
pub struct MatchDonationState {
    pub donation_amount: u64,
    pub match_key: Pubkey,
    pub seed: u64,
}

impl Space for MatchDonationState {
    const INIT_SPACE: usize = 8 + 8 + 8 + 8;
}