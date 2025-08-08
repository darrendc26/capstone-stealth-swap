import * as anchor from "@coral-xyz/anchor";
import { Program, BN } from "@coral-xyz/anchor";
import { CapstoneStealthSwap } from "../target/types/capstone_stealth_swap";
import { 
  createMint,
  createAssociatedTokenAccount,
  mintTo,
  TOKEN_PROGRAM_ID,
  getAccount,
  ASSOCIATED_TOKEN_PROGRAM_ID 
} from "@solana/spl-token";
import { PublicKey, Signer, SystemProgram } from "@solana/web3.js";
import { assert } from "chai";

describe("onchain", () => {
  anchor.setProvider(anchor.AnchorProvider.env());

  const user: Signer = anchor.web3.Keypair.generate();
  const program = anchor.workspace.CapstoneStealthSwap as Program<CapstoneStealthSwap>;
  const inputToken = anchor.web3.Keypair.generate();
  const outputToken = anchor.web3.Keypair.generate();
  const id = 0;
  const connection = anchor.getProvider().connection;

  let inputTokenMint: PublicKey;
  let userTokenAccount: PublicKey;

  before(async () => {
    // Airdrop SOL to user
    const airdropSignature = await connection.requestAirdrop(
      user.publicKey,
      anchor.web3.LAMPORTS_PER_SOL
    );

    await connection.confirmTransaction({
      signature: airdropSignature,
      blockhash: (await connection.getLatestBlockhash()).blockhash,
      lastValidBlockHeight: (await connection.getLatestBlockhash()).lastValidBlockHeight,
    });
    console.log("User address:", user.publicKey.toBase58());
    console.log("SOL airdrop successful!:",await connection.getBalance(user.publicKey));

    // Create the input token mint
    inputTokenMint = await createMint(
      connection,
      user,
      user.publicKey,  // mint authority
      user.publicKey,  // freeze authority
      9,              // decimals
      inputToken,     // mint keypair
      undefined,
      TOKEN_PROGRAM_ID
    );
    console.log("Input token mint created successfully:", inputTokenMint.toString());

    // Create associated token account for user
    userTokenAccount = await createAssociatedTokenAccount(
      connection,
      user,
      inputTokenMint,  // mint address, not user.publicKey
      user.publicKey,  // owner
      undefined,
      TOKEN_PROGRAM_ID
    );
    console.log("User token account created successfully:", userTokenAccount.toString());
    // Mint some tokens to user's account
    await mintTo(
      connection,
      user,
      inputTokenMint,
      userTokenAccount,
      user.publicKey,  // mint authority
      1000000000,     // amount (1000 tokens with 9 decimals)
      [],
      undefined,
      TOKEN_PROGRAM_ID
    );
    console.log("User token account mint successful:", userTokenAccount.toString());
  });

  it("Create Intent", async () => {

    const ataInfo = await getAccount(connection, userTokenAccount);
console.log("Mint:", ataInfo.mint.toBase58());
console.log("Owner:", ataInfo.owner.toBase58());
console.log("Amount:", ataInfo.amount.toString());

    const [intent, bump] = await PublicKey.findProgramAddressSync(
      [ 
        Buffer.from("intent"), 
        user.publicKey.toBuffer(), 
        // Buffer.from(new BN(id).toArray("le", 8))  // Include the id in seeds
      ],
      program.programId
    );
    console.log("Intent address:", intent.toString());

    const [escrow, escrowBump] = await PublicKey.findProgramAddressSync(
      [ 
        Buffer.from("escrow"), 
        user.publicKey.toBuffer(),           // Use intent key, not user key
        // inputTokenMint.toBuffer()    // Use mint address, not inputToken keypair
      ],
      program.programId
    );
  console.log("Escrow address:", escrow.toString());

  // Ensure these two are the same public key
console.log("Mint in user token account:", (await getAccount(connection, userTokenAccount)).mint.toString());
console.log("inputTokenMint:", inputTokenMint.toString());
    try {
      console.log("Creating intent...");
      await program.methods.createIntent(
        {
          inputToken: inputTokenMint,
          outputToken: outputToken.publicKey,
          inputAmount: new BN(10),
          minReceive: new BN(5),
        },
        new BN(id)
      )
        .accounts({
          user: user.publicKey,
          userTokenAccount: userTokenAccount,
          inputTokenMint: inputTokenMint,
        })
        .signers([user])
        .rpc();

      const intentAccount = await program.account.intent.fetch(intent);
      console.log("Intent created successfully:", intentAccount);

      assert.equal(intentAccount.user.toString(), user.publicKey.toString());
      assert.equal(intentAccount.inputToken.toString(), inputTokenMint.toString());
      assert.equal(intentAccount.outputToken.toString(), outputToken.publicKey.toString());
      assert.equal(intentAccount.inputAmount.toString(), new BN(10).toString());
      assert.equal(intentAccount.minReceive.toString(), new BN(5).toString());
      assert.equal(intentAccount.active, true);
      
    } catch (e) {
      console.error("Error creating intent:", e);
      throw e;
    }
  });
});