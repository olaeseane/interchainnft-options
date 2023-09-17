use cosmwasm_std::{to_binary, Binary, Deps, StdResult};

use common::types::TokenId;

use crate::state::{MULTI_VAULTS, SOLO_VAULTS};

/// Gets the address of a vault for a particular id NFT token.
pub(crate) fn get_vault(deps: Deps, nft_addr: &str, nft_id: &TokenId) -> StdResult<Binary> {
    let nft_addr = deps.api.addr_validate(nft_addr)?;
    let vault_addr = SOLO_VAULTS.may_load(deps.storage, (&nft_addr, nft_id))?;
    to_binary(&vault_addr)
}

/// Gets the address of a multi-asset vault for a particular NFT contract, if one exists.
pub(crate) fn get_multi_vault(deps: Deps, nft_addr: &str) -> StdResult<Binary> {
    let nft_addr = deps.api.addr_validate(nft_addr)?;
    let vault_addr = MULTI_VAULTS.may_load(deps.storage, &nft_addr)?;
    to_binary(&vault_addr)
}

/// Gets the address of a multi-asset vault if one exists, if no exists gets solo vault for a particular id NFT token.
pub(crate) fn get_multi_or_solo_vault(
    deps: Deps,
    nft_addr: &str,
    nft_id: Option<TokenId>,
) -> StdResult<Binary> {
    let nft_addr = deps.api.addr_validate(nft_addr)?;
    let mut vault_addr = MULTI_VAULTS.may_load(deps.storage, &nft_addr)?;
    if vault_addr.is_none() && nft_id.is_some() {
        vault_addr = SOLO_VAULTS.may_load(deps.storage, (&nft_addr, &nft_id.unwrap()))?;
    }
    to_binary(&vault_addr)
}
