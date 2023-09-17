use std::sync::Mutex;

use common::errors::ContractError;
use cosmwasm_std::{
    testing::{mock_dependencies, mock_env, mock_info, MockApi, MockQuerier, MockStorage},
    to_binary, Addr, CosmosMsg, MessageInfo, OwnedDeps, ReplyOn, Response, SubMsg, Uint128,
    WasmMsg,
};
use cw2::ContractVersion;
use cw_ownable::OwnershipError;
use once_cell::sync::Lazy;

#[allow(unused_imports)]
use log::debug;

use crate::{
    contract::{self, *},
    execute::INSTANTIATE_CALL_INSTRUMENT_ID,
    msg::*,
    state::*,
};

static SETUP_LOGGER: Lazy<Mutex<()>> = Lazy::new(|| {
    env_logger::init();
    Mutex::new(())
});

const OWNER: &str = "owner_addr";
const USER: &str = "user_addr";
const CALL_INSTRUMENT_CODE_ID: u64 = 11;

#[allow(dead_code)]
#[allow(let_underscore_lock)]
fn setup(info_sender: &str) -> (OwnedDeps<MockStorage, MockApi, MockQuerier>, MessageInfo) {
    let _ = SETUP_LOGGER.lock().unwrap();

    let mut deps = mock_dependencies();
    let info = mock_info(info_sender, &[]);

    let msg = InstantiateMsg {
        protocol_addr: "protocol_addr".to_string(),
        call_code_id: CALL_INSTRUMENT_CODE_ID,
        owner: Some(OWNER.to_string()),
        nft_symbol: "nft_symbol".to_string(),
        nft_name: "nft_name".to_string(),
        default_minimum_option_duration: 1,
        default_allowed_denom: "ATOM".to_string(),
        default_min_bid_inc_bips: Uint128::new(1),
        vault_factory_addr: "vault_factory_addr".to_string(),
    };
    let res = instantiate(deps.as_mut(), mock_env(), info.clone(), msg);

    assert!(res.is_ok());

    (deps, info)
}

#[test]
fn proper_initialization() {
    let msg = InstantiateMsg {
        protocol_addr: "protocol_addr".to_string(),
        call_code_id: CALL_INSTRUMENT_CODE_ID,
        owner: Some(OWNER.to_string()),
        nft_symbol: "nft_symbol".to_string(),
        nft_name: "nft_name".to_string(),
        default_minimum_option_duration: 1,
        default_allowed_denom: "ATOM".to_string(),
        default_min_bid_inc_bips: Uint128::new(1),
        vault_factory_addr: "vault_factory_addr".to_string(),
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
    assert_eq!(config.call_code_id, CALL_INSTRUMENT_CODE_ID);
    assert_eq!(config.nft_symbol, "nft_symbol");
    assert_eq!(config.nft_name, "nft_name");
    assert_eq!(config.default_minimum_option_duration, 1);
    assert_eq!(config.default_allowed_denom, "ATOM");
    assert_eq!(config.default_min_bid_inc_bips, Uint128::new(1));
    assert_eq!(config.vault_factory_addr, "vault_factory_addr");

    let version = cw2::get_contract_version(&deps.storage).unwrap();
    assert_eq!(
        version,
        ContractVersion {
            contract: "crates.io:interchainnft-options-call-factory".to_string(),
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
fn make_call_instrument() {
    let (mut deps, info) = setup(OWNER);

    let msg = ExecuteMsg::MakeCallInstrument {
        nft_addr: "call_instrument_addr".to_string(),
    };

    let resp = contract::execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    assert_eq!(
        resp,
        Response::new()
            .add_submessage(SubMsg {
                id: INSTANTIATE_CALL_INSTRUMENT_ID,
                msg: CosmosMsg::Wasm(WasmMsg::Instantiate {
                    admin: None,
                    code_id: CALL_INSTRUMENT_CODE_ID,
                    msg: to_binary(&call::msg::InstantiateMsg {
                        name: "nft_name".to_string(),
                        symbol: "nft_symbol".to_string(),
                        protocol_addr: "protocol_addr".to_string(),
                        allowed_underlying_nft: "call_instrument_addr".to_string(),
                        minimum_option_duration: 1,
                        allowed_denom: "ATOM".to_string(),
                        min_bid_inc_bips: Uint128::new(1),
                        vault_factory_addr: "vault_factory_addr".to_string(),
                    })
                    .unwrap(),
                    funds: vec![],
                    label: "Interchainnft-options call option nft".into(),
                }),
                reply_on: ReplyOn::Success,
                gas_limit: None,
            })
            .add_attribute("action", "make_call_instrument")
            .add_attribute("sender", OWNER)
    );
}

#[test]
fn not_owner_cant_make_call_instrument() {
    let (mut deps, info) = setup(USER);

    let msg = ExecuteMsg::MakeCallInstrument {
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
