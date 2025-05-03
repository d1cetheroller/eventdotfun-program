use anchor_lang::prelude::*;

use crate::BondingCurve;

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

    /// CHECK:
    #[account(mut)]
    pub collection: UncheckedAccount<'info>,
    // #[account(mut)]
    // pub collection: Option<Account<'info, BaseCollectionV1>>,
    #[account(mut, seeds = [bonding_curve.key().as_ref()], bump)]
    pub vault: SystemAccount<'info>,

    #[account(mut)]
    pub asset: Signer<'info>,

    #[account(mut)]
    pub user: Signer<'info>,

    pub system_program: Program<'info, System>,

    // #[account(address = MPL_CORE_ID)]
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
        initial_price: f64,
        last_price: f64,
        max_ticket_to_sold: u64,
        bumps: &CreateBondingCurveBumps,
    ) -> Result<()> {
        let multiplier =
            (last_price - initial_price) / (max_ticket_to_sold ^ exponent as u64) as f64;

        self.bonding_curve.set_inner(BondingCurve {
            creator: self.user.key(),
            sales_type,
            start_at,
            end_at,
            exponent,
            initial_price,
            last_price,
            multiplier,
            max_ticket_to_sold,
            current_ticket_sold: 0,
            total_sol: 0,
            total_refund: 0,
            bump: bumps.bonding_curve,
            vault_bump: bumps.vault,
        });

        Ok(())
    }
}
