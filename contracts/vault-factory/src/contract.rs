use cosmwasm_std::{
    ensure, entry_point, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Reply, Response, StdError,
    StdResult,
};
use cw_utils::parse_reply_instantiate_data;

use common::errors::ContractError;

use crate::{
    execute::{self, INSTANTIATE_MULTI_VAULT_ID, INSTANTIATE_SOLO_VAULT_ID},
    msg::{ExecuteMsg, InstantiateMsg, QueryMsg},
    query,
    state::{Config, MULTI_VAULTS, SOLO_VAULTS, TMP_VAULT},
};

const CONTRACT_NAME: &str = "crates.io:interchainnft-options-vault-factory";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    cw2::set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let owner_addr = if let Some(owner) = msg.owner.clone() {
        deps.api.addr_validate(&owner)?
    } else {
        info.sender.clone()
    };
    cw_ownable::initialize_owner(deps.storage, deps.api, Some(owner_addr.as_str()))?;

    let config: Config = msg.into();
    config.validate(deps.api)?;
    config.save(deps.storage)?;

    Ok(Response::new()
        .add_attribute("action", "instantiate")
        .add_attribute("sender", info.sender))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    let config = Config::load(deps.storage)?;

    match msg {
        ExecuteMsg::MakeMultiVault { nft_addr } => {
            execute::make_multi_vault(deps, &info.sender, nft_addr, &config)
        }

        ExecuteMsg::MakeSoloVault { nft_addr, nft_id } => {
            execute::make_solo_vault(deps, &info.sender, nft_addr, nft_id, &config)
        }
    }
}

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    let _config = Config::load(deps.storage)?; // TODO use ContractError::COnfigNotFound

    match msg {
        QueryMsg::GetVault { nft_addr, nft_id } => query::get_vault(deps, &nft_addr, &nft_id),

        QueryMsg::GetMultiVault { nft_addr } => query::get_multi_vault(deps, &nft_addr),

        QueryMsg::GetMultiOrSoloVault { nft_addr, nft_id } => {
            query::get_multi_or_solo_vault(deps, &nft_addr, nft_id)
        }
    }
}

/// The entry point to the contract for processing replies from submessages.
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, _env: Env, msg: Reply) -> Result<Response, ContractError> {
    match msg.id {
        INSTANTIATE_MULTI_VAULT_ID => {
            let tmp = TMP_VAULT.load(deps.storage)?;
            ensure!(
                !MULTI_VAULTS.has(deps.storage, &tmp.nft_addr),
                StdError::generic_err("make_multi_vault - vault already exist")
            );
            let res = parse_reply_instantiate_data(msg)?;
            MULTI_VAULTS.save(
                deps.storage,
                &tmp.nft_addr,
                &Addr::unchecked(res.contract_address),
            )?;
            Ok(Response::new())
        }

        INSTANTIATE_SOLO_VAULT_ID => {
            let tmp = TMP_VAULT.load(deps.storage)?;

            match tmp.nft_id {
                Some(nft_id) => {
                    ensure!(
                        !SOLO_VAULTS.has(deps.storage, (&tmp.nft_addr, &nft_id)),
                        StdError::generic_err("make_solo_vault - vault already exist")
                    );
                    let res = parse_reply_instantiate_data(msg)?;
                    SOLO_VAULTS.save(
                        deps.storage,
                        (&tmp.nft_addr, &nft_id),
                        &Addr::unchecked(res.contract_address),
                    )?;
                    Ok(Response::new())
                }
                None => Err(StdError::generic_err("make_solo_vault - nft_id must be Some").into()),
            }
        }

        _ => Err(ContractError::UnknownReplyID {}),
    }
}
