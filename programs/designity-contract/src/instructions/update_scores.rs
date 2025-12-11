use anchor_lang::prelude::*;
use anchor_spl::metadata::{update_metadata_accounts_v2, UpdateMetadataAccountsV2};
use mpl_token_metadata::state::Collection;

use super::*;

pub fn update_scores(
    ctx: Context<ScoreCTX>,
    scores_sum: Vec<f32>,
    reviews_recieved: Vec<u16>,
    last_update: i64,
    levels: Vec<u8>,
    override_levels: bool,
) -> Result<()> {
    msg!("update transaction");
    assert_eq!(ctx.accounts.org.weights.len(), scores_sum.len());
    assert_eq!(ctx.accounts.org.weights.len(), reviews_recieved.len());
    assert_eq!(ctx.accounts.org.ranges.len() + 1, levels.len());
    assert_eq!(
        ctx.accounts.org.authority.key(),
        ctx.accounts.authority.key()
    );

    ctx.accounts.score.scores_sum = scores_sum.clone();
    ctx.accounts.score.reviews_recieved = reviews_recieved;

    let next_level = ctx.accounts.score.reconcile(&ctx.accounts.org);
    ctx.accounts.score.levels = next_level;
    ctx.accounts.score.last_update = last_update;

    if override_levels {
        ctx.accounts.score.levels = levels;
    }
    let mut level_string: String = ctx
        .accounts
        .score
        .levels
        .iter()
        .map(|&id| id.to_string() + "-")
        .collect();
    level_string.pop();
    let mut uri = ctx.accounts.org.domain.clone();
    uri.push('/');
    uri.push_str(&level_string);
    uri.push_str(".json");

    msg!("Updating NFT");
    let org_mint = ctx.accounts.org.mint.key();
    let signer: &[&[&[u8]]] = &[&[
        b"org",
        org_mint.as_ref(),
        ctx.accounts.authority.key.as_ref(),
        &[ctx.accounts.org.bump],
    ]];
    let data_v2 = mpl_token_metadata::state::DataV2 {
        name: ctx.accounts.metadata.data.name.to_string(),
        symbol: "SCORE".to_string(),
        uri,
        seller_fee_basis_points: 0,
        creators: None,
        collection: Some(Collection {
            verified: true,
            key: ctx.accounts.org.mint.key(),
        }),
        uses: None,
    };
    let cpi_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_metadata_program.to_account_info(),
        UpdateMetadataAccountsV2 {
            metadata: ctx.accounts.metadata.to_account_info(),
            update_authority: ctx.accounts.org.to_account_info(),
        },
        signer,
    );
    update_metadata_accounts_v2(cpi_ctx, None, Some(data_v2), Some(true), Some(true))?;
    Ok(())
}
