use cosmwasm_std::{
    ensure, to_binary, Addr, CosmosMsg, DepsMut, Empty, Env, MessageInfo, Response, StdError,
    Uint128, WasmMsg,
};
use cw_utils::{nonpayable, Expiration};

use common::{
    denom::find_allowed_coin,
    errors::ContractError,
    msg::bank_send_msg,
    nft,
    types::{AssetId, OptionId, TokenId},
};
use vault::{
    msg::{
        CurrentEntitlementOperatorResponse, ExecuteMsg as VaultExecuteMsg,
        QueryMsg as VaultQueryMsg, SetEntitlement,
    },
    utils::{
        clear_entitlement_and_distribute_wasm_msg, clear_entitlement_wasm_msg,
        set_beneficial_owner_wasm_msg,
    },
};
use vault_factory::msg::QueryMsg;

use crate::{
    state::{CallInstrument, Config, OPTION_CLAIMS},
    utils::{burn_option_nft, is_beneficial_owner_or_operator, mint_call, option_owner},
};

// MESSAGE HANDLERS

/// Mints a new call option for a particular "underlying" NFT with a given strike price and expiration.
#[allow(clippy::too_many_arguments)]
pub fn mint_with_nft(
    deps: DepsMut,
    env: &Env,
    info: MessageInfo,
    nft_addr: String,
    nft_id: TokenId,
    strike: Uint128,
    expiration: Expiration,
    config: &Config,
) -> Result<Response, ContractError> {
    // let nft_addr = into_addr(deps.api, nft, PREFIX)?;
    let nft_addr = deps.api.addr_validate(&nft_addr)?;

    // check that sender uses allowed nft
    if config.allowed_underlying_nft != nft_addr {
        return Err(StdError::generic_err("mint_with_nft - nft not allowed").into());
    }

    // check if the sender is an owner, or has a approval, or is an nft operator
    let owner = nft::owner_of(&deps.querier, &nft_addr, &nft_id)?.owner;
    let operators = nft::all_operators(&deps.querier, &owner, &nft_addr, None, None, false)?;
    let approvals = nft::approvals(&deps.querier, &nft_addr, &nft_id, None)?.approvals;
    let is_sender_operator = operators.iter().any(|a| a.spender == info.sender);
    let is_sender_has_approval = approvals.iter().any(|a| a.spender == info.sender);
    let is_contract_operator = operators.iter().any(|a| a.spender == env.contract.address);
    let is_contract_has_approval = approvals.iter().any(|a| a.spender == env.contract.address);

    ensure!(
        owner == info.sender || is_sender_operator || is_sender_has_approval,
        StdError::generic_err("mint_with_nft - caller not owner or operator")
    );
    ensure!(
        is_contract_operator || is_contract_has_approval,
        StdError::generic_err("mint_with_nft - call option contract not operator or has approval",)
    );

    // find appropriate vault
    let vault_addr = deps
        .querier
        .query_wasm_smart::<Option<Addr>>(
            &config.vault_factory_addr,
            &QueryMsg::GetMultiOrSoloVault {
                nft_addr: nft_addr.as_str().into(),
                nft_id: Some(nft_id.clone()),
            },
        )?
        .ok_or(StdError::generic_err(
            "mint_with_nft - appropriate vault not found",
        ))?;

    let new_option_id = mint_call(
        deps,
        env,
        &owner,
        &vault_addr,
        &nft_id,
        strike,
        expiration,
        config,
    )?;

    let set_entitlement = SetEntitlement {
        beneficial_owner: info.sender,
        entitled_operator: env.contract.address.clone(),
        approved_operator: None,
        expiry: expiration,
    };

    // send the underlying asset into our vault, passing along the entitlement. The entitlement specified
    // here will be accepted by the vault because we are also simultaneously tendering the asset.
    let send_nft_msg = nft::send_nft(
        &nft_addr,
        &nft_id,
        &vault_addr,
        to_binary(&Some(set_entitlement))?,
    )?;

    Ok(Response::new()
        .add_submessage(send_nft_msg)
        .add_attribute("action", "mint_with_nft")
        .add_attribute("option_id", new_option_id.to_string()))
}

