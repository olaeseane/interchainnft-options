use cosmwasm_std::{ensure, from_binary, Addr, DepsMut, Env, MessageInfo, Response, StdError};
use cw_utils::{maybe_addr, Expiration};

use common::{errors::ContractError, nft, types::AssetId};

use crate::{
    msg::Entitlement,
    state::{update_or_create_asset, Asset, Config, APPROVALS, ASSETS},
    utils::{has_active_entitlement, register_entitlement},
};

/// Add an entitlement claim to the asset held within the vault
pub fn impose_entitlement(
    deps: DepsMut,
    env: &Env,
    sender: &Addr,
    asset_id: AssetId,
    operator: String,
    expiry: Expiration,
    // config: &Config,
) -> Result<Response, ContractError> {
    let asset = ASSETS
        .may_load(deps.storage, &asset_id)?
        .ok_or(ContractError::AssetNotFound(asset_id.clone()))?;

    // check that the asset has a current beneficial owner before creating a new entitlement
    ensure!(
        asset.beneficial_owner.is_some(),
        StdError::generic_err(
            "impose_entitlement - beneficial owner must be set to impose an entitlement",
        )
    );

    let operator = maybe_addr(deps.api, Some(operator))?;

    register_entitlement(
        deps,
        env,
        &asset_id,
        Some(&asset),
        Asset {
            beneficial_owner: asset.beneficial_owner.clone(),
            operator,
            expiry: Some(expiry),
        },
    )?;

    Ok(Response::default()
        .add_attribute("action", "impose_entitlement")
        .add_attribute("sender", sender)
        .add_attribute("asset_id", asset_id))
}

/// Allows the beneficial owner to grant an entitlement to an asset within the contract.
pub fn grant_entitlement(
    deps: DepsMut,
    env: &Env,
    sender: &Addr,
    entitlement: &Entitlement,
) -> Result<Response, ContractError> {
    let asset_id = entitlement.asset_id.clone();
    let asset = ASSETS
        .may_load(deps.storage, &asset_id)?
        .ok_or(ContractError::AssetNotFound(asset_id.clone()))?;

    // the entitlement must be sent by the current beneficial owner or approved operator
    if asset
        .beneficial_owner
        .as_ref()
        .map_or(true, |b| b != sender)
    {
        let approval = APPROVALS.may_load(deps.storage, &asset_id)?;

        if approval.map_or(true, |a| a != sender) {
            return Err(StdError::generic_err(
                "grant_entitlement - only the beneficial owner or approved operator can grant an entitlement",
            ).into());
        }
    }

    let beneficial_owner = maybe_addr(deps.api, Some(entitlement.beneficial_owner.clone()))?;
    let operator = maybe_addr(deps.api, Some(entitlement.operator.clone()))?;

    // the beneficial owner of an asset is able to directly set any entitlement on their own asset
    // as long as it has not already been committed to someone else.
    register_entitlement(
        deps,
        env,
        &entitlement.asset_id,
        Some(&asset),
        Asset {
            beneficial_owner,
            operator,
            expiry: Some(entitlement.expiry),
        },
    )?;

    Ok(Response::default()
        .add_attribute("action", "grant_entitlement")
        .add_attribute("sender", sender)
        .add_attribute("asset_id", entitlement.asset_id.clone()))
}

/// Withdrawal an unencumbered asset from this vault.
/// Сan only be performed to the beneficial owner if there are no entitlements.
pub fn withdrawal_asset(
    deps: DepsMut,
    env: &Env,
    sender: &Addr,
    asset_id: AssetId,
    config: &Config,
) -> Result<Response, ContractError> {
    let asset = ASSETS
        .may_load(deps.storage, &asset_id)?
        .ok_or(ContractError::AssetNotFound(asset_id.to_string()))?;

    // only the beneficial owner can withdrawal an asset
    let owner = asset
        .beneficial_owner
        .as_ref()
        .ok_or(ContractError::Unauthorized {})?;
    ensure!(owner == sender, ContractError::Unauthorized {});

    // the asset cannot be withdrawn with an active entitlement
    ensure!(
        !has_active_entitlement(&asset, env),
        ContractError::WithdrawalFailed {}
    );

    // _nftContract.safeTransferFrom(address(this),assets[assetId].beneficialOwner,_assetTokenId(assetId));
    let transfer_nft_msg = nft::transfer_nft(&config.nft_addr, &asset_id, owner)?;

    // TODO maybe we must delete entitlement in ASSETS?
    ASSETS.remove(deps.storage, &asset_id);

    Ok(Response::default()
        .add_submessage(transfer_nft_msg)
        .add_attribute("action", "withdrawal_asset")
        .add_attribute("sender", sender)
        .add_attribute("asset_id", asset_id.to_string()))
}

