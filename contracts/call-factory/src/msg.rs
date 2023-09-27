use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Addr;

#[cw_serde]
pub struct InstantiateMsg {
    /// The main protocol contract address
    pub protocol_addr: String,
    /// Call option contract code identifier
    pub call_code_id: u64,
    /// Address allowed to change contract parameters
    pub owner: Option<String>,
    /// Symbol of call option nft
    pub nft_symbol: String,
    /// Name of call option nft
    pub nft_name: String,
    /// Default min duration of an created option
    pub default_minimum_option_duration: u64,
    /// TODO
    pub default_allowed_denom: String,
    /// TODO
    pub default_min_bid_increment_bps: u64,
    /// The address of the vault factory
    pub vault_factory_addr: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    ///  Create a call option instrument for a specific underlying asset address
    MakeCallInstrument { nft_addr: String },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    /// Lookup the call instrument contract based on the asset address
    #[returns(Option<Addr>)]
    GetCallInstrument { nft_addr: String },
}
