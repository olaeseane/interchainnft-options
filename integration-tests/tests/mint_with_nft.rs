#[allow(unused_imports)]
use cosmwasm_std::{
    testing::{mock_dependencies, mock_env as _mock_env, mock_info},
    Uint128,
};
use cw_utils::Expiration;
use testing::{mock_contracts::CallInstrument, mock_env::MockEnvBuilder};

#[test]
fn mint_with_nft_flow() {
    // 1. deploy and instantiate protocol, vault/call factories and underlying nft contracts
    let mut mock_env = MockEnvBuilder::new(None, None, None, None).build();
    let underlying_nft = mock_env.underlying_nft.clone();
    let call_factory = mock_env.call_factory.clone();
    let vault_factory = mock_env.vault_factory.clone();

    let token_id = "id001";
    let token_owner = mock_env.writer.clone();
    let strike = Uint128::new(5);

    // let expiration = Expiration::AtTime(mock_env.query_block_info().time.plus_days(1));

    // 2. mint nft
    underlying_nft.mint(&mut mock_env, token_id, token_owner.as_str());

    // 3. make multi vault for nft
    let vault_addr =
        vault_factory.make_multi_vault(&mut mock_env, underlying_nft.contract_addr.as_str());

    // 4. make call instrument for nft
    call_factory.make_call_instrument(&mut mock_env, underlying_nft.contract_addr.as_str());
    let call_instrument = CallInstrument {
        contract_addr: call_factory
            .query_call_instrument(&mock_env, underlying_nft.contract_addr.as_str())
            .unwrap(),
    };

    // 5. grant nft approval to call instrument
    underlying_nft.approve(
        &mut mock_env,
        token_owner.as_str(),
        call_instrument.contract_addr.as_str(),
        token_id,
    );

    // 4. mint call-option with transfering nft to vault
    let nft_expired = mock_env.query_block_info().time.plus_days(1);
    let option_id = call_instrument.mint_with_nft(
        &mut mock_env,
        underlying_nft.contract_addr.as_str(),
        token_id.to_owned(),
        strike,
        Expiration::AtTime(nft_expired),
    );

    // 5. run checks call-option info
    assert_eq!(
        call_instrument.query_get_vault_address(&mock_env, option_id),
        vault_addr,
    );
    assert_eq!(
        call_instrument.query_get_option_id_for_asset(
            &mock_env,
            vault_addr.as_str(),
            &token_id.to_string()
        ),
        option_id
    );
    assert_eq!(
        call_instrument.query_get_asset_id(&mock_env, option_id),
        token_id,
    );
    assert_eq!(
        call_instrument.query_get_strike_price(&mock_env, option_id),
        Uint128::new(5),
    );
    assert_eq!(
        call_instrument.query_get_expiration(&mock_env, option_id),
        Expiration::AtTime(nft_expired),
    );

    // 6. nft must owned by vault
    assert_eq!(underlying_nft.query_owner(&mock_env, token_id), vault_addr);
}

/*
   // 5. send nft to the vault
   cw721_nft.send(
       &mut mock_env,
       token_owner,
       token_id,
       vault_addr.as_str(),
       entitlement,
   );
*/
