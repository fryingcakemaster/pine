import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Pine } from "../target/types/pine";
import { expect } from "chai";

describe("pine", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.Pine as Program<Pine>;
  const payer = anchor.web3.Keypair.generate()
  const signer = anchor.web3.Keypair.generate()
  console.log("Payer Public Key:", payer.publicKey.toString());
  console.log("Signer Public Key:", signer.publicKey.toString());

  const  LAMPORTS_PER_SOL = 1000000000;
  const x = async () => {
    const airdropSignature = await provider.connection.requestAirdrop(
      signer.publicKey, 2 * LAMPORTS_PER_SOL);

    const latestBlockHash = await provider.connection.getLatestBlockhash();

    await provider.connection.confirmTransaction({
      blockhash: latestBlockHash.blockhash,
      lastValidBlockHeight: latestBlockHash.lastValidBlockHeight,
      signature: airdropSignature,
    });
  }

  it("Is initialized!", async () => {
    await x()

    const dex = anchor.web3.Keypair.generate();
    const tx = await program.methods.initialize()
    .accounts({ 
      dexState: dex,
      authority: signer.publicKey,
    })
    .signers([signer])
    .rpc()

    console.log("Your transaction signature", tx);
  });

  // it("Fetches DexState account info", async () => {
  //   const seeds = []
  //   const [dexState, _bump] = anchor.web3.PublicKey.findProgramAddressSync(seeds, program.programId);

  //   const dexStateAccount = await program.account.dexState.fetch(dexState);
  //   console.log("DexState account info authority = ", dexStateAccount.authority);
  //   console.log("DexState account info orderCount = ", dexStateAccount.orderCount.toNumber());
  //   console.log("DexState account info orders = ", dexStateAccount.orders);
  // });

  // it("Checks if initialized successfully", async () => {
  //   const seeds = []
  //   const [dexState, _bump] = anchor.web3.PublicKey.findProgramAddressSync(seeds, program.programId);

  //   const dexStateAccount = await program.account.dexState.fetch(dexState);

  //   // AssertionError: expected <BN: 0> to equal +0
  //   expect(dexStateAccount.orderCount.toNumber()).to.equal(0);
  //   expect(dexStateAccount.authority.toString()).to.equal(signer.publicKey.toString());
  //   expect(dexStateAccount.orders.length).to.equal(0);

  //   console.log("Initialization check passed.");
  // });

  it("Places an order successfully", async () => {
    await x()

    const userTokenAccount = anchor.web3.Keypair.generate();
    const dexTokenAccount = anchor.web3.Keypair.generate();

    console.log("User Token Account:", userTokenAccount.publicKey.toString());
    console.log("DEX Token Account:", dexTokenAccount.publicKey.toString());
    // Error Number: 2006. Error Message: A seeds constraint was violated
    // const [udexState, _a] = anchor.web3.PublicKey.findProgramAddressSync([], program.programId);
    // const [ddexState, _d] = anchor.web3.PublicKey.findProgramAddressSync([], program.programId);

    await program.methods.initialize()
    .accounts({ 
      dexState: userTokenAccount.publicKey,
      authority: userTokenAccount.publicKey,
    })
    .signers([userTokenAccount])
    .rpc()

    await program.methods.initialize()
    .accounts({ 
      dexState: dexTokenAccount.publicKey,
      authority: dexTokenAccount.publicKey,
    })
    .signers([dexTokenAccount])
    .rpc()

    await program.methods.initialize()
    .accounts({ 
      dexState: signer.publicKey,
      authority: signer.publicKey,
    })
    .signers([signer])
    .rpc()

    // const tokenProgram = anchor.web3.PublicKey.findProgramAddressSync(
    //   [Buffer.from("token")],
    //   anchor.web3.PublicKey.default
    // );
    
    console.log("Payer:", payer.publicKey.toString());

    const od = { buy:{} };
    // src.toArrayLike is not a function
    // u64 -> BN

    console.log("User Token Account:", userTokenAccount.toString());
    console.log("DEX Token Account:", dexTokenAccount.toString());

    const tx = await program.methods.placeOrder(od, new anchor.BN(100), new anchor.BN(10))
    .accounts({
      dexState: signer.publicKey,
      signer: signer.publicKey,
      userTokenAccount: userTokenAccount.publicKey,
      dexTokenAccount: dexTokenAccount.publicKey,
    })
    .signers([signer])
    .rpc();
    console.log("Transaction signature for placing order:", tx);

    const dexStateAccount = await program.account.dexState.fetch(payer.publicKey);
    expect(dexStateAccount.orderCount.toNumber()).to.equal(1);
    expect(dexStateAccount.orders.length).to.equal(1);
    expect(dexStateAccount.orders[0].orderId).to.equal(0);
    expect(dexStateAccount.orders[0].owner.toString()).to.equal(userTokenAccount.publicKey.toString());
    expect(dexStateAccount.orders[0].od).to.equal({buy: {}});
    expect(dexStateAccount.orders[0].amount).to.equal(100);
    expect(dexStateAccount.orders[0].price).to.equal(10);
    expect(dexStateAccount.orders[0].fulfilled).to.equal(0);

    console.log("Order placement check passed.");
  });
});
