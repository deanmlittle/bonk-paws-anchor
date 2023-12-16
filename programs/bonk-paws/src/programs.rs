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

    #[derive(AnchorSerialize, AnchorDeserialize)]
    pub struct RoutePlanStep {
        pub swap: Swap,
        pub percent: u8,
        pub input_index: u8,
        pub output_index: u8
    }

    impl Discriminator for SharedAccountsRoute {
        const DISCRIMINATOR: [u8; 8] = [0xc1, 0x20, 0x9b, 0x33, 0x41, 0xd6, 0x9c, 0x81];
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
        MeteoraDlmm
    }
}