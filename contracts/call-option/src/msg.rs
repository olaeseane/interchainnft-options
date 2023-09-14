use common::types::{AssetId, OptionId, TokenId};
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{CustomMsg, Uint128};
use cw_utils::Expiration;

#[cw_serde]
pub struct InstantiateMsg {
    /// TODO
    pub name: String,
    /// TODO
    pub symbol: String,
    /// The main protocol contract address
    pub protocol_addr: String,
    /// Address allowed to change contract parameters
    pub owner: Option<String>,
    /// The address of the token contract permitted to serve as underlying assets for this instrument.
    pub allowed_underlying_nft: String,
    /// The address of the vault factory.
    pub factory_addr: String,
    /// Default min duration of an created option
    pub minimum_option_duration: u64,
    /// TODO
    pub allowed_denom: String,
    /// TODO
    pub min_bid_inc_bips: Uint128,
}

#[cw_serde]
pub enum CallOptionExecuteMsg {
    /// Mints a new call option for a particular "underlying" NFT with a given strike price and expiration.
    MintWithNFT {
        nft_addr: String,
        nft_id: TokenId,
        strike: Uint128,
        expiration: Expiration,
    },

    /// Mints a new call option for the assets deposited in a particular vault given strike price and expiration.
    MintWithVault {
        vault_addr: String,
        asset_id: AssetId,
        strike: Uint128,
        expiration: Expiration,
    },

    /// Mints a new call option for the assets deposited in a particular vault given strike price and expiration.
    /// That vault must already have a registered entitlement for this contract with the an expiration equal to expiration.
    MintWithEntitledVault {
        vault_addr: String,
        asset_id: AssetId,
        strike: Uint128,
        expiration: Expiration,
    },

    /// Bid in the settlement auction for an option. The paid amount is the bid, and the bidder
    /// is required to escrow this amount until either the auction ends or another bidder bids higher
    Bid { option_id: OptionId },

    /// Allows the writer to reclaim an entitled asset. This is only
    /// possible when the writer holds the option nft and calls this function.
    ReclaimAsset { option_id: OptionId, withdraw: bool },

    /// Permissionlessly settle an expired option when the option expires in the money,
    /// distributing the proceeds to the Writer, Holder, and Bidder.
    SettleOption { option_id: OptionId },

    /// Allows anyone to burn the instrument NFT for an expired option.
    BurnExpiredOption { option_id: OptionId },

    /// Allows the option owner to claim proceeds if the option was settled
    /// by another account. The option NFT is burned after settlement.
    ClaimOptionProceeds { option_id: OptionId },
}

impl CustomMsg for CallOptionExecuteMsg {}

#[cw_serde]
pub enum CallOptionQueryMsg {
    /// Gets the current high settlement bid of an option, or None if there is no high bid
    CurrentBid {
        option_id: OptionId,
    },

    // Gets the current high bidder for an option settlement auction, or the None if no
    CurrentBidder {
        option_id: OptionId,
    },

    GetVaultAddress {
        option_id: OptionId,
    },

    /// Looks up the latest optionId that covers a particular asset, if one exists.
    /// This option may be already settled.
    GetOptionIdForAsset {
        vault: String,
        asset_id: AssetId,
    },

    GetAssetId {
        option_id: OptionId,
    },

    GetStrikePrice {
        option_id: OptionId,
    },

    GetExpiration {
        option_id: OptionId,
    },
}

impl CustomMsg for CallOptionQueryMsg {}
