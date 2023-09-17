use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Addr;

use common::types::TokenId;

#[cw_serde]
pub struct InstantiateMsg {
    /// The main protocol contract address
    pub protocol_addr: String,
    /// Valut contract code identifier
    pub vault_code_id: u64,
    /// Address allowed to change contract parameters
    pub owner: Option<String>,
}

#[cw_serde]
pub enum ExecuteMsg {
    /// Deploy a multi-asset vault if one has not already been deployed.
    MakeMultiVault { nft_addr: String },

    /// Make a new vault that can contain a single asset only.
    MakeSoloVault { nft_addr: String, nft_id: TokenId },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    /// Gets the address of a vault for a particular ERC-721 token.
    #[returns(Option<Addr>)]
    GetVault { nft_addr: String, nft_id: TokenId },

    /// Gets the address of a multi-asset vault for a particular ERC-721 contract, if one exists
    #[returns(Option<Addr>)]
    GetMultiVault { nft_addr: String },

    /// Gets the address of a multi-asset vault if one exists, if no exists gets solo vault for a particular id NFT token.
    #[returns(Option<Addr>)]
    GetMultiOrSoloVault {
        nft_addr: String,
        nft_id: Option<TokenId>,
    },
}
