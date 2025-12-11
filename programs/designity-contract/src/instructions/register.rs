use anchor_lang::prelude::*;
use anchor_spl::metadata::{
    create_metadata_accounts_v3, CreateMetadataAccountsV3, MasterEditionAccount,
};
use anchor_spl::token::{mint_to, Mint, MintTo, Token, TokenAccount};
use mpl_token_metadata::state::{Collection, DataV2};
use crate::utils::Realloc;

use crate::state::Org;
use crate::state::Score;

#[derive(Accounts)]
pub struct RegisterCTX<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account()]
    pub applicant: AccountInfo<'info>,
    #[account()]
    pub org: Account<'info, Org>,
    #[account(
        init_if_needed,
        payer = authority,
        seeds = [b"score", org.key().as_ref(), applicant.key().as_ref()],
        bump,
        space= 10 + std::mem::size_of::<Score>()
    )]
    pub score: Account<'info, Score>,
    #[account(mut)]
    pub register_mint: Account<'info, Mint>,
    #[account(mut)]
    pub token_account: Account<'info, TokenAccount>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(mut)]
    pub metadata: UncheckedAccount<'info>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(mut)]
    pub collection_master: Account<'info, MasterEditionAccount>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub token_metadata_program: UncheckedAccount<'info>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub rent: AccountInfo<'info>,
}

pub fn register(
    ctx: Context<RegisterCTX>,
    name: String,
    levels: Vec<u8>,
    last_update: i64,
) -> Result<()> {
    assert_eq!(
        ctx.accounts.org.authority.key(),
        ctx.accounts.authority.key()
    );
    assert_eq!(levels.len(), ctx.accounts.org.levels.len());
    let org = &mut ctx.accounts.org;
    let mint = org.mint.key();
    let signer: &[&[&[u8]]] = &[&[
        b"org",
        mint.as_ref(),
        ctx.accounts.authority.key.as_ref(),
        &[org.bump],
    ]];

    ctx.accounts.score.set_inner(Score::new(
        *ctx.bumps
            .get("score")
            .expect("Failed to fetch bump for `score`"),
    ));
    let space_to_add = 4 * (org.ranges.len() + 1) // scores
    + (4 * org.weights.len())  //scores_sum
    + org.levels.len() // levels
    + (name.len() * 4) // name
    + (2 * org.weights.len());
    msg!("space to add:{}", space_to_add);
    ctx.accounts.score.realloc(
        space_to_add,
        &ctx.accounts.authority,
        &ctx.accounts.system_program,
    )?;
    ctx.accounts.score.scores = vec![0 as f32; org.ranges.len() + 1];
    ctx.accounts.score.levels = levels;
    ctx.accounts.score.reviews_recieved = vec![0_u16; org.weights.len()];
    ctx.accounts.score.name = name.clone();
    ctx.accounts.score.last_update = last_update;
    msg!("last update:{}", last_update);
    ctx.accounts.score.applicant = ctx.accounts.applicant.key();
    ctx.accounts.score.mint = ctx.accounts.register_mint.key();
    ctx.accounts.score.scores_sum = vec![0 as f32; org.weights.len()];

    msg!("Minting token");
    let mint_to_cpi_accounts = MintTo {
        mint: ctx.accounts.register_mint.to_account_info(),
        to: ctx.accounts.token_account.to_account_info(),
        authority: org.to_account_info(),
    };
    let token_program = ctx.accounts.token_program.to_account_info();
    let mint_to_cpi_ctx = CpiContext::new_with_signer(token_program, mint_to_cpi_accounts, signer);
    mint_to(mint_to_cpi_ctx, 1)?;
    msg!("Token minted");
    msg!("register instruction");
    let mut metadata_name = org.name.clone();
    metadata_name.push_str(" - ");
    metadata_name.push_str(&name);
    let mut level_string: String = ctx
        .accounts
        .score
        .levels
        .iter()
        .map(|&id| id.to_string() + "-")
        .collect();
    level_string.pop();
    let mut uri = org.domain.clone();
    uri.push('/');
    uri.push_str(&level_string);
    uri.push_str(".json");
    let data_v2 = DataV2 {
        name: metadata_name.to_string(),
        symbol: "SCORE".to_string(),
        uri,
        seller_fee_basis_points: 0,
        creators: None,
        collection: Some(Collection {
            verified: false,
            key: org.mint.key(),
        }),
        uses: None,
    };
    let create_metadata_cpi_accounts = CreateMetadataAccountsV3 {
        metadata: ctx.accounts.metadata.to_account_info(),
        mint: ctx.accounts.register_mint.to_account_info(),
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

    create_metadata_accounts_v3(create_metadata_cpi_ctx, data_v2, true, true, None)?;
    msg!("Metadata Account Created !!!");
    Ok(())
}
