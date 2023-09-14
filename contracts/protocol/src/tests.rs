use std::sync::Mutex;

use cosmwasm_std::{
    from_binary,
    testing::{mock_dependencies, mock_env, mock_info, MockApi, MockQuerier, MockStorage},
    Addr, MessageInfo, OwnedDeps, Response,
};
use cw2::ContractVersion;
use once_cell::sync::Lazy;

#[allow(unused_imports)]
use log::debug;

use crate::{
    contract::{self, *},
    msg::*,
    state::*,
};

static SETUP_LOGGER: Lazy<Mutex<()>> = Lazy::new(|| {
    env_logger::init();
    Mutex::new(())
});

const OWNER: &str = "owner_addr";
const USER: &str = "user_addr";

#[allow(dead_code)]
#[allow(let_underscore_lock)]
fn setup(info_sender: &str) -> (OwnedDeps<MockStorage, MockApi, MockQuerier>, MessageInfo) {
    let _ = SETUP_LOGGER.lock().unwrap();

    let mut deps = mock_dependencies();
    let info = mock_info(info_sender, &[]);

    let vault_factory_addr = "cosmos_vault_factory_addr".to_string();
    let option_factory_addr = "cosmos_option_factory_addr".to_string();

    let msg = InstantiateMsg {
        owner: Some(OWNER.to_string()),
        vault_factory_addr: vault_factory_addr.clone(),
        option_factory_addr: option_factory_addr.clone(),
    };
    let res = instantiate(deps.as_mut(), mock_env(), info.clone(), msg);

    assert!(res.is_ok());

    (deps, info)
}

#[test]
fn proper_initialization() {
    let vault_factory_addr = "cosmos_vault_factory_addr".to_string();
    let option_factory_addr = "cosmos_option_factory_addr".to_string();

    let msg = InstantiateMsg {
        owner: Some(OWNER.to_string()),
        vault_factory_addr: vault_factory_addr.clone(),
        option_factory_addr: option_factory_addr.clone(),
    };

    let mut deps = mock_dependencies();
    let info = mock_info(USER, &[]);

    let res = instantiate(deps.as_mut(), mock_env(), info, msg);

    assert_eq!(
        res,
        Ok(Response::new()
            .add_attribute("action", "instantiate")
            .add_attribute("sender", "user_addr"))
    );

    let config = Config::load(&deps.storage).unwrap();
    assert_eq!(config.vault_factory_addr, vault_factory_addr);
    assert_eq!(config.vault_factory_addr, vault_factory_addr);

    let version = cw2::get_contract_version(&deps.storage).unwrap();
    assert_eq!(
        version,
        ContractVersion {
            contract: "crates.io:interchainnft-options-protocol".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        }
    );

    let ownership = cw_ownable::get_ownership(&deps.storage).unwrap();
    assert_eq!(
        ownership.owner.expect("owner is None"),
        Addr::unchecked(OWNER)
    );
}

#[test]
fn set_vault_factory_and_covered_call_factory() {
    let (mut deps, info) = setup(OWNER);

    let msg = ExecuteMsg::SetCoveredCallFactory {
        contract_addr: "new_covered_call_factory".to_string(),
    };
    let _ = contract::execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

    let msg = ExecuteMsg::SetVaultFactory {
        contract_addr: "new_vault_factory_addr".to_string(),
    };
    let _ = contract::execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    let msg = QueryMsg::Config {};
    let res = contract::query(deps.as_ref(), mock_env(), msg).unwrap();
    let config: Config = from_binary(&res).unwrap();
    assert_eq!(
        config.option_factory_addr,
        Addr::unchecked("new_covered_call_factory")
    );
    assert_eq!(
        config.vault_factory_addr,
        Addr::unchecked("new_vault_factory_addr")
    )
}

#[test]
fn not_owner_cant_set_vault_factory() {
    let (mut deps, info) = setup(USER);

    let msg = ExecuteMsg::SetCoveredCallFactory {
        contract_addr: "new_covered_call_factory".to_string(),
    };
    let res = contract::execute(deps.as_mut(), mock_env(), info.clone(), msg);

    assert!(res.is_err()); // TODO add Err equals
}

// TODO test pause()
