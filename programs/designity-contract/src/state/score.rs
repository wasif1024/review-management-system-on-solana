use anchor_lang::{prelude::*, system_program};

use crate::utils::Realloc;

use super::Org;

#[account]
pub struct Score {
    pub name: String,
    pub scores: Vec<f32>,
    pub scores_sum: Vec<f32>,
    pub applicant: Pubkey,
    pub mint: Pubkey,
    pub reviews_recieved: Vec<u16>,
    pub reviews_sent: u16,
    pub levels: Vec<u8>,
    pub last_update: i64,
    pub bump: u8,
}

impl Score {
    pub fn new(bump: u8) -> Self {
        Self {
            name: "".to_string(),
            scores: vec![],
            scores_sum: vec![],
            reviews_recieved: vec![],
            reviews_sent: 0,
            applicant: Pubkey::new_from_array([0; 32_usize]),
            mint: Pubkey::new_from_array([0; 32_usize]),
            levels: vec![],
            last_update: 0,
            bump,
        }
    }

    pub fn update_scores(&mut self, org: &Account<'_, Org>) {
        msg!("Calculating new score");
        let mut r_index = 0;
        let mut group_sum = 0 as f32;
        let mut counter = 0 as f32;
        let mut next;
        for (p1, _) in self.scores_sum.clone().iter().enumerate() {
            if r_index > org.ranges.len() - 1 {
                next = org.weights.len() as u8;
            } else {
                next = org.ranges[r_index];
            }
            if self.reviews_recieved[p1] != 0 {
                group_sum +=
                    self.scores_sum[p1] * org.weights[p1] / self.reviews_recieved[p1] as f32;
                counter += org.weights[p1];
            }
            if p1 >= next as usize - 1 {
                let avg = group_sum / counter;
                self.scores[r_index] = avg;
                r_index += 1;
                group_sum = 0 as f32;
                counter = 0 as f32;
            }
        }
    }

    pub fn calculate_potential_level(&self, org: &Account<'_, Org>) -> Vec<u8> {
        msg!("Updating potential levels");
        let mut levels: Vec<u8> = vec![0_u8; self.levels.len()];
        //let mut dumm_level=0.0;
        for (p1, e1) in self.scores.iter().enumerate() {
            let mut level = 0;
            //for (inner_index,l) in org.levels[p1].iter().enumerate() {
            for l in org.levels[p1].iter() {
                if l < e1 {
                    /*if p1==1&&inner_index==0{
                        dumm_level=l+26.0;
                    }else{
                        dumm_level=l+1.0;
                    }
                msg!("dummy values {}",&dumm_level);
                if &dumm_level < e1 {*/
                    level += 1
                } else {
                    break;
                }
            }
            levels[p1] = level;
        }
        msg!("potential levels:{:?}", levels);
        levels
    }

    pub fn calculate_next_level(&self, potential_levels: Vec<u8>) -> Vec<u8> {
        msg!("Calculating next level");
        let mut levels = self.levels.clone();
        for (p, e1) in self.levels.iter().enumerate() {
            if *e1 < potential_levels[p] {
                levels[p] += 1;
                break;
            } else if *e1 > potential_levels[p] {
                levels[p] -= 1;
                break;
            }
        }
        msg!("next levels:{:?}", levels);
        levels
    }

    pub fn reconcile(&mut self, org: &Account<'_, Org>) -> Vec<u8> {
        self.update_scores(org);
        let potential_levels = self.calculate_potential_level(org);
        self.calculate_next_level(potential_levels)
    }
}

impl<'info> Realloc<'info> for Account<'info, Score> {
    fn realloc(
        &mut self,
        space_to_add: usize,
        payer: &Signer<'info>,
        system_program: &Program<'info, System>,
    ) -> Result<()> {
        let account_info = self.to_account_info();
        let new_account_size = account_info.data_len() + space_to_add;

        // Determine additional rent required
        let lamports_required = (Rent::get()?).minimum_balance(new_account_size);
        let additional_rent_to_fund = lamports_required - account_info.lamports();

        // Perform transfer of additional rent
        system_program::transfer(
            CpiContext::new(
                system_program.to_account_info(),
                system_program::Transfer {
                    from: payer.to_account_info(),
                    to: account_info.clone(),
                },
            ),
            additional_rent_to_fund,
        )?;

        // Reallocate the account
        account_info.realloc(new_account_size, false)?;
        Ok(())
    }
}
