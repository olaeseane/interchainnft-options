use cosmwasm_schema::{cw_serde, QueryResponses};
use cw_utils::Duration;

#[allow(unused_imports)]
use crate::state::Config;

#[cw_serde]
pub struct InstantiateMsg {
    /// Address allowed to change contract parameters
    pub owner: Option<String>,
}

/// This structure describes the execute messages of the contract.
#[cw_serde]
pub enum ExecuteMsg {
    /// Pauses the protocol contracts for a set duration.
    /// When paused the protocol contracts is unable to execute messages.
    Pause { duration: Duration },

    /// Allows an admin to set the address of the deployed vault factory.
    /// All protocol components, including the call factory, to look up the vault factory.
    SetVaultFactory { contract_addr: String },

    /// Allows an admin to set the address of the deployed covered call factory
    /// This address is used by other protocols searching for the registry of protocols.
    SetCallFactory { contract_addr: String },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    /// Config returns contract settings.
    #[returns(Config)]
    Config {},
}
