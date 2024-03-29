use anchor_lang::prelude::*;

mod instructions;
mod state;
mod utils;

use instructions::*;

declare_id!("86kpZGxTEDUF3VShNZfykKM8ow1TEeuHEzJ3tduLFyQY");

#[program]
pub mod designity {
    use super::*;

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
        instructions::create_organization(
            ctx,
            weights,
            ranges,
            levels,
            name,
            min_reviews,
            domain,
            level_wait,
        )
    }

    pub fn register(
        ctx: Context<RegisterCTX>,
        name: String,
        levels: Vec<u8>,
        last_update: i64,
    ) -> Result<()> {
        instructions::register(ctx, name, levels, last_update)
    }

    pub fn receive_score(
        ctx: Context<ScoreCTX>,
        scores: Vec<f32>,
        submission_ts: i64,
    ) -> Result<()> {
        instructions::receive_score(ctx, scores, submission_ts)
    }

    pub fn verify(ctx: Context<VerifyCTX>) -> Result<()> {
        instructions::verify(ctx)
    }

    pub fn send_score(ctx: Context<ScoreCTX>) -> Result<()> {
        instructions::send_score(ctx)
    }

    pub fn update_scores(
        ctx: Context<ScoreCTX>,
        scores_sum: Vec<f32>,
        reviews_recieved: Vec<u16>,
        last_update: i64,
        levels: Vec<u8>,
        override_levels: bool,
    ) -> Result<()> {
        instructions::update_scores(
            ctx,
            scores_sum,
            reviews_recieved,
            last_update,
            levels,
            override_levels,
        )
    }
}
