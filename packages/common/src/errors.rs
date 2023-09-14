use cosmwasm_std::{Addr, DivideByZeroError, OverflowError, StdError};
use cw_ownable::OwnershipError;
use cw_utils::ParseReplyError;
use thiserror::Error;

use crate::types::AssetId;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error(transparent)]
    Std(#[from] StdError),

    #[error(transparent)]
    Rbac(#[from] rbac::RbacError),

    #[error(transparent)]
    OwnershipError(#[from] OwnershipError),

    #[error(transparent)]
    ParseReplyError(#[from] ParseReplyError),

    #[error(transparent)]
    Cw721BaseError(#[from] cw721_base::ContractError),

    #[error(transparent)]
    OverflowError(#[from] OverflowError),

    #[error(transparent)]
    DivideByZeroError(#[from] DivideByZeroError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Configuration not found.")]
    ConfigNotFound {},

    #[error("Invalid config parameters")]
    InvalidConfig {},

    #[error("Vault was already created")]
    VaultWasCreated {},

    #[error("Failed to parse or process reply message")]
    FailedToParseReply {},

    #[error("An unknown reply ID was received")]
    UnknownReplyID {},

    #[error("Invalid nft. Got ({received}), expected ({expected})")]
    InvalidNft { received: Addr, expected: Addr },

    #[error("Asset {0} not found")]
    AssetNotFound(AssetId),

    #[error("Asset has expired")]
    Expired {},

    #[error("Existing entitlement must be cleared before registering a new one")]
    HasActiveEntitlement {},

    #[error("The asset cannot be withdrawn with an active entitlement")]
    WithdrawalFailed {},

    #[error("Expiry has already expired")]
    InvalidExpiry {},

    #[error("Beneficial owner must be set to impose an entitlement")]
    BeneficialOwnerNotSet {},

    #[error("Invalid address: {0}")]
    InvalidAddress(String),

    #[error("Invalid chain prefix: {0}")]
    InvalidChainPrefix(String),
}
