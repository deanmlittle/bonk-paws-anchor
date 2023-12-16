import * as anchor from "@coral-xyz/anchor";
import { Program, BN } from "@coral-xyz/anchor";
import { AddressLookupTableAccount, ComputeBudgetProgram, Connection, Ed25519Program, Keypair, PublicKey, SYSVAR_INSTRUCTIONS_PUBKEY, SystemProgram, TransactionInstruction, TransactionMessage, VersionedTransaction } from "@solana/web3.js";
import { IDL, BonkPaws } from "./idl";
import { ASSOCIATED_TOKEN_PROGRAM_ID, TOKEN_PROGRAM_ID, getAssociatedTokenAddressSync } from "@solana/spl-token";

const API_ENDPOINT = "https://quote-api.jup.ag/v6";

const connection = new Connection("https://api.mainnet-beta.solana.com", "confirmed");

/* Define our keys */

let wallet = Keypair.fromSecretKey(new Uint8Array([
  107, 124, 200,  32,  31,  45,  98, 103,  55, 227, 228,
  154, 121, 245,  78,  40,  25, 138, 115, 102,  67, 192,
  115, 128,  60, 137,  15, 181, 249, 147,   9,  77,  47,
   26,  39, 232,  45, 169,   1, 211,  40,  34,  48,  31,
   26, 105, 170,  81, 126,  33, 104, 147, 102,  14,  28,
   96, 137, 225, 240,  33, 153, 135, 190, 231
]));

// pawsyGgeZrkzVnfNt88BW4pihiUP5LLYAoddVBhjfJN / 0xc30ae5cac0f5cbc585a3c64a75ac217d4834435b45c74e89b9a64f8680f8997
const BONK_SIGNER = Keypair.fromSecretKey(new Uint8Array([25,155,170,242,135,13,252,244,233,109,183,245,77,233,32,190,188,215,185,85,57,68,127,75,236,212,53,71,96,190,160,225,12,48,174,92,172,15,92,188,88,90,60,100,167,90,194,23,212,131,68,53,180,92,116,232,155,154,100,248,104,15,137,151]));
const CHARITY_ADDRESS = Keypair.generate().publicKey;

anchor.setProvider(new anchor.AnchorProvider(connection, new anchor.Wallet(wallet), {}));

console.log(BONK_SIGNER.publicKey);

const programId = new PublicKey("AVWhsnDDwm7PEaijsyQEv4aJ6YnjvnW4WgL4569mf6Gt");
const program = new anchor.Program<BonkPaws>(IDL, programId, anchor.getProvider());

const getQuote = async (
  fromMint: PublicKey,
  toMint: PublicKey,
  amount: number
): Promise<SwapQuote> => {
  return fetch(
    `${API_ENDPOINT}/quote?outputMint=${toMint.toBase58()}&inputMint=${fromMint.toBase58()}&amount=${amount}&slippage=0.5&maxAccounts=54&destinationTokenAccount=${wallet.publicKey.toBase58()}`
  ).then(async (response) => await response.json() as SwapQuote );
};

const getSwapIx = async (user: PublicKey, quote: any) => {
  const data = {
    quoteResponse: quote,
    userPublicKey: user.toBase58(),
  };

  return fetch(`${API_ENDPOINT}/swap-instructions`, {
    method: "POST",
    headers: {
      Accept: "application/json",
      "Content-Type": "application/json",
    },
    body: JSON.stringify(data),
  }).then(async (response) => await response.json() as SwapInstruction);
};

const signatureInstruction = Ed25519Program.createInstructionWithPrivateKey({
  privateKey: BONK_SIGNER.secretKey,
  message: CHARITY_ADDRESS.toBuffer(),
})

const BONK = new PublicKey('DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263');
const wSOL = new PublicKey('So11111111111111111111111111111111111111112');

const donorBonk = getAssociatedTokenAddressSync(BONK, wallet.publicKey);
const poolBonk = getAssociatedTokenAddressSync(BONK, BONK_SIGNER.publicKey);
const poolWsol = getAssociatedTokenAddressSync(wSOL, BONK_SIGNER.publicKey);

const deserializeInstruction = (instruction) => {
  return new TransactionInstruction({
    programId: new PublicKey(instruction.programId),
    keys: instruction.accounts.map((key) => ({
      pubkey: new PublicKey(key.pubkey),
      isSigner: key.isSigner,
      isWritable: key.isWritable,
    })),
    data: Buffer.from(instruction.data, "base64"),
  });
};

