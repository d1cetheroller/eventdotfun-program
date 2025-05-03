#![allow(unexpected_cfgs)]

pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("H8jsrvMP58WfPdBHNyjbcLLjik1a8VJ4udA2aD6n5uox");

#[program]
pub mod eventdotfun_program {
    use super::*;

    pub fn initialize(ctx: Context<SetupConfig>, fee: u64, fee_recipient: Pubkey) -> Result<()> {
        ctx.accounts
            .initialize_config(fee, fee_recipient, &ctx.bumps)
    }

    pub fn update_config(ctx: Context<SetupConfig>, fee: u64, fee_recipient: Pubkey) -> Result<()> {
        ctx.accounts.update_config(fee, fee_recipient)
    }

    pub fn create_bonding_curve(
        ctx: Context<CreateBondingCurve>,
        sales_type: u8,
        start_at: u64,
        end_at: u64,
        exponent: u8,
        initial_price: f64,
        last_price: f64,
        max_ticket_to_sold: u64,
    ) -> Result<()> {
        ctx.accounts.create_bonding_curve(
            sales_type,
            start_at,
            end_at,
            exponent,
            initial_price,
            last_price,
            max_ticket_to_sold,
            &ctx.bumps,
        )
    }
}