/// Mints a new call option for the assets deposited in a particular vault given strike price and expiration.
#[allow(clippy::too_many_arguments)]
pub(crate) fn mint_with_vault(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    vault: String,
    asset_id: AssetId,
    strike: Uint128,
    expiration: Expiration,
    config: &Config,
) -> Result<Response, ContractError> {
    deps.api.addr_validate(&vault)?;
    // assert_valid_addr(deps.api, vec![&vault], PREFIX)?;

    // check that sender uses allowed nft
    let nft_addr: Addr = deps
        .querier
        .query_wasm_smart(&vault, &VaultQueryMsg::AssetAddress {})?;
    if config.allowed_underlying_nft != nft_addr {
        return Err(StdError::generic_err("mint_with_vault - nft not allowes").into());
    }

    // check that asset already in the vault
    if !deps.querier.query_wasm_smart::<bool>(
        &vault,
        &VaultQueryMsg::HoldsAsset {
            asset_id: asset_id.clone(),
        },
    )? {
        return Err(StdError::generic_err("mint_with_vault - asset not in vault").into());
    }

    // TODO verify that a particular vault was created by the protocol's vault factory
    // _allowedVaultImplementation(vaultAddress, allowedUnderlyingAddress, assetId)

    // the beneficial owner is the only one able to impose entitlements, so
    // we need to require that they've done so here
    let (ok, beneficial_owner) =
        is_beneficial_owner_or_operator(&deps.querier, &vault, asset_id.clone(), &info.sender)?;
    ensure!(
        ok,
        StdError::generic_err(
            "mint_with_vault - called by someone other than the beneficial owner or operator",
        )
    );
    let beneficial_owner = beneficial_owner.ok_or(StdError::generic_err(
        "mint_with_entitled_vault - beneficial owner not set",
    ))?;

    let new_option_id = mint_call(
        deps,
        &env,
        beneficial_owner,
        &vault,
        &asset_id,
        strike,
        expiration,
        config,
    )?;

    let impose_entitlement_msg = CosmosMsg::<Empty>::Wasm(WasmMsg::Execute {
        contract_addr: vault,
        msg: to_binary(&VaultExecuteMsg::ImposeEntitlement {
            operator: env.contract.address.into_string(),
            expiry: expiration,
            asset_id,
        })?,
        funds: vec![],
    });

    Ok(Response::new()
        .add_message(impose_entitlement_msg)
        .add_attribute("action", "mint_with_vault")
        .add_attribute("option_id", new_option_id.to_string()))
}