const getAddressLookupTableAccounts = async (
  keys: string[]
): Promise<AddressLookupTableAccount[]> => {
  const addressLookupTableAccountInfos =
    await connection.getMultipleAccountsInfo(
      keys.map((key) => new PublicKey(key))
    );

  return addressLookupTableAccountInfos.reduce((acc, accountInfo, index) => {
    const addressLookupTableAddress = keys[index];
    if (accountInfo) {
      const addressLookupTableAccount = new AddressLookupTableAccount({
        key: new PublicKey(addressLookupTableAddress),
        state: AddressLookupTableAccount.deserialize(accountInfo.data),
      });
      acc.push(addressLookupTableAccount);
    }

    return acc;
  }, new Array<AddressLookupTableAccount>());
};

(async () => { 
  // Get quote
  const quote = await getQuote(
      BONK,
      wSOL,
      1e10
  );

  console.log(quote);

  const {
      tokenLedgerInstruction, // If you are using `useTokenLedger = true`.
      computeBudgetInstructions, // The necessary instructions to setup the compute budget.
      setupInstructions, // Setup missing ATA for the users.
      swapInstruction: swapInstructionPayload, // The actual swap instruction.
      cleanupInstruction, // Unwrap the SOL if `wrapAndUnwrapSol = true`.
      addressLookupTableAddresses, // The lookup table addresses that you can use if you are using versioned transaction.
  } = await getSwapIx(
      wallet.publicKey,
      quote
  )

  const addressLookupTableAccounts: AddressLookupTableAccount[] = [];

  addressLookupTableAccounts.push(
    ...(await getAddressLookupTableAccounts(addressLookupTableAddresses))
  );

    const donateInstruction = await program.methods.donate(new BN(quote.inAmount), new BN(quote.outAmount))
    .accounts({
      instructions: SYSVAR_INSTRUCTIONS_PUBKEY,
      donor: wallet.publicKey,
      charity: CHARITY_ADDRESS,
      bonk: BONK,
      wsol: wSOL,
      donorBonk,
      poolBonk,
      poolWsol,
      associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
      tokenProgram: TOKEN_PROGRAM_ID,
      systemProgram: SystemProgram.programId
    })
    .instruction();

    // const finalizeInstruction = await program.methods.finalize(new BN(quote.outAmount))
    // .accounts({
    //   instructions: SYSVAR_INSTRUCTIONS_PUBKEY,
    //   donor: wallet.publicKey,
    //   charity: CHARITY_ADDRESS,
    //   wsol: wSOL,
    //   poolWsol,
    //   associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
    //   tokenProgram: TOKEN_PROGRAM_ID,
    //   systemProgram: SystemProgram.programId
    // })
    // .instruction();

        const recentBlockhash = (await connection.getLatestBlockhash()).blockhash;

    const messageV0 = new TransactionMessage({
      payerKey: wallet.publicKey,
      recentBlockhash,
      instructions: [
        ComputeBudgetProgram.setComputeUnitLimit({ units: 1_400_000 }),
        ComputeBudgetProgram.setComputeUnitPrice({ microLamports: 1458 }),
        signatureInstruction,
        donateInstruction,
        deserializeInstruction(swapInstructionPayload),
        // finalizeInstruction
      ],
    }).compileToV0Message(addressLookupTableAccounts);
    const transaction = new VersionedTransaction(messageV0);
    transaction.sign([wallet]);
    // const recentBlockhash = (await connection.getLatestBlockhash()).blockhash;
    // const messageV0 = new TransactionMessage({
    //   payerKey: wallet.publicKey,
    //   recentBlockhash,
    //   instructions: [
    //   // Create a compute unit limit ix
    //   ComputeBudgetProgram.setComputeUnitLimit({ units: 1_400_000 }),
    //   // Create a compute unit price ix (>=1400 should be good)
    //   ComputeBudgetProgram.setComputeUnitPrice({ microLamports: 1458 }),
    //   // Signature ix
    //   // signatureInstruction,
    //   // Donate ix
    //   // donateInstruction,
    //   // Jup swap ix
    //   // deserializeInstruction(ix.swapInstruction),
    //   // Finalize ix
    //   // finalizeInstruction
    //   ]
    // }).compileToV0Message();
    // try {
    //   let transaction = new VersionedTransaction(messageV0);
    console.log(transaction.serialize());
    //   // transaction.sign([wallet]);
    //   // anchor.getProvider().send(transaction, [wallet]);
    //   // console.log(transaction.);
    // } catch(e) {
    //   console.log(e);
    // }
})()