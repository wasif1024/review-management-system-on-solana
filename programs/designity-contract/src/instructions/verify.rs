use anchor_lang::prelude::*;
use anchor_spl::{
    metadata::{
        verify_sized_collection_item, MasterEditionAccount, MetadataAccount,
        VerifySizedCollectionItem,
    },
    token::Mint,
};

use crate::state::Org;

#[derive(Accounts)]
pub struct VerifyCTX<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account()]
    pub org: Account<'info, Org>,
    #[account()]
    pub org_mint: Account<'info, Mint>,
    #[account(mut)]
    pub collection_master: Account<'info, MasterEditionAccount>,
    #[account(mut)]
    pub collection_metadata: Account<'info, MetadataAccount>,
    #[account(mut)]
    pub metadata: Account<'info, MetadataAccount>,
    pub system_program: Program<'info, System>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub token_metadata_program: UncheckedAccount<'info>,
}

pub fn verify(ctx: Context<VerifyCTX>) -> Result<()> {
    assert_eq!(
        ctx.accounts.org.authority.key(),
        ctx.accounts.authority.key()
    );
    if ctx.accounts.metadata.collection.as_ref().unwrap().verified{
        return Ok(());
    }
    let mint = ctx.accounts.org.mint.key();
    let signer: &[&[&[u8]]] = &[&[
        b"org",
        mint.as_ref(),
        ctx.accounts.authority.key.as_ref(),
        &[ctx.accounts.org.bump],
    ]];
    msg!("Verifying collection");
    let verify_cpi_accounts = VerifySizedCollectionItem {
        collection_authority: ctx.accounts.org.to_account_info(),
        collection_master_edition: ctx.accounts.collection_master.to_account_info(),
        collection_metadata: ctx.accounts.collection_metadata.to_account_info(),
        collection_mint: ctx.accounts.org_mint.to_account_info(),
        metadata: ctx.accounts.metadata.to_account_info(),
        payer: ctx.accounts.authority.to_account_info(),
    };
    let verify_cpi_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_metadata_program.to_account_info(),
        verify_cpi_accounts,
        signer,
    );
    verify_sized_collection_item(verify_cpi_ctx, None)?;
    msg!("Collection verified");
    Ok(())
}
