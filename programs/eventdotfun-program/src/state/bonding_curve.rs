use anchor_lang::prelude::*;

#[account]
pub struct BondingCurve {
    pub creator: Pubkey,
    pub sales_type: u8,
    pub start_at: u64,
    pub end_at: u64,

    pub collection: Pubkey,

    pub exponent: u8,
    pub initial_price: u64,
    pub last_price: u64,
    pub multiplier: u64,
    pub max_ticket_to_sold: u64,
    pub current_ticket_sold: u64,

    pub total_sol: u64,
    pub total_refund: u64,

    pub bump: u8,
    pub vault_bump: u8,
}

impl BondingCurve {
    pub const INIT_SPACE: usize = 8 + 32 + 1 + 8 + 8 + 32 + 1 + (8 * 7) + 1 + 1;

    pub const SEED: &'static str = "bonding_curve";
}
