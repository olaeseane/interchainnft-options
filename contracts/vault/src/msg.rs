use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Api};
use cw_utils::Expiration;

use common::{
    addr::{assert_valid_addr, PREFIX},
    errors::ContractError,
    types::{AssetId, TokenId},
};

#[cw_serde]
pub struct InstantiateMsg {
    pub nft_addr: String,
    pub nft_id: Option<TokenId>,
    pub protocol_addr: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    /// Executed when the contract receives a cw721 token.
    ReceiveNft(cw721::Cw721ReceiveMsg),

    /// Add an entitlement claim to the asset held within the contract.
    ImposeEntitlement {
        asset_id: AssetId,
        operator: String,
        expiry: Expiration,
    },

    /// Allows the beneficial owner to grant an entitlement to an asset within the contract.
    GrantEntitlement { entitlement: Entitlement },

    /// Withdrawal an unencumbered asset from this vault.
    WithdrawalAsset { asset_id: AssetId },

    /// Updates the current address that can claim the asset when it is free of entitlements.
    SetBeneficialOwner {
        asset_id: AssetId,
        new_beneficial_owner: String,
    },

    /// Allows the entitled address to release their claim on the asset.
    ClearEntitlement { asset_id: AssetId },

    /// Allows the entitled address to release their claim on the asset.
    ClearEntitlementAndDistribute { asset_id: AssetId, receiver: String },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    /// Looks up the current beneficial owner of the asset.
    #[returns(Option<Addr>)]
    BeneficialOwner { asset_id: AssetId },

    /// Checks if the asset is currently stored in the vault.
    #[returns(bool)]
    HoldsAsset { asset_id: AssetId },

    /// Returns the contract address of the vaulted asset.
    #[returns(Addr)]
    AssetAddress {},

    #[returns(Option<Addr>)]
    ApprovedOperator { asset_id: AssetId },

    /// Looks up the expiration timestamp of the current entitlement.
    #[returns(Option<Expiration>)]
    EntitlementExpiration { asset_id: AssetId },

    /// Looks up the current operator of an entitlement on an asset.
    #[returns(CurrentEntitlementOperatorResponse)]
    CurrentEntitlementOperator { asset_id: AssetId },
}

#[cw_serde]
pub struct CurrentEntitlementOperatorResponse {
    pub is_active: bool,
    pub operator: Option<Addr>,
}

#[cw_serde]
pub struct VaultInstantiateData {
    pub nft_addr: Addr,
    pub nft_id: Option<TokenId>,
}

#[cw_serde]
#[derive(Default)]
pub struct Entitlement {
    /// The beneficial owner address this entitlement applies to. This address will also be the signer.
    pub beneficial_owner: String,
    /// The operating contract that can change ownership during the entitlement period.
    pub operator: String,
    /// The contract address for the vault that contains the underlying assets.
    pub vault_address: String,
    /// The assetId of the asset or assets within the vault.
    pub asset_id: AssetId,
    /// The block timestamp after which the asset is free of the entitlement.
    pub expiry: Expiration,
}

impl Entitlement {
    pub fn validate(&self, api: &dyn Api) -> Result<(), ContractError> {
        assert_valid_addr(api, vec![&self.beneficial_owner, &self.operator], PREFIX)
    }
}
