use testing::mock_env::MockEnvBuilder;
use vault::msg::Entitlement;

#[test]
fn mint_with_cw721_flow() {
    env_logger::init();

    let token_id = "id001";
    let token_owner = "token_owner_addr";

    // 1. deploy and instantiate vault factory and underlying nft
    let mut mock_env = MockEnvBuilder::new(None, None, None, None).build();
    let cw721_nft = mock_env.cw721_nft.clone();
    let factory = mock_env.factory.clone();

    // 2. mint nft contract
    cw721_nft.mint(&mut mock_env, token_id, token_owner);

    // 3. make multi vault for nft
    factory.make_multi_vault(&mut mock_env, cw721_nft.contract_addr.as_str());
    let vault_addr = factory
        .query_multi_or_solo_vault(&mock_env, cw721_nft.contract_addr.as_str(), None)
        .unwrap();
    assert_eq!(vault_addr, "contract3");

    // 4. prep entitlement
    let entitlement = Entitlement::default();

    // 4.send nft to the vault
    cw721_nft.send(
        &mut mock_env,
        token_owner,
        token_id,
        vault_addr.as_str(),
        entitlement,
    );
}
