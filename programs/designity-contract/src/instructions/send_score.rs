use anchor_lang::prelude::*;

use super::ScoreCTX;

pub fn send_score(ctx: Context<ScoreCTX>) -> Result<()> {
    let score = &mut ctx.accounts.score;
    let org = &mut ctx.accounts.org;
    assert_eq!(org.authority.key(), ctx.accounts.authority.key());
    score.reviews_sent += 1;
    Ok(())
}
