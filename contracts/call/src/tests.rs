use common::{denom::Denom, errors::ContractError};
use cosmwasm_std::{
    from_binary,
    testing::{mock_dependencies, mock_env, mock_info, MockApi, MockQuerier, MockStorage},
    Coin, Empty, MessageInfo, OwnedDeps, StdError, Uint128,
};
use cw2::ContractVersion;

use crate::{
    contract::{self, *},
    msg::*,
    state::*,
};

const USER: &str = "user_addr";
const NFT_ADDR: &str = "bayc_nft_addr";
const NFT_ID: &str = "id001";

#[allow(dead_code)]
#[allow(let_underscore_lock)]
fn setup(
    info_sender: &str,
    funds: &[Coin],
) -> (OwnedDeps<MockStorage, MockApi, MockQuerier>, MessageInfo) {
    let mut deps = mock_dependencies();
    let info = mock_info(info_sender, funds);

    let msg = InstantiateMsg {
        name: "call-option-nft".to_string(),
        symbol: "CALOPT".to_string(),
        protocol_addr: "protocol_addr".to_string(),
        allowed_underlying_nft: NFT_ADDR.to_string(),
        vault_factory_addr: "vault_factory_addr".to_string(),
        minimum_option_duration: 1,
        allowed_denom: "ATOM".to_string(),
        min_bid_increment_bps: 100,
    };
    let res = instantiate(deps.as_mut(), mock_env(), info.clone(), msg);

    assert!(res.is_ok());

    (deps, info)
}

#[test]
fn proper_initialization() {
    let msg = InstantiateMsg {
        name: "call-option-nft".to_string(),
        symbol: "CALOPT".to_string(),
        protocol_addr: "protocol_addr".to_string(),
        allowed_underlying_nft: NFT_ADDR.to_string(),
        vault_factory_addr: "vault_factory_addr".to_string(),
        minimum_option_duration: 1,
        allowed_denom: "ATOM".to_string(),
        min_bid_increment_bps: 100,
    };

    let mut deps = mock_dependencies();
    let info = mock_info(USER, &[]);

    let res = instantiate(deps.as_mut(), mock_env(), info, msg);

    assert!(res.is_ok());

    let config = Config::load(&deps.storage).unwrap();
    assert_eq!(config.protocol_addr, "protocol_addr");
    assert_eq!(config.vault_factory_addr, "vault_factory_addr");
    assert_eq!(config.allowed_underlying_nft, "bayc_nft_addr".to_string());
    assert_eq!(config.minimum_option_duration, 1);
    assert_eq!(config.allowed_denom, Denom::from("ATOM"));
    assert_eq!(config.min_bid_increment_bps, 100);

    let version = cw2::get_contract_version(&deps.storage).unwrap();
    assert_eq!(
        version,
        ContractVersion {
            contract: "crates.io:interchainnft-options-call-instrument".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        }
    );

    let minter = from_binary::<cw721_base::MinterResponse>(
        &contract::query(deps.as_ref(), mock_env(), cw721_base::QueryMsg::Minter {}).unwrap(),
    )
    .unwrap();

    assert!(minter.minter.is_some());
}

#[test]
fn cant_mint_with_wrong_nft() {
    let (mut deps, info) = setup(USER, &[]);

    let wrong_nft = "blabla_nft_addr";
    let msg = cw721_base::ExecuteMsg::Extension {
        msg: crate::ExecuteMsg::MintWithNFT {
            nft_addr: wrong_nft.to_string(),
            nft_id: wrong_nft.to_string(),
            strike: Uint128::zero(),
            expiration: cw_utils::Expiration::Never {},
        },
    };

    let res = contract::execute(deps.as_mut(), mock_env(), info, msg);

    assert_eq!(
        res,
        Err(ContractError::Std(StdError::GenericErr {
            msg: "mint_with_nft - nft not allowed".to_string()
        }))
    );
}

#[test]
fn cant_direct_minting() {
    let (mut deps, info) = setup(USER, &[]);

    let msg = cw721_base::ExecuteMsg::Mint {
        token_id: NFT_ID.to_string(),
        owner: USER.to_string(),
        token_uri: None,
        extension: Empty {},
    };
    let res = contract::execute(deps.as_mut(), mock_env(), info, msg);

    assert_eq!(
        res,
        // TODO replace transparent
        Err(ContractError::Std(StdError::GenericErr {
            msg: "direct minting is forbidden".to_string()
        }))
    )
}
