use anchor_lang::{prelude::*, system_program};

use crate::utils::Realloc;

#[account]
// #[derive(Default)]
pub struct Org {
    pub name: String,
    pub min_reviews: u8,
    pub weights: Vec<f32>,
    pub ranges: Vec<u8>,
    pub levels: Vec<Vec<f32>>,
    pub mint: Pubkey,
    pub authority: Pubkey,
    pub domain: String,
    pub bump: u8,
    pub level_wait: i32,
}

impl<'info> Realloc<'info> for Account<'info, Org> {
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
