use common::errors::ContractError;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Api, StdResult, Storage, Uint128};
use cw_storage_plus::{Item, Map};

use macros::ConfigStorage;

use crate::msg::InstantiateMsg;

/// Saves factory settings
pub const CONFIG: Item<Config> = Item::new("config");

#[cw_serde]
#[derive(ConfigStorage)]
pub struct Config {
    /// The main protocol contract address
    pub protocol_addr: Addr,
    /// Valut contract code identifier
    pub call_code_id: u64,
    /// Symbol of call option nft
    pub nft_symbol: String,
    /// Name of call option nft
    pub nft_name: String,
    /// Default min duration of an created option
    pub default_minimum_option_duration: u64,
    /// TODO
    pub default_allowed_denom: String,
    /// TODO
    pub default_min_bid_inc_bips: Uint128,
    /// The address of the vault factory
    pub vault_factory_addr: String,
}

impl Config {
    pub fn validate(&self, api: &dyn Api) -> Result<(), ContractError> {
        api.addr_validate(self.protocol_addr.as_str())?;
        Ok(())
    }
}

impl From<InstantiateMsg> for Config {
    fn from(value: InstantiateMsg) -> Self {
        Config {
            protocol_addr: Addr::unchecked(value.protocol_addr),
            call_code_id: value.call_code_id,
            default_minimum_option_duration: value.default_minimum_option_duration,
            default_allowed_denom: value.default_allowed_denom,
            default_min_bid_inc_bips: value.default_min_bid_inc_bips,
            nft_symbol: value.nft_symbol,
            nft_name: value.nft_name,
            vault_factory_addr: value.vault_factory_addr,
        }
    }
}

/// Registry of all of the active markets projects with supported call instruments
pub(crate) const CALL_INSTRUMENTS: Map<&Addr, Addr> = Map::new("call_instrument");

/// This is an intermediate structure for storing a vault info. It is used in a submessage response.
#[cw_serde]
pub struct TmpInstrumentInfo {
    pub nft_addr: Addr,
}

/// Saves a temporary vault info for submessages response.
pub const TMP_INSTRUMENT: Item<TmpInstrumentInfo> = Item::new("tmp_instrument_info");

// TODO use IndexedMap
// TODO maybe remove protocol_addr
