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
pub struct MatchDonation {
    pub id: u64,
    pub amount: u64,
}

impl Space for MatchDonation {
    const INIT_SPACE: usize = 8 + 8 + 8;
}