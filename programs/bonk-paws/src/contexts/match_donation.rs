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
        transfer as spl_transfer, Mint, Token, TokenAccount, Transfer as SplTransfer
    }, 
    associated_token::AssociatedToken
};

use crate::{
    constants::{bonk, signing_authority, wsol}, errors::BonkPawsError, programs::jupiter::{
        self, SharedAccountsExactOutRoute, SharedAccountsExactOutRouteAccountMetas
    }, require_discriminator_eq, require_instruction_eq, state::{DonationState, MatchDonationState}
};

#[derive(Accounts)]
pub struct MatchDonation<'info> {
    #[account(mut, address = signing_authority::ID)]
    signer: Signer<'info>,
    #[account(
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
        seeds = [b"donation_state"],
        bump,    
    )]
    donation_state: Account<'info, DonationState>,
    #[account(
        mut,
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
        let swap_ix = load_instruction_at_checked(current_index + 1, &ixs).map_err(|_| BonkPawsError::MissingSwapIx)?;

        // Discriminator check
        require_instruction_eq!(swap_ix, jupiter::ID, SharedAccountsExactOutRoute::DISCRIMINATOR, BonkPawsError::InvalidInstruction);

        // Instruction data checks
        let swap_ix_data = SharedAccountsExactOutRoute::try_from_slice(&swap_ix.data[8..])?;
        require_gte!(50, swap_ix_data.slippage_bps, BonkPawsError::InvalidSlippage);
        require_eq!(swap_ix_data.out_amount, self.match_donation_state.donation_amount, BonkPawsError::InvalidSolanaAmount);

        // Account checks
        let swap_ix_accounts = SharedAccountsExactOutRouteAccountMetas::try_from(&swap_ix.accounts)?;

        // BONK account checks
        require_keys_eq!(swap_ix_accounts.source_mint.pubkey, self.bonk.key(), BonkPawsError::InvalidBonkMint);
        require_keys_eq!(swap_ix_accounts.source_token_account.pubkey, self.signer_bonk.key(), BonkPawsError::InvalidBonkATA);

        // wSOL account checks
        require_keys_eq!(swap_ix_accounts.destination_mint.pubkey, self.wsol.key(), BonkPawsError::InvalidwSolMint);
        require_keys_eq!(swap_ix_accounts.destination_token_account.pubkey, self.signer_wsol.key(), BonkPawsError::InvalidwSolATA);

        // Save sum of BONK vault plus user BONK ATA balance in MatchState PDA for cost comparison in finalization
        self.match_donation_state.donation_amount = self.bonk_vault.amount.checked_add(self.signer_bonk.amount).ok_or(BonkPawsError::Overflow)?;
        //  Send maximum donation amount after slippage to user's ATA
        let max_donation_amount: u64 = swap_ix_data.quoted_in_amount.checked_mul(10050).ok_or(BonkPawsError::Overflow)?.checked_div(10000).ok_or(BonkPawsError::Overflow)?;

        /* 

            Transfer the maximum amount of Bonk needed for the Signer to match the donation
        
        */

        let seeds = &[
            b"donation_state".as_ref(),
            &[bumps.donation_state],
        ];
        let signer_seeds = &[&seeds[..]];

        // Transfer the exact amount needed to match the donation
        let transfer_accounts = SplTransfer {
            from: self.bonk_vault.to_account_info(),
            to: self.signer_bonk.to_account_info(),
            authority: self.donation_state.to_account_info(),
        };
        let transfer_ctx = CpiContext::new_with_signer(self.token_program.to_account_info(), transfer_accounts, signer_seeds);

        spl_transfer(transfer_ctx, max_donation_amount)?;

        /* 
        
            Match Finalize Instruction

        */

        if let Ok(ix) = load_instruction_at_checked(current_index.checked_add(2).ok_or(BonkPawsError::Overflow)?, &ixs) {
            // Instruction checks
            require_instruction_eq!(ix, crate::ID, crate::instruction::FinalizeDonation::DISCRIMINATOR, BonkPawsError::InvalidInstruction);
            // Make sure match donation state key matches
            require_keys_eq!(
                ix.accounts.get(8).ok_or(BonkPawsError::InvalidMatchKey)?.pubkey, 
                self.match_donation_state.key(), 
                BonkPawsError::InvalidMatchKey
            );
        } else {
            return Err(BonkPawsError::MissingFinalizeIx.into());
        }
        
        Ok(())
    }
}