/// Mints a new call option for the assets deposited in a particular vault given strike price and expiration.
/// That vault must already have a registered entitlement for this contract with the an expiration equal to {expirationTime}
#[allow(clippy::too_many_arguments)]
pub(crate) fn mint_with_entitled_vault(
    deps: DepsMut,
    env: &Env,
    info: MessageInfo,
    vault: &String,
    asset_id: AssetId,
    strike: Uint128,
    expiration: Expiration,
    config: &Config,
) -> Result<Response, ContractError> {
    // check that sender uses allowed nft
    let nft_addr: Addr = deps
        .querier
        .query_wasm_smart(vault, &VaultQueryMsg::AssetAddress {})?;
    if config.allowed_underlying_nft != nft_addr {
        return Err(StdError::generic_err("mint_with_vault - nft not allowes").into());
    }

    // check that asset already in the vault
    if !deps.querier.query_wasm_smart::<bool>(
        vault,
        &VaultQueryMsg::HoldsAsset {
            asset_id: asset_id.clone(),
        },
    )? {
        return Err(StdError::generic_err("mint_with_vault - asset not in vault").into());
    }

    // only current operator can mint option with already entitled asset
    let CurrentEntitlementOperatorResponse {
        is_active,
        operator,
    } = deps.querier.query_wasm_smart(
        vault,
        &VaultQueryMsg::CurrentEntitlementOperator {
            asset_id: asset_id.clone(),
        },
    )?;
    if !is_active && operator.map_or(true, |op| op != env.contract.address) {
        return Err(
            StdError::generic_err("mint_with_entitled_vault - call contract not operator").into(),
        );
    }

    // expiration must be equal
    let entitlement_expiration: Option<Expiration> = deps.querier.query_wasm_smart(
        vault,
        &VaultQueryMsg::EntitlementExpiration {
            asset_id: asset_id.clone(),
        },
    )?;
    if entitlement_expiration.map_or(true, |e| e != expiration) {
        return Err(StdError::generic_err(
            "mint_with_entitled_vault - entitlement expiration different",
        )
        .into());
    }

    // the beneficial owner owns the asset so they should receive the option
    let (ok, writer) =
        is_beneficial_owner_or_operator(&deps.querier, vault, asset_id.clone(), &info.sender)?;
    if !ok {
        return Err(StdError::generic_err(
            "mint_with_entitled_vault - only owner or operator may mint",
        )
        .into());
    }

    // TODO we need this checks?
    let writer_addr = writer.ok_or(StdError::generic_err(
        "mint_with_entitled_vault - beneficial owner not set",
    ))?;

    let new_option_id = mint_call(
        deps,
        env,
        writer_addr,
        vault,
        &asset_id,
        strike,
        expiration,
        config,
    )?;

    Ok(Response::new()
        .add_attribute("action", "mint_with_entitled_vault")
        .add_attribute("option_id", new_option_id.to_string()))
}

pub(crate) fn bid(
    deps: DepsMut,
    info: MessageInfo,
    option_id: &OptionId,
    config: &Config,
) -> Result<Response, ContractError> {
    // TODO use this macros or use if?
    // ensure!(
    //     !has_allowed_denoms(&info.funds, &config.allowed_denoms),
    //     Err(StdError::generic_err(
    //         "bid - not allowed denom",
    //     ));
    // );
    let mut new_bid = find_allowed_coin(info.funds.clone(), config.allowed_denom.as_ref())
        .ok_or(ContractError::DenomNotAllowed {})?;

    let mut call = CallInstrument::load(deps.storage, option_id)?;

    if info.sender == call.writer_addr {
        // Handle the case where an option writer bids on an underlying asset that they owned.
        // In this case, as they would be the recipient of the spread after the auction,
        // they are able to bid paying only the difference between their bid and the strike.
        new_bid.amount += call.strike;
    }

    ensure!(
        call.bid.u128() / 10_000 * 10_000 == call.bid.u128(),
        StdError::generic_err("bid - bid amount too small")
    ); // TODO why am I doing it?
    let bid_increment: u128 =
        (call.bid.u128() * config.min_bid_increment_bps as u128) / 10_000_u128;
    let min_required_amount = call.bid.checked_add(bid_increment.into())?;

    // min_required_amount = call_bid + ((call_bid * min_bid_increment_bips) / 10000)
    // let network_fee = payment * params.trading_fee_percent / Uint128::from(100u128);
    // new_bid.amount.multiply_ratio(90u128, 100u128);

    // 1_000_000_000_000_000_000
    // let bid_increment = call
    //     .bid
    //     .checked_mul(Decimal::percent(config.min_bid_increment_bps))?
    //     .checked_div(Uint128::new(10000))?;
    // let min_required_amount = call.bid.checked_add(bid_increment)?;

    ensure!(
        new_bid.amount >= min_required_amount,
        StdError::generic_err("bid - must overbid by minBidIncrementBips")
    );

    ensure!(
        new_bid.amount >= call.strike,
        ContractError::BidIsLowerStrikePrice {}
    );

    let resp = Response::new()
        .add_attribute("action", "bid")
        .add_attribute("bid_amount", new_bid.amount);

    // return bid to previous bidder
    let resp = match call.bidder {
        Some(high_bidder) => {
            let bid_to_return = call.bid;
            if high_bidder == call.writer_addr {
                bid_to_return.checked_sub(call.strike)?;
            }
            if bid_to_return > Uint128::zero() {
                // handle the case when high_bidder is Some and bid_to_return is greater than zero
                let return_bid_msg = bank_send_msg(
                    high_bidder.into_string(),
                    config.allowed_denom.coins(&bid_to_return),
                );
                resp.add_message(return_bid_msg)
            } else {
                resp
            }
        }
        _ => resp,
    };

    // set the new bidder
    call.bid = new_bid.amount;
    call.bidder = Some(info.sender.clone());
    call.save(deps.storage, option_id)?;

    // the new high bidder is the beneficial owner of the asset.
    // the beneficial owner must be set here instead of with a settlement
    // because otherwise the writer will be able to remove the asset from the vault
    // between the expiration and the settlement call, effectively stealing the asset.
    let resp = resp.add_message(set_beneficial_owner_wasm_msg(
        call.vault_addr.as_str(),
        call.asset_id.as_str(),
        info.sender.as_str(),
    )?);

    Ok(resp)
}

