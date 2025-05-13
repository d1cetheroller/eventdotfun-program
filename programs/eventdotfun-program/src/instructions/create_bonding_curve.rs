use anchor_lang::prelude::*;
use mpl_core::{
    instructions::{CreateCollectionV2CpiBuilder, CreateV2CpiBuilder},
    ID as MPL_CORE_ID,
};

use crate::{error::ErrorCode, BondingCurve};
#[derive(Accounts)]
pub struct CreateBondingCurve<'info> {
    #[account(
        init,
        space = BondingCurve::INIT_SPACE,
        payer = user,
        seeds = [BondingCurve::SEED.as_bytes(), collection.key().as_ref()],
        bump
    )]
    pub bonding_curve: Account<'info, BondingCurve>,

    #[account(mut, seeds = [b"vault", bonding_curve.key().as_ref()], bump)]
    pub vault: SystemAccount<'info>,

    #[account(mut)]
    pub collection: Signer<'info>,

    #[account(mut)]
    pub user: Signer<'info>,

    pub system_program: Program<'info, System>,

    #[account(address = MPL_CORE_ID)]
    /// CHECK: this account is checked by the address constraint
    pub mpl_core_program: UncheckedAccount<'info>,
}

impl<'info> CreateBondingCurve<'info> {
    pub fn create_bonding_curve(
        &mut self,
        sales_type: u8,
        start_at: u64,
        end_at: u64,
        exponent: u8,
        initial_price: u64,
        last_price: u64,
        min_ticket_to_sold: u64,
        max_ticket_to_sold: u64,
        refund_window: u64,
        bumps: &CreateBondingCurveBumps,
    ) -> Result<()> {
        require!(
            sales_type > 0 || sales_type < 3,
            ErrorCode::InvalidSalesType
        );

        let now = Clock::get()?.unix_timestamp as u64;
        require!(
            start_at > now && end_at > start_at,
            ErrorCode::InvalidTimestamp
        );

        require!(exponent > 0 || exponent < 3, ErrorCode::InvalidExponent);
        require!(
            initial_price > 0 || last_price >= initial_price,
            ErrorCode::InvalidPrice
        );
        require!(
            min_ticket_to_sold < max_ticket_to_sold,
            ErrorCode::InvalidTicketConfiguration
        );

        let multiplier = (last_price - initial_price) / (min_ticket_to_sold ^ exponent as u64);

        self.bonding_curve.set_inner(BondingCurve {
            creator: self.user.key(),
            sales_type,
            start_at,
            end_at,
            collection: self.collection.key(),
            exponent,
            initial_price,
            last_price,
            multiplier,
            max_ticket_to_sold,
            current_ticket_sold: 0,
            min_ticket_to_sold,
            total_sol: 0,
            total_refund: 0,
            refund_window,
            bump: bumps.bonding_curve,
            vault_bump: bumps.vault,
        });

        CreateCollectionV2CpiBuilder::new(&self.mpl_core_program.to_account_info())
            .collection(&self.collection.to_account_info())
            .payer(&self.user.to_account_info())
            .update_authority(Some(self.bonding_curve.as_ref()))
            .system_program(&self.system_program.to_account_info())
            .name("EVENTDOTFUN Collection".to_owned())
            .uri("https://devnet.irys.xyz/2x17GpZTmXPKGGiUKuaQA5b9jg3tQuG7VquatBLdkFB2".to_owned())
            .invoke()?;

        Ok(())
    }
}
