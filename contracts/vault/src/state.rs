use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Api, DepsMut, StdResult, Storage};
use cw_storage_plus::{Item, Map};
use cw_utils::Expiration;

use common::{
    errors::ContractError,
    types::{AssetId, TokenId},
};
use macros::ConfigStorage;

use crate::msg::{InstantiateMsg, SetEntitlement};

/// Saves factory settings
pub const CONFIG: Item<Config> = Item::new("config");

#[cw_serde]
#[derive(ConfigStorage)]
pub struct Config {
    /// The NFT contract address the vault is covering
    pub nft_addr: Addr,
    /// The NFT token id the vault is covering. None means multi vault
    pub nft_id: Option<TokenId>,
    /// The main protocol contract address
    pub protocol_addr: Addr,
}

impl Config {
    pub fn validate(&self, api: &dyn Api) -> Result<(), ContractError> {
        api.addr_validate(self.protocol_addr.as_str())?;
        Ok(())
    }
}

impl From<InstantiateMsg> for Config {
    fn from(val: InstantiateMsg) -> Self {
        Config {
            nft_addr: Addr::unchecked(val.nft_addr),
            protocol_addr: Addr::unchecked(val.protocol_addr),
            nft_id: val.nft_id,
        }
    }
}

#[cw_serde]
/// Entitlements applied to each asset, which includes the beneficialOwner for the asset and the entitled operator
/// If the entitled operator field is non-null, it means an unreleased entitlement has been applied
pub struct Entitlement {
    /// The address that can claim the asset when it is free of entitlements.
    pub beneficial_owner: Option<Addr>, // TODO without Option?
    /// The operating contract that can change ownership during the entitlement period.
    pub operator: Option<Addr>,
    /// The block timestamp after which the asset is free of the entitlement.
    pub expiry: Option<Expiration>,
}

impl From<SetEntitlement> for Entitlement {
    fn from(value: SetEntitlement) -> Self {
        Self {
            beneficial_owner: Some(value.beneficial_owner),
            operator: Some(value.entitled_operator),
            expiry: Some(value.expiry),
        }
    }
}

/// Current entitlements applied to each asset
pub const ASSETS: Map<&AssetId, Entitlement> = Map::new("assets");

// update some attributes for a particular asset within the vault or create new one
pub fn update_or_create_entitlement(
    deps: DepsMut,
    asset_id: &AssetId,
    entitlement: &Entitlement,
) -> StdResult<()> {
    let entitlement_clone = entitlement.clone();
    if let Err(ContractError::AssetNotFound(asset_id)) =
        ASSETS.update::<_, ContractError>(deps.storage, asset_id, |e| {
            let mut entitlement = e.ok_or(ContractError::AssetNotFound(asset_id.to_string()))?;
            entitlement.beneficial_owner = entitlement_clone
                .beneficial_owner
                .or(entitlement.beneficial_owner);
            entitlement.operator = entitlement_clone.operator.or(entitlement.operator);
            entitlement.expiry = entitlement_clone.expiry.or(entitlement.expiry);
            Ok(entitlement)
        })
    {
        ASSETS.save(deps.storage, &asset_id, entitlement)?;
    };

    Ok(())
}

/// Mapping from asset ID to approved address
pub const APPROVALS: Map<&AssetId, Addr> = Map::new("approvals");

// TODO assetId == tokenId?
// TODO use IndexedMap
