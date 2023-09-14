use cosmwasm_std::{to_binary, Binary, Deps, Env, StdResult};
use cw_utils::Expiration;

use common::{nft, types::AssetId};

use crate::{
    msg::CurrentEntitlementOperatorResponse,
    state::{Config, APPROVALS, ASSETS},
    utils::has_active_entitlement,
};

/// Looks up the current beneficial owner of the asset.
pub fn beneficial_owner(deps: Deps, asset_id: &AssetId) -> StdResult<Binary> {
    let asset = ASSETS.load(deps.storage, asset_id)?;
    to_binary(&asset.beneficial_owner)
}

/// Checks if the asset is currently stored in the vault.
pub fn holds_asset(
    deps: Deps,
    env: &Env,
    asset_id: &AssetId,
    config: &Config,
) -> StdResult<Binary> {
    let owner = nft::owner_of(&deps.querier, &config.nft_addr, asset_id)?.owner;
    to_binary(&(owner == env.contract.address))
}

/// Returns the contract address of the vaulted asset.
pub fn asset_address(config: &Config) -> StdResult<Binary> {
    to_binary(&config.nft_addr)
}

// TODO fix description
/// Returns the account approved for `tokenId` token.
pub fn approved_operator(deps: Deps, asset_id: &AssetId) -> StdResult<Binary> {
    let operator = APPROVALS.may_load(deps.storage, asset_id)?;
    to_binary(&operator)
}

/// Looks up the expiration timestamp of the current entitlement.
pub fn entitlement_expiration(deps: Deps, env: &Env, asset_id: &AssetId) -> StdResult<Binary> {
    let asset = ASSETS.load(deps.storage, asset_id)?;
    if !has_active_entitlement(&asset, env) {
        return to_binary::<Option<Expiration>>(&None);
    }
    to_binary(&Some(asset.expiry))
}

/// Looks up the current operator of an entitlement on an asset.
pub fn current_entitlement_operator(
    deps: Deps,
    env: &Env,
    asset_id: &AssetId,
) -> StdResult<Binary> {
    let asset = ASSETS.load(deps.storage, asset_id)?;
    let is_active = has_active_entitlement(&asset, env);

    to_binary(&CurrentEntitlementOperatorResponse {
        is_active,
        operator: asset.operator,
    })
}
