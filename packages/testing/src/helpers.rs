use common::types::OptionId;
use cosmwasm_std::{Addr, StdError, StdResult};
use cw_utils::Expiration;

use crate::{
    mock_contracts::{CallInstrument, Vault},
    mock_env::MockEnv,
    Params,
};

/// Assert elements in vecs one by one in order to get a more meaningful error
/// when debugging tests
pub fn assert_eq_vec<T: std::fmt::Debug + PartialEq>(expected: Vec<T>, actual: Vec<T>) {
    assert_eq!(expected.len(), actual.len());

    for (i, element) in expected.iter().enumerate() {
        assert_eq!(*element, actual[i]);
    }
}

/// Assert StdError::GenericErr message with expected_msg
pub fn assert_generic_error_message<T>(response: StdResult<T>, expected_msg: &str) {
    match response {
        Err(StdError::GenericErr { msg, .. }) => assert_eq!(msg, expected_msg),
        Err(other_err) => panic!("Unexpected error: {other_err:?}"),
        Ok(_) => panic!("SHOULD NOT ENTER HERE!"),
    }
}

// TODO think about naming
pub fn prep(mock_env: &mut MockEnv, params: &Params) -> (CallInstrument, Vault, OptionId) {
    let underlying_nft = mock_env.underlying_nft.clone();
    let call_factory = mock_env.call_factory.clone();
    let vault_factory = mock_env.vault_factory.clone();

    // 2. mint nft
    underlying_nft.mint(mock_env, params.token_id, params.token_owner);

    // 3. make multi vault for nft
    let vault_addr =
        vault_factory.make_multi_vault(mock_env, underlying_nft.contract_addr.as_str());
    let vault = Vault {
        contract_addr: Addr::unchecked(vault_addr.clone()),
    };

    // 4. make call instrument for nft
    call_factory.make_call_instrument(mock_env, underlying_nft.contract_addr.as_str());
    let call_instrument = CallInstrument {
        contract_addr: call_factory
            .query_call_instrument(mock_env, underlying_nft.contract_addr.as_str())
            .unwrap(),
    };

    // 5. grant nft approval to call instrument
    underlying_nft.approve(
        mock_env,
        params.token_owner,
        call_instrument.contract_addr.as_str(),
        params.token_id,
    );

    // 4. mint call-option with transfering nft to vault
    let expiration_timestamp = mock_env
        .query_block_info()
        .time
        .plus_days(params.expired_after);
    let option_id = call_instrument.mint_with_nft(
        mock_env,
        underlying_nft.contract_addr.as_str(),
        params.token_id.to_string(),
        params.strike,
        Expiration::AtTime(expiration_timestamp),
    );

    (call_instrument, vault, option_id)
}
