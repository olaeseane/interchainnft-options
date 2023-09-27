use cosmwasm_std::{
    ensure, to_binary, CosmosMsg, DepsMut, Empty, Env, StdError, StdResult, WasmMsg,
};

use common::{
    errors::ContractError,
    types::{AssetId, TokenId},
};

use crate::{
    msg::{ExecuteMsg, InstantiateMsg},
    state::{update_or_create_entitlement, Entitlement},
};

pub fn vault_instantiate_wasm_msg(
    code_id: u64,
    nft_addr: String,
    nft_id: Option<String>,
    protocol_addr: String,
    label: String,
) -> StdResult<CosmosMsg<Empty>> {
    let msg = to_binary(&InstantiateMsg {
        nft_addr,
        nft_id,
        protocol_addr,
    })?;

    Ok(CosmosMsg::Wasm(WasmMsg::Instantiate {
        admin: None, // TODO set admin
        code_id,
        msg,
        funds: vec![],
        label,
    }))
}

pub fn set_beneficial_owner_wasm_msg(
    contract_addr: &str,
    asset_id: &str,
    new_beneficial_owner: &str,
) -> StdResult<CosmosMsg> {
    Ok(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: contract_addr.into(),
        msg: to_binary(&ExecuteMsg::SetBeneficialOwner {
            asset_id: asset_id.into(),
            new_beneficial_owner: new_beneficial_owner.into(),
        })?,
        funds: vec![],
    }))
}

pub fn clear_entitlement_wasm_msg(contract_addr: &str, asset_id: &str) -> StdResult<CosmosMsg> {
    Ok(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: contract_addr.into(),
        msg: to_binary(&ExecuteMsg::ClearEntitlement {
            asset_id: asset_id.into(),
        })?,
        funds: vec![],
    }))
}

pub fn clear_entitlement_and_distribute_wasm_msg(
    contract_addr: &str,
    asset_id: &str,
    receiver: &str,
) -> StdResult<CosmosMsg> {
    Ok(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: contract_addr.into(),
        msg: to_binary(&ExecuteMsg::ClearEntitlementAndDistribute {
            asset_id: asset_id.into(),
            receiver: receiver.into(),
        })?,
        funds: vec![],
    }))
}

pub(crate) fn has_active_entitlement(entitlement: &Entitlement, env: &Env) -> bool {
    // block.timestamp < assets[assetId].expiry && assets[assetId].operator != address(0);
    // TODO check condition
    entitlement
        .expiry
        .is_some_and(|expiry| !expiry.is_expired(&env.block))
        && entitlement.operator.is_some()
}

/// Returns the underlying token ID for a given asset.
/// In this case the tokenId == the assetId
pub(crate) fn _asset_id_to_nft_id(asset_id: AssetId) -> TokenId {
    asset_id
}

pub(crate) fn register_entitlement(
    deps: DepsMut,
    env: &Env,
    asset_id: &AssetId,
    prev: Option<&Entitlement>,
    new: &Entitlement,
) -> Result<(), ContractError> {
    // TODO optimize
    prev.map(|e| {
        if has_active_entitlement(e, env) {
            Err(ContractError::HasActiveEntitlement {})
        } else {
            Ok(())
        }
    })
    .transpose()?;

    ensure!(
        !new.expiry.unwrap().is_expired(&env.block),
        StdError::generic_err("register_entitlement - entitlement must expire in the future",)
    );

    update_or_create_entitlement(deps, asset_id, new)?;

    Ok(())
}
