import * as anchor from "@coral-xyz/anchor";
import { Program, BN } from "@coral-xyz/anchor";
import { BonkPaws } from "../target/types/bonk_paws";
import { ComputeBudgetProgram, Keypair, LAMPORTS_PER_SOL, PublicKey, SystemProgram, Transaction } from "@solana/web3.js";
import { ASSOCIATED_PROGRAM_ID, TOKEN_PROGRAM_ID, associatedAddress } from "@coral-xyz/anchor/dist/cjs/utils/token";
import { associated } from "@coral-xyz/anchor/dist/cjs/utils/pubkey";
import { getOrCreateAssociatedTokenAccount } from "@solana/spl-token";

describe("bonk-paws", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const signer = Keypair.generate();
  const charity = Keypair.generate();
  let donorBonk: PublicKey;
  let poolBonk: PublicKey;
  let poolWsol: PublicKey;

  const connection = anchor.getProvider().connection;

  const confirm = async (signature: string): Promise<string> => {
    const block = await connection.getLatestBlockhash();
    await connection.confirmTransaction({
      signature,
      ...block
    })
    return signature
  }
  
  const log = async(signature: string): Promise<string> => {
    console.log(`Your transaction signature: https://explorer.solana.com/transaction/${signature}?cluster=custom&customUrl=${connection.rpcEndpoint}`);
    return signature;
  }

  const bonk = new PublicKey('DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263');
  const wsol = new PublicKey('So11111111111111111111111111111111111111112');

  const program = anchor.workspace.BonkPaws as Program<BonkPaws>;

  it("Airdrop", async () => {
    await connection.requestAirdrop(signer.publicKey, LAMPORTS_PER_SOL * 10)
    .then(confirm)
    .then(log)
  })
  it("ATAs", async () => {
    await getOrCreateAssociatedTokenAccount(connection, { mint: bonk, owner: signer.publicKey }).
  })
  it("Is initialized!", async () => {
    // Add your test here.
    const tx = new Transaction();
    // Set compute unit limit to 1.4mil
    const setComputeUnitLImitIx = ComputeBudgetProgram.setComputeUnitLimit({
      units: 1_400_000,
    });

    const donateIx = program.methods.donate(
      new BN(1e6),
      new BN(1000),
    )
    .accounts({
      donor: signer.publicKey,
      charity: charity.publicKey,
      bonk,
      wsol,
      donorBonk,
      poolBonk,
      poolWsol,
      tokenProgram: TOKEN_PROGRAM_ID,
      associatedTokenProgram: ASSOCIATED_PROGRAM_ID,
      systemProgram: SystemProgram.programId
    })
    tx.instructions.push();
     await program.methods.donate().rpc();
    console.log("Your transaction signature", tx);
  });
});
