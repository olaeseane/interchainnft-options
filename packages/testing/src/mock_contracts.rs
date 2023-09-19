use cosmwasm_std::{to_binary, Addr, Coin, Empty, Uint128};
use cw_multi_test::{App, Contract, ContractWrapper, Executor};
use cw_utils::Expiration;

use call::QueryMsg as CallInstrumentQueryMsg;
use common::types::{AssetId, OptionId, TokenId};
use cw721_base::QueryMsg::Extension as Cw721Extension;
use vault::msg::{CurrentEntitlementOperatorResponse, SetEntitlement};

use crate::mock_env::MockEnv;

type Cw721BaseExecuteMsg = cw721_base::ExecuteMsg<Empty, Empty>;
type Cw721BaseQueryMsg = cw721_base::QueryMsg<Empty>;

pub fn mock_app() -> App {
    App::default()
}

/*
    Protocol contract mock
*/

#[derive(Clone, Debug)]
pub struct Protocol {
    pub contract_addr: Addr,
}

impl Protocol {
    pub fn mock_contract() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(
            protocol::contract::execute,
            protocol::contract::instantiate,
            protocol::contract::query,
        );
        Box::new(contract)
    }

    pub fn set_vault_factory(&self, env: &mut MockEnv, contract_addr: String) {
        env.app
            .execute_contract(
                env.admin.clone(),
                self.contract_addr.clone(),
                &protocol::ExecuteMsg::SetVaultFactory { contract_addr },
                &[],
            )
            .unwrap();
    }

    pub fn set_call_factory(&self, env: &mut MockEnv, contract_addr: String) {
        env.app
            .execute_contract(
                env.admin.clone(),
                self.contract_addr.clone(),
                &protocol::ExecuteMsg::SetCallFactory { contract_addr },
                &[],
            )
            .unwrap();
    }
}

/*
    Vault factory contract mock
*/

#[derive(Clone, Debug)]
pub struct VaultFactory {
    pub contract_addr: Addr,
}

impl VaultFactory {
    pub fn mock_contract() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(
            vault_factory::contract::execute,
            vault_factory::contract::instantiate,
            vault_factory::contract::query,
        )
        .with_reply(vault_factory::contract::reply);
        Box::new(contract)
    }

    pub fn make_multi_vault(&self, env: &mut MockEnv, nft_addr: &str) -> String {
        let resp = env
            .app
            .execute_contract(
                env.admin.clone(),
                self.contract_addr.clone(),
                &vault_factory::ExecuteMsg::MakeMultiVault {
                    nft_addr: nft_addr.into(),
                },
                &[],
            )
            .unwrap();

        // find vault_addr in events
        resp.events[5]
            .attributes
            .iter()
            .find(|attr| attr.key == "vault_addr")
            .unwrap()
            .value
            .clone()
    }

    #[track_caller]
    pub fn make_solo_vault(&self, env: &mut MockEnv, nft_addr: &str, nft_id: &str) {
        env.app
            .execute_contract(
                env.admin.clone(),
                self.contract_addr.clone(),
                &vault_factory::ExecuteMsg::MakeSoloVault {
                    nft_addr: nft_addr.into(),
                    nft_id: nft_id.into(),
                },
                &[],
            )
            .unwrap();
    }

    pub fn query_multi_or_solo_vault(
        &self,
        env: &MockEnv,
        nft_addr: &str,
        nft_id: Option<&str>,
    ) -> Option<Addr> {
        env.app
            .wrap()
            .query_wasm_smart(
                &self.contract_addr,
                &vault_factory::QueryMsg::GetMultiOrSoloVault {
                    nft_addr: nft_addr.into(),
                    nft_id: nft_id.map(|n| n.to_string()),
                },
            )
            .unwrap()
    }

    pub fn query_solo_vault(&self, env: &MockEnv, nft_addr: &str, nft_id: &str) -> Option<Addr> {
        env.app
            .wrap()
            .query_wasm_smart(
                &self.contract_addr,
                &vault_factory::QueryMsg::GetVault {
                    nft_addr: nft_addr.into(),
                    nft_id: nft_id.to_string(),
                },
            )
            .unwrap()
    }
}

/*
    Vaut contract mock
*/

#[derive(Clone, Debug)]
pub struct Vault {
    pub contract_addr: Addr,
}

