use std::sync::Mutex;

use cosmwasm_std::{
    from_binary,
    testing::{mock_dependencies, mock_env, mock_info, MockApi, MockQuerier, MockStorage},
    Addr, Attribute, MessageInfo, OwnedDeps,
};
use cw2::ContractVersion;
use once_cell::sync::Lazy;

use crate::{contract::*, msg::*, state::*};

static SETUP_LOGGER: Lazy<Mutex<()>> = Lazy::new(|| {
    env_logger::init();
    Mutex::new(())
});

const SENDER: &str = "sender_addr";

#[allow(dead_code)]
#[allow(let_underscore_lock)]
fn setup_multi_vault(
    info_sender: &str,
) -> (OwnedDeps<MockStorage, MockApi, MockQuerier>, MessageInfo) {
    let _ = SETUP_LOGGER.lock().unwrap();

    let mut deps = mock_dependencies();
    let info = mock_info(info_sender, &[]);

    let msg = InstantiateMsg {
        nft_addr: "nft_addr".to_string(),
        nft_id: None,
        protocol_addr: "protocol_addr".to_string(),
    };
    let res = instantiate(deps.as_mut(), mock_env(), info.clone(), msg);

    assert!(res.is_ok());

    (deps, info)
}

#[test]
fn proper_multi_vault_initialization() {
    let msg = InstantiateMsg {
        nft_addr: "nft_addr".to_string(),
        nft_id: None,
        protocol_addr: "protocol_addr".to_string(),
    };

    let mut deps = mock_dependencies();
    let info = mock_info(SENDER, &[]);

    let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    assert_eq!(
        res.attributes,
        vec![
            Attribute {
                key: "action".into(),
                value: "instantiate".into(),
            },
            Attribute {
                key: "sender".into(),
                value: "sender_addr".into(),
            }
        ]
    );
    assert!(res.data.is_some());
    assert_eq!(
        from_binary::<VaultInstantiateData>(&res.data.unwrap()).unwrap(),
        VaultInstantiateData {
            nft_addr: Addr::unchecked("nft_addr"),
            nft_id: None,
        }
    );

    let config = Config::load(&deps.storage).expect("failed to load config");
    assert_eq!(config.protocol_addr, "protocol_addr".to_string());
    assert_eq!(config.nft_addr, "nft_addr".to_string());
    assert_eq!(config.nft_id, None);

    let version = cw2::get_contract_version(&deps.storage).unwrap();
    assert_eq!(
        version,
        ContractVersion {
            contract: "crates.io:interchainnft-options-vault".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        }
    );
}

/*
#[test]
fn make_multi_vault() {
    let (mut deps, info) = setup(OWNER);

    let msg = ExecuteMsg::MakeMultiVault {
        nft_addr: "cosmos_nft_addr".to_string(),
    };

    let resp = contract::execute(deps.as_mut(), mock_env(), info, msg)
        .expect("failed to execute make_multi_vault()");

    assert_eq!(
        resp,
        Response::new()
            .add_submessage(SubMsg {
                id: INSTANTIATE_MULTI_VAULT_ID,
                msg: CosmosMsg::Wasm(WasmMsg::Instantiate {
                    admin: None,
                    code_id: MULTI_VAULT_CODE_ID,
                    msg: to_binary(&vault::msg::InstantiateMsg {
                        nft_addr: "cosmos_nft_addr".to_string(),
                        nft_id: None,
                        protocol_addr: "cosmos_protocol".to_string(),
                    })
                    .expect("failed to_binary()"),
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
        nft_addr: "cosmos_nft_addr".to_string(),
        nft_id: "nft_id".to_string(),
    };

    let resp = contract::execute(deps.as_mut(), mock_env(), info, msg)
        .expect("failed to execute make_solo_vault()");

    assert_eq!(
        resp,
        Response::new()
            .add_submessage(SubMsg {
                id: INSTANTIATE_SOLO_VAULT_ID,
                msg: CosmosMsg::Wasm(WasmMsg::Instantiate {
                    admin: None,
                    code_id: MULTI_VAULT_CODE_ID,
                    msg: to_binary(&vault::msg::InstantiateMsg {
                        nft_addr: "cosmos_nft_addr".to_string(),
                        nft_id: Some("nft_id".to_string()),
                        protocol_addr: "cosmos_protocol".to_string(),
                    })
                    .expect("failed to_binary()"),
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
 */

// TODO integration test create vault if already exists
// TODO integration test get created vault
