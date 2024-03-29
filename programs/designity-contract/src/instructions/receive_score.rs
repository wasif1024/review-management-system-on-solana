use anchor_lang::prelude::*;
use anchor_spl::{
    metadata::{update_metadata_accounts_v2, MetadataAccount, UpdateMetadataAccountsV2},
    token::Token,
};
use mpl_token_metadata::state::Collection;

use crate::state::{Org, Score};

#[derive(Accounts)]
pub struct ScoreCTX<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account()]
    pub applicant: AccountInfo<'info>,
    #[account(
        mut,
        seeds = [b"score", org.key().as_ref(), applicant.key().as_ref()],
        bump,
    )]
    pub score: Account<'info, Score>,
    #[account()]
    pub org: Account<'info, Org>,
    #[account(mut)]
    pub metadata: Account<'info, MetadataAccount>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub token_metadata_program: UncheckedAccount<'info>,
}

pub fn receive_score(
    ctx: Context<ScoreCTX>,
    scores: Vec<f32>,
    timestamp_override: i64,
) -> Result<()> {
    let score = &mut ctx.accounts.score;
    let clock = Clock::get()?;
    let mut submission_ts = clock.unix_timestamp;
    if timestamp_override != 0 {
        submission_ts = timestamp_override;
    }
    assert_eq!(ctx.accounts.org.weights.len(), scores.len());
    assert_eq!(
        ctx.accounts.org.authority.key(),
        ctx.accounts.authority.key()
    );

    for (p1, e1) in scores.iter().enumerate() {
        score.scores_sum[p1] += e1;
        if *e1 != 0 as f32 {
            score.reviews_recieved[p1] += 1;
        }
    }

    let next_level = score.reconcile(&ctx.accounts.org);

    if !ctx.accounts.metadata.collection.as_ref().unwrap().verified{
        return Ok(());
    }

    msg!(
        "check debug last_update:{} level_wait:{} current_ts:{} current_level:{:?} next_level:{:?}",
        score.last_update,
        ctx.accounts.org.level_wait,
        clock.unix_timestamp,
        score.levels,
        next_level
    );
    if score.levels == next_level {
        score.last_update = clock.unix_timestamp;
    } else if score.levels != next_level
        && score.last_update + (ctx.accounts.org.level_wait as i64) < submission_ts
        && *score.reviews_recieved.iter().max().unwrap() >= ctx.accounts.org.min_reviews as u16
    {
        score.levels = next_level;
        score.last_update = submission_ts;

        let mut level_string: String = score
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
    }
    Ok(())
}
