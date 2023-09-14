use cosmwasm_std::{to_binary, Binary, Deps, StdResult};

use crate::state::Config;

pub fn config(deps: Deps) -> StdResult<Binary> {
    let config = Config::load(deps.storage)?;

    to_binary(&config)
}
