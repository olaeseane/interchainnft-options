use common::types::{AssetId, OptionId};
use cosmwasm_std::{to_binary, Binary, Deps, StdResult};

use crate::state::{CallInstrument, VAULT_ASSET_OPTION};

pub fn current_bid(deps: Deps, option_id: &OptionId) -> StdResult<Binary> {
    let call = CallInstrument::load(deps.storage, option_id)?;

    to_binary(&call.bid)
}

pub fn current_bidder(deps: Deps, option_id: &OptionId) -> StdResult<Binary> {
    let call = CallInstrument::load(deps.storage, option_id)?;

    to_binary(&call.bidder)
}

pub fn get_vault_address(deps: Deps, option_id: &OptionId) -> StdResult<Binary> {
    let call = CallInstrument::load(deps.storage, option_id)?;

    to_binary(&call.vault_addr)
}

pub fn get_option_id_for_asset(deps: Deps, vault: String, asset_id: &AssetId) -> StdResult<Binary> {
    let vault_addr = deps.api.addr_validate(&vault)?;
    let option_id = VAULT_ASSET_OPTION.load(deps.storage, (&vault_addr, asset_id))?;

    to_binary(&option_id)
}

pub fn get_asset_id(deps: Deps, option_id: &OptionId) -> StdResult<Binary> {
    let call = CallInstrument::load(deps.storage, option_id)?;

    to_binary(&call.asset_id)
}

pub fn get_strike_price(deps: Deps, option_id: &OptionId) -> StdResult<Binary> {
    let call = CallInstrument::load(deps.storage, option_id)?;

    to_binary(&call.strike)
}

pub fn get_expiration(deps: Deps, option_id: &OptionId) -> StdResult<Binary> {
    let call = CallInstrument::load(deps.storage, option_id)?;

    to_binary(&call.expiration)
}
