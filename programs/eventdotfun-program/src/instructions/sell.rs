use anchor_lang::{
    prelude::*,
    system_program::{transfer, Transfer},
};
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{TokenAccount, TokenInterface},
};
use mpl_core::{
    accounts::BaseAssetV1,
    fetch_plugin,
    instructions::{BurnV1CpiBuilder, CreateV2CpiBuilder},
    types::{Attributes, PluginType},
    ID as MPL_CORE_ID,
};

use crate::{error::ErrorCode, BondingCurve};

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

    #[account(mut)]
    pub user: Signer<'info>,

    pub system_program: Program<'info, System>,

    #[account(address = MPL_CORE_ID)]
    /// CHECK: this account is checked by the address constraint
    pub mpl_core_program: UncheckedAccount<'info>,
}

impl<'info> Sell<'info> {
    pub fn sell(&mut self) -> Result<()> {
        let ticket_index = self.bonding_curve.current_ticket_sold;
        let initial_price = self.bonding_curve.initial_price;
        let exponent = self.bonding_curve.exponent;
        let multiplier = self.bonding_curve.multiplier;
        let min_ticket_to_sold = self.bonding_curve.min_ticket_to_sold;

        let now = Clock::get()?.unix_timestamp as u64;
        let end_at = self.bonding_curve.end_at;

        require!(now <= end_at, ErrorCode::CurveEnded);
        require!(
            ticket_index < min_ticket_to_sold,
            ErrorCode::CurveReachesThreshold
        );

        // 1. calculate lamports
        let lamports = initial_price + ((ticket_index - 1) ^ exponent as u64) * multiplier;

        // 2. update bonding curve state
        self.bonding_curve.current_ticket_sold -= 1_u64;
        self.bonding_curve.total_sol -= lamports;

        // 3. transfer from vault
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

        // 4. burn Core asset
        // let collection_key = self.collection.key();
        // let seeds = &[
        //     BondingCurve::SEED.as_bytes(),
        //     collection_key.as_ref(),
        //     &[self.bonding_curve.bump],
        // ];
        // let signer_seeds = &[&seeds[..]];

        BurnV1CpiBuilder::new(&self.mpl_core_program.to_account_info())
            .asset(&self.asset.to_account_info())
            .collection(Some(self.collection.as_ref()))
            .authority(Some(&self.user.to_account_info()))
            .payer(&self.user.to_account_info())
            .system_program(Some(&self.system_program.to_account_info()))
            .invoke()?;

        Ok(())
    }
}
