use anchor_lang::{
    prelude::*, 
    solana_program::sysvar::{
        self, 
        instructions::{
            load_current_index_checked, 
            load_instruction_at_checked
        }
    }, 
    Discriminator
};

use anchor_spl::{
    token::{
        burn, transfer as spl_transfer, Burn, Mint, Token, TokenAccount, Transfer as SplTransfer
    }, 
    associated_token::AssociatedToken
};

use crate::{
    constants::bonk, 
    constants::wsol, 
    programs::jupiter::{
        SharedAccountsRoute, 
        self
    },
    require_discriminator_eq, 
    require_instruction_eq, errors::BonkPawsError,
    state::{DonationState, MatchDonationState}
};

#[derive(Accounts)]
pub struct MatchDonation<'info> {
    #[account(mut)]
    signer: Signer<'info>,

    #[account(
        mut,
        address = bonk::ID
    )]
    bonk: Account<'info, Mint>,
    #[account(
        init_if_needed,
        payer = signer,
        associated_token::mint = bonk,
        associated_token::authority = signer,
    )]
    signer_bonk: Account<'info, TokenAccount>,
    #[account(
        address = wsol::ID
    )]
    wsol: Account<'info, Mint>,
    #[account(
        init_if_needed,
        payer = signer,
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
        seeds = [b"match_donation", match_donation_state.seed.to_le_bytes().as_ref()],
        bump,
    )]
    match_donation_state: Account<'info, MatchDonationState>,

    #[account(
        mut,
        associated_token::mint = bonk,
        associated_token::authority = donation_state,
    )]
    bonk_vault: Account<'info, TokenAccount>,

    #[account(address = sysvar::instructions::ID)]
    /// CHECK: InstructionsSysvar account
    instructions: UncheckedAccount<'info>,
    associated_token_program: Program<'info, AssociatedToken>,
    token_program: Program<'info, Token>,
    system_program: Program<'info, System>
}

