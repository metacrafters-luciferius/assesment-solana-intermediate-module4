use anchor_lang::prelude::*;
use anchor_spl::{token::{Mint, TokenAccount, Token, MintTo, Approve, Revoke}, associated_token::AssociatedToken};
use mpl_token_metadata::{ID as METADATA_PROGRAM_ID};

declare_id!("AHqbhaYrNwAXhH7X4w8cC8y26P2PAATBKzWMnEZP5hnq");

#[program]
pub mod create_nft_program {
    use anchor_lang::AccountsClose;
    use anchor_spl::token::{mint_to, approve, revoke};
    use mpl_token_metadata::{instruction::{create_metadata_accounts_v3, create_master_edition_v3, freeze_delegated_account, thaw_delegated_account}, state::Creator};
    use solana_program::program::{invoke, invoke_signed};

    use super::*;

    pub fn initialize_mint(ctx: Context<InitializeMint>) -> Result<()> {
        msg!("Created token {}", ctx.accounts.token_mint.key());
        Ok(())
    }

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

    pub fn stake(ctx: Context<StakeNFT>) -> Result<()> {
        //let user = ctx.accounts.user.key();
        //let signer = &[user.as_ref()];
        //approve(ctx.accounts.approve_ctx().with_signer(&[&signer[..]]), 1)?;
        approve(ctx.accounts.approve_ctx(), 1)?;

        let authority_bump = *ctx.bumps.get("program_authority").unwrap();
        let authority_seeds = &["authority".as_bytes(), &[authority_bump]];
        let signer = &[&authority_seeds[..]];

        let freeze_ix = freeze_delegated_account(
            ctx.accounts.metadata_program.key(), 
            ctx.accounts.program_authority.key(), 
            ctx.accounts.user_token_account.key(), 
            ctx.accounts.master_edition.key(), 
            ctx.accounts.nft_mint.key()
        );

        invoke_signed(
            &freeze_ix, 
            &[
                ctx.accounts.metadata_program.to_account_info(),
                ctx.accounts.program_authority.to_account_info(),
                ctx.accounts.user_token_account.to_account_info(),
                ctx.accounts.master_edition.to_account_info(),
                ctx.accounts.nft_mint.to_account_info()
            ],
            signer
        )?;

        msg!("Staked NFT successfully.");

        ctx.accounts.stake.timestamp = Clock::get()?.unix_timestamp.unsigned_abs();

        Ok(())
    }

    pub fn unstake(ctx: Context<UnstakeNFT>) -> Result<()> {
        let reward = Clock::get()?.unix_timestamp.unsigned_abs() - ctx.accounts.stake.timestamp;

        //Thaw account
        let authority_bump = *ctx.bumps.get("program_authority").unwrap();
        let authority_seeds = &["authority".as_bytes(), &[authority_bump]];
        let signer = &[&authority_seeds[..]];

        let thaw_ix = thaw_delegated_account(
            ctx.accounts.metadata_program.key(), 
            ctx.accounts.program_authority.key(), 
            ctx.accounts.nft_token_account.key(), 
            ctx.accounts.master_edition.key(), 
            ctx.accounts.nft_mint.key()
        );

        invoke_signed(
            &thaw_ix, 
            &[
                ctx.accounts.metadata_program.to_account_info(),
                ctx.accounts.program_authority.to_account_info(),
                ctx.accounts.nft_token_account.to_account_info(),
                ctx.accounts.master_edition.to_account_info(),
                ctx.accounts.nft_mint.to_account_info()
            ],
            signer
        )?;

        msg!("Unstaked NFT.");

        //Revoke
        revoke(ctx.accounts.revoke_ctx())?;

        msg!("Revoked delegate.");

        //Mint reward token        
        let mint_bump = *ctx.bumps.get("mint_authority").unwrap();
        let mint_seeds = &["mint-authority".as_bytes(), &[mint_bump]];
        let signer = &[&mint_seeds[..]];

        let mint_to_ctx = ctx.accounts.mint_to_ctx().with_signer(signer);
        let result = mint_to(mint_to_ctx, reward);

        if result.is_err() {
            let error = result.err().unwrap();
            msg!("Mint {} reward token failed: {}", reward, error);
        }
        else{
            msg!("Mint {} reward token completed successfully.", reward);
        }

        //Close state account
        ctx.accounts.stake.close(ctx.accounts.user.to_account_info())?;

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

#[derive(Accounts)]
pub struct InitializeMint<'info> {
    #[account(
        init,
        mint::authority = mint_authority,
        mint::decimals = 8, 
        seeds = ["token-mint".as_bytes()], 
        bump, 
        payer=payer)]
    pub token_mint: Account<'info, Mint>,
    #[account(seeds = ["mint-authority".as_bytes()], bump)]
    /// CHECK: using as signer
    pub mint_authority: AccountInfo<'info>,
    #[account(mut)]
    pub payer: Signer<'info>,
    pub rent: Sysvar<'info, Rent>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