/// Allows the writer to reclaim an entitled asset. This is only possible
/// when the writer holds the option nft and calls this function.
pub(crate) fn reclaim_asset(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    option_id: &OptionId,
    withdraw: bool,
    config: &Config,
) -> Result<Response, ContractError> {
    nonpayable(&info)?; // TODO ???

    let mut call = CallInstrument::load(deps.storage, option_id)?;

    ensure!(
        call.writer_addr == info.sender,
        ContractError::Unauthorized {} // TODO change type error
    );
    ensure!(
        !call.settled,
        ContractError::OptionAlreadySettled(option_id.to_owned())
    );
    ensure!(
        !call.expiration.is_expired(&env.block),
        ContractError::OptionIsExpired {}
    );

    let owner_option = option_owner(&deps, &env, option_id.to_string())?;
    ensure!(
        call.writer_addr.to_owned().into_string() == owner_option,
        StdError::generic_err("reclaim_asset - writer must own option",)
    );

    // settle the option
    call.settled = true;
    call.save(deps.storage, option_id)?;

    // burn the option NFT
    burn_option_nft(deps, env, info, option_id.to_string())?;

    let mut msgs = vec![];

    // return current bidder's money
    if let Some(high_bidder) = call.bidder {
        let returned_amount = if high_bidder == call.writer_addr {
            call.bid.checked_sub(call.strike)?
        } else {
            call.bid
        };
        msgs.push(bank_send_msg(
            high_bidder.into_string(),
            config.allowed_denom.coins(&returned_amount),
        ))
    }

    // if we have a bid, we may have set the bidder, so make sure to revert it here.
    msgs.push(set_beneficial_owner_wasm_msg(
        call.vault_addr.as_str(),
        call.asset_id.as_str(),
        call.writer_addr.as_str(),
    )?);

    if withdraw {
        msgs.push(clear_entitlement_and_distribute_wasm_msg(
            call.vault_addr.as_str(),
            call.asset_id.as_str(),
            call.writer_addr.as_str(),
        )?)
    } else {
        msgs.push(clear_entitlement_wasm_msg(
            call.vault_addr.as_str(),
            call.asset_id.as_str(),
        )?)
    }

    Ok(Response::new().add_messages(msgs))
}

