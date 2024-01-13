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
        Token, 
        Mint, 
        Burn, 
        burn
    }, 
    associated_token::{AssociatedToken, get_associated_token_address}
};

use crate::{
    mints::bonk, 
    mints::wsol, 
    programs::jupiter::{
        SharedAccountsRoute, 
        self
    },
    constants::signing_authority, 
    require_discriminator_eq, 
    require_instruction_eq, errors::BonkPawsError,
    state::{DonationState, MatchDonation}
};

#[derive(Accounts)]
#[instruction(_seed: u64)]
pub struct FinalizeSolDonation<'info> {
    #[account(mut)] // Hardcode to Bonk Wallet
    authority: Signer<'info>,
    #[account(mut)]
    charity: SystemAccount<'info>,
    #[account(
        mut,
        address = bonk::ID
    )]
    bonk: Account<'info, Mint>,
    #[account(
        mut,
        associated_token::mint = bonk,
        associated_token::authority = authority,
    )]
    authority_bonk: Account<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [b"state"],
        bump,    
    )]
    bonk_state: Account<'info, DonationState>,
    #[account(
        mut,
        close = authority,
        seeds = [b"match_donation", _seed.to_le_bytes().as_ref()],
        bump,
    )]
    match_donation_state: Account<'info, MatchDonation>,

    #[account(address = sysvar::instructions::ID)]
    /// CHECK: InstructionsSysvar account
    instructions: UncheckedAccount<'info>,
    associated_token_program: Program<'info, AssociatedToken>,
    token_program: Program<'info, Token>,
    system_program: Program<'info, System>
}

impl<'info> FinalizeSolDonation<'info> {        
    pub fn finalize_sol_donation(&mut self, _seed: u64, bonk_donation: u64) -> Result<()> {
        
        // Burn 1% of deposit in Bonk -> Do we want to burn it only if you match?
        let burn_accounts = Burn {
            mint: self.bonk.to_account_info(),
            from: self.authority_bonk.to_account_info(),
            authority: self.authority.to_account_info()
        };

        let burn_ctx = CpiContext::new(self.token_program.to_account_info(), burn_accounts);

        burn(burn_ctx, bonk_donation.checked_div(100).unwrap())?; // ???
        self.bonk_state.bonk_burned = self.bonk_state.bonk_burned.checked_add(bonk_donation.checked_div(100).unwrap() as u128).unwrap();

        // Updated the BonkState
        self.bonk_state.bonk_donated = self.bonk_state.bonk_donated.checked_add(bonk_donation as u128).unwrap();

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

        let swap_amount = bonk_donation.checked_mul(99).unwrap().checked_div(100).unwrap();
        let min_lamports_out = self.match_donation_state.amount.checked_mul(99).unwrap().checked_div(100).unwrap();
        let max_lamports_out = min_lamports_out + min_lamports_out.checked_mul(5).unwrap().checked_div(10).unwrap();


        if let Ok(ix) = load_instruction_at_checked(current_index + 1, &ixs) {
            // Instruction checks
            require_instruction_eq!(ix, jupiter::ID, SharedAccountsRoute::DISCRIMINATOR, BonkPawsError::InvalidInstruction);
            let shared_account_route_ix = SharedAccountsRoute::try_from_slice(&ix.data[8..])?;
            require_eq!(shared_account_route_ix.in_amount, swap_amount, BonkPawsError::InvalidAmount);
            require_eq!(shared_account_route_ix.slippage_bps, 50, BonkPawsError::InvalidSlippage);
            require_gte!(shared_account_route_ix.route_plan.len(), 1, BonkPawsError::InvalidRoute);
            require!(shared_account_route_ix.quoted_out_amount > min_lamports_out && shared_account_route_ix.quoted_out_amount < max_lamports_out, BonkPawsError::InvalidSolanaAmount);

            // BONK account checks
            require_keys_eq!(ix.accounts.get(7).ok_or(BonkPawsError::InvalidBonkMint)?.pubkey, self.bonk.key(), BonkPawsError::InvalidBonkMint);
            require_keys_eq!(ix.accounts.get(3).ok_or(BonkPawsError::InvalidBonkATA)?.pubkey, self.authority_bonk.key(), BonkPawsError::InvalidBonkATA);
            require_keys_eq!(ix.accounts.get(2).ok_or(BonkPawsError::InvalidBonkAccount)?.pubkey, self.authority.key(), BonkPawsError::InvalidBonkAccount);

            // wSOL account checks
            let wsol_ata = get_associated_token_address(&self.authority.key(), &wsol::ID);

            require_keys_eq!(ix.accounts.get(8).ok_or(BonkPawsError::InvalidwSolMint)?.pubkey, wsol::ID, BonkPawsError::InvalidwSolMint);
            require_keys_eq!(ix.accounts.get(6).ok_or(BonkPawsError::InvalidwSolATA)?.pubkey, wsol_ata, BonkPawsError::InvalidwSolATA);
        } else {
            return Err(BonkPawsError::MissingSwapIx.into());
        }

        /* 
        
        Transfer to Charity Instruction - ToDo Docs

        Ensure that the next instruction after the swap is a transfer
        */

        // Ensure we have a finalize ix
        if let Ok(ix) = load_instruction_at_checked(current_index + 3, &ixs) {
            // Instruction checks
            require_keys_eq!(ix.program_id, self.system_program.key(),  BonkPawsError::InvalidInstruction);
            require_eq!(ix.data[0], 2u8, BonkPawsError::InvalidInstruction);

            require!(ix.data[4..12].eq(&min_lamports_out.to_le_bytes()), BonkPawsError::InvalidAmount);
            require_keys_eq!(ix.accounts.get(1).unwrap().pubkey, charity_key, BonkPawsError::InvalidCharityAddress);
        } else {
            return Err(BonkPawsError::MissingFinalizeIx.into());
        }
        
        Ok(())
    }
}