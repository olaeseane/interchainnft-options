use cosmwasm_std::{Addr, DepsMut, Env, Response};
use cw_utils::Duration;

use common::errors::ContractError;

use crate::state::{Config, PAUSED, PAUSERS};

pub fn set_vault_factory(
    deps: DepsMut,
    sender: &Addr,
    contract_addr: String,
) -> Result<Response, ContractError> {
    cw_ownable::assert_owner(deps.as_ref().storage, sender)?;

    // TODO UPGRADERS.has(&deps.storage, sender);
    // TODO require(Address.isContract(vaultFactoryContract), "setVaultFactory: implementation is not a contract");

    let contract_addr = deps.api.addr_validate(contract_addr.as_str())?;
    Config::update(deps.storage, Some(contract_addr), None)?;

    Ok(Response::default()
        .add_attribute("action", "set_vault_factory")
        .add_attribute("sender", sender))
}

pub fn set_call_factory(
    deps: DepsMut,
    sender: &Addr,
    contract_addr: String,
) -> Result<Response, ContractError> {
    cw_ownable::assert_owner(deps.as_ref().storage, sender)?;

    // TODO UPGRADERS.has(&deps.storage, sender);
    // TODO require(Address.isContract(vaultFactoryContract), "setVaultFactory: implementation is not a contract");

    let contract_addr = deps.api.addr_validate(contract_addr.as_str())?;
    Config::update(deps.storage, None, Some(contract_addr))?;

    Ok(Response::default()
        .add_attribute("action", "set_call_factory")
        .add_attribute("sender", sender))
}

pub fn pause(
    deps: DepsMut,
    env: Env,
    sender: Addr,
    pause_duration: Duration,
) -> Result<Response, ContractError> {
    // only the core contract or pausers may call this method.
    if sender != env.contract.address {
        PAUSERS.check(deps.storage, &sender)?;
    }

    let until = pause_duration.after(&env.block);

    PAUSED.save(deps.storage, &until)?;

    Ok(Response::new()
        .add_attribute("action", "pause")
        .add_attribute("sender", sender)
        .add_attribute("until", until.to_string()))
}
