use anchor_lang::prelude::*;

pub mod contexts;
pub mod programs;
pub mod mints;
pub mod errors;
pub mod macros;
pub mod constants;

use contexts::*;

declare_id!("AVWhsnDDwm7PEaijsyQEv4aJ6YnjvnW4WgL4569mf6Gt");

#[program]
pub mod bonk_paws {
    use super::*;

    pub fn donate(ctx: Context<Donate>, bonk_amount_in: u64, min_lamports_out: u64) -> Result<()> {
        ctx.accounts.match_burn_and_swap(bonk_amount_in, min_lamports_out)
    }

    pub fn finalize(ctx: Context<Finalize>, min_lamports_out: u64) -> Result<()> {
        ctx.accounts.finalize(min_lamports_out)
    }
}