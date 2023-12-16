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
    mints::wsol, 
    errors::BonkPawsError, require_instruction_eq
};

#[derive(Accounts)]
pub struct Finalize<'info> {
    #[account(mut)]
    donor: Signer<'info>,
    charity: SystemAccount<'info>,
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
    /// CHECK: InstructionsSysvar account
    #[account(address = sysvar::instructions::ID)]
    instructions: UncheckedAccount<'info>,
    associated_token_program: Program<'info, AssociatedToken>,
    token_program: Program<'info, Token>,
    system_program: Program<'info, System>
}

impl<'info> Finalize<'info> {        
    pub fn finalize(&self, min_lamports_out: u64) -> Result<()> {
        /*
        
        Check wSOL and Lamports balances

        It would be exceedingly difficult to hack the program at this
        point, but just in case, we're going to check our lamports and
        wSOL balances to ensure they fall within expected range in the
        donate instruction. min_lamport_out is also introspected from 
        the donate instruction to ensure they match.
        */
        require_gte!(self.pool_wsol.amount, min_lamports_out, BonkPawsError::InvalidwSolBalance);
        require_gte!(self.pool_wsol.get_lamports(), self.pool_wsol.amount, BonkPawsError::InvalidLamportsBalance);

        /*
        
        Match Donate Instruction

        By ensuring we have the Donate instruction, we can ensure
        that this instruction:

        - Won't work with CPI
        - Is directly preceded by a Jupiter Swap instruction
        - Which is directly preceded by a Donate instruction
        - min_lamports_out matches the Donate instruction
        - <insert numerous safety guarantees I can't be bothered to list>
        */
        let ixs = self.instructions.to_account_info();

        let current_index = load_current_index_checked(&ixs)? as usize;
        require_gte!(current_index, 3, BonkPawsError::InvalidInstructionIndex);

        if let Ok(ix) = load_instruction_at_checked(current_index - 2, &ixs) {
            // Instruction checks
            require_instruction_eq!(ix, crate::ID, crate::instruction::Donate::DISCRIMINATOR, BonkPawsError::InvalidInstruction);
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

        // Get the amount of non-rent-exempt SOL
        let donation_amount = self.pool_wsol.amount;
        // Close wSOL account and send to the user
        let close_wsol_accounts = CloseAccount {
            account: self.pool_wsol.to_account_info(),
            destination: self.donor.to_account_info(),
            authority: self.pool_wsol.to_account_info()
        };

        let signer_seeds: [&[&[u8]];1] = [
            &[
                b"pool_wsol", 
                self.donor.to_account_info().key.as_ref(), 
                self.charity.to_account_info().key.as_ref()
            ]
        ];

        let close_wsol_ctx = CpiContext::new_with_signer(
            self.token_program.to_account_info(),
            close_wsol_accounts,
            &signer_seeds
        );

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

        let transfer_ctx = CpiContext::new(
            self.system_program.to_account_info(),
            transfer_accounts
        );

        transfer(transfer_ctx, donation_amount)
    }
}