/// Updates the current address that can claim the asset when it is free of entitlements.
pub fn set_beneficial_owner(
    deps: DepsMut,
    env: &Env,
    sender: &Addr,
    asset_id: AssetId,
    new_beneficial_owner: String,
) -> Result<Response, ContractError> {
    let asset = ASSETS
        .may_load(deps.storage, &asset_id)?
        .ok_or(ContractError::AssetNotFound(asset_id.clone()))?;

    // only the contract with the active entitlement can update the beneficial owner
    // otherwise only the current owner can update the beneficial owner
    if has_active_entitlement(&asset, env) {
        if asset.operator.map_or(true, |operator| operator != sender) {
            return Err(StdError::generic_err("set_beneficial_owner - only the contract with the active entitlement can update the beneficial owner").into());
        }
    } else if asset.beneficial_owner.map_or(true, |owner| owner != sender) {
        return Err(StdError::generic_err(
            "set_beneficial_owner - only the current owner can update the beneficial owner",
        )
        .into());
    }

    let beneficial_owner = maybe_addr(deps.api, Some(new_beneficial_owner))?;

    // sets the new beneficial owner for a particular asset within the vault
    update_or_create_asset(
        deps,
        &asset_id,
        Asset {
            beneficial_owner,
            operator: None,
            expiry: None,
        },
    )?;

    Ok(Response::default()
        .add_attribute("action", "set_beneficial_owner")
        .add_attribute("sender", sender)
        .add_attribute("asset_id", asset_id.to_string()))
}

/// Allows the entitled address to release their claim on the asset
pub fn clear_entitlement(
    deps: DepsMut,
    env: &Env,
    sender: &Addr,
    asset_id: AssetId,
) -> Result<Response, ContractError> {
    let mut asset = ASSETS
        .may_load(deps.storage, &asset_id)?
        .ok_or(ContractError::AssetNotFound(asset_id.clone()))?;

    // this can only be called if an entitlement currently exists
    ensure!(
        has_active_entitlement(&asset, env),
        StdError::generic_err("an active entitlement must exist")
    );

    ensure!(
        asset.operator.map_or(false, |operator| operator != sender),
        StdError::generic_err(
            "clear_entitlement - only the entitled address can clear the entitlement",
        )
    );

    // TODO maybe just delete storage record?
    asset.expiry = None;
    asset.operator = None;
    ASSETS.save(deps.storage, &asset_id, &asset)?;

    Ok(Response::default()
        .add_attribute("action", "clear_entitlement")
        .add_attribute("sender", sender))
}

/// Removes the active entitlement from a vault and returns the asset to the beneficial owner
/// The entitlement must be exist, and must be called by the {operator}. The operator can specify
/// an intended receiver, which should match the beneficial owner. The function will throw if
/// the receiver and owner do not match.
pub fn clear_entitlement_and_distribute(
    deps: DepsMut,
    env: &Env,
    sender: &Addr,
    asset_id: AssetId,
    receiver: String,
    config: &Config,
) -> Result<Response, ContractError> {
    let receiver_addr = deps.api.addr_validate(&asset_id)?;

    let asset = ASSETS
        .may_load(deps.storage, &asset_id)?
        .ok_or(ContractError::AssetNotFound(asset_id.clone()))?;

    ensure!(
        asset
            .beneficial_owner
            .map_or(false, |owner| owner == receiver),
        StdError::generic_err(
            "clear_entitlement_and_distribute - only the beneficial owner can receive the asset",
        )
    );

    let transfer_nft_msg = nft::transfer_nft(&config.nft_addr, &asset_id, &receiver_addr)?;

    clear_entitlement(deps, env, sender, asset_id)?;

    Ok(Response::new()
        .add_submessage(transfer_nft_msg)
        .add_attribute("action", "clear_entitlement_and_distribute"))
}

pub fn receive_cw721(
    deps: DepsMut,
    env: &Env,
    info: &MessageInfo,
    wrapper: cw721::Cw721ReceiveMsg,
) -> Result<Response, ContractError> {
    // info.sender - NFT contract
    // wrapper.sender - owner of NFT or user with approvals for NFT

    let config = Config::load(deps.storage)?;
    ensure!(
        info.sender == config.nft_addr,
        ContractError::InvalidNft {
            received: info.sender.clone(),
            expected: config.nft_addr,
        }
    );

    let sender = deps.api.addr_validate(&wrapper.sender)?;

    let Entitlement {
        // beneficial_owner,
        operator,
        // vault_address,
        // asset_id,
        expiry,
        ..
    } = from_binary(&wrapper.msg)?;

    let operator = maybe_addr(deps.api, Some(operator))?;

    register_entitlement(
        deps,
        env,
        &wrapper.token_id,
        None,
        Asset {
            beneficial_owner: Some(sender.clone()),
            // beneficial_owner: Some(beneficial_owner),
            operator,
            expiry: Some(expiry),
        },
    )?;

    Ok(Response::default()
        .add_attribute("action", "receive_cw721")
        .add_attribute("from", sender)
        .add_attribute("token_id", wrapper.token_id))
}

// TODO function clearEntitlementAndDistribute(uint32 assetId, address receiver) external nonReentrant
