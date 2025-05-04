import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { EventdotfunProgram } from "../target/types/eventdotfun_program";
import { Keypair, LAMPORTS_PER_SOL, PublicKey } from "@solana/web3.js";
import { SYSTEM_PROGRAM_ID } from "@coral-xyz/anchor/dist/cjs/native/system";
import { BN } from "bn.js";

const deployerKeypair = Keypair.fromSecretKey(
  new Uint8Array(require("../keys/turbin3.json")),
);

const feeRecipientKeypair = Keypair.fromSecretKey(
  new Uint8Array(require("../keys/fee_recipient.json")),
);

describe("eventdotfun-program", () => {
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace
    .eventdotfunProgram as Program<EventdotfunProgram>;

  let globalCollection;
  let globalAsset;

  it("Config State Initialized!", async () => {
    const fee = new anchor.BN(1000); // 10%
    const feeRecipient = feeRecipientKeypair.publicKey;

    const tx = await program.methods.initialize(fee, feeRecipient).rpc();
    console.log("Your transaction signature", tx);
  });

  it("Config State Updated!", async () => {
    const fee = new anchor.BN(1000); // 10%
    const feeRecipient = feeRecipientKeypair.publicKey;

    const tx = await program.methods.updateConfig(fee, feeRecipient).rpc();
    console.log("Your transaction signature", tx);
  });

  it("Bonding Curve Created!", async () => {
    const now = Math.floor(Date.now() / 1000);

    const salesType = 1;
    const startAt = new anchor.BN(now + 5);
    const endAt = new anchor.BN(now + 20);
    const exponent = 2;
    const initialPrice = new anchor.BN(0.001 * LAMPORTS_PER_SOL);
    const lastPrice = new anchor.BN(1 * LAMPORTS_PER_SOL);
    const minTicketToSold = new BN(80);
    const maxTicketToSold = new anchor.BN(100);
    const refundWindow = new BN(1746320400);

    const collection = Keypair.generate();

    globalCollection = collection.publicKey;

    const [bondingCurve] = PublicKey.findProgramAddressSync(
      [Buffer.from("bonding_curve"), collection.publicKey.toBuffer()],
      program.programId,
    );

    const [vault] = PublicKey.findProgramAddressSync(
      [Buffer.from("vault"), bondingCurve.toBuffer()],
      program.programId,
    );

    const tx = await program.methods
      .createBondingCurve(
        salesType,
        startAt,
        endAt,
        exponent,
        initialPrice,
        lastPrice,
        minTicketToSold,
        maxTicketToSold,
        refundWindow,
      )
      .accounts({
        // @ts-ignore
        bondingCurve,
        vault,
        // asset: asset.publicKey,
        collection: collection.publicKey,
        user: deployerKeypair.publicKey,
        systemProgram: SYSTEM_PROGRAM_ID,
        mplCoreProgram: new PublicKey(
          "CoREENxT6tW1HoK8ypY1SxRMZTcVPm7R94rH4PZNhX7d",
        ),
      })
      .signers([collection])
      .rpc();
    console.log("Your transaction signature", tx);

    await sleep(10000);
  });

  it("Buy!", async () => {
    const asset = Keypair.generate();

    globalAsset = asset.publicKey;

    const [bondingCurve] = PublicKey.findProgramAddressSync(
      [Buffer.from("bonding_curve"), globalCollection.toBuffer()],
      program.programId,
    );

    const [vault] = PublicKey.findProgramAddressSync(
      [Buffer.from("vault"), bondingCurve.toBuffer()],
      program.programId,
    );

    // const numOfTicket = 1;

    const tx = await program.methods
      .buy()
      .accounts({
        // @ts-ignore
        bondingCurve,
        vault,
        collection: globalCollection,
        asset: asset.publicKey,
        user: deployerKeypair.publicKey,
        systemProgram: SYSTEM_PROGRAM_ID,
        mplCoreProgram: new PublicKey(
          "CoREENxT6tW1HoK8ypY1SxRMZTcVPm7R94rH4PZNhX7d",
        ),
      })
      .signers([asset])
      .rpc({
        commitment: "confirmed",
      });
    console.log("Your transaction signature", tx);
  });

  it("Sell!", async () => {
    const [bondingCurve] = PublicKey.findProgramAddressSync(
      [Buffer.from("bonding_curve"), globalCollection.toBuffer()],
      program.programId,
    );

    const [vault] = PublicKey.findProgramAddressSync(
      [Buffer.from("vault"), bondingCurve.toBuffer()],
      program.programId,
    );

    // const numOfTicket = 1;

    const tx = await program.methods
      .sell()
      .accounts({
        // @ts-ignore
        bondingCurve,
        vault,
        collection: globalCollection,
        asset: globalAsset,
        user: deployerKeypair.publicKey,
        systemProgram: SYSTEM_PROGRAM_ID,
        mplCoreProgram: new PublicKey(
          "CoREENxT6tW1HoK8ypY1SxRMZTcVPm7R94rH4PZNhX7d",
        ),
      })
      .rpc({
        commitment: "confirmed",
      });
    console.log("Your transaction signature", tx);

    // const txDetails = await program.provider.connection.getParsedTransaction(
    //   tx,
    //   {
    //     commitment: "confirmed",
    //   },
    // );
    // console.log(txDetails);
  });

  it("withdraw!", async () => {
    // await sleep(10000);

    const [bondingCurve] = PublicKey.findProgramAddressSync(
      [Buffer.from("bonding_curve"), globalCollection.toBuffer()],
      program.programId,
    );

    const [vault] = PublicKey.findProgramAddressSync(
      [Buffer.from("vault"), bondingCurve.toBuffer()],
      program.programId,
    );

    const tx = await program.methods
      .withdraw()
      .accounts({
        // @ts-ignore
        bondingCurve,
        vault,
        collection: globalCollection,
        user: deployerKeypair.publicKey,
        systemProgram: SYSTEM_PROGRAM_ID,
      })
      .rpc();
    console.log("Your transaction signature", tx);
  });

  it("read account data", async () => {
    const [bondingCurve] = PublicKey.findProgramAddressSync(
      [Buffer.from("bonding_curve"), globalCollection.toBuffer()],
      program.programId,
    );

    const accInfo = await program.account.bondingCurve.fetch(bondingCurve);
    console.log(accInfo);
  });
});

const sleep = (ms: number) => new Promise((resolve) => setTimeout(resolve, ms));
