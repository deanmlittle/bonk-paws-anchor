use anchor_lang::{
    prelude::*, 
    solana_program::sysvar::{
        self, 
        instructions::{
            load_current_index_checked, 
            load_instruction_at_checked
        }
    },
    system_program::{
        Transfer,
        transfer
    },
    Discriminator
};

use anchor_spl::{
    token::{
        Transfer as SplTransfer,
        transfer as spl_transfer,
        Burn,
        burn,
        TokenAccount,
        Token,
        Mint, 
        CloseAccount, 
        close_account
    }, 
    associated_token::AssociatedToken
};

use crate::{
    constants::{bonk, signing_authority, wsol}, errors::BonkPawsError, programs::jupiter::{
        self, SharedAccountsExactOutRoute
    }, require_discriminator_eq, require_instruction_eq, state::{DonationState, MatchDonationState}
};


#[derive(Accounts)]
pub struct FinalizeDonation<'info> {
    #[account(
        mut,
        address = signing_authority::ID
    )]
    signer: Signer<'info>,
    #[account(mut)]
    match_key: SystemAccount<'info>,
    #[account(
        mut,
        address = bonk::ID
    )]
    bonk: Account<'info, Mint>,
    #[account(
        mut,
        associated_token::mint = bonk,
        associated_token::authority = signer,
    )]
    signer_bonk: Account<'info, TokenAccount>,
    #[account(
        mut,
        associated_token::mint = bonk,
        associated_token::authority = donation_state,
    )]
    bonk_vault: Account<'info, TokenAccount>,
    #[account(
        address = wsol::ID
    )]
    wsol: Account<'info, Mint>,
    #[account(
        mut,
        associated_token::mint = wsol,
        associated_token::authority = signer,
    )]
    signer_wsol: Account<'info, TokenAccount>,
    #[account(
        mut,
        seeds = [b"donation_state"],
        bump,    
    )]
    donation_state: Account<'info, DonationState>,
    #[account(
        mut,
        close = signer,
        has_one = match_key,
        seeds = [b"match_donation", match_donation_state.seed.to_le_bytes().as_ref()],
        bump,
    )]
    match_donation_state: Account<'info, MatchDonationState>,
    /// CHECK: InstructionsSysvar account
    #[account(address = sysvar::instructions::ID)]
    instructions: UncheckedAccount<'info>,
    associated_token_program: Program<'info, AssociatedToken>,
    token_program: Program<'info, Token>,
    system_program: Program<'info, System>
}

impl<'info> FinalizeDonation<'info> {        
    pub fn finalize_donation(&mut self, bumps: FinalizeDonationBumps) -> Result<()> {
        
        /*
        
            Match Donate Instruction

            By ensuring we have the Donate instruction, we can ensure
            that this instruction:

            - Won't work with CPI
            - Is directly preceded by a Jupiter Swap instruction
            - Which is directly preceded by a Donate instruction
            - Contains the correct match Donation amount
        
        */

        let ixs = self.instructions.to_account_info();

        let current_index = load_current_index_checked(&ixs)? as usize;
        require_gte!(current_index, 2, BonkPawsError::InvalidInstructionIndex);

        // Make sure we have a match IX
        if let Ok(ix) = load_instruction_at_checked(current_index.checked_sub(2).ok_or(BonkPawsError::Overflow)?, &ixs) {
            // Instruction checks
            require_instruction_eq!(ix, crate::ID, crate::instruction::MatchDonation::DISCRIMINATOR, BonkPawsError::InvalidInstruction);
        } else {
            return Err(BonkPawsError::MissingDonateIx.into());
        }

        // Make sure we have a swap IX
        let swap_ix = load_instruction_at_checked(
            current_index.checked_sub(1).ok_or(BonkPawsError::Overflow)?,
            &ixs
        ).map_err(|_| BonkPawsError::MissingDonateIx)?;
        require_instruction_eq!(swap_ix, jupiter::ID, SharedAccountsExactOutRoute::DISCRIMINATOR, BonkPawsError::InvalidInstruction);
        let swap_ix_data = SharedAccountsExactOutRoute::try_from_slice(&swap_ix.data[8..])?;
        
        let donation_amount = swap_ix_data.out_amount;

        /*
        
            Close wSOL Account

            To avoid adding an intermediate vault account and an additional
            transfer instruction, we will simply close the wSOL ATA, sending 
            its entire lamports balance to the donor, followed by refunding
            the non-rent-exempt lamports to the charity account. To do this,
            we must first save the "token" balance of the wSOL account which
            should be slightly lower than the total lamports balance. 

        */

        // Close wSOL account and send to the signer
        let close_wsol_accounts = CloseAccount {
            account: self.signer_wsol.to_account_info(),
            destination: self.signer.to_account_info(),
            authority: self.signer.to_account_info()
        };
        let close_wsol_ctx = CpiContext::new(self.token_program.to_account_info(), close_wsol_accounts);

        close_account(close_wsol_ctx)?;

        /*
        
            Donate Native SOL To Charity Account

        */
        
        let transfer_accounts = Transfer {
            from: self.signer.to_account_info(),
            to: self.match_key.to_account_info()
        };

        let transfer_ctx = CpiContext::new(self.system_program.to_account_info(), transfer_accounts);

        transfer(transfer_ctx, donation_amount)?;

        // Transfer surplus bonk from the signer back to the vault
        let transfer_accounts = SplTransfer {
            from: self.signer_bonk.to_account_info(),
            to: self.bonk_vault.to_account_info(),
            authority: self.signer.to_account_info(),
        };
        let transfer_ctx = CpiContext::new(self.token_program.to_account_info(), transfer_accounts);

        spl_transfer(transfer_ctx, self.signer_bonk.amount)?;

        // Calculate how much BONK was spent to match
        let bonk_matched_amount: u64 = self.match_donation_state.donation_amount.checked_sub(self.bonk_vault.amount).ok_or(BonkPawsError::Overflow)?;
        // Calculate burn amount
        let bonk_burn_amount: u64 = bonk_matched_amount.checked_div(100).ok_or(BonkPawsError::Overflow)?;

        // Burn 1% of the bonk donated
        let seeds = &[
            b"donation_state".as_ref(),
            &[bumps.donation_state],
        ];
        let signer_seeds = &[&seeds[..]];

        let burn_accounts = Burn {
            mint: self.bonk.to_account_info(),
            from: self.bonk_vault.to_account_info(),
            authority: self.donation_state.to_account_info(),
        };
        let burn_ctx = CpiContext::new_with_signer(self.token_program.to_account_info(), burn_accounts, signer_seeds);

        burn(burn_ctx, bonk_burn_amount)?;

        // Update the donation state
        self.donation_state.sol_matched = self.donation_state.sol_matched.checked_add(donation_amount).ok_or(BonkPawsError::Overflow)?;
        self.donation_state.bonk_burned = self.donation_state.bonk_burned.checked_add(bonk_burn_amount).ok_or(BonkPawsError::Overflow)?;

        Ok(())
    }
}