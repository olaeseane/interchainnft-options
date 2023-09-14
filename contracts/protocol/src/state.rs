use cosmwasm_schema::cw_serde;
use cosmwasm_std::StdResult;
use cosmwasm_std::{Addr, Api, StdError, Storage};
use cw_storage_plus::Item;
use cw_utils::Expiration;

use common::errors::ContractError;
use macros::ConfigStorage;
use rbac::Role;

use crate::msg::InstantiateMsg;

/// Saves factory settings
pub const CONFIG: Item<Config> = Item::new("config");

/// This structure holds the main parameters for the ?
#[cw_serde]
#[derive(ConfigStorage)]
pub struct Config {
    /// The address of the deployed vault factory contract.
    pub vault_factory_addr: Addr,
    /// The address of the deployed covered call factory contract.
    pub option_factory_addr: Addr,
}

impl From<InstantiateMsg> for Config {
    fn from(value: InstantiateMsg) -> Self {
        Self {
            vault_factory_addr: Addr::unchecked(value.vault_factory_addr),
            option_factory_addr: Addr::unchecked(value.option_factory_addr),
        }
    }
}

impl Config {
    pub fn validate(&self, api: &dyn Api) -> Result<(), ContractError> {
        api.addr_validate(self.vault_factory_addr.as_str())?;
        api.addr_validate(self.option_factory_addr.as_str())?;
        Ok(())
    }

    pub fn update(
        store: &mut dyn Storage,
        new_vault_factory: Option<Addr>,
        new_option_factory: Option<Addr>,
    ) -> StdResult<()> {
        CONFIG
            .update::<_, StdError>(store, |mut config| {
                config.option_factory_addr =
                    new_option_factory.unwrap_or(config.option_factory_addr);
                config.vault_factory_addr = new_vault_factory.unwrap_or(config.vault_factory_addr);
                Ok(config)
            })
            .map(|_| ())
    }
}

/*
// TODO add to Config?
   // pub covered_call_contract: Addr,
   // pub vault_contract: Addr,
   /// Default min amount of the current bid that the new bid
   /// must exceed the current bid by in order to be considered valid.
   /// This amount is expressed in basis points (i.e. 1/100th of 1%)
   min_bid_inc_bips: u16,
   /// Default amount of time before the expiration of the option
   /// that the settlement auction will begin.
   settlement_auction_start_offset: Duration,
   market_paused: bool,
   min_option_duration: Duration,
*/

/// Roles
pub const ADMINS: rbac::Role = rbac::Role::new("admins");
pub const PAUSERS: rbac::Role = rbac::Role::new("pausers");
pub const UPGRADERS: Role = Role::new("upgraders");

/// The time the Protocol will unpause.
pub const PAUSED: Item<Expiration> = Item::new("paused");

// TODO use IndexedMap
// TODO mapping(address => mapping(bytes32 => bool)) collectionConfigs;
