use anchor_lang::prelude::*;

#[account]
pub struct Config {
    pub fee: u64,
    pub fee_recipient: Pubkey,
    pub bump: u8,
}

impl Config {
    pub const INIT_SPACE: usize = 8 + 8 + 32 + 1;

    pub const SEED: &'static str = "config";
}
