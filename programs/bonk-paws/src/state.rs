use anchor_lang::prelude::*;

#[account]
pub struct DonationState {
    pub bonk_burned: u128,
    pub bonk_donated: u128,
    pub sol_donated: u128, // Do we want to track $SOL donated?

}

impl Space for DonationState {
    const INIT_SPACE: usize = 8 + 16 + 16 + 16;
}

#[account]
pub struct MatchDonation {
    pub id: u64,
    pub amount: u64,
}

impl Space for MatchDonation {
    const INIT_SPACE: usize = 8 + 8 + 8;
}