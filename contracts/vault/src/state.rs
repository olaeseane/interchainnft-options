use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Api, DepsMut, StdResult, Storage};
use cw_storage_plus::{Item, Map};
use cw_utils::Expiration;

use common::{
    errors::ContractError,
    types::{AssetId, TokenId},
};
use macros::ConfigStorage;

use crate::msg::InstantiateMsg;

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
pub struct Asset {
    /// The address that can claim the asset when it is free of entitlements.
    pub beneficial_owner: Option<Addr>,
    pub operator: Option<Addr>,
    pub expiry: Option<Expiration>,
}

/// Current entitlements applied to each asset
pub const ASSETS: Map<&AssetId, Asset> = Map::new("assets");

// update some attributes for a particular asset within the vault or create new one
pub fn update_or_create_asset(
    deps: DepsMut,
    asset_id: &AssetId,
    new_asset: Asset,
) -> StdResult<()> {
    let updated_asset = new_asset.clone();
    if let Err(ContractError::AssetNotFound(asset_id)) =
        ASSETS.update::<_, ContractError>(deps.storage, asset_id, |a| {
            let mut asset = a.ok_or(ContractError::AssetNotFound(asset_id.to_string()))?;
            asset.beneficial_owner = updated_asset.beneficial_owner.or(asset.beneficial_owner);
            asset.operator = updated_asset.operator.or(asset.operator);
            asset.expiry = updated_asset.expiry.or(asset.expiry);
            Ok(asset)
        })
    {
        ASSETS.save(deps.storage, &asset_id, &new_asset)?;
    };

    Ok(())
}

/// Mapping from asset ID to approved address
pub const APPROVALS: Map<&AssetId, Addr> = Map::new("approvals");

// TODO assetId == tokenId?
// TODO use IndexedMap
