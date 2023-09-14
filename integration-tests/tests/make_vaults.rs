use testing::mock_env::MockEnvBuilder;

#[test]
fn make_multi_and_solo_vaults_flow() {
    env_logger::init();

    let token_id = "nft_id001";
    let token_owner = "nft_token_owner_addr";

    // 1. deploy and instantiate vault factory and underlying nft
    let mut mock_env = MockEnvBuilder::new(None, None, None, None).build();
    let cw721_nft = mock_env.cw721_nft.clone();
    let factory = mock_env.factory.clone();

    // 2. mint nft contract
    cw721_nft.mint(&mut mock_env, token_id, token_owner);

    // 3. make multi vault for nft
    factory.make_multi_vault(&mut mock_env, cw721_nft.contract_addr.as_str());
    let addr = factory.query_multi_or_solo_vault(&mock_env, cw721_nft.contract_addr.as_str(), None);
    assert_eq!(addr.unwrap(), "contract3");

    // 4. make solo vault for nft
    factory.make_solo_vault(&mut mock_env, cw721_nft.contract_addr.as_str(), token_id);
    let addr = factory.query_solo_vault(&mock_env, cw721_nft.contract_addr.as_str(), token_id);
    assert_eq!(addr.unwrap(), "contract4");
}
