#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};

use common::errors::ContractError;

use crate::{
    execute,
    msg::{ExecuteMsg, InstantiateMsg, QueryMsg, VaultInstantiateData},
    query::{self},
    state::{Config, CONFIG},
};

const CONTRACT_NAME: &str = "crates.io:interchainnft-options-vault";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    cw2::set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let config: Config = msg.clone().into();
    config.validate(deps.api)?;
    config.save(deps.storage)?;

    // TODO delete?
    let vault_instantiate_data = VaultInstantiateData {
        nft_addr: config.nft_addr,
        nft_id: config.nft_id,
    };

    Ok(Response::new()
        .set_data(to_binary(&vault_instantiate_data)?)
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
    let config = Config::load(deps.storage)?;

    match msg {
        ExecuteMsg::ReceiveNft(msg) => execute::receive_cw721(deps, &env, &info, msg),

        ExecuteMsg::ImposeEntitlement {
            asset_id,
            operator,
            expiry,
        } => execute::impose_entitlement(deps, &env, &info.sender, asset_id, operator, expiry),

        ExecuteMsg::GrantEntitlement {
            asset_id,
            beneficial_owner,
            operator,
            expiry,
        } => execute::grant_entitlement(
            deps,
            &env,
            &info.sender,
            asset_id,
            beneficial_owner,
            operator,
            expiry,
        ),

        ExecuteMsg::WithdrawalAsset { asset_id } => {
            execute::withdrawal_asset(deps, &env, &info.sender, asset_id, &config)
        }

        ExecuteMsg::SetBeneficialOwner {
            asset_id,
            new_beneficial_owner,
        } => {
            execute::set_beneficial_owner(deps, &env, &info.sender, asset_id, new_beneficial_owner)
        }

        ExecuteMsg::ClearEntitlement { asset_id } => {
            execute::clear_entitlement(deps, &env, &info.sender, asset_id)
        }

        ExecuteMsg::ClearEntitlementAndDistribute { asset_id, receiver } => {
            execute::clear_entitlement_and_distribute(
                deps,
                &env,
                &info.sender,
                asset_id,
                receiver,
                &config,
            )
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    let config = CONFIG.load(deps.storage)?; // TODO use ContractError::ConfigNotFound

    match msg {
        QueryMsg::BeneficialOwner { asset_id } => query::beneficial_owner(deps, &asset_id),

        QueryMsg::HoldsAsset { asset_id } => query::holds_asset(deps, &env, &asset_id, &config),

        QueryMsg::EntitlementExpiration { asset_id } => {
            query::entitlement_expiration(deps, &env, &asset_id)
        }

        // TODO add get by asset_id
        QueryMsg::AssetAddress {} => query::asset_address(&config),

        QueryMsg::ApprovedOperator { asset_id } => query::approved_operator(deps, &asset_id),

        QueryMsg::CurrentEntitlementOperator { asset_id } => {
            query::current_entitlement_operator(deps, &env, &asset_id)
        }
    }
}
