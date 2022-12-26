import * as anchor from "@project-serum/anchor"
import { Program } from "@project-serum/anchor"
import { CreateNftProgram } from "../target/types/create_nft_program"
import { Connection, PublicKey, LAMPORTS_PER_SOL, Keypair, SystemProgram, SYSVAR_RENT_PUBKEY } from '@solana/web3.js'
import { safeAirdrop, delay } from './utils/utils'
import { BN } from "bn.js"
import { Key, PROGRAM_ID as METADATA_PROGRAM_ID } from '@metaplex-foundation/mpl-token-metadata'
import { TOKEN_PROGRAM_ID, getAssociatedTokenAddress, ASSOCIATED_TOKEN_PROGRAM_ID, getAccount, getNonTransferable } from '@solana/spl-token'
import { assert, expect } from "chai"

describe("create-nft-demo", async () => {
  anchor.setProvider(anchor.AnchorProvider.env())

  const program = anchor.workspace.CreateNftProgram as Program<CreateNftProgram>
  const provider = anchor.AnchorProvider.env()

  const nftMint = Keypair.generate()
  const user = Keypair.generate()
  const userTokenAccount = await getAssociatedTokenAddress(nftMint.publicKey, user.publicKey)

  const [metadata, metadataBump] = PublicKey.findProgramAddressSync(
    [Buffer.from("metadata"), METADATA_PROGRAM_ID.toBuffer(), nftMint.publicKey.toBuffer()],
    METADATA_PROGRAM_ID
  )

  const [masterEdition, masterBump] = PublicKey.findProgramAddressSync(
    [Buffer.from("metadata"), METADATA_PROGRAM_ID.toBuffer(), nftMint.publicKey.toBuffer(), Buffer.from("edition")],
    METADATA_PROGRAM_ID
  )

  const stake = PublicKey.findProgramAddressSync(
    [user.publicKey.toBuffer(), userTokenAccount.toBuffer()],
    program.programId
  )

  const programAuthority = PublicKey.findProgramAddressSync(
    [Buffer.from("authority")],
    program.programId
  )

  const tokenMint = PublicKey.findProgramAddressSync(
    [Buffer.from("token-mint")],
    program.programId
  )

  const mintAuthority = PublicKey.findProgramAddressSync(
    [Buffer.from("mint-authority")],
    program.programId
  )
  
  const rewardTokenAccount = await getAssociatedTokenAddress(tokenMint[0], user.publicKey)

  it("Create and mint NFT!", async () => {
    await safeAirdrop(user.publicKey, provider.connection)
    const name = "my test NFT"
    const symbol = "DDR"
    const uri = "test-uri"

    const txid = await program.methods.createNft(name, symbol, uri)
    .accounts({
      user: user.publicKey,
      userTokenAccount: userTokenAccount,
      nftMint: nftMint.publicKey,
      metadataAccount: metadata,
      masterEdition: masterEdition,
      tokenProgram: TOKEN_PROGRAM_ID,
      associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
      metadataProgram: METADATA_PROGRAM_ID,
      systemProgram: SystemProgram.programId,
      rent: SYSVAR_RENT_PUBKEY
    })
    .signers([user, nftMint])
    .rpc()
    console.log("View transaction in explorer:")
    console.log(`https://explorer.solana.com/tx/${txid}?cluster=devnet`)

    console.log("View NFT in explorer:")
    console.log(`https://explorer.solana.com/address/${nftMint.publicKey}?cluster=devnet`)

  })

  it("Stake NFT!", async () => {
    const txid = await program.methods.stake()
    .accounts({
      user: user.publicKey,
      userTokenAccount: userTokenAccount,
      nftMint: nftMint.publicKey,
      stake: stake[0],
      masterEdition: masterEdition,
      tokenProgram: TOKEN_PROGRAM_ID,
      metadataProgram: METADATA_PROGRAM_ID,
      systemProgram: SystemProgram.programId,
      rent: SYSVAR_RENT_PUBKEY,
      programAuthority: programAuthority[0]
    })
    .signers([user])
    .rpc()
    console.log("View staking transaction in explorer:")
    console.log(`https://explorer.solana.com/tx/${txid}?cluster=devnet`)

    const tokenAccountInfo = await getAccount(
      provider.connection,
      userTokenAccount
    );
    expect(tokenAccountInfo.isFrozen).to.be.true;
    expect(tokenAccountInfo.amount).to.equal(BigInt(1));
  })

  it.skip("Initialize mint.", async () => {
    const txid = await program.methods.initializeMint()
    .accounts({
      payer: provider.wallet.publicKey,
      tokenMint: tokenMint[0],
      mintAuthority: mintAuthority[0],
      tokenProgram: TOKEN_PROGRAM_ID,
      systemProgram: SystemProgram.programId,
      rent: SYSVAR_RENT_PUBKEY
    })
    .rpc()
    console.log("View initialize mint transaction in explorer:")
    console.log(`https://explorer.solana.com/tx/${txid}?cluster=devnet`)
  })

  it("Unstake NFT.", async () => {
    //wait at least 1s to get at least one reward token
    await delay(1000)

    const txid = await program.methods.unstake()
    .accounts({
      user: user.publicKey,
      nftMint: nftMint.publicKey,
      stake: stake[0],
      nftTokenAccount: userTokenAccount,
      masterEdition: masterEdition,
      programAuthority: programAuthority[0],
      tokenMint: tokenMint[0],
      mintAuthority: mintAuthority[0],
      userTokenAccount: rewardTokenAccount,
      metadataProgram: METADATA_PROGRAM_ID,
      tokenProgram: TOKEN_PROGRAM_ID,
      systemProgram: SystemProgram.programId,
      associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
      rent: SYSVAR_RENT_PUBKEY
    })
    .signers([user])
    .rpc()
    console.log("View unstake transaction in explorer:")
    console.log(`https://explorer.solana.com/tx/${txid}?cluster=devnet`)

    const nftAccountInfo = await getAccount(
      provider.connection,
      userTokenAccount
    );
    expect(nftAccountInfo.isFrozen).to.be.false;
    expect(nftAccountInfo.delegate).to.be.null;
    
    const tokenAccountInfo = await getAccount(
      provider.connection,
      rewardTokenAccount
    );
    expect(Number.parseInt(tokenAccountInfo.amount.toString())).to.be.greaterThanOrEqual(1);
  })
})