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


    inputTokenMint = await createMint(
      connection,
      user,
      user.publicKey, 
      user.publicKey,  
      9,              
      inputToken,   
      undefined,
      TOKEN_PROGRAM_ID
    );
    console.log("Input token mint created successfully:", inputTokenMint.toString());

    userTokenAccount = await createAssociatedTokenAccount(
      connection,
      user,
      inputTokenMint, 
      user.publicKey, 
      undefined,
      TOKEN_PROGRAM_ID
    );
    console.log("User token account created successfully:", userTokenAccount.toString());

    await mintTo(
      connection,
      user,
      inputTokenMint,
      userTokenAccount,
      user.publicKey,  
      1000000000,     
      [],
      undefined,
      TOKEN_PROGRAM_ID
    );
    console.log("User token account mint successful:", userTokenAccount.toString());

    outputTokenMint = await createMint(
      connection,
      user,
      user.publicKey, 
      user.publicKey, 
      9,              
      outputToken,   
      undefined,
      TOKEN_PROGRAM_ID
    );
    console.log("Output token mint created successfully:", outputTokenMint.toString());

    

     userOutputTokenAccount = await createAssociatedTokenAccount(
      connection,
      user,
      outputTokenMint,
      user.publicKey,  
      undefined,
      TOKEN_PROGRAM_ID
    );
    console.log("User output token account created successfully:", userOutputTokenAccount.toString());
    
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
  console.log("User escrow address:", escrow.toString());

  const [solverOutputEscrow, solverOutputEscrowBump] = await PublicKey.findProgramAddressSync(
    [ 
      Buffer.from("solver_escrow"), 
      solver.publicKey.toBuffer(),
      outputTokenMint.toBuffer()
    ],
    program.programId
  );
  console.log("Solver output escrow address:", solverOutputEscrow.toString());

  try {
    console.log("Filling intent...");
    await program.methods.fillIntent(
      {
        id: new BN(id),
        inputAmount: new BN(10),
        minReceive: new BN(5),
        receiveAmount: new BN(5),
        inputToken: inputTokenMint,
        outputToken: outputTokenMint,
        user: user.publicKey,
      })
      .accounts({
        solver: solver.publicKey,
        user: user.publicKey,
        intent: intent,
      
        solverOutputAta: solverOutputTokenAccount,
        userInputEscrow: escrow,
       
        inputTokenMint: inputTokenMint,
        outputTokenMint: outputTokenMint,
      })
      .signers([solver, user])
      .rpc();
      
    console.log("Intent filled successfully!");
    
    const intentAccount = await program.account.intent.fetch(intent);
    console.log("Intent account:", intentAccount);

    assert.equal(intentAccount.active, false);
      
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