impl Vault {
    pub fn mock_contract() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(
            vault::contract::execute,
            vault::contract::instantiate,
            vault::contract::query,
        );
        Box::new(contract)
    }

    pub fn query_beneficial_owner(&self, env: &MockEnv, asset_id: &str) -> Option<Addr> {
        env.app
            .wrap()
            .query_wasm_smart(
                &self.contract_addr,
                &vault::QueryMsg::BeneficialOwner {
                    asset_id: asset_id.into(),
                },
            )
            .unwrap()
    }

    pub fn query_holds_asset(&self, env: &MockEnv, asset_id: &str) -> bool {
        env.app
            .wrap()
            .query_wasm_smart(
                &self.contract_addr,
                &vault::QueryMsg::HoldsAsset {
                    asset_id: asset_id.into(),
                },
            )
            .unwrap()
    }

    pub fn query_asset_address(&self, env: &MockEnv) -> Addr {
        env.app
            .wrap()
            .query_wasm_smart(&self.contract_addr, &vault::QueryMsg::AssetAddress {})
            .unwrap()
    }

    pub fn query_approved_operator(&self, env: &MockEnv, asset_id: &str) -> Option<Addr> {
        env.app
            .wrap()
            .query_wasm_smart(
                &self.contract_addr,
                &vault::QueryMsg::ApprovedOperator {
                    asset_id: asset_id.into(),
                },
            )
            .unwrap()
    }

    pub fn query_entitlement_expiration(
        &self,
        env: &MockEnv,
        asset_id: &str,
    ) -> Option<Expiration> {
        env.app
            .wrap()
            .query_wasm_smart(
                &self.contract_addr,
                &vault::QueryMsg::EntitlementExpiration {
                    asset_id: asset_id.into(),
                },
            )
            .unwrap()
    }

    pub fn query_current_entitlement_operator(
        &self,
        env: &MockEnv,
        asset_id: &str,
    ) -> CurrentEntitlementOperatorResponse {
        env.app
            .wrap()
            .query_wasm_smart(
                &self.contract_addr,
                &vault::QueryMsg::CurrentEntitlementOperator {
                    asset_id: asset_id.into(),
                },
            )
            .unwrap()
    }
}

/*
    Call factory contract mock
*/

#[derive(Clone, Debug)]
pub struct CallFactory {
    pub contract_addr: Addr,
}

impl CallFactory {
    pub fn mock_contract() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(
            call_factory::contract::execute,
            call_factory::contract::instantiate,
            call_factory::contract::query,
        )
        .with_reply(call_factory::contract::reply);
        Box::new(contract)
    }

    pub fn make_call_instrument(&self, env: &mut MockEnv, nft_addr: &str) {
        env.app
            .execute_contract(
                env.admin.clone(),
                self.contract_addr.clone(),
                &call_factory::ExecuteMsg::MakeCallInstrument {
                    nft_addr: nft_addr.into(),
                },
                &[],
            )
            .unwrap();
    }

    pub fn query_call_instrument(&self, env: &MockEnv, nft_addr: &str) -> Option<Addr> {
        env.app
            .wrap()
            .query_wasm_smart(
                &self.contract_addr,
                &call_factory::QueryMsg::GetCallInstrument {
                    nft_addr: nft_addr.into(),
                },
            )
            .unwrap()
    }
}

/*
    Call instrument contract mock
*/

#[derive(Clone, Debug)]
pub struct CallInstrument {
    pub contract_addr: Addr,
}

