#![allow(dead_code)]

use std::mem::take;

use cosmwasm_std::Addr;
use cw_multi_test::{App, BasicApp, Executor};

use crate::mock_contracts::{
    mock_cw721_contract, mock_factory_contract, mock_protocol_contract, mock_vault_contract,
    CallOption, Cw721Nft, Factory, Protocol, Vault,
};

const ADMIN: &str = "admin";
const WRITER: &str = "writer";
const HOLDER: &str = "holder";
const BIDDER: &str = "bidder";

pub struct MockEnv {
    pub app: App,

    // user's addresses
    pub admin: Addr,  // owner for interchain-options and cw721 contracts
    pub writer: Addr, // 'underlying nft' owner and 'option nft' creator
    pub holder: Addr, // current owner of 'option nft'
    pub bidder: Addr, // use who wants to buy 'option nft'

    // contract's addresses
    pub protocol: Protocol,
    pub factory: Factory,
    pub vault: Vault,
    pub call_option: CallOption,
    pub cw721_nft: Cw721Nft,
}

pub struct MockEnvBuilder {
    app: BasicApp,

    admin: Addr,
    writer: Addr,
    holder: Addr,
    bidder: Addr,

    nft_symbol: String,
    nft_name: String,
    protocol_addr: String,
    // chain_pref
    // base_denom: String,
    // base_denom_decimals: u8,
}

impl MockEnvBuilder {
    pub fn new(
        admin: Option<Addr>,
        writer: Option<Addr>,
        holder: Option<Addr>,
        bidder: Option<Addr>,
    ) -> Self {
        Self {
            app: App::default(),
            admin: admin.unwrap_or(Addr::unchecked(ADMIN)),
            writer: writer.unwrap_or(Addr::unchecked(WRITER)),
            holder: holder.unwrap_or(Addr::unchecked(HOLDER)),
            bidder: bidder.unwrap_or(Addr::unchecked(BIDDER)),
            nft_symbol: "NFTSYMBOL".to_string(),
            nft_name: "NFTNAME".to_string(),
            protocol_addr: "protocol_addr".to_string(),
        }
    }

    pub fn build(&mut self) -> MockEnv {
        // deploy underlying NFT contract
        let cw721_nft_addr = self.deploy_cw721_nft();

        // deploy factory contract
        let vault_code_id = self.app.store_code(mock_vault_contract());
        let factory_addr = self.deploy_factory(vault_code_id);
        // TODO update protocol_addr in factory contract

        // deploy protocol contract
        let protocol_addr = self.deploy_protocol(&factory_addr);

        MockEnv {
            app: take(&mut self.app),

            admin: self.admin.clone(),
            writer: self.writer.clone(),
            holder: self.holder.clone(),
            bidder: self.bidder.clone(),

            factory: Factory {
                contract_addr: factory_addr,
            },
            protocol: Protocol {
                contract_addr: protocol_addr,
            },
            vault: Vault {
                contract_addr: Addr::unchecked(""),
            },
            call_option: CallOption {
                contract_addr: Addr::unchecked(""),
            },
            cw721_nft: Cw721Nft {
                contract_addr: cw721_nft_addr,
            },
        }
    }

    fn deploy_cw721_nft(&mut self) -> Addr {
        let code_id = self.app.store_code(mock_cw721_contract());

        self.app
            .instantiate_contract(
                code_id,
                self.admin.clone(),
                &cw721_base::InstantiateMsg {
                    name: self.nft_name.clone(),
                    symbol: self.nft_symbol.clone(),
                    minter: self.admin.clone().into_string(),
                },
                &[],
                "cw721_nft",
                None,
            )
            .unwrap()
    }

    fn deploy_factory(&mut self, vault_code_id: u64) -> Addr {
        let code_id = self.app.store_code(mock_factory_contract());

        self.app
            .instantiate_contract(
                code_id,
                self.admin.clone(),
                &factory::InstantiateMsg {
                    owner: None, // owner is sender
                    protocol_addr: self.protocol_addr.clone(),
                    vault_code_id,
                },
                &[],
                "vault",
                None,
            )
            .unwrap()
    }

    fn deploy_protocol(&mut self, vault_factory_addr: &Addr) -> Addr {
        let code_id = self.app.store_code(mock_protocol_contract());

        self.app
            .instantiate_contract(
                code_id,
                self.admin.clone(),
                &protocol::InstantiateMsg {
                    owner: None, // owner is sender
                    vault_factory_addr: vault_factory_addr.clone().into(),
                    option_factory_addr: vault_factory_addr.into(), // TODO what is option_factory?
                },
                &[],
                "protocol",
                None,
            )
            .unwrap()
    }

    /*
    pub fn chain_prefix(&mut self, prefix: &str) -> &mut Self {
        self.chain_prefix = prefix.to_string();
        self
    }

    pub fn base_denom(&mut self, denom: &str) -> &mut Self {
        self.base_denom = denom.to_string();
        self
    }
    */
}

/*
impl MockEnv {
    pub fn increment_by_blocks(&mut self, num_of_blocks: u64) {
        self.app.update_block(|block| {
            block.height += num_of_blocks;
            // assume block time = 6 sec
            block.time = block.time.plus_seconds(num_of_blocks * 6);
        })
    }

    pub fn increment_by_time(&mut self, seconds: u64) {
        self.app.update_block(|block| {
            block.height += seconds / 6;
            // assume block time = 6 sec
            block.time = block.time.plus_seconds(seconds);
        })
    }

    pub fn fund_account(&mut self, addr: &Addr, coins: &[Coin]) {
        self.app
            .sudo(SudoMsg::Bank(BankSudo::Mint {
                to_address: addr.to_string(),
                amount: coins.to_vec(),
            }))
            .unwrap();
    }

    pub fn query_balance(&self, addr: &Addr, denom: &str) -> StdResult<Coin> {
        self.app.wrap().query_balance(addr, denom)
    }
}
*/
