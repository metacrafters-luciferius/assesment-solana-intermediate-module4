import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { CreateNftProgram } from "../target/types/create_nft_program";

describe("create-nft-program", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.CreateNftProgram as Program<CreateNftProgram>;

  it("Is initialized!", async () => {
    // Add your test here.
    const tx = await program.methods.initialize().rpc();
    console.log("Your transaction signature", tx);
  });
});
