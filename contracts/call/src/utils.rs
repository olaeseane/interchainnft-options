// HELPERS
use cosmwasm_std::{
    ensure, from_binary, to_binary, Addr, CosmosMsg, DepsMut, Empty, Env, MessageInfo,
    QuerierWrapper, StdError, StdResult, Uint128, WasmMsg,
};
use cw721_base::state::TokenInfo;
use cw_utils::Expiration;

use common::{
    errors::ContractError,
    types::{AssetId, OptionId},
};
use vault::msg::QueryMsg as VaultQueryMsg;

use crate::{
    contract::CallInstrumentContract,
    state::{update_vault_asset_option, CallOption, Config, CALL_OPTIONS, VAULT_ASSET_OPTION},
    InstantiateMsg,
};

#[allow(clippy::too_many_arguments)]
pub(crate) fn mint_call(
    deps: DepsMut,
    env: &Env,
    writer: impl Into<String>,
    vault: impl Into<String>,
    asset_id: &AssetId,
    strike: Uint128,
    expiration: Expiration,
    config: &Config,
) -> Result<OptionId, ContractError> {
    // TODO it's ok?
    let writer_addr = Addr::unchecked(writer);
    let vault_addr = Addr::unchecked(vault);

    let mut block_info = env.block.clone();
    block_info.time = env.block.time.plus_seconds(config.minimum_option_duration);

    // the settlement auction always occurs one day before expiration
    ensure!(
        !expiration.is_expired(&block_info),
        StdError::generic_err("mint_option_with_vault - expires sooner than min duration",)
    );

    // verify that, if there is a previous option on this asset, it has already settled.
    let prev_option_id = VAULT_ASSET_OPTION.may_load(deps.storage, (&vault_addr, asset_id))?;
    if let Some(prev_option_id) = prev_option_id {
        let prev_option = CALL_OPTIONS.load(deps.storage, &prev_option_id)?;
        ensure!(
            !prev_option.settled,
            StdError::generic_err("mint_option_with_vault - previous option must be settled",)
        );
    }

    // save the new option metadata
    let call_option = CallOption {
        asset_id: asset_id.clone(),
        writer_addr: writer_addr.clone(),
        expiration,
        vault_addr: vault_addr.clone(),
        strike,
        bid: Uint128::zero(),
        high_bidder: None,
        settled: false,
    };
    let next_option_id = CallOption::inc(deps.storage)?;
    call_option.save(deps.storage, &next_option_id)?;

    update_vault_asset_option(deps.storage, &vault_addr, asset_id, next_option_id)?;

    // TODO if msg.sender and tokenOwner are different accounts, approve the msg.sender
    // to transfer the option NFT as it already had the right to transfer the underlying NFT.

    // info.sender =

    // mint the option NFT to the underlying token owner.
    /* Cw721CallOptionContract::default().mint(
       deps,
       info.to_owned(),
       next_option_id.to_string(),
       writer_addr.into_string(),
       None,
       Empty {},
    )?;*/

    // create the token
    let nft_contract = CallInstrumentContract::default();
    let token = TokenInfo {
        owner: writer_addr,
        approvals: vec![],
        token_uri: None,
        extension: Empty {},
    };
    nft_contract
        .tokens
        .update(deps.storage, &next_option_id.to_string(), |old| match old {
            Some(_) => Err(cw721_base::ContractError::Claimed {}),
            None => Ok(token),
        })?;

    nft_contract.increment_tokens(deps.storage)?;

    Ok(next_option_id)
}

pub(crate) fn is_beneficial_owner_or_operator(
    querier: &QuerierWrapper,
    vault_addr: &String,
    asset_id: AssetId,
    sender: &Addr,
) -> StdResult<(bool, Option<Addr>)> {
    let beneficial_owner: Option<Addr> = querier.query_wasm_smart(
        vault_addr.clone(),
        &VaultQueryMsg::BeneficialOwner {
            asset_id: asset_id.clone(),
        },
    )?;
    if let Some(owner) = beneficial_owner.clone() {
        if owner == sender {
            return Ok((true, beneficial_owner));
        }
    }

    let operator: Option<Addr> =
        querier.query_wasm_smart(vault_addr, &VaultQueryMsg::ApprovedOperator { asset_id })?;
    if let Some(op) = operator {
        if op == sender {
            return Ok((true, beneficial_owner));
        }
    }

    Ok((false, beneficial_owner))
}

pub(crate) fn option_owner(deps: &DepsMut, env: &Env, token_id: String) -> StdResult<String> {
    let resp: cw721::OwnerOfResponse = from_binary(&CallInstrumentContract::default().query(
        deps.as_ref(),
        env.clone(),
        cw721_base::QueryMsg::OwnerOf {
            token_id,
            include_expired: None,
        },
    )?)?;

    Ok(resp.owner)
}

pub fn burn_option_nft(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    token_id: String,
) -> Result<(), ContractError> {
    // burn the option NFT
    CallInstrumentContract::default().execute(
        deps,
        env,
        info,
        cw721_base::ExecuteMsg::Burn { token_id },
    )?;

    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub fn call_instrument_instantiate_wasm_msg(
    name: String,
    symbol: String,
    code_id: u64,
    nft_addr: String,
    protocol_addr: String,
    vault_factory_addr: String,
    minimum_option_duration: u64,
    allowed_denom: String,
    min_bid_inc_bips: Uint128,
    label: String,
) -> StdResult<CosmosMsg<Empty>> {
    let msg = to_binary(&InstantiateMsg {
        name,
        symbol,
        protocol_addr,
        allowed_underlying_nft: nft_addr,
        vault_factory_addr,
        minimum_option_duration,
        allowed_denom,
        min_bid_inc_bips,
    })?;

    Ok(CosmosMsg::Wasm(WasmMsg::Instantiate {
        admin: None, // TODO set admin
        code_id,
        msg,
        funds: vec![],
        label,
    }))
}
