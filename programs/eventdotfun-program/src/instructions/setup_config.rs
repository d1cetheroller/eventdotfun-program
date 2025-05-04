use std::str::FromStr;

use anchor_lang::prelude::*;

use crate::{error::ErrorCode, Config};

#[derive(Accounts)]
pub struct SetupConfig<'info> {
    #[account(
        init_if_needed,
        space = Config::INIT_SPACE,
        payer = user,
        seeds = [Config::SEED.as_bytes()],
        bump
    )]
    pub config: Account<'info, Config>,

    #[account(
        mut,
        constraint = user.key() == Pubkey::from_str("2FKE2ooggeaLszVmKzLLSxZQAuHqmGmxA7ogFWbX2EE5").unwrap() @ ErrorCode::InvalidAuthority
    )]
    pub user: Signer<'info>,

    pub system_program: Program<'info, System>,
}

impl<'info> SetupConfig<'info> {
    pub fn initialize_config(
        &mut self,
        fee: u64,
        fee_recipient: Pubkey,
        bumps: &SetupConfigBumps,
    ) -> Result<()> {
        self.config.set_inner(Config {
            fee,
            fee_recipient,
            bump: bumps.config,
        });

        Ok(())
    }

    pub fn update_config(&mut self, fee: u64, fee_recipient: Pubkey) -> Result<()> {
        self.config.fee = fee;
        self.config.fee_recipient = fee_recipient;

        Ok(())
    }
}
