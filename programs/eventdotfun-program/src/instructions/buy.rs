use anchor_lang::{
    prelude::*,
    system_program::{transfer, Transfer},
};
use mpl_core::{
    accounts::{BaseAssetV1, BaseCollectionV1},
    fetch_external_plugin_adapter, fetch_external_plugin_adapter_data_info,
    instructions::{
        BurnV1CpiBuilder, CreateV2CpiBuilder, WriteExternalPluginAdapterDataV1CpiBuilder,
    },
    types::{
        AppDataInitInfo, Attribute, Attributes, ExternalPluginAdapter,
        ExternalPluginAdapterInitInfo, ExternalPluginAdapterKey, ExternalPluginAdapterSchema,
        Plugin, PluginAuthority, PluginAuthorityPair,
    },
    ID as MPL_CORE_ID,
};

use crate::{error::ErrorCode, BondingCurve};

#[derive(Accounts)]
pub struct Buy<'info> {
    #[account(
        mut,
        seeds = [BondingCurve::SEED.as_bytes(), collection.key().as_ref()],
        bump = bonding_curve.bump
    )]
    pub bonding_curve: Account<'info, BondingCurve>,

    #[account(mut, seeds = [b"vault", bonding_curve.key().as_ref()], bump = bonding_curve.vault_bump)]
    pub vault: SystemAccount<'info>,

    #[account(mut, constraint = collection.key() == bonding_curve.collection)]
    pub collection: Account<'info, BaseCollectionV1>,

    #[account(mut)]
    pub asset: Signer<'info>,

    #[account(mut)]
    pub user: Signer<'info>,

    pub system_program: Program<'info, System>,

    #[account(address = MPL_CORE_ID)]
    /// CHECK: this account is checked by the address constraint
    pub mpl_core_program: UncheckedAccount<'info>,
}

impl<'info> Buy<'info> {
    pub fn buy(&mut self) -> Result<()> {
        let ticket_index = self.bonding_curve.current_ticket_sold + 1;
        let min_ticket_to_sold = self.bonding_curve.min_ticket_to_sold;
        let max_ticket_to_sold = self.bonding_curve.max_ticket_to_sold;
        let current_ticket_sold = self.bonding_curve.current_ticket_sold;

        let now = Clock::get()?.unix_timestamp as u64;
        let start_at = self.bonding_curve.start_at;
        let end_at = self.bonding_curve.end_at;

        let initial_price = self.bonding_curve.initial_price;
        let exponent = self.bonding_curve.exponent;
        let multiplier = self.bonding_curve.multiplier;

        require!(now >= start_at, ErrorCode::CurveNotStarted);
        require!(now <= end_at, ErrorCode::CurveEnded);
        require!(
            ticket_index <= max_ticket_to_sold,
            ErrorCode::MaxTicketReached
        );

        // 1. calculate lamports
        let lamports = if current_ticket_sold > min_ticket_to_sold {
            initial_price + (min_ticket_to_sold ^ exponent as u64) * multiplier
        } else {
            initial_price + (ticket_index ^ exponent as u64) * multiplier
        };

        // 2. update bonding curve state
        self.bonding_curve.current_ticket_sold += 1_u64;
        self.bonding_curve.total_sol += lamports;

        // 3. transfer to vault
        let cpi_program = self.system_program.to_account_info();
        let cpi_account = Transfer {
            from: self.user.to_account_info(),
            to: self.vault.to_account_info(),
        };
        let cpi_ctx = CpiContext::new(cpi_program.clone(), cpi_account);
        transfer(cpi_ctx, lamports)?;

        // 4. create Core asset
        let mut ticket_plugin: Vec<PluginAuthorityPair> = vec![];

        let attribute_list: Vec<Attribute> = vec![Attribute {
            key: "Ticket Number".to_string(),
            value: ticket_index.to_string(),
        }];

        ticket_plugin.push(PluginAuthorityPair {
            plugin: Plugin::Attributes(Attributes { attribute_list }),
            authority: Some(PluginAuthority::UpdateAuthority),
        });

        let collection_key = self.collection.key();

        let seeds = &[
            BondingCurve::SEED.as_bytes(),
            collection_key.as_ref(),
            &[self.bonding_curve.bump],
        ];
        let signer_seeds = &[&seeds[..]];

        CreateV2CpiBuilder::new(&self.mpl_core_program.to_account_info())
            .asset(&self.asset.to_account_info())
            .name(("EVENTDOTFUN Ticket").to_owned())
            .collection(Some(self.collection.as_ref()))
            .uri("https://devnet.irys.xyz/2x17GpZTmXPKGGiUKuaQA5b9jg3tQuG7VquatBLdkFB2".to_owned())
            .plugins(ticket_plugin)
            .owner(Some(self.user.as_ref()))
            .payer(&self.user.to_account_info())
            .authority(Some(self.bonding_curve.as_ref()))
            .system_program(&self.system_program.to_account_info())
            .invoke_signed(signer_seeds)?;

        Ok(())
    }
}
