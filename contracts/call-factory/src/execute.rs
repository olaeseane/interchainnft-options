use cosmwasm_std::{ensure, Addr, DepsMut, Response, StdError, SubMsg};

use call::utils::call_instrument_instantiate_wasm_msg;
use common::errors::ContractError;

use crate::state::{Config, TmpInstrumentInfo, CALL_INSTRUMENTS, TMP_INSTRUMENT};

/// A `reply` call code ID used in a sub-message.
pub(crate) const INSTANTIATE_CALL_INSTRUMENT_ID: u64 = 10;

/// Create a call option instrument for a specific underlying asset address
pub(crate) fn make_call_instrument(
    deps: DepsMut,
    sender: &Addr,
    nft_addr: String,
    config: &Config,
) -> Result<Response, ContractError> {
    cw_ownable::assert_owner(deps.as_ref().storage, sender)?;

    let call_instrument_instantiate_wasm_msg = call_instrument_instantiate_wasm_msg(
        config.nft_name.clone(),
        config.nft_symbol.clone(),
        config.call_code_id,
        nft_addr.clone(),
        config.protocol_addr.to_string(),
        config.vault_factory_addr.to_string(),
        config.default_minimum_option_duration,
        config.default_allowed_denom.to_string(),
        config.default_min_bid_increment_bps,
        "Interchainnft-options call option nft".into(),
    )?;

    let nft_addr = deps.api.addr_validate(&nft_addr)?;

    ensure!(
        !CALL_INSTRUMENTS.has(deps.storage, &nft_addr),
        StdError::generic_err("make_call_instrument - instrument already exist")
    );

    TMP_INSTRUMENT.save(deps.storage, &TmpInstrumentInfo { nft_addr })?;

    Ok(Response::default()
        .add_submessage(SubMsg::reply_on_success(
            call_instrument_instantiate_wasm_msg,
            INSTANTIATE_CALL_INSTRUMENT_ID,
        ))
        .add_attribute("action", "make_call_instrument")
        .add_attribute("sender", sender))
}
