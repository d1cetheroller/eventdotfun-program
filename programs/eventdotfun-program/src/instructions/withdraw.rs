use anchor_lang::{
    prelude::*,
    system_program::{transfer, Transfer},
};

use crate::{error::ErrorCode, BondingCurve};

#[derive(Accounts)]
pub struct Withdraw<'info> {
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

    #[account(mut)]
    pub user: Signer<'info>,

    pub system_program: Program<'info, System>,
}

impl<'info> Withdraw<'info> {
    pub fn withdraw(&mut self) -> Result<()> {
        let now = Clock::get()?.unix_timestamp as u64;
        let end = self.bonding_curve.end_at;
        let current_ticket_to_sold = self.bonding_curve.current_ticket_sold;
        let min_ticket_to_sold = self.bonding_curve.min_ticket_to_sold;

        require!(now >= end, ErrorCode::CurveStillOnProgress);
        require!(
            current_ticket_to_sold >= min_ticket_to_sold,
            ErrorCode::CurveStillBelowThreshold
        );

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
        transfer(cpi_ctx, self.vault.lamports())?;

        Ok(())
    }
}
