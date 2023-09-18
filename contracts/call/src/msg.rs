use common::types::{AssetId, OptionId, TokenId};
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, CustomMsg, Uint128};
use cw_utils::Expiration;

#[cw_serde]
pub struct InstantiateMsg {
    /// TODO
    pub name: String,
    /// TODO
    pub symbol: String,
    /// The main protocol contract address
    pub protocol_addr: String,
    /// The address of the token contract permitted to serve as underlying assets for this instrument.
    pub allowed_underlying_nft: String,
    /// The address of the vault factory.
    pub vault_factory_addr: String,
    /// Default min duration of an created option
    pub minimum_option_duration: u64,
    /// TODO
    pub allowed_denom: String,
    /// TODO
    pub min_bid_inc_bips: Uint128,
}

#[cw_serde]
pub enum CallInstrumentExecuteMsg {
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
    Bid {
        option_id: OptionId,
    },

    /// Allows the writer to reclaim an entitled asset. This is only
    /// possible when the writer holds the option nft and calls this function.
    ReclaimAsset {
        option_id: OptionId,
        withdraw: bool,
    },

    /// Permissionlessly settle an expired option when the option expires in the money,
    /// distributing the proceeds to the Writer, Holder, and Bidder.
    SettleOption {
        option_id: OptionId,
    },

    /// Allows anyone to burn the instrument NFT for an expired option.
    BurnExpiredOption {
        option_id: OptionId,
    },

    /// Allows the option owner to claim proceeds if the option was settled
    /// by another account. The option NFT is burned after settlement.
    ClaimOptionProceeds {
        option_id: OptionId,
    },

    ForTest {}, // TODO delete
}

impl CustomMsg for CallInstrumentExecuteMsg {}

#[cw_serde]
#[derive(QueryResponses)]
pub enum CallInstrumentQueryMsg {
    /// Gets the current high settlement bid of an option, or None if there is no high bid
    #[returns(Uint128)]
    CurrentBid { option_id: OptionId },

    // Gets the current high bidder for an option settlement auction, or the None if no
    #[returns(Option<Addr>)]
    CurrentBidder { option_id: OptionId },

    #[returns(Addr)]
    GetVaultAddress { option_id: OptionId },

    /// Looks up the latest optionId that covers a particular asset, if one exists.
    /// This option may be already settled.
    #[returns(u64)]
    GetOptionIdForAsset { vault: String, asset_id: AssetId },

    #[returns(String)]
    GetAssetId { option_id: OptionId },

    #[returns(Uint128)]
    GetStrikePrice { option_id: OptionId },

    #[returns(Expiration)]
    GetExpiration { option_id: OptionId },
}

impl CustomMsg for CallInstrumentQueryMsg {}