impl CallInstrument {
    pub fn mock_contract() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(
            call::contract::execute,
            call::contract::instantiate,
            call::contract::query,
        );
        Box::new(contract)
    }

    pub fn mint_with_nft(
        &self,
        env: &mut MockEnv,
        nft_addr: &str,
        nft_id: TokenId,
        strike: Uint128,
        expiration: Expiration,
    ) -> OptionId {
        let resp = env
            .app
            .execute_contract(
                env.writer.clone(),
                self.contract_addr.clone(),
                &cw721_base::ExecuteMsg::<Empty, call::ExecuteMsg>::Extension {
                    msg: call::ExecuteMsg::MintWithNFT {
                        nft_addr: nft_addr.to_owned(),
                        nft_id,
                        strike,
                        expiration,
                    },
                },
                &[],
            )
            .unwrap();

        // find option_id in events
        resp.events[1]
            .attributes
            .iter()
            .find(|attr| attr.key == "option_id")
            .unwrap()
            .value
            .parse::<u64>()
            .unwrap()
    }

    pub fn bid(&self, env: &mut MockEnv, option_id: OptionId, coin: Coin) {
        env.app
            .execute_contract(
                env.bidder.clone(),
                self.contract_addr.clone(),
                &cw721_base::ExecuteMsg::<Empty, call::ExecuteMsg>::Extension {
                    msg: call::ExecuteMsg::Bid { option_id },
                },
                &[coin],
            )
            .unwrap();
    }

    pub fn query_current_bid(&self, env: &MockEnv, option_id: OptionId) -> Uint128 {
        env.app
            .wrap()
            .query_wasm_smart(
                &self.contract_addr,
                &Cw721Extension {
                    msg: CallInstrumentQueryMsg::CurrentBid { option_id },
                },
            )
            .unwrap()
    }

    pub fn query_current_bidder(&self, env: &MockEnv, option_id: OptionId) -> Option<Addr> {
        env.app
            .wrap()
            .query_wasm_smart(
                &self.contract_addr,
                &Cw721Extension {
                    msg: CallInstrumentQueryMsg::CurrentBidder { option_id },
                },
            )
            .unwrap()
    }

    pub fn query_get_vault_address(&self, env: &MockEnv, option_id: OptionId) -> Addr {
        env.app
            .wrap()
            .query_wasm_smart(
                &self.contract_addr,
                &Cw721Extension {
                    msg: CallInstrumentQueryMsg::GetVaultAddress { option_id },
                },
            )
            .unwrap()
    }

    pub fn query_get_asset_id(&self, env: &MockEnv, option_id: OptionId) -> AssetId {
        env.app
            .wrap()
            .query_wasm_smart(
                &self.contract_addr,
                &Cw721Extension {
                    msg: CallInstrumentQueryMsg::GetAssetId { option_id },
                },
            )
            .unwrap()
    }

    pub fn query_get_strike_price(&self, env: &MockEnv, option_id: OptionId) -> Uint128 {
        env.app
            .wrap()
            .query_wasm_smart(
                &self.contract_addr,
                &Cw721Extension {
                    msg: CallInstrumentQueryMsg::GetStrikePrice { option_id },
                },
            )
            .unwrap()
    }

    pub fn query_get_expiration(&self, env: &MockEnv, option_id: OptionId) -> Expiration {
        env.app
            .wrap()
            .query_wasm_smart(
                &self.contract_addr,
                &Cw721Extension {
                    msg: CallInstrumentQueryMsg::GetExpiration { option_id },
                },
            )
            .unwrap()
    }

    pub fn query_get_option_id_for_asset(
        &self,
        env: &MockEnv,
        vault: &str,
        asset_id: &AssetId,
    ) -> OptionId {
        env.app
            .wrap()
            .query_wasm_smart(
                &self.contract_addr,
                &Cw721Extension {
                    msg: CallInstrumentQueryMsg::GetOptionIdForAsset {
                        vault: vault.to_string(),
                        asset_id: asset_id.to_string(),
                    },
                },
            )
            .unwrap()
    }
}

// Cw721 NFT contract mock

#[derive(Clone, Debug)]
pub struct Cw721Nft {
    pub contract_addr: Addr,
}

impl Cw721Nft {
    pub fn mock_contract() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(
            cw721_base::entry::execute,
            cw721_base::entry::instantiate,
            cw721_base::entry::query,
        );
        Box::new(contract)
    }

    pub fn mint(&self, env: &mut MockEnv, token_id: &str, token_owner: &str) {
        env.app
            .execute_contract(
                env.admin.clone(),
                self.contract_addr.clone(),
                &Cw721BaseExecuteMsg::Mint {
                    token_id: token_id.to_string(),
                    owner: token_owner.to_string(),
                    token_uri: None,
                    extension: Empty::default(),
                },
                &[],
            )
            .unwrap();
    }

    pub fn send(
        &self,
        env: &mut MockEnv,
        sender: &str,
        token_id: &str,
        vault: &str,
        entitlement: Option<SetEntitlement>,
    ) {
        env.app
            .execute_contract(
                Addr::unchecked(sender),
                self.contract_addr.clone(),
                &Cw721BaseExecuteMsg::SendNft {
                    contract: vault.to_string(),
                    token_id: token_id.to_string(),
                    msg: to_binary(&entitlement).unwrap(),
                },
                &[],
            )
            .unwrap();
    }

    pub fn approve(&self, env: &mut MockEnv, owner: &str, spender: &str, token_id: &str) {
        env.app
            .execute_contract(
                Addr::unchecked(owner),
                self.contract_addr.clone(),
                &Cw721BaseExecuteMsg::Approve {
                    spender: spender.to_string(),
                    token_id: token_id.to_string(),
                    expires: None,
                },
                &[],
            )
            .unwrap();
    }

    pub fn query_owner(&self, env: &MockEnv, token_id: &str) -> String {
        env.app
            .wrap()
            .query_wasm_smart::<cw721::OwnerOfResponse>(
                &self.contract_addr,
                &Cw721BaseQueryMsg::OwnerOf {
                    token_id: token_id.to_string(),
                    include_expired: None,
                },
            )
            .unwrap()
            .owner
    }
}
