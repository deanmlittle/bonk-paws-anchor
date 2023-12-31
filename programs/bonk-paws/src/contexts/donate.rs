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
        TokenAccount, 
        Transfer as TransferSPL, 
        transfer as transfer_spl, 
        Token, 
        Mint, 
        Burn, 
        burn
    }, 
    associated_token::AssociatedToken
};

use crate::{
    mints::bonk, 
    mints::wsol, 
    programs::jupiter::{
        SharedAccountsRoute, 
        self
    },
    constants::{ MATCH_THRESHOLD, signing_authority}, 
    require_discriminator_eq, 
    require_instruction_eq, errors::BonkPawsError
};

#[derive(Accounts)]
pub struct Donate<'info> {
    #[account(mut)]
    donor: Signer<'info>,
    charity: SystemAccount<'info>,
    #[account(
        mut,
        address = bonk::ID
    )]
    bonk: Account<'info, Mint>,
    #[account(
        address = wsol::ID
    )]
    donor_bonk: Account<'info, TokenAccount>,
    #[account(
        mut,
        seeds = [b"pool_bonk"],
        bump,
        token::mint = bonk,
        token::authority = pool_bonk
    )]
    pool_bonk: Account<'info, TokenAccount>,
    #[account(
        address = wsol::ID
    )]
    wsol: Account<'info, Mint>,
    #[account(
        mut,
        seeds = [b"pool_wsol", donor.key().as_ref(), charity.key().as_ref()],
        bump,
        token::mint = wsol,
        token::authority = pool_wsol
    )]
    pool_wsol: Account<'info, TokenAccount>,
    #[account(address = sysvar::instructions::ID)]
    /// CHECK: InstructionsSysvar account
    instructions: UncheckedAccount<'info>,
    associated_token_program: Program<'info, AssociatedToken>,
    token_program: Program<'info, Token>,
    system_program: Program<'info, System>
}

impl<'info> Donate<'info> {        
    pub fn match_burn_and_swap(&self, bonk_donation: u64, min_lamports_out: u64) -> Result<()> {
        let signer_seeds: [&[&[u8]];1] = [&[b"pool_bonk"]];

        /* 
        
        Burn 1% of Donation

        After configuring the finaly donation amount, we calculate 1% of 
        it and burn it from our BONK pool

        */
        // Burn 1% of deposit in Bonk
        let burn_accounts = Burn {
            mint: self.bonk.to_account_info(),
            from: self.pool_bonk.to_account_info(),
            authority: self.pool_bonk.to_account_info()
        };

        let burn_ctx = CpiContext::new_with_signer(
            self.token_program.to_account_info(), 
            burn_accounts,
            &signer_seeds);

        burn(burn_ctx, bonk_donation)?;

        /* 
        
        Match Donation

        We need to check to see if the donated amount of BONK exceeds
        the match threshold. If it does, we will match the donation 1:1
        from our BONK pool. We will also count this as the donation
        amount in our burn calculation. 
        
        To save on resources, instead of creating a vault for the matched
        BONK, we simply send it to the donor's ATA. 
        
        This is secure by virtue of the transaction only being able
        to succeed if there also a Jupiter swap and Finalize instruction 
        present after the Donate instruction. We also don't allow CPI.

        */

        let mut swap_amount = bonk_donation;

        if bonk_donation >= MATCH_THRESHOLD {
            let transfer_accounts = TransferSPL {
                from: self.pool_bonk.to_account_info(),
                to: self.donor_bonk.to_account_info(),
                authority: self.pool_bonk.to_account_info()
            };


            let transfer_ctx = CpiContext::new_with_signer(
                self.token_program.to_account_info(),
                transfer_accounts,
                &signer_seeds
            );

            transfer_spl(transfer_ctx, bonk_donation)?;

            swap_amount = swap_amount.checked_add(bonk_donation).ok_or(BonkPawsError::InvalidAmount)?;
        }

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
        require_gte!(current_index, 1, BonkPawsError::InvalidInstructionIndex);
        let current_ix = load_instruction_at_checked(current_index, &ixs)?;
        require!(crate::check_id(&current_ix.program_id), BonkPawsError::ProgramMismatch);

        /*
        Make sure previous IX is an ed25519 signature verifying the donation address
        */
        
        // Check program ID is instructions sysvar
        let signature_ix = load_instruction_at_checked(current_index-1, &ixs)?;
        require!(sysvar::instructions::check_id(&signature_ix.program_id), BonkPawsError::ProgramMismatch);

        // Ensure a strict instruction header format: 
        require!([0x01, 0x00, 0x30, 0x00, 0xff, 0xff, 0x10, 0x00, 0xff, 0xff, 0x70, 0x00, 0x20, 0x00, 0xff, 0xff].eq(&signature_ix.data[0..16]), BonkPawsError::SignatureHeaderMismatch);

        // Ensure signing authority is correct
        require!(signing_authority::ID.to_bytes().eq(&signature_ix.data[16..48]), BonkPawsError::SignatureAuthorityMismatch);

        // Get charity account key for later verification:
        let mut charity_key_data: [u8;32] = [0u8;32]; 
        charity_key_data.copy_from_slice(&signature_ix.data[0x70..0x90]);
        let charity_key = Pubkey::from(charity_key_data);

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

        if let Ok(ix) = load_instruction_at_checked(current_index + 1, &ixs) {
            // Instruction checks
            require_instruction_eq!(ix, jupiter::ID, SharedAccountsRoute::DISCRIMINATOR, BonkPawsError::InvalidInstruction);
            let shared_account_route_ix = SharedAccountsRoute::try_from_slice(&ix.data[8..])?;
            require_eq!(shared_account_route_ix.in_amount, swap_amount, BonkPawsError::InvalidAmount);
            require_eq!(shared_account_route_ix.slippage_bps, 50, BonkPawsError::InvalidSlippage);
            require_gte!(shared_account_route_ix.route_plan.len(), 1, BonkPawsError::InvalidRoute);
            require_gte!(shared_account_route_ix.quoted_out_amount, min_lamports_out, BonkPawsError::InvalidSolanaAmount);

            // BONK account checks
            require_keys_eq!(ix.accounts.get(7).ok_or(BonkPawsError::InvalidBonkMint)?.pubkey, self.bonk.key(), BonkPawsError::InvalidBonkMint);
            require_keys_eq!(ix.accounts.get(3).ok_or(BonkPawsError::InvalidBonkATA)?.pubkey, self.donor_bonk.key(), BonkPawsError::InvalidBonkATA);
            require_keys_eq!(ix.accounts.get(2).ok_or(BonkPawsError::InvalidBonkAccount)?.pubkey, self.donor_bonk.key(), BonkPawsError::InvalidBonkAccount);
        } else {
            return Err(BonkPawsError::MissingSwapIx.into());
        }

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
            require_instruction_eq!(ix, crate::ID, crate::instruction::Finalize::DISCRIMINATOR, BonkPawsError::InvalidInstruction);
            let finalize_ix = crate::instruction::Finalize::try_from_slice(&ix.data[8..])?;

            require_eq!(min_lamports_out, finalize_ix.min_lamports_out, BonkPawsError::InvalidLamportsBalance);

            // SOL account checks
            require_keys_eq!(ix.accounts.get(1).ok_or(BonkPawsError::InvalidCharityAddress)?.pubkey, charity_key);
        } else {
            return Err(BonkPawsError::MissingFinalizeIx.into());
        }
        Ok(())
    }
}