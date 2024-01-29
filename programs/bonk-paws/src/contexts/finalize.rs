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
        TokenAccount,
        Token, 
        Mint, 
        CloseAccount, 
        close_account
    }, 
    associated_token::AssociatedToken
};

use crate::{
    constants::wsol, 
    programs::jupiter::{
        SharedAccountsRoute, 
        self
    },
    require_discriminator_eq, 
    require_instruction_eq, errors::BonkPawsError,
};


#[derive(Accounts)]
pub struct FinalizeDonation<'info> {
    #[account(mut)]
    donor: Signer<'info>,
    #[account(mut)]
    charity: SystemAccount<'info>,

    #[account(
        address = wsol::ID
    )]
    wsol: Account<'info, Mint>,
    #[account(
        mut,
        associated_token::mint = wsol,
        associated_token::authority = donor,
    )]
    donor_wsol: Account<'info, TokenAccount>,

    /// CHECK: InstructionsSysvar account
    #[account(address = sysvar::instructions::ID)]
    instructions: UncheckedAccount<'info>,
    associated_token_program: Program<'info, AssociatedToken>,
    token_program: Program<'info, Token>,
    system_program: Program<'info, System>
}

impl<'info> FinalizeDonation<'info> {        
    pub fn finalize_donation(&self) -> Result<()> {
        
        /*
        
            Match Donate Instruction

            By ensuring we have the Donate instruction, we can ensure
            that this instruction:

            - Won't work with CPI
            - Is directly preceded by a Jupiter Swap instruction
            - Which is directly preceded by a Donate instruction
            - min_lamports_out matches the Donate instruction
        
        */

        let ixs = self.instructions.to_account_info();

        let current_index = load_current_index_checked(&ixs)? as usize;
        require_gte!(current_index, 2, BonkPawsError::InvalidInstructionIndex);

        if let Ok(ix) = load_instruction_at_checked(current_index - 2, &ixs) {
            // Instruction checks
            require_instruction_eq!(ix, crate::ID, crate::instruction::MatchDonation::DISCRIMINATOR, BonkPawsError::InvalidInstruction);
        } else {
            return Err(BonkPawsError::MissingDonateIx.into());
        }

        let donation_amount: u64;

        if let Ok(ix) = load_instruction_at_checked(current_index - 1, &ixs) {
            require_instruction_eq!(ix, jupiter::ID, SharedAccountsRoute::DISCRIMINATOR, BonkPawsError::InvalidInstruction);
            let shared_account_route_ix = SharedAccountsRoute::try_from_slice(&ix.data[8..])?;
            donation_amount = shared_account_route_ix.quoted_out_amount.checked_mul(100_000).unwrap();
        } else {
            return Err(BonkPawsError::MissingDonateIx.into());
        }

        /*
        
            Close wSOL Account

            To avoid adding an intermediate vault account and an additional
            transfer instruction, we will simply close the wSOL ATA, sending 
            its entire lamports balance to the donor, followed by refunding
            the non-rent-exempt lamports to the charity account. To do this,
            we must first save the "token" balance of the wSOL account which
            should be slightly lower than the total lamports balance. 

        */

        // Close wSOL account and send to the user
        let close_wsol_accounts = CloseAccount {
            account: self.donor_wsol.to_account_info(),
            destination: self.donor.to_account_info(),
            authority: self.donor.to_account_info()
        };
        let close_wsol_ctx = CpiContext::new(self.token_program.to_account_info(), close_wsol_accounts);

        close_account(close_wsol_ctx)?;

        /*
        
            Donate Native SOL To Charity Account

            Finally, we refund the non-rent-exempt balance, or in other
            words, the swapped SOL amount, to the Charity account,
            completing our atomic optional match, swap and burn of funds
            in a single transaction. :)

        */
        
        let transfer_accounts = Transfer {
            from: self.donor.to_account_info(),
            to: self.charity.to_account_info()
        };

        let transfer_ctx = CpiContext::new(self.system_program.to_account_info(), transfer_accounts);

        transfer(transfer_ctx, donation_amount)?;

        Ok(())
    }
}