/// Permissionlessly settle an expired option when the option expires in the money,
/// distributing the proceeds to the Writer, Holder, and Bidder
/// WRITER (who originally called mint() and owned underlying asset) receives the `strike`
/// HOLDER (ownerOf(optionId)) receives `bid - strike`
/// HIGH BIDDER (call.bidder) that pays `bid`, becomes ownerOf NFT,
pub(crate) fn settle_option(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    option_id: &OptionId,
    config: &Config,
) -> Result<Response, ContractError> {
    let mut call = CallInstrument::load(deps.storage, option_id)?;

    let high_bidder_addr = call
        .bidder
        .clone()
        .ok_or(ContractError::NoWinningBidder())?;
    ensure!(
        call.expiration.is_expired(&env.block),
        ContractError::OptionNotExpired(option_id.to_owned())
    );
    ensure!(
        !call.settled,
        ContractError::OptionAlreadySettled(option_id.to_owned())
    );

    let spread = call.bid.checked_sub(call.strike)?;

    let owner_option = option_owner(&deps, &env, option_id.to_string())?;

    // set settled to prevent an additional attempt to settle the option
    call.settled = true;
    call.save(deps.storage, option_id)?;

    let mut msgs = vec![];

    // if the option writer is the high bidder they don't receive the strike because they bid on the spread.
    if high_bidder_addr != call.writer_addr {
        msgs.push(bank_send_msg(
            call.writer_addr.into_string(),
            config.allowed_denom.coins(&call.strike),
        ));
    };

    let mut claimable = false;
    if info.sender == owner_option {
        // send option holder their earnings
        msgs.push(bank_send_msg(
            owner_option,
            config.allowed_denom.coins(&spread),
        ));
        burn_option_nft(deps, env, info, option_id.to_string())?;
    } else {
        OPTION_CLAIMS.save(deps.storage, option_id, &spread)?;
        claimable = true;
    }

    Ok(Response::new()
        .add_messages(msgs)
        .add_attribute("action", "settle_option")
        .add_attribute("claimable", claimable.to_string()))
}

/// Allows anyone to burn the instrument NFT for an expired option
pub(crate) fn burn_expired_option(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    option_id: &OptionId,
) -> Result<Response, ContractError> {
    let call = CallInstrument::load(deps.storage, option_id)?;

    ensure!(
        call.bidder.is_none(),
        StdError::generic_err("option has bids")
    );
    ensure!(
        call.expiration.is_expired(&env.block),
        ContractError::OptionNotExpired(option_id.to_owned())
    );
    ensure!(
        !call.settled,
        ContractError::OptionAlreadySettled(option_id.to_owned())
    );

    call.update(deps.storage, option_id)?;

    burn_option_nft(deps, env, info, option_id.to_string())?;

    Ok(Response::new().add_attribute("action", "burn_expired_option"))
}

/// Allows the option owner to claim proceeds if the option was settled
/// by another account. The option NFT is burned after settlement.
pub(crate) fn claim_option_proceeds(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    option_id: &OptionId,
    config: &Config,
) -> Result<Response, ContractError> {
    let owner_option = option_owner(&deps, &env, option_id.to_string())?;
    ensure!(
        info.sender.clone().into_string() == owner_option,
        ContractError::OnlyOptionOwner(info.sender.to_string())
    );
    let claim = OPTION_CLAIMS.load(deps.storage, option_id)?;
    OPTION_CLAIMS.remove(deps.storage, option_id);

    ensure!(!claim.is_zero(), StdError::generic_err("claim is zero"));
    burn_option_nft(deps, env, info, option_id.to_string())?;

    let send_claim_msg = bank_send_msg(owner_option, config.allowed_denom.coins(&claim));
    Ok(Response::new()
        .add_message(send_claim_msg)
        .add_attribute("action", "claim_option_proceeds"))
}

pub fn for_test() -> Result<Response, ContractError> {
    let r = 10;
    let _ = r;
    Ok(Response::default())
}

/*
function _returnBidToPreviousBidder(CallOption storage call) internal {
     uint256 unNormalizedHighBid = call.bid;
     if (call.highBidder == call.writer) {
         unNormalizedHighBid -= call.strike;
     }

     // return current bidder's money
     if (unNormalizedHighBid > 0) {
         _safeTransferETHWithFallback(call.highBidder, unNormalizedHighBid);
     }
 }
  */
