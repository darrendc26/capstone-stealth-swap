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
  const solver: Signer = anchor.web3.Keypair.generate();

  let inputTokenMint: PublicKey;
  let userTokenAccount: PublicKey;
  let outputTokenMint: PublicKey;
  let solverOutputTokenAccount: PublicKey;
  let solverInputTokenAccount: PublicKey;
  let userOutputTokenAccount: PublicKey;

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

    // Airdrop SOL to solver
const solverAirdropSig = await connection.requestAirdrop(
  solver.publicKey,
  anchor.web3.LAMPORTS_PER_SOL
);

await connection.confirmTransaction({
  signature: solverAirdropSig,
  ...(await connection.getLatestBlockhash())
});

console.log("Solver address:", solver.publicKey.toBase58());
console.log("SOL airdrop successful!:", await connection.getBalance(solver.publicKey));


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

    // Create the output token mint
    outputTokenMint = await createMint(
      connection,
      user,
      user.publicKey,  // mint authority
      user.publicKey,  // freeze authority
      9,              // decimals
      outputToken,     // mint keypair
      undefined,
      TOKEN_PROGRAM_ID
    );
    console.log("Output token mint created successfully:", outputTokenMint.toString());

    

     userOutputTokenAccount = await createAssociatedTokenAccount(
      connection,
      user,
      outputTokenMint,  // mint address, not user.publicKey
      user.publicKey,  // owner
      undefined,
      TOKEN_PROGRAM_ID
    );
    console.log("User output token account created successfully:", userOutputTokenAccount.toString());
    
        // Get the solver's output token account (where they have the tokens to give)
     solverOutputTokenAccount = await createAssociatedTokenAccount(
      connection,
      solver,
      outputTokenMint,
      solver.publicKey,
      undefined,
      TOKEN_PROGRAM_ID
    );
    console.log("Solver OUTPUT token account created:", solverOutputTokenAccount.toString());

    await mintTo(
      connection,
      user,
      outputTokenMint,
      solverOutputTokenAccount,
      user.publicKey,
      1000,
      [],
      undefined,
      TOKEN_PROGRAM_ID
    );
    console.log("Solver output tokens minted");

    // const userOutputTokenAccount = await createAssociatedTokenAccount(
    //   connection,
    //   user,
    //   outputTokenMint,
    //   user.publicKey,
    //   undefined,
    //   TOKEN_PROGRAM_ID
    // );
    // console.log("User output token account created successfully:", userOutputTokenAccount.toString());

     solverInputTokenAccount = await createAssociatedTokenAccount(
      connection,
      solver,
      inputTokenMint,
      solver.publicKey,
      undefined,
      TOKEN_PROGRAM_ID
    );
    console.log("Solver input token account created successfully:", solverInputTokenAccount.toString());

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

  it("Fill Intent", async () => {
    const [intent, bump] = await PublicKey.findProgramAddressSync(
      [ 
        Buffer.from("intent"), 
        user.publicKey.toBuffer(), 
      ],
      program.programId
    );
    console.log("Intent address:", intent.toString());
    
    const [escrow, escrowBump] = await PublicKey.findProgramAddressSync(
      [ 
        Buffer.from("escrow"), 
        user.publicKey.toBuffer(),
      ],
      program.programId
    );
    console.log("Escrow address:", escrow.toString());

    const [solverOutputEscrow, solverOutputEscrowBump] = await PublicKey.findProgramAddressSync(
      [ 
        Buffer.from("escrow"), 
        solver.publicKey.toBuffer(),
      ],
      program.programId
    );
    console.log("Solver escrow address:", solverOutputEscrow.toString());


    try {
      console.log("Filling intent...");
      await program.methods.fillIntent(
        {
          id: new BN(id),
          inputAmount: new BN(10),
          minReceive: new BN(5),
          receiveAmount: new BN(5),
          inputToken: inputTokenMint,
          outputToken: outputTokenMint, // Use outputTokenMint instead of outputToken.publicKey
          user: user.publicKey,
        })
        .accounts({
          solver: solver.publicKey,
          // user: user.publicKey, // Add the user account
          intent: intent,
          // userReceiveAta: userOutputTokenAccount, // User's output token account
          // solverReceiveAta: solverInputTokenAccount, // Solver's input token account  
          solverOutputAta: solverOutputTokenAccount, // ADDED: Solver's output token account
          userInputEscrow: escrow,
          // solverOutputEscrow: solverOutputEscrow,
          inputTokenMint: inputTokenMint,
          outputTokenMint: outputTokenMint,
     
        })
        .signers([ solver]) // Sign with both solver and user
        .rpc();
        
      console.log("Intent filled successfully! Tx:");
      
      const intentAccount = await program.account.intent.fetch(intent);
      console.log("Intent account:", intentAccount);

      assert.equal(intentAccount.active, false);
        
      // Verify balances
      const userOutputBalance = await connection.getTokenAccountBalance(userOutputTokenAccount);
      const solverInputBalance = await connection.getTokenAccountBalance(solverInputTokenAccount);
      
      console.log("User received:", userOutputBalance.value.amount, "output tokens");
      console.log("Solver received:", solverInputBalance.value.amount, "input tokens");

      assert.equal(userOutputBalance.value.amount, "5");
      assert.equal(solverInputBalance.value.amount, "10");

    } catch (e) {
      console.error("Error filling intent:", e);
      throw e;
    }
  });
});