use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Api, StdResult, Storage};
use cw_storage_plus::{Item, Map};

use common::{errors::ContractError, types::TokenId};
use macros::ConfigStorage;

use crate::msg::InstantiateMsg;

/// Saves factory settings
pub const CONFIG: Item<Config> = Item::new("config");

#[cw_serde]
#[derive(ConfigStorage)]
pub struct Config {
    /// The main protocol contract address
    pub protocol_addr: Addr,
    /// Vault contract code identifier
    pub vault_code_id: u64,
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
            vault_code_id: value.vault_code_id,
        }
    }
}

/// Registry of all of the active multi-vaults within the protocol
pub(crate) const MULTI_VAULTS: Map<&Addr, Addr> = Map::new("multi_vaults");

/// Registry of all of the active vaults within the protocol, allowing users
/// to find vaults by project address and tokenId;
pub(crate) const SOLO_VAULTS: Map<(&Addr, &TokenId), Addr> = Map::new("solo_vaults");

/// This is an intermediate structure for storing a vault info. It is used in a submessage response.
#[cw_serde]
pub struct TmpVaultInfo {
    pub nft_addr: Addr,
    pub nft_id: Option<TokenId>,
}

/// Saves a temporary vault info for submessages response.
pub const TMP_VAULT: Item<TmpVaultInfo> = Item::new("tmp_vault_info");

// TODO use IndexedMap
// TODO maybe remove protocol_addr
