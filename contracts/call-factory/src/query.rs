use cosmwasm_std::{to_binary, Binary, Deps, StdResult};

use crate::state::CALL_INSTRUMENTS;

/// Lookup the call instrument contract based on the asset address
pub(crate) fn get_call_instrument(deps: Deps, nft_addr: &str) -> StdResult<Binary> {
    let nft_addr = deps.api.addr_validate(nft_addr)?;
    let call_instrument_addr = CALL_INSTRUMENTS.may_load(deps.storage, &nft_addr)?;
    to_binary(&call_instrument_addr)
}