impl<'info> MatchDonation<'info> {        
    pub fn match_donation(&mut self, bumps: MatchDonationBumps) -> Result<()> {

        /* 
        
            Instruction Introspection

            This is the primary means by which we secure our program,
            enforce atomicity while making a great UX for our users.

        */
        let ixs = self.instructions.to_account_info();

        /*

            Disable CPIs
            
            Although we have taken numerous measures to secure this program,
            we can kill CPI to close off even more attack vectors as our 
            current use case doesn't need it.

        */
        let current_index = load_current_index_checked(&ixs)? as usize;
        require_gte!(current_index, 0, BonkPawsError::InvalidInstructionIndex);
        let current_ix = load_instruction_at_checked(current_index, &ixs)?;
        require!(crate::check_id(&current_ix.program_id), BonkPawsError::ProgramMismatch);

        /* 
        
            Match Jupiter Swap Instruction
            
            Ensure that the next instruction after this one is a swap in the
            Jupiter program. Checks include:

            - Program ID and IX discriminator
            - Token account matching
            - Mint account matching
            - Deposit amount matching
            - Minimum SOL amount matching
            - Max slippage protection

            By matching token accounts against our account struct which already 
            enforces mint constraints, we should be able to deduce the mint
            accounts in the instruction also match. Alas, we check them anyway
            just to be extra safe.

            Basically, the only way this rugs is if Jupiter gets hacked.

        */
        let donation_amount: u64;

        if let Ok(ix) = load_instruction_at_checked(current_index + 1, &ixs) {
            // Instruction checks
            require_instruction_eq!(ix, jupiter::ID, SharedAccountsRoute::DISCRIMINATOR, BonkPawsError::InvalidInstruction);
            let shared_account_route_ix = SharedAccountsRoute::try_from_slice(&ix.data[8..])?;
            require_eq!(shared_account_route_ix.slippage_bps, 50, BonkPawsError::InvalidSlippage);
            require_gte!(shared_account_route_ix.route_plan.len(), 1, BonkPawsError::InvalidRoute);
            require_eq!(shared_account_route_ix.quoted_out_amount.checked_mul(100_000).unwrap(), self.match_donation_state.donation_amount, BonkPawsError::InvalidSolanaAmount);

            // BONK account checks
            require_keys_eq!(ix.accounts.get(7).ok_or(BonkPawsError::InvalidBonkMint)?.pubkey, self.bonk.key(), BonkPawsError::InvalidBonkMint);
            require_keys_eq!(ix.accounts.get(3).ok_or(BonkPawsError::InvalidBonkATA)?.pubkey, self.signer_bonk.key(), BonkPawsError::InvalidBonkATA);

            // wSOL account checks
            require_keys_eq!(ix.accounts.get(8).ok_or(BonkPawsError::InvalidwSolMint)?.pubkey, self.wsol.key(), BonkPawsError::InvalidwSolMint);
            require_keys_eq!(ix.accounts.get(6).ok_or(BonkPawsError::InvalidwSolATA)?.pubkey, self.signer_wsol.key(), BonkPawsError::InvalidwSolATA);

            donation_amount = shared_account_route_ix.in_amount.checked_mul(100_000).unwrap();
        } else {
            return Err(BonkPawsError::MissingSwapIx.into());
        }

        /* 

            Transfer the amount of bonk needed to the Signer to match the donation
            done by the user before and burn 1% of the bonk donated.
        
        */

        let seeds = &[
            b"donation_state".as_ref(),
            &[bumps.donation_state],
        ];
        let signer_seeds = &[&seeds[..]];

        // Transfer the exact amount needed to match the donation
        let transfer_account = SplTransfer {
            from: self.bonk_vault.to_account_info(),
            to: self.signer_bonk.to_account_info(),
            authority: self.donation_state.to_account_info(),
        };
        let transfer_ctx = CpiContext::new_with_signer(self.token_program.to_account_info(), transfer_account, signer_seeds);

        spl_transfer(transfer_ctx, donation_amount)?;

        // Burn 1% of the bonk donated
        let burn_account = Burn {
            mint: self.bonk.to_account_info(),
            from: self.bonk_vault.to_account_info(),
            authority: self.signer.to_account_info(),
        };
        let burn_ctx = CpiContext::new_with_signer(self.token_program.to_account_info(), burn_account, signer_seeds);

        burn(burn_ctx, donation_amount.checked_div(100).ok_or(BonkPawsError::Overflow)?)?;

        // Update the donation state
        self.donation_state.bonk_donated = self.donation_state.bonk_donated.checked_add(donation_amount).ok_or(BonkPawsError::Overflow)?;
        self.donation_state.bonk_matched = self.donation_state.bonk_matched.checked_add(donation_amount).ok_or(BonkPawsError::Overflow)?;
        self.donation_state.bonk_burned = self.donation_state.bonk_burned.checked_add(donation_amount.checked_div(100).ok_or(BonkPawsError::Overflow)?).ok_or(BonkPawsError::Overflow)?;

        /* 
        
            Match Finalize Instruction
            
            We also ensure that after swapping on Jupiter, we clean up our 
            wSOL ATA, sending swapped lamports to the charity address and
            refunding rent-exempt sats to the donor. 
            
            To avoid adding another account for a vault, we simply save the
            "amount" from the wSOL ATA, close it, sending the entire lamport 
            balance to the donor's signing keypair, before finally sending 
            the non-rent except amount from the donor to the charity account.

            We also peform the following checks:

            - Program ID and IX discriminator
            - SOL account matching
            - min_lamports_out balance

        */

        // Ensure we have a finalize ix
        if let Ok(ix) = load_instruction_at_checked(current_index + 2, &ixs) {
            // Instruction checks
            require_instruction_eq!(ix, crate::ID, crate::instruction::FinalizeDonation::DISCRIMINATOR, BonkPawsError::InvalidInstruction);

            // We check if the money are actually sent to the charity in the end
            require_keys_eq!(ix.accounts.get(1).ok_or(BonkPawsError::InvalidCharityAddress)?.pubkey, self.match_donation_state.match_key, BonkPawsError::InvalidCharityAddress);
        } else {
            return Err(BonkPawsError::MissingFinalizeIx.into());
        }
        
        Ok(())
    }
}