use cosmwasm_std::{to_binary, Addr, Empty};
use cw_multi_test::{App, Contract, ContractWrapper, Executor};
use vault::msg::Entitlement;

use crate::mock_env::MockEnv;

pub fn mock_app() -> App {
    App::default()
}

// Protocol contract mock

#[derive(Clone)]
pub struct Protocol {
    pub contract_addr: Addr,
}

impl Protocol {
    pub fn set_vault_factory(&self, env: &mut MockEnv, vault_factory_addr: String) {
        env.app
            .execute_contract(
                env.admin.clone(),
                self.contract_addr.clone(),
                &protocol::ExecuteMsg::SetVaultFactory {
                    contract_addr: vault_factory_addr,
                },
                &[],
            )
            .unwrap();
    }
}

pub fn mock_protocol_contract() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        protocol::contract::execute,
        protocol::contract::instantiate,
        protocol::contract::query,
    );
    Box::new(contract)
}

// Factory contract mock

#[derive(Clone)]
pub struct Factory {
    pub contract_addr: Addr,
}

impl Factory {
    pub fn make_multi_vault(&self, env: &mut MockEnv, nft_addr: &str) {
        env.app
            .execute_contract(
                env.admin.clone(),
                self.contract_addr.clone(),
                &factory::ExecuteMsg::MakeMultiVault {
                    nft_addr: nft_addr.into(),
                },
                &[],
            )
            .unwrap();
    }

    #[track_caller]
    pub fn make_solo_vault(&self, env: &mut MockEnv, nft_addr: &str, nft_id: &str) {
        env.app
            .execute_contract(
                env.admin.clone(),
                self.contract_addr.clone(),
                &factory::ExecuteMsg::MakeSoloVault {
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
                self.contract_addr.clone(),
                &factory::QueryMsg::GetMultiOrSoloVault {
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
                self.contract_addr.clone(),
                &factory::QueryMsg::GetVault {
                    nft_addr: nft_addr.into(),
                    nft_id: nft_id.to_string(),
                },
            )
            .unwrap()
    }
}

pub fn mock_factory_contract() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        factory::contract::execute,
        factory::contract::instantiate,
        factory::contract::query,
    )
    .with_reply(factory::contract::reply);
    Box::new(contract)
}

// Vaut contract mock

#[derive(Clone)]
pub struct Vault {
    pub contract_addr: Addr,
}

pub fn mock_vault_contract() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        vault::contract::execute,
        vault::contract::instantiate,
        vault::contract::query,
    );
    Box::new(contract)
}

// CallOption contract mock

#[derive(Clone)]
pub struct CallOption {
    pub contract_addr: Addr,
}

// Cw721 NFT contract mock

#[derive(Clone)]
pub struct Cw721Nft {
    pub contract_addr: Addr,
}

impl Cw721Nft {
    pub fn mint(&self, env: &mut MockEnv, token_id: &str, token_owner: &str) {
        env.app
            .execute_contract(
                env.admin.clone(),
                self.contract_addr.clone(),
                &cw721_base::ExecuteMsg::<Empty, Empty>::Mint {
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
        entitlement: Entitlement,
    ) {
        env.app
            .execute_contract(
                Addr::unchecked(sender),
                self.contract_addr.clone(),
                &cw721_base::ExecuteMsg::<Empty, Empty>::SendNft {
                    contract: vault.to_string(),
                    token_id: token_id.to_string(),
                    msg: to_binary(&entitlement).unwrap(),
                },
                &[],
            )
            .unwrap();
    }
}

pub fn mock_cw721_contract() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        cw721_base::entry::execute,
        cw721_base::entry::instantiate,
        cw721_base::entry::query,
    );
    Box::new(contract)
}
