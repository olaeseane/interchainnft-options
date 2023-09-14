use cosmwasm_std::{
    entry_point, Binary, Deps, DepsMut, Empty, Env, MessageInfo, Response, StdResult,
};

use cw_utils::maybe_addr;

use common::errors::ContractError;

use crate::{
    execute,
    msg::{CallOptionExecuteMsg, CallOptionQueryMsg, InstantiateMsg},
    query,
    state::Config,
};

pub type Cw721CallOptionContract<'a> =
    cw721_base::Cw721Contract<'a, Empty, Empty, CallOptionExecuteMsg, CallOptionQueryMsg>;

pub(crate) const CONTRACT_NAME: &str = "crates.io:interchainnft-options-factory";
pub(crate) const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    cw2::set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let config: Config = msg.clone().into();
    config.validate(deps.api)?;
    config.save(deps.storage)?;

    let owner = maybe_addr(deps.api, msg.owner).unwrap_or(Some(info.sender.to_owned()));

    let cw721_base_instantiate_msg = cw721_base::InstantiateMsg {
        name: msg.name,
        symbol: msg.symbol,
        minter: owner.unwrap().into_string(),
    };

    Cw721CallOptionContract::default().instantiate(deps, env, info, cw721_base_instantiate_msg)?;

    // TODO add sender?
    Ok(Response::new().add_attribute("action", "instantiate"))
}

#[allow(unused_variables)] // TODO remove
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: cw721_base::ExecuteMsg<Empty, CallOptionExecuteMsg>,
) -> Result<Response, ContractError> {
    let config = Config::load(deps.storage)?;

    match msg {
        // handle custom call option messages
        cw721_base::ExecuteMsg::Extension { msg } => match msg {
            CallOptionExecuteMsg::MintWithNFT {
                nft_addr,
                nft_id,
                strike,
                expiration,
            } => execute::mint_with_nft(
                deps, &env, info, nft_addr, nft_id, strike, expiration, &config,
            ),

            CallOptionExecuteMsg::MintWithVault {
                vault_addr,
                asset_id,
                strike,
                expiration,
            } => execute::mint_with_vault(
                deps, env, info, vault_addr, asset_id, strike, expiration, &config,
            ),

            CallOptionExecuteMsg::MintWithEntitledVault {
                vault_addr,
                asset_id,
                strike,
                expiration,
            } => execute::mint_with_entitled_vault(
                deps,
                &env,
                info,
                &vault_addr,
                asset_id,
                strike,
                expiration,
                &config,
            ),

            CallOptionExecuteMsg::Bid { option_id } => {
                execute::bid(deps, info, &option_id, &config)
            }

            CallOptionExecuteMsg::ReclaimAsset {
                option_id,
                withdraw,
            } => execute::reclaim_asset(deps, env, info, &option_id, withdraw, &config),

            CallOptionExecuteMsg::SettleOption { option_id } => {
                execute::settle_option(deps, env, info, &option_id, &config)
            }

            CallOptionExecuteMsg::BurnExpiredOption { option_id } => {
                execute::burn_expired_option(deps, env, info, &option_id)
            }

            CallOptionExecuteMsg::ClaimOptionProceeds { option_id } => {
                execute::claim_option_proceeds(deps, env, info, &option_id, &config)
            }
        },

        // handle standard cw721 messages
        _ => Cw721CallOptionContract::default()
            .execute(deps, env, info, msg)
            .map_err(|e| e.into()),
    }
}

#[entry_point]
pub fn query(
    deps: Deps,
    env: Env,
    msg: cw721_base::QueryMsg<CallOptionQueryMsg>,
) -> StdResult<Binary> {
    match msg {
        cw721_base::QueryMsg::Extension { msg } => match msg {
            CallOptionQueryMsg::CurrentBid { option_id } => query::current_bid(deps, &option_id),

            CallOptionQueryMsg::CurrentBidder { option_id } => {
                query::current_bidder(deps, &option_id)
            }

            CallOptionQueryMsg::GetVaultAddress { option_id } => {
                query::get_vault_address(deps, &option_id)
            }

            CallOptionQueryMsg::GetOptionIdForAsset { vault, asset_id } => {
                query::get_option_id_for_asset(deps, vault, &asset_id)
            }

            CallOptionQueryMsg::GetAssetId { option_id } => query::get_asset_id(deps, &option_id),

            CallOptionQueryMsg::GetStrikePrice { option_id } => {
                query::get_strike_price(deps, &option_id)
            }

            CallOptionQueryMsg::GetExpiration { option_id } => {
                query::get_expiration(deps, &option_id)
            }
        },
        _ => Cw721CallOptionContract::default().query(deps, env, msg),
    }
}

/*
/// The entry point to the contract for processing replies from submessages.
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, _env: Env, msg: Reply) -> Result<Response, ContractError> {
    match msg {
        Reply {
            id: INSTANTIATE_PAIR_REPLY_ID,
            result:
                SubMsgResult::Ok(SubMsgResponse {
                    data: Some(data), ..
                }),
        } => {
            let tmp = TMP_PAIR_INFO.load(deps.storage)?;
            if PAIRS.has(deps.storage, &tmp.pair_key) {
                return Err(ContractError::PairWasRegistered {});
            }

            let init_response = parse_instantiate_response_data(data.as_slice())
                .map_err(|e| StdError::generic_err(format!("{e}")))?;

            let pair_contract = deps.api.addr_validate(&init_response.contract_address)?;

            PAIRS.save(deps.storage, &tmp.pair_key, &pair_contract)?;

            Ok(Response::new().add_attributes(vec![
                attr("action", "register"),
                attr("pair_contract_addr", pair_contract),
            ]))
        }
        _ => Err(ContractError::FailedToParseReply {}),
    }
}
 */
