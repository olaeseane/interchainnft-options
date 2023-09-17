use std::sync::Mutex;

use common::errors::ContractError;
use cosmwasm_std::{
    testing::{mock_dependencies, mock_env, mock_info, MockApi, MockQuerier, MockStorage},
    to_binary, Addr, CosmosMsg, MessageInfo, OwnedDeps, ReplyOn, Response, SubMsg, WasmMsg,
};
use cw2::ContractVersion;
use cw_ownable::OwnershipError;
use once_cell::sync::Lazy;

#[allow(unused_imports)]
use log::debug;

use crate::{
    contract::{self, *},
    execute::{INSTANTIATE_MULTI_VAULT_ID, INSTANTIATE_SOLO_VAULT_ID},
    msg::*,
    state::*,
};

static SETUP_LOGGER: Lazy<Mutex<()>> = Lazy::new(|| {
    env_logger::init();
    Mutex::new(())
});

const OWNER: &str = "owner_addr";
const USER: &str = "user_addr";
const VAULT_CODE_ID: u64 = 11;

#[allow(dead_code)]
#[allow(let_underscore_lock)]
fn setup(info_sender: &str) -> (OwnedDeps<MockStorage, MockApi, MockQuerier>, MessageInfo) {
    let _ = SETUP_LOGGER.lock().unwrap();

    let mut deps = mock_dependencies();
    let info = mock_info(info_sender, &[]);

    let msg = InstantiateMsg {
        protocol_addr: "protocol_addr".to_string(),
        vault_code_id: VAULT_CODE_ID,
        owner: Some(OWNER.to_string()),
    };
    let res = instantiate(deps.as_mut(), mock_env(), info.clone(), msg);

    assert!(res.is_ok());

    (deps, info)
}

#[test]
fn proper_initialization() {
    let msg = InstantiateMsg {
        protocol_addr: "protocol_addr".to_string(),
        vault_code_id: VAULT_CODE_ID,
        owner: Some(OWNER.to_string()),
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
    assert_eq!(config.protocol_addr, "protocol_addr");
    assert_eq!(config.vault_code_id, VAULT_CODE_ID);

    let version = cw2::get_contract_version(&deps.storage).unwrap();
    assert_eq!(
        version,
        ContractVersion {
            contract: "crates.io:interchainnft-options-vault-factory".to_string(),
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
fn make_multi_vault() {
    let (mut deps, info) = setup(OWNER);

    let msg = ExecuteMsg::MakeMultiVault {
        nft_addr: "nft_addr".to_string(),
    };

    let resp = contract::execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    assert_eq!(
        resp,
        Response::new()
            .add_submessage(SubMsg {
                id: INSTANTIATE_MULTI_VAULT_ID,
                msg: CosmosMsg::Wasm(WasmMsg::Instantiate {
                    admin: None,
                    code_id: VAULT_CODE_ID,
                    msg: to_binary(&vault::msg::InstantiateMsg {
                        nft_addr: "nft_addr".to_string(),
                        nft_id: None,
                        protocol_addr: "protocol_addr".to_string(),
                    })
                    .unwrap(),
                    funds: vec![],
                    label: "Interchainnft-options multi vault".into(),
                }),
                reply_on: ReplyOn::Success,
                gas_limit: None,
            })
            .add_attribute("action", "make_multi_vault")
            .add_attribute("sender", OWNER)
    );
}

#[test]
fn make_solo_vault() {
    let (mut deps, info) = setup(OWNER);

    let msg = ExecuteMsg::MakeSoloVault {
        nft_addr: "nft_addr".to_string(),
        nft_id: "nft_id".to_string(),
    };

    let resp = contract::execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    assert_eq!(
        resp,
        Response::new()
            .add_submessage(SubMsg {
                id: INSTANTIATE_SOLO_VAULT_ID,
                msg: CosmosMsg::Wasm(WasmMsg::Instantiate {
                    admin: None,
                    code_id: VAULT_CODE_ID,
                    msg: to_binary(&vault::msg::InstantiateMsg {
                        nft_addr: "nft_addr".to_string(),
                        nft_id: Some("nft_id".to_string()),
                        protocol_addr: "protocol_addr".to_string(),
                    })
                    .unwrap(),
                    funds: vec![],
                    label: "Interchainnft-options solo vault".into(),
                }),
                reply_on: ReplyOn::Success,
                gas_limit: None,
            })
            .add_attribute("action", "make_solo_vault")
            .add_attribute("sender", OWNER)
    );
}

#[test]
fn not_owner_cant_make_vaults() {
    let (mut deps, info) = setup(USER);

    let msg = ExecuteMsg::MakeMultiVault {
        nft_addr: "nft_addr".to_string(),
    };

    let resp = contract::execute(deps.as_mut(), mock_env(), info, msg);
    assert_eq!(
        resp,
        Err(ContractError::OwnershipError(OwnershipError::NotOwner))
    );
}

// TODO integration test create vault if already exists
// TODO integration test get created vault
