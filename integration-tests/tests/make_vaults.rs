use testing::mock_env::MockEnvBuilder;

#[test]
fn make_multi_and_solo_vaults_flow() {
    env_logger::init();

    let token_id = "nft_id001";
    let token_owner = "nft_token_owner_addr";

    // 1. deploy and instantiate vault factory and underlying nft
    let mut mock_env = MockEnvBuilder::new(None, None, None, None).build();
    let underlying_nft = mock_env.underlying_nft.clone();
    let vault_factory = mock_env.vault_factory.clone();

    mock_env.print_env();

    // 2. mint nft contract
    underlying_nft.mint(&mut mock_env, token_id, token_owner);

    // 3. make multi vault for nft
    vault_factory.make_multi_vault(&mut mock_env, underlying_nft.contract_addr.as_str());
    let addr = vault_factory.query_multi_or_solo_vault(
        &mock_env,
        underlying_nft.contract_addr.as_str(),
        None,
    );
    assert_eq!(addr.unwrap(), "contract4");

    // 4. make solo vault for nft
    vault_factory.make_solo_vault(
        &mut mock_env,
        underlying_nft.contract_addr.as_str(),
        token_id,
    );
    let addr =
        vault_factory.query_solo_vault(&mock_env, underlying_nft.contract_addr.as_str(), token_id);
    assert_eq!(addr.unwrap(), "contract5");

    // let env = cosmwasm_std::testing::mock_env();
    // call_factory::contract::instantiate(
    //     mock_dependencies().as_mut(),
    //     env,
    //     mock_info("sender", &[]),
    //     call_factory::InstantiateMsg {
    //         protocol_addr: "s".to_string(),
    //         call_code_id: 1,
    //         owner: None,
    //         nft_symbol: "s".to_string(),
    //         nft_name: "s".to_string(),
    //         default_minimum_option_duration: 1,
    //         default_allowed_denom: "s".to_string(),
    //         default_min_bid_inc_bips: Uint128::new(1),
    //         vault_factory_addr: "protocol_addr".to_string(),
    //     },
    // )
    // .unwrap();
}
