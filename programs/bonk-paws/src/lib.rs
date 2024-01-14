use anchor_lang::prelude::*;

pub mod contexts;
pub mod programs;
pub mod mints;
pub mod errors;
pub mod macros;
pub mod constants;
pub mod state;

use contexts::*;

declare_id!("AVWhsnDDwm7PEaijsyQEv4aJ6YnjvnW4WgL4569mf6Gt");

#[program]
pub mod bonk_paws {
    use super::*;

    pub fn donate(ctx: Context<Donate>, bonk_amount_in: u64, min_lamports_out: u64) -> Result<()> {
        ctx.accounts.match_burn_and_swap(bonk_amount_in, min_lamports_out)
    }

    pub fn donate_sol(ctx: Context<DonateSol>, seeds: u64, sol_donation: u64, id: u64) -> Result<()> {
        ctx.accounts.donate_sol(seeds, sol_donation, id)
    }

    pub fn match_sol(ctx: Context<MatchSolDonation>, seeds: u64, bonk_donation: u64) -> Result<()> {
        ctx.accounts.match_sol_donation(seeds, bonk_donation)
    }

    pub fn finalize(ctx: Context<Finalize>) -> Result<()> {
        ctx.accounts.finalize()
    }
}