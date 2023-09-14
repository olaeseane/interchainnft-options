use cosmwasm_std::{Addr, Api};

use crate::errors::ContractError;

pub const PREFIX: &str = "cosmos";

/// Assert an address is valid
///
/// NOTE: The `deps.api.addr_validate` function can only verify addresses of the current chain, e.g.
/// a contract on Osmosis can only verify addresses with the `osmo1` prefix. If the provided address
/// does not start with this prefix, we use bech32 decoding (valid address should be successfully decoded).
pub fn assert_valid_addr(
    api: &dyn Api,
    humans: Vec<&str>,
    prefix: &str,
) -> Result<(), ContractError> {
    for &h in humans.iter() {
        if h.starts_with(prefix) {
            api.addr_validate(h)
                .map_err(|_| ContractError::InvalidAddress(h.to_string()))?;
        } else {
            bech32::decode(h).map_err(|_| ContractError::InvalidAddress(h.to_string()))?;
        }
    }
    Ok(())
}

/// Prefix should be related to owner address prefix on a specific chain
pub fn assert_valid_prefix(owner: &str, prefix: &str) -> Result<(), ContractError> {
    if !owner.starts_with(prefix) {
        return Err(ContractError::InvalidChainPrefix(prefix.to_string()));
    }
    Ok(())
}

// TODO check bech32::decode() result
// TODO use instead of deps.api.addr_validate()
pub fn into_addr(api: &dyn Api, human: String, prefix: &str) -> Result<Addr, ContractError> {
    if human.starts_with(prefix) {
        api.addr_validate(&human)
            .map_err(|_| ContractError::InvalidAddress(human))
    } else {
        bech32::decode(&human)
            .map(|(h, _, _)| Addr::unchecked(h))
            .map_err(|_| ContractError::InvalidAddress(human.to_string()))
    }
}
