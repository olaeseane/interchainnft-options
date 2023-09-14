use cosmwasm_std::{ensure, Addr, DepsMut, Response, StdError, SubMsg};

use common::{errors::ContractError, types::TokenId};
use vault::utils::vault_instantiate_wasm_msg;

use crate::state::{Config, TmpVaultInfo, MULTI_VAULTS, SOLO_VAULTS, TMP_VAULT};

/// A `reply` call code ID used in a sub-message.
pub(crate) const INSTANTIATE_MULTI_VAULT_ID: u64 = 1;
pub(crate) const INSTANTIATE_SOLO_VAULT_ID: u64 = 2;

pub fn make_multi_vault(
    deps: DepsMut,
    sender: &Addr,
    nft_addr: String,
    config: &Config,
) -> Result<Response, ContractError> {
    cw_ownable::assert_owner(deps.as_ref().storage, sender)?;

    let vault_instantiate_wasm_msg = vault_instantiate_wasm_msg(
        config.vault_code_id,
        nft_addr.clone(),
        None,
        config.protocol_addr.clone().into_string(),
        "Interchainnft-options multi vault".into(),
    )?;

    let nft_addr = deps.api.addr_validate(&nft_addr)?;

    ensure!(
        !MULTI_VAULTS.has(deps.storage, &nft_addr),
        StdError::generic_err("make_multi_vault - vault already exist")
    );

    TMP_VAULT.save(
        deps.storage,
        &TmpVaultInfo {
            nft_addr,
            nft_id: None,
        },
    )?;

    Ok(Response::default()
        .add_submessage(SubMsg::reply_on_success(
            vault_instantiate_wasm_msg,
            INSTANTIATE_MULTI_VAULT_ID,
        ))
        .add_attribute("action", "make_multi_vault")
        .add_attribute("sender", sender))
}

pub fn make_solo_vault(
    deps: DepsMut,
    sender: &Addr,
    nft_addr: String,
    nft_id: TokenId,
    config: &Config,
) -> Result<Response, ContractError> {
    cw_ownable::assert_owner(deps.as_ref().storage, sender)?;

    let vault_instantiate_wasm_msg = vault_instantiate_wasm_msg(
        config.vault_code_id,
        nft_addr.clone(),
        Some(nft_id.clone()),
        config.protocol_addr.clone().into_string(),
        "Interchainnft-options solo vault".into(),
    )?;

    let nft_addr = deps.api.addr_validate(&nft_addr)?;

    ensure!(
        !SOLO_VAULTS.has(deps.storage, (&nft_addr, &nft_id)),
        StdError::generic_err("make_solo_vault - vault already exist")
    );

    TMP_VAULT.save(
        deps.storage,
        &TmpVaultInfo {
            nft_addr,
            nft_id: Some(nft_id),
        },
    )?;

    Ok(Response::default()
        .add_submessage(SubMsg::reply_on_success(
            vault_instantiate_wasm_msg,
            INSTANTIATE_SOLO_VAULT_ID,
        ))
        .add_attribute("action", "make_solo_vault")
        .add_attribute("sender", sender))
}
