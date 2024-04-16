use anchor_lang::{prelude::*, Discriminator};
pub mod jupiter {
    use super::*;
    declare_id!("JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4");

    #[derive(AnchorDeserialize, AnchorSerialize)]
    pub struct SharedAccountsRoute {
        pub id: u8,
        pub route_plan: Vec<RoutePlanStep>,
        pub in_amount: u64,
        pub quoted_out_amount: u64,
        pub slippage_bps: u16,
        pub platform_fee_bps: u8
    }

    #[derive(AnchorDeserialize, AnchorSerialize)]
    pub struct ExactOutRoute {
        pub route_plan: Vec<RoutePlanStep>,
        pub out_amount: u64,
        pub quoted_in_amount: u64,
        pub slippage_bps: u16,
        pub platform_fee_bps: u8
    }

    #[derive(AnchorDeserialize, AnchorSerialize)]
    pub struct SharedAccountsExactOutRoute {
        pub id: u8,
        pub route_plan: Vec<RoutePlanStep>,
        pub out_amount: u64,
        pub quoted_in_amount: u64,
        pub slippage_bps: u16,
        pub platform_fee_bps: u8
    }

    #[derive(AnchorSerialize, AnchorDeserialize)]
    pub struct RoutePlanStep {
        pub swap: Swap,
        pub percent: u8,
        pub input_index: u8,
        pub output_index: u8
    }

    pub struct SharedAccountsExactOutRouteAccountMetas<'info> {
        pub token_program: &'info AccountMeta,
        pub program_authority: &'info AccountMeta,
        pub user_transfer_authority: &'info AccountMeta,
        pub source_token_account: &'info AccountMeta,
        pub program_source_token_account: &'info AccountMeta,
        pub program_destination_token_account: &'info AccountMeta,
        pub destination_token_account: &'info AccountMeta,
        pub source_mint: &'info AccountMeta,
        pub destination_mint: &'info AccountMeta,
        pub platform_fee_account: Option<&'info AccountMeta>,
        pub token2022_program: Option<&'info AccountMeta>,
        pub event_authority: &'info AccountMeta,
        pub program: &'info AccountMeta,
    }

    impl<'info> TryFrom<&'info Vec<AccountMeta>> for SharedAccountsExactOutRouteAccountMetas<'info> {
        type Error = Error;
    
        fn try_from(value: &'info Vec<AccountMeta>) -> Result<Self> {
            if value.len() < 13 {
                return Err(ProgramError::NotEnoughAccountKeys.into());
            }

            let [
                token_program,
                program_authority,
                user_transfer_authority,
                source_token_account,
                program_source_token_account,
                program_destination_token_account,
                destination_token_account,
                source_mint,
                destination_mint,
                platform_fee_account,
                token2022_program,
                event_authority,
                program,
            ] = [
                &value[0],
                &value[1],
                &value[2],
                &value[3],
                &value[4],
                &value[5],
                &value[6],
                &value[7],
                &value[8],
                &value[9],
                &value[10],
                &value[11],
                &value[12]
            ];
    
            let token2022_program = match token2022_program.pubkey.eq(&ID) {
                true => None,
                false => Some(token2022_program),
            };
    
            let platform_fee_account = match platform_fee_account.pubkey.eq(&ID) {
                true => None,
                false => Some(platform_fee_account),
            };
    
            let accounts = SharedAccountsExactOutRouteAccountMetas {
                token_program,
                program_authority,
                user_transfer_authority,
                source_token_account,
                program_source_token_account,
                program_destination_token_account,
                destination_token_account,
                source_mint,
                destination_mint,
                platform_fee_account,
                token2022_program,
                event_authority,
                program,
            };
    
            Ok(accounts)
        }
    }

    impl Discriminator for SharedAccountsRoute {
        const DISCRIMINATOR: [u8; 8] = [0xc1, 0x20, 0x9b, 0x33, 0x41, 0xd6, 0x9c, 0x81];
    }

    impl Discriminator for ExactOutRoute {
        const DISCRIMINATOR: [u8; 8] = [0xd0, 0x33, 0xef, 0x97, 0x7b, 0x2b, 0xed, 0x5c];
    }

    impl Discriminator for SharedAccountsExactOutRoute {
        const DISCRIMINATOR: [u8; 8] = [0xb0, 0xd1, 0x69, 0xa8, 0x9a, 0x7d, 0x45, 0x3e];
    }

    #[derive(AnchorSerialize, AnchorDeserialize)]
    pub enum Side {
        Bid,
        Ask
    }

    #[derive(AnchorSerialize, AnchorDeserialize)]
    pub enum Swap {
        Saber,
        SaberAddDecimalsDeposit,
        SaberAddDecimalsWithdraw,
        TokenSwap,
        Sencha,
        Step,
        Cropper,
        Raydium,
        Crema { x_to_y: bool },
        Lifinity,
        Mercurial,
        Cykura,
        Serum { side: Side },
        MarinadeDeposit,
        MarinadeUnstake,
        Aldrin { side: Side },
        AldrinV2 { side: Side },
        Whirlpool { a_to_b: bool },
        Invariant { x_to_y: bool },
        Meteora,
        GooseFX,
        DeltaFi { stable: bool },
        Balansol,
        MarcoPolo { x_to_y: bool },
        Dradex { side: Side },
        LifinityV2,
        RaydiumClmm,
        Openbook { side: Side },
        Phoenix { side: Side },
        Symmetry { from_token_id: u64, to_token_id: u64 },
        TokenSwapV2,
        HeliumTreasuryManagementRedeemV0,
        StakeDexStakeWrappedSol,
        StakeDexSwapViaStake { bridge_stake_seed: u32 },
        GooseFXV2,
        Perps,
        PerpsAddLiquidity,
        PerpsRemoveLiquidity,
        MeteoraDlmm,
        OpenBookV2 { side: Side },
        RaydiumClmmV2,
    }
}