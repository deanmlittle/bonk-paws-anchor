use anchor_lang::prelude::*;

pub mod contexts;
pub mod programs;
pub mod errors;
pub mod macros;
pub mod constants;
pub mod state;

use contexts::*;

declare_id!("bfpP4enQQ7ajSLaMWhAy6wYZYmRV6uxVid3r5hphh68");

#[program]
pub mod bonk_paws {
    use super::*;

    pub fn donate(ctx: Context<DonateSol>, seeds: u64, sol_donation: u64) -> Result<()> {
        ctx.accounts.donate_sol(seeds, sol_donation)
    }

    pub fn match_donation(ctx: Context<MatchDonation>) -> Result<()> {
        ctx.accounts.match_donation(ctx.bumps)
    }

    pub fn finalize_donation(ctx: Context<FinalizeDonation>) -> Result<()> {
        ctx.accounts.finalize_donation(ctx.bumps)
    }
}