pub struct StakeNFT<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    pub nft_mint: Account<'info, Mint>,
    #[account(
        init,
        payer = user,
        seeds = [user.key().as_ref(), user_token_account.key().as_ref()],
        bump,
        space = 8 + 8
    )]
    pub stake: Account<'info, StakingData>,
    #[account(
        mut,
        associated_token::mint = nft_mint,
        associated_token::authority = user
    )]
    pub user_token_account: Account<'info, TokenAccount>,
    /// CHECK: Manual validation
    #[account(owner=METADATA_PROGRAM_ID)]
    pub master_edition: UncheckedAccount<'info>,
    /// CHECK: Manual validation
    #[account(mut, seeds=["authority".as_bytes().as_ref()], bump)]
    pub program_authority: UncheckedAccount<'info>,
    /// CHECK: Safe because verification through contraint
    #[account(
        constraint = metadata_program.key() == METADATA_PROGRAM_ID
    )]
    pub metadata_program: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

#[account]
pub struct StakingData{
    pub timestamp: u64
}

impl <'info> StakeNFT<'info> {
    pub fn approve_ctx(&self) -> CpiContext<'_, '_, '_, 'info, Approve<'info>>{
        let cpi_program = self.token_program.to_account_info();
        let cpi_accounts = Approve { 
            to: self.user_token_account.to_account_info(), 
            delegate: self.program_authority.to_account_info(), 
            authority: self.user.to_account_info() 
        };

        CpiContext::new(cpi_program, cpi_accounts)
    }
}

#[derive(Accounts)]
pub struct UnstakeNFT<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    pub nft_mint: Account<'info, Mint>,
    #[account(
        mut,
        seeds = [user.key().as_ref(), nft_token_account.key().as_ref()],
        bump
    )]
    pub stake: Account<'info, StakingData>,
    #[account(
        mut,
        associated_token::mint = nft_mint,
        associated_token::authority = user
    )]
    pub nft_token_account: Account<'info, TokenAccount>,
    /// CHECK: Manual validation
    #[account(owner=METADATA_PROGRAM_ID)]
    pub master_edition: UncheckedAccount<'info>,
    /// CHECK: Manual validation
    #[account(mut, seeds=["authority".as_bytes().as_ref()], bump)]
    pub program_authority: UncheckedAccount<'info>,
    /// CHECK: Safe because verification through contraint
    #[account(mut, seeds = ["token-mint".as_bytes()], bump)]
    pub token_mint: Account<'info, Mint>,
    #[account(mut, seeds = ["mint-authority".as_bytes()], bump)]
    /// CHECK: using as signer
    pub mint_authority: AccountInfo<'info>,
    #[account(
        init_if_needed,
        payer = user,
        associated_token::mint = token_mint,
        associated_token::authority = user,
    )]
    pub user_token_account: Account<'info, TokenAccount>,
    /// CHECK: Safe because verification through contraint
    #[account(
        constraint = metadata_program.key() == METADATA_PROGRAM_ID
    )]
    pub metadata_program: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub rent: Sysvar<'info, Rent>,
}

impl <'info> UnstakeNFT<'info> {
    pub fn revoke_ctx(&self) -> CpiContext<'_, '_, '_, 'info, Revoke<'info>>{
        let cpi_program = self.token_program.to_account_info();
        let cpi_accounts = Revoke { 
            source: self.nft_token_account.to_account_info(), 
            authority: self.user.to_account_info()
        };

        CpiContext::new(cpi_program, cpi_accounts)
    }
    pub fn mint_to_ctx(&self) -> CpiContext<'_, '_, '_, 'info, MintTo<'info>>{
        let cpi_program = self.token_program.to_account_info();
        let cpi_accounts = MintTo {
            mint: self.token_mint.to_account_info(),
            to: self.user_token_account.to_account_info(),
            authority: self.mint_authority.to_account_info(),
        };

        CpiContext::new(cpi_program, cpi_accounts)
    }
}
