#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};

use common::errors::ContractError;

use crate::{
    execute,
    msg::{ExecuteMsg, InstantiateMsg, QueryMsg},
    query,
    state::{Config, ADMINS},
};

const CONTRACT_NAME: &str = "crates.io:interchainnft-options-protocol";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    cw2::set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    // TODO If no admin is specified, the contract is its own admin?

    let config = Config::default();
    config.save(deps.storage)?;

    let owner_addr = if let Some(owner) = msg.owner.clone() {
        deps.api.addr_validate(&owner)?
    } else {
        info.sender.clone()
    };

    cw_ownable::initialize_owner(deps.storage, deps.api, Some(owner_addr.as_ref()))?;
    ADMINS.grant(deps.storage, owner_addr)?;

    Ok(Response::new()
        .add_attribute("action", "instantiate")
        .add_attribute("sender", info.sender))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Pause { duration } => execute::pause(deps, env, info.sender, duration),

        ExecuteMsg::SetVaultFactory { contract_addr } => {
            execute::set_vault_factory(deps, &info.sender, contract_addr)
        }

        ExecuteMsg::SetCallFactory { contract_addr } => {
            execute::set_call_factory(deps, &info.sender, contract_addr)
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => query::config(deps),
    }
}
