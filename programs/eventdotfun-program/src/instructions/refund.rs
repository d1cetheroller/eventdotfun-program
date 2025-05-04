use anchor_lang::{
    prelude::*,
    system_program::{transfer, Transfer},
};
use mpl_core::{
    accounts::{BaseAssetV1, BaseCollectionV1},
    fetch_plugin,
    instructions::{BurnV1CpiBuilder, CreateV2CpiBuilder},
    types::{Attributes, PluginType},
    ID as MPL_CORE_ID,
};

use crate::{error::ErrorCode, BondingCurve};

#[derive(Accounts)]
pub struct Refund<'info> {
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
    pub collection: Account<'info, BaseCollectionV1>,

    #[account(mut)]
    pub asset: Account<'info, BaseAssetV1>,

    #[account(mut)]
    pub user: Signer<'info>,

    pub system_program: Program<'info, System>,

    #[account(address = MPL_CORE_ID)]
    /// CHECK: this account is checked by the address constraint
    pub mpl_core_program: UncheckedAccount<'info>,
}

impl<'info> Refund<'info> {
    pub fn refund(&mut self) -> Result<()> {
        let now = Clock::get()?.unix_timestamp as u64;
        let end = self.bonding_curve.end_at;
        let current_ticket_to_sold = self.bonding_curve.current_ticket_sold;
        let min_ticket_to_sold = self.bonding_curve.min_ticket_to_sold;
        let sales_type = self.bonding_curve.sales_type;

        if sales_type == 1 {
            require!(now >= end, ErrorCode::CurveStillOnProgress);
            require!(
                current_ticket_to_sold < min_ticket_to_sold,
                ErrorCode::CurveReachesThreshold
            );

            BurnV1CpiBuilder::new(&self.mpl_core_program.to_account_info())
                .asset(&self.asset.to_account_info())
                .collection(Some(self.collection.as_ref()))
                .authority(Some(&self.user.to_account_info()))
                .payer(&self.user.to_account_info())
                .system_program(Some(&self.system_program.to_account_info()))
                .invoke()?;
        } else if sales_type == 2 {
            let refund_at = end + self.bonding_curve.refund_window;
            require!(now >= refund_at, ErrorCode::RefundNotOpened);
        }

        let (_, asset_attribute_list, _) = fetch_plugin::<BaseAssetV1, Attributes>(
            &self.asset.to_account_info(),
            PluginType::Attributes,
        )?;

        let ticket_number_attribute = asset_attribute_list
            .attribute_list
            .iter()
            .find(|attr| attr.key == "Ticket Number")
            .unwrap();

        let ticket_number = ticket_number_attribute.value.parse::<u64>().unwrap();

        let initial_price = self.bonding_curve.initial_price;
        let exponent = self.bonding_curve.exponent;
        let multiplier = self.bonding_curve.multiplier;
        let lamports = initial_price + (ticket_number ^ exponent as u64) * multiplier;

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
        transfer(cpi_ctx, lamports)?;

        self.bonding_curve.total_refund += lamports;

        Ok(())
    }
}
