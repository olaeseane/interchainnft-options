use cosmwasm_std::{
    ensure, entry_point, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Reply, Response, StdError,
    StdResult,
};
use cw_utils::parse_reply_instantiate_data;

use common::errors::ContractError;

use crate::{
    execute::{self, INSTANTIATE_CALL_INSTRUMENT_ID},
    msg::{ExecuteMsg, InstantiateMsg, QueryMsg},
    query,
    state::{Config, CALL_INSTRUMENTS, TMP_INSTRUMENT},
};

const CONTRACT_NAME: &str = "crates.io:interchainnft-options-call-factory";
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
        ExecuteMsg::MakeCallInstrument { nft_addr } => {
            execute::make_call_instrument(deps, &info.sender, nft_addr, &config)
        }
    }
}

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    let _config = Config::load(deps.storage)?; // TODO use ContractError::COnfigNotFound

    match msg {
        QueryMsg::GetCallInstrument { nft_addr } => query::get_call_instrument(deps, &nft_addr),
    }
}

/// The entry point to the contract for processing replies from submessages.
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, _env: Env, msg: Reply) -> Result<Response, ContractError> {
    match msg.id {
        INSTANTIATE_CALL_INSTRUMENT_ID => {
            let tmp = TMP_INSTRUMENT.load(deps.storage)?;
            ensure!(
                !CALL_INSTRUMENTS.has(deps.storage, &tmp.nft_addr),
                StdError::generic_err("make_call_instrument - instrument already exist")
            );
            let res = parse_reply_instantiate_data(msg)?;
            CALL_INSTRUMENTS.save(
                deps.storage,
                &tmp.nft_addr,
                &Addr::unchecked(res.contract_address),
            )?;
            Ok(Response::new())
        }

        _ => Err(ContractError::UnknownReplyID {}),
    }
}
