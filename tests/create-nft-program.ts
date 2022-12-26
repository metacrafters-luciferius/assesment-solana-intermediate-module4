import * as anchor from "@project-serum/anchor"
import { Program } from "@project-serum/anchor"
import { CreateNftProgram } from "../target/types/create_nft_program"
import { Connection, PublicKey, LAMPORTS_PER_SOL, Keypair, SystemProgram, SYSVAR_RENT_PUBKEY } from '@solana/web3.js'
import { safeAirdrop, delay } from './utils/utils'
import { BN } from "bn.js"
import { Key, PROGRAM_ID as METADATA_PROGRAM_ID } from '@metaplex-foundation/mpl-token-metadata'
import { TOKEN_PROGRAM_ID, getAssociatedTokenAddress, ASSOCIATED_TOKEN_PROGRAM_ID, getAccount } from '@solana/spl-token'
import { assert } from "chai"

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
})