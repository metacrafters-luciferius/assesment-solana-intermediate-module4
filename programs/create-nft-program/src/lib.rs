use anchor_lang::prelude::*;
use anchor_spl::{token::{Mint, TokenAccount, Token, MintTo}, associated_token::AssociatedToken};
use mpl_token_metadata::{ID as METADATA_PROGRAM_ID};

declare_id!("AHqbhaYrNwAXhH7X4w8cC8y26P2PAATBKzWMnEZP5hnq");

#[program]
pub mod create_nft_program {
    use anchor_spl::token::mint_to;
    use mpl_token_metadata::{instruction::{create_metadata_accounts_v3, create_master_edition_v3}, state::Creator};
    use solana_program::program::invoke;

    use super::*;

    pub fn create_nft(ctx: Context<CreateNFT>, name: String, symbol: String, uri: String) -> Result<()> {
        //define NFT creators
        let creators = Some(vec![
            Creator{ 
                address: ctx.accounts.user.key(), 
                verified: true, 
                share: 100 
            }
        ]);

        //create metadata account
        let create_metadata_instruction = create_metadata_accounts_v3(
            ctx.accounts.metadata_program.key(), 
            ctx.accounts.metadata_account.key(), 
            ctx.accounts.nft_mint.key(), 
            ctx.accounts.user.key(), 
            ctx.accounts.user.key(), 
            ctx.accounts.user.key(), 
            name, 
            symbol, 
            uri, 
            creators, 
            0, 
            false, 
            false, 
            None, 
            None, 
            None
        );

        //submit
        invoke(
            &create_metadata_instruction, 
            &[
                ctx.accounts.metadata_program.to_account_info(),
                ctx.accounts.metadata_account.to_account_info(),
                ctx.accounts.nft_mint.to_account_info(),
                ctx.accounts.user.to_account_info(),
            ]
        )?;

        //mint nft
        mint_to(ctx.accounts.mint_to_ctx(), 1)?;

        let create_master_edition_ix = create_master_edition_v3(
            ctx.accounts.metadata_program.key(), 
            ctx.accounts.master_edition.key(), 
            ctx.accounts.nft_mint.key(), 
            ctx.accounts.user.key(), 
            ctx.accounts.user.key(), 
            ctx.accounts.metadata_account.key(), 
            ctx.accounts.user.key(), 
            Some(1)
        );

        invoke(
            &create_master_edition_ix, 
            &[
                ctx.accounts.metadata_program.to_account_info(),
                ctx.accounts.master_edition.to_account_info(),
                ctx.accounts.nft_mint.to_account_info(),
                ctx.accounts.user.to_account_info(),
                ctx.accounts.metadata_account.to_account_info(),
            ]
        )?;

        Ok(())
    }
}

#[derive(Accounts)]
pub struct CreateNFT<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(
        init,
        payer = user,
        mint::decimals = 0,
        mint::authority = user,
        mint::freeze_authority = user
    )]
    pub nft_mint: Account<'info, Mint>,
    #[account(
        init,
        payer = user,
        associated_token::mint = nft_mint,
        associated_token::authority = user
    )]
    pub user_token_account: Account<'info, TokenAccount>,
    /// CHECK: Safe metadata account
    #[account(
        mut,
        seeds = [b"metadata", metadata_program.key().as_ref(), nft_mint.key().as_ref()],
        bump,
        seeds::program = metadata_program.key()
    )]
    pub metadata_account: AccountInfo<'info>,
    /// CHECK: Safe master edition account
    #[account(
        mut,
        seeds = [b"metadata", metadata_program.key().as_ref(), nft_mint.key().as_ref(), b"edition"],
        bump,
        seeds::program = metadata_program.key()
    )]
    pub master_edition: AccountInfo<'info>,
    /// CHECK: Safe because verification through contraint
    #[account(
        constraint = metadata_program.key() == METADATA_PROGRAM_ID
    )]
    pub metadata_program: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub rent: Sysvar<'info, Rent>
}

impl <'info> CreateNFT<'info> {
    pub fn mint_to_ctx(&self) -> CpiContext<'_, '_, '_, 'info, MintTo<'info>>{
        let cpi_program = self.token_program.to_account_info();
        let cpi_accounts = MintTo {
            mint: self.nft_mint.to_account_info(),
            to: self.user_token_account.to_account_info(),
            authority: self.user.to_account_info(),
        };

        CpiContext::new(cpi_program, cpi_accounts)
    }
}
