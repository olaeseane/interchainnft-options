use cosmwasm_std::{ensure, from_binary, Addr, DepsMut, Env, MessageInfo, Response, StdError};
use cw_utils::{maybe_addr, Expiration};

use common::{errors::ContractError, nft, types::AssetId};

use crate::{
    msg::SetEntitlement,
    state::{update_or_create_entitlement, Config, Entitlement, APPROVALS, ASSETS},
    utils::{has_active_entitlement, register_entitlement},
};

/// Add an entitlement claim to the asset held within the vaultv (replace existed entitlement with new operator)
pub fn impose_entitlement(
    deps: DepsMut,
    env: &Env,
    sender: &Addr,
    asset_id: AssetId,
    operator: String,
    expiry: Expiration,
) -> Result<Response, ContractError> {
    let entitlement = ASSETS
        .may_load(deps.storage, &asset_id)?
        .ok_or(ContractError::AssetNotFound(asset_id.clone()))?;

    // check that the entitlement has a current beneficial owner before creating a new entitlement
    let beneficial_owner = entitlement
        .beneficial_owner
        .clone()
        .ok_or(StdError::generic_err(
            "impose_entitlement - beneficial owner must be set to impose an entitlement",
        ))?;

    let operator = deps.api.addr_validate(&operator)?;

    register_entitlement(
        deps,
        env,
        &asset_id,
        Some(&entitlement),
        &Entitlement {
            beneficial_owner: Some(beneficial_owner),
            operator: Some(operator),
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
    asset_id: AssetId,
    beneficial_owner: String,
    operator: String,
    expiry: Expiration,
) -> Result<Response, ContractError> {
    let entitlement = ASSETS
        .may_load(deps.storage, &asset_id)?
        .ok_or(ContractError::AssetNotFound(asset_id.clone()))?;

    // the entitlement must be sent by the current beneficial owner or approved operator
    if entitlement
        .beneficial_owner
        .clone()
        .is_some_and(|beneficial_owner| beneficial_owner != sender)
    {
        let approval = APPROVALS.may_load(deps.storage, &asset_id)?;

        ensure!(
            approval.is_some_and(|approval_address| approval_address == sender),
            ContractError::OnlyBeneficialOwnerOrOperator {}
        )
    }

    let beneficial_owner = maybe_addr(deps.api, Some(beneficial_owner))?;
    let operator = maybe_addr(deps.api, Some(operator))?;

    // the beneficial owner of an asset is able to directly set any entitlement on their own asset
    // as long as it has not already been committed to someone else.
    register_entitlement(
        deps,
        env,
        &asset_id,
        Some(&entitlement),
        &Entitlement {
            beneficial_owner,
            operator,
            expiry: Some(expiry),
        },
    )?;

    Ok(Response::default()
        .add_attribute("action", "grant_entitlement")
        .add_attribute("sender", sender)
        .add_attribute("asset_id", asset_id))
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
    let entitlement = ASSETS
        .may_load(deps.storage, &asset_id)?
        .ok_or(ContractError::AssetNotFound(asset_id.to_string()))?;

    // only the beneficial owner can withdrawal an asset
    let beneficial_owner = entitlement
        .beneficial_owner
        .clone()
        .ok_or(ContractError::Unauthorized {})?;

    ensure!(beneficial_owner == sender, ContractError::Unauthorized {});

    // the asset cannot be withdrawn with an active entitlement
    ensure!(
        !has_active_entitlement(&entitlement, env),
        ContractError::WithdrawalFailed {}
    );

    let transfer_nft_msg = nft::transfer_nft(&config.nft_addr, &asset_id, &beneficial_owner)?;

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
    let entitlement = ASSETS
        .may_load(deps.storage, &asset_id)?
        .ok_or(ContractError::AssetNotFound(asset_id.clone()))?;

    // only the contract with the active entitlement can update the beneficial owner
    // otherwise only the current owner can update the beneficial owner
    if has_active_entitlement(&entitlement, env) {
        ensure!(
            entitlement.operator.is_some_and(|operator| operator == sender),
            StdError::generic_err("set_beneficial_owner - only the contract with the active entitlement can update the beneficial owner")
        );
    } else {
        ensure!(
            entitlement
                .beneficial_owner
                .is_some_and(|beneficial_owner| beneficial_owner == sender),
            StdError::generic_err(
                "set_beneficial_owner - only the current owner can update the beneficial owner"
            )
        );
    }

    let new_beneficial_owner = maybe_addr(deps.api, Some(new_beneficial_owner))?;

    // sets the new beneficial owner for a particular asset within the vault
    update_or_create_entitlement(
        deps,
        &asset_id,
        &Entitlement {
            beneficial_owner: new_beneficial_owner,
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
    let mut entitlement = ASSETS
        .may_load(deps.storage, &asset_id)?
        .ok_or(ContractError::AssetNotFound(asset_id.clone()))?;

    // this can only be called if an entitlement currently exists
    ensure!(
        has_active_entitlement(&entitlement, env),
        ContractError::NoActiveEntitlement {}
    );

    ensure!(
        entitlement
            .operator
            .is_some_and(|operator| operator == sender),
        StdError::generic_err(
            "clear_entitlement - only the entitled address can clear the entitlement",
        )
    );

    entitlement.expiry = None;
    entitlement.operator = None;
    ASSETS.save(deps.storage, &asset_id, &entitlement)?;

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
    let receiver_addr = deps.api.addr_validate(&receiver)?;

    let entitlement = ASSETS
        .may_load(deps.storage, &asset_id)?
        .ok_or(ContractError::AssetNotFound(asset_id.clone()))?;

    ensure!(
        entitlement
            .beneficial_owner
            .is_some_and(|beneficial_owner| beneficial_owner == receiver),
        StdError::generic_err(
            "clear_entitlement_and_distribute - only the beneficial owner can receive the asset",
        )
    );

    let transfer_nft_msg = nft::transfer_nft(&config.nft_addr, &asset_id, &receiver_addr)?;

    // TODO maybe delete entitlement from ASSETS
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

    let entitlement: Option<SetEntitlement> = from_binary(&wrapper.msg)?;

    match entitlement {
        None => {
            update_or_create_entitlement(
                deps,
                &wrapper.token_id,
                &Entitlement {
                    beneficial_owner: Some(sender.clone()),
                    operator: None,
                    expiry: None,
                },
            )?;
        }
        Some(entitlement) => {
            if let Some(approved_operator) = entitlement.approved_operator {
                APPROVALS.save(deps.storage, &wrapper.token_id, &approved_operator)?;
            }

            register_entitlement(
                deps,
                env,
                &wrapper.token_id,
                None,
                &Entitlement {
                    beneficial_owner: Some(entitlement.beneficial_owner),
                    operator: Some(entitlement.entitled_operator),
                    expiry: Some(entitlement.expiry),
                },
            )?;
        }
    }

    Ok(Response::default()
        .add_attribute("action", "receive_cw721")
        .add_attribute("from", sender)
        .add_attribute("token_id", wrapper.token_id))
}
