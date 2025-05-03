use anchor_lang::{
    prelude::*,
    system_program::{transfer, Transfer},
};
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{TokenAccount, TokenInterface},
};
use mpl_core::{
    instructions::{BurnV1CpiBuilder, CreateV2CpiBuilder},
    ID as MPL_CORE_ID,
};

use crate::BondingCurve;

#[derive(Accounts)]
pub struct Sell<'info> {
    #[account(
        mut,
        seeds = [BondingCurve::SEED.as_bytes(), collection.key().as_ref()],
        bump = bonding_curve.bump
    )]
    pub bonding_curve: Account<'info, BondingCurve>,

    #[account(mut, seeds = [b"vault", bonding_curve.key().as_ref()], bump = bonding_curve.vault_bump)]
    pub vault: SystemAccount<'info>,

    /// CHECK:
    #[account(mut, constraint = collection.key() == bonding_curve.collection)]
    pub collection: UncheckedAccount<'info>,

    /// CHECK:
    #[account(mut)]
    pub asset: UncheckedAccount<'info>,

    // #[account(mut)]
    // pub user_associated_token_account: InterfaceAccount<'info, TokenAccount>,
    #[account(mut)]
    pub user: Signer<'info>,

    pub system_program: Program<'info, System>,

    // pub associated_token_program: Program<'info, AssociatedToken>,

    // pub token_program: Interface<'info, TokenInterface>,
    #[account(address = MPL_CORE_ID)]
    /// CHECK: this account is checked by the address constraint
    pub mpl_core_program: UncheckedAccount<'info>,
}

impl<'info> Sell<'info> {
    pub fn sell(&mut self, num_of_tickets: u8) -> Result<()> {
        let mut total_lamports = 0;

        for _i in 0..num_of_tickets {
            let ticket_index = self.bonding_curve.current_ticket_sold;
            let initial_price = self.bonding_curve.initial_price;
            let exponent = self.bonding_curve.exponent;
            let multiplier = self.bonding_curve.multiplier;

            total_lamports = initial_price + ((ticket_index - 1) ^ exponent as u64) * multiplier;
        }

        self.bonding_curve.current_ticket_sold -= num_of_tickets as u64;
        self.bonding_curve.total_sol += total_lamports;

        let cpi_program = self.system_program.to_account_info();
        let cpi_account = Transfer {
            from: self.vault.to_account_info(),
            to: self.user.to_account_info(),
        };

        let bonding_curve_key = self.bonding_curve.key();
        let seeds = &[
            b"vault",
            bonding_curve_key.as_ref(),
            &[self.bonding_curve.vault_bump],
        ];
        let signer_seeds = &[&seeds[..]];

        let cpi_ctx = CpiContext::new_with_signer(cpi_program.clone(), cpi_account, signer_seeds);
        transfer(cpi_ctx, total_lamports)?;

        let collection_key = self.collection.key();
        let seeds = &[
            BondingCurve::SEED.as_bytes(),
            collection_key.as_ref(),
            &[self.bonding_curve.bump],
        ];
        let signer_seeds = &[&seeds[..]];

        for _ in 0..num_of_tickets {
            BurnV1CpiBuilder::new(&self.mpl_core_program.to_account_info())
                .asset(&self.asset.to_account_info())
                .collection(Some(self.collection.as_ref()))
                .authority(Some(&self.user.to_account_info()))
                .payer(&self.user.to_account_info())
                .system_program(Some(&self.system_program.to_account_info()))
                .invoke()?;
            // .invoke_signed(signer_seeds)?;
        }

        Ok(())
    }
}
