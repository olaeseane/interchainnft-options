use testing::mock_env::MockEnvBuilder;

#[test]
fn make_call_instruments_flow() {
    let token_id = "nft_id001";
    let token_owner = "nft_token_owner_addr";

    // 1. deploy and instantiate call instrument factory and underlying nft
    let mut mock_env = MockEnvBuilder::new(None, None, None, None).build();
    let underlying_nft = mock_env.underlying_nft.clone();
    let call_factory = mock_env.call_factory.clone();

    // 2. mint underlying nft
    underlying_nft.mint(&mut mock_env, token_id, token_owner);

    // 3. make call instrument for nft
    call_factory.make_call_instrument(&mut mock_env, underlying_nft.contract_addr.as_str());
    let addr = call_factory.query_call_instrument(&mock_env, underlying_nft.contract_addr.as_str());
    assert_eq!(addr.unwrap(), "contract4");
}
