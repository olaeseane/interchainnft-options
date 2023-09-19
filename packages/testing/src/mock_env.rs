#![allow(dead_code)]

use std::mem::take;

use cosmwasm_std::{Addr, BlockInfo, Coin, StdResult, Uint128};
use cw_multi_test::{App, BankSudo, BasicApp, Executor, SudoMsg};

use crate::mock_contracts::{CallFactory, CallInstrument, Cw721Nft, Protocol, Vault, VaultFactory};

pub const WRITER: &str = "writer";
pub const ADMIN: &str = "admin";
pub const HOLDER: &str = "holder";
pub const BIDDER: &str = "bidder";

const UNDERLYING_NFT_SYMBOL: &str = "BAYC";
const UNDERLYING_NFT_NAME: &str = "BORED APE YACHT CLUB";

const CALL_OPTION_NFT_SYMBOL: &str = "CALOPT";
const CALL_OPTION_NFT_NAME: &str = "call option nft";

const MINIMUM_OPTION_DURATION: u64 = 1;
const ALLOWED_DENOM: &str = "ATOM";
const MIN_BID_INC_BIPS: Uint128 = Uint128::new(1);

pub struct MockEnv {
    pub app: App,

    // user's addresses
    pub admin: Addr,  // owner for interchain-options and cw721 contracts
    pub writer: Addr, // 'underlying nft' owner and 'option nft' creator
    pub holder: Addr, // current owner of 'option nft'
    pub bidder: Addr, // use who wants to buy 'option nft'

    // contract's addresses
    pub underlying_nft: Cw721Nft,
    pub protocol: Protocol,
    pub vault_factory: VaultFactory,
    pub call_factory: CallFactory,
    // pub call_instrument: CallInstrument,
}

impl MockEnv {
    pub fn increment_by_time(&mut self, seconds: u64) {
        self.app.update_block(|block| {
            block.height += seconds / 6; // assume block time = 6 sec
            block.time = block.time.plus_seconds(seconds);
        })
    }

    pub fn query_block_info(&self) -> BlockInfo {
        self.app.block_info()
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

    pub fn print_env(&self) {
        println!("underlying_nft: {:?}", self.underlying_nft);
        println!("protocol: {:?}", self.protocol);
        println!("vault_factory: {:?}", self.vault_factory);
        // println!("vault: {:?}", self.vault);
        println!("call_factory: {:?}", self.call_factory);
        // println!("call: {:?}", self.call);
    }
}

pub struct MockEnvBuilder {
    app: BasicApp,

    admin: Addr,
    writer: Addr,
    holder: Addr,
    bidder: Addr,

    nft_symbol: String,
    nft_name: String,
    // protocol_addr: String,
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
            nft_symbol: UNDERLYING_NFT_SYMBOL.to_string(),
            nft_name: UNDERLYING_NFT_NAME.to_string(),
        }
    }

    pub fn build(&mut self) -> MockEnv {
        // deploy underlying NFT contract - contract0
        let underlying_nft_addr = self.deploy_cw721_nft();

        // first of all, deploy protocol contract - contract1
        let protocol_addr = self.deploy_protocol();

        // deploy vault factory contract - contract2
        let vault_code_id = self.app.store_code(Vault::mock_contract());
        let vault_factory_addr = self.deploy_vault_factory(vault_code_id, &protocol_addr);

        // deploy call factory contract - contract3
        let call_code_id = self.app.store_code(CallInstrument::mock_contract());
        let call_factory_addr =
            self.deploy_call_factory(call_code_id, &protocol_addr, &vault_factory_addr);

        // update factories addresses in protocol contract
        self.set_protocol_factory(
            &protocol_addr,
            protocol::ExecuteMsg::SetCallFactory {
                contract_addr: call_factory_addr.to_string(),
            },
        );
        self.set_protocol_factory(
            &protocol_addr,
            protocol::ExecuteMsg::SetVaultFactory {
                contract_addr: vault_factory_addr.to_string(),
            },
        );

        MockEnv {
            app: take(&mut self.app),

            admin: self.admin.clone(),
            writer: self.writer.clone(),
            holder: self.holder.clone(),
            bidder: self.bidder.clone(),

            underlying_nft: Cw721Nft {
                contract_addr: underlying_nft_addr,
            },
            protocol: Protocol {
                contract_addr: protocol_addr,
            },
            vault_factory: VaultFactory {
                contract_addr: vault_factory_addr,
            },
            call_factory: CallFactory {
                contract_addr: call_factory_addr,
            },
        }
    }

    fn deploy_cw721_nft(&mut self) -> Addr {
        let code_id = self.app.store_code(Cw721Nft::mock_contract());

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

    fn deploy_protocol(&mut self) -> Addr {
        let code_id = self.app.store_code(Protocol::mock_contract());

        self.app
            .instantiate_contract(
                code_id,
                self.admin.clone(),
                &protocol::InstantiateMsg {
                    owner: None, // owner is sender
                },
                &[],
                "protocol",
                None,
            )
            .unwrap()
    }

    fn deploy_vault_factory(&mut self, vault_code_id: u64, protocol_addr: &Addr) -> Addr {
        let code_id = self.app.store_code(VaultFactory::mock_contract());

        self.app
            .instantiate_contract(
                code_id,
                self.admin.clone(),
                &vault_factory::InstantiateMsg {
                    owner: None, // owner is sender
                    protocol_addr: protocol_addr.to_string(),
                    vault_code_id,
                },
                &[],
                "vault-factory",
                None,
            )
            .unwrap()
    }

    #[track_caller]
    fn deploy_call_factory(
        &mut self,
        call_code_id: u64,
        protocol_addr: &Addr,
        vault_factory_addr: &Addr,
    ) -> Addr {
        let code_id = self.app.store_code(CallFactory::mock_contract());

        self.app
            .instantiate_contract(
                code_id,
                self.admin.clone(),
                &call_factory::InstantiateMsg {
                    protocol_addr: protocol_addr.to_string(),
                    call_code_id,
                    owner: None, // owner is sender
                    nft_symbol: CALL_OPTION_NFT_SYMBOL.to_string(),
                    nft_name: CALL_OPTION_NFT_NAME.to_string(),
                    default_minimum_option_duration: MINIMUM_OPTION_DURATION,
                    default_allowed_denom: ALLOWED_DENOM.to_string(),
                    default_min_bid_inc_bips: MIN_BID_INC_BIPS,
                    vault_factory_addr: vault_factory_addr.to_string(),
                },
                &[],
                "call-factory",
                None,
            )
            .unwrap()
    }

    pub fn set_protocol_factory<T>(&mut self, protocol_addr: &Addr, msg: T)
    where
        T: Into<protocol::ExecuteMsg>,
    {
        self.app
            .execute_contract(self.admin.clone(), protocol_addr.clone(), &msg.into(), &[])
            .unwrap();
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

}
*/
