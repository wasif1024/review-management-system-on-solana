#![allow(deprecated)]

use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::{create, AssociatedToken, Create},
    metadata::{
        create_master_edition_v3, create_metadata_accounts_v3, CreateMasterEditionV3,
        CreateMetadataAccountsV3,
    },
    token::{mint_to, Mint, MintTo, Token},
};
use mpl_token_metadata::state::CollectionDetails;

use crate::{state::Org, utils::Realloc};

#[derive(Accounts)]
pub struct CreateOrgCTX<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(
        init,
        payer = authority,
        space = 8 + std::mem::size_of::<Org>(),
        seeds = [b"org", org_mint.key().as_ref(), authority.key().as_ref()],
        bump
    )]
    pub org: Account<'info, Org>,
    #[account(
        init,
        payer = authority,
        mint::decimals = 0,
        mint::authority = org,
        mint::freeze_authority = org,
    )]
    pub org_mint: Account<'info, Mint>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(mut)]
    pub metadata: UncheckedAccount<'info>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(mut)]
    pub master_edition: UncheckedAccount<'info>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(mut)]
    pub token_account: UncheckedAccount<'info>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub token_metadata_program: UncheckedAccount<'info>,
    ///CHECK: This is not dangerous because we don't read or write from this account
    pub rent: AccountInfo<'info>,
}

pub fn create_organization(
    ctx: Context<CreateOrgCTX>,
    weights: Vec<f32>,
    ranges: Vec<u8>,
    levels: Vec<Vec<f32>>,
    name: String,
    min_reviews: u8,
    domain: String,
    level_wait: i32,
) -> Result<()> {
    assert_eq!(ranges.len() + 1, levels.len());
    let org = &mut ctx.accounts.org;
    org.weights = weights;
    org.ranges = ranges;
    org.levels = levels;
    org.mint = ctx.accounts.org_mint.key();
    org.authority = ctx.accounts.authority.key();
    org.name = name.clone();
    org.min_reviews = min_reviews;
    org.domain = domain;
    org.level_wait = level_wait;
    let mut total_levels = 0;
    for l in org.levels.iter() {
        for _l2 in l.iter() {
            total_levels += 1;
        }
    }
    org.bump = *ctx.bumps.get("org").unwrap();
    let mint = ctx.accounts.org_mint.key();
    let signer: &[&[&[u8]]] = &[&[
        b"org",
        mint.as_ref(),
        ctx.accounts.authority.key.as_ref(),
        &[org.bump],
    ]];

    msg!("Creating Metadata");
    let mut metadata_name = name.clone();
    metadata_name.push_str(" Organization");
    let mut uri = org.domain.clone();
    uri.push_str("/org.json");
    let data_v2 = mpl_token_metadata::state::DataV2 {
        name: metadata_name.to_string(),
        symbol: "GRWTH".to_string(),
        uri: uri.to_string(),
        seller_fee_basis_points: 0,
        creators: None,
        collection: None,
        uses: None,
    };
    let create_metadata_cpi_accounts = CreateMetadataAccountsV3 {
        metadata: ctx.accounts.metadata.to_account_info(),
        mint: ctx.accounts.org_mint.to_account_info(),
        mint_authority: org.to_account_info(),
        update_authority: org.to_account_info(),
        payer: ctx.accounts.authority.to_account_info(),
        system_program: ctx.accounts.system_program.to_account_info(),
        rent: ctx.accounts.rent.to_account_info(),
    };
    let create_metadata_cpi_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_metadata_program.to_account_info(),
        create_metadata_cpi_accounts,
        signer,
    );
    create_metadata_accounts_v3(
        create_metadata_cpi_ctx,
        data_v2,
        true,
        true,
        Some(CollectionDetails::V1 { size: (0) }),
    )?;
    msg!("Metadata created");

    msg!("Creating token account");
    let create_ata_cpi_accounts = Create {
        associated_token: ctx.accounts.token_account.to_account_info(),
        authority: org.to_account_info(),
        mint: ctx.accounts.org_mint.to_account_info(),
        payer: ctx.accounts.authority.to_account_info(),
        system_program: ctx.accounts.system_program.to_account_info(),
        token_program: ctx.accounts.token_program.to_account_info(),
    };
    let create_ata_cpi_ctx = CpiContext::new(
        ctx.accounts.associated_token_program.to_account_info(),
        create_ata_cpi_accounts,
    );
    create(create_ata_cpi_ctx)?;
    msg!("Token account created");

    msg!("Minting Token");
    let mint_to_cpi_accounts = MintTo {
        mint: ctx.accounts.org_mint.to_account_info(),
        to: ctx.accounts.token_account.to_account_info(),
        authority: org.to_account_info(),
    };
    let mint_to_cpi_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        mint_to_cpi_accounts,
        signer,
    );
    mint_to(mint_to_cpi_ctx, 1)?;
    msg!("Token minted");

    msg!("Creating master edition");
    let create_master_cpi_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_metadata_program.to_account_info(),
        CreateMasterEditionV3 {
            payer: ctx.accounts.authority.to_account_info(),
            update_authority: org.to_account_info(),
            token_program: ctx.accounts.token_metadata_program.to_account_info(),
            system_program: ctx.accounts.system_program.to_account_info(),
            rent: ctx.accounts.rent.to_account_info(),
            edition: ctx.accounts.master_edition.to_account_info(),
            metadata: ctx.accounts.metadata.to_account_info(),
            mint_authority: org.to_account_info(),
            mint: ctx.accounts.org_mint.to_account_info(),
        },
        signer,
    );
    create_master_edition_v3(create_master_cpi_ctx, Some(0))?;

    org.realloc(
        total_levels * 4 + org.levels.len() + 50 + 50,
        &ctx.accounts.authority,
        &ctx.accounts.system_program,
    )?;
    msg!("Master edition created");
    Ok(())
}
