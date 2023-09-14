use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Api, StdError, StdResult, Storage, Uint128};
use cw_storage_plus::{Item, Map};
use cw_utils::Expiration;

use common::{
    addr::{assert_valid_addr, PREFIX},
    denom::Denom,
    errors::ContractError,
    types::{AssetId, OptionId},
};
use macros::ConfigStorage;

use crate::msg::InstantiateMsg;

/// Covered call option settings store
const CONFIG: Item<Config> = Item::new("config");

#[cw_serde]
#[derive(ConfigStorage)]
pub struct Config {
    /// The main protocol contract address (which contains configurations)
    pub protocol_addr: Addr,
    /// The address of the token contract permitted to serve as underlying assets for this instrument.
    pub allowed_underlying_nft: Addr,
    /// The address of the vault factory.
    pub factory_addr: Addr,
    /// Default min duration in seconds of an created option
    pub minimum_option_duration: u64,
    /// TODO
    pub allowed_denom: Denom,
    /// Min amount of the current bid that the new bid
    /// must exceed the current bid by in order to be considered valid.
    /// This amount is expressed in basis points (i.e. 1/100th of 1%)
    pub min_bid_inc_bips: Uint128,
}

impl Config {
    pub fn validate(&self, api: &dyn Api) -> Result<(), ContractError> {
        assert_valid_addr(
            api,
            vec![
                self.protocol_addr.as_str(),
                self.allowed_underlying_nft.as_str(),
                self.factory_addr.as_str(),
            ],
            PREFIX,
        )
    }
}

impl From<InstantiateMsg> for Config {
    fn from(value: InstantiateMsg) -> Self {
        Self {
            protocol_addr: Addr::unchecked(value.protocol_addr),
            allowed_underlying_nft: Addr::unchecked(value.allowed_underlying_nft),
            factory_addr: Addr::unchecked(value.factory_addr),
            minimum_option_duration: value.minimum_option_duration,
            allowed_denom: value.allowed_denom.into(),
            min_bid_inc_bips: value.min_bid_inc_bips,
        }
    }
}

/// @param protocol the address of the Hook protocol (which contains configurations)
/// @param nftContract the address for the ERC-721 contract that can serve as underlying instruments
/// @param hookVaultFactory the address of the ERC-721 vault registry
/// @param preApprovedMarketplace the address of the contract which will automatically approved

/// Counter for options
pub const CALL_OPTIONS_COUNT: Item<u64> = Item::new("call_options_count");

/// Storage of all existing options contracts.
pub const CALL_OPTIONS: Map<&OptionId, CallOption> = Map::new("call_options");

/// The metadata for each covered call option stored within the protocol
#[cw_serde]
pub struct CallOption {
    // TODO remove?
    /// The asset id of the underlying within the vault
    pub asset_id: AssetId,
    /// The address of the writer that created the call option
    pub writer_addr: Addr,
    /// The expiration time of the call option
    pub expiration: Expiration,
    /// The address of the vault holding the underlying asset
    pub vault_addr: Addr,
    /// The strike price to exercise the call option
    pub strike: Uint128,
    /// The current high bid in the settlement auction
    pub bid: Uint128,
    /// The address that made the current winning bid in the settlement auction
    pub high_bidder: Option<Addr>,
    // TODO Once this flag is set, ETH should not?
    /// Flag that marks when a settlement action has taken place successfully.
    pub settled: bool,
}

impl CallOption {
    pub fn validate(&self, api: &dyn Api) -> Result<(), ContractError> {
        assert_valid_addr(
            api,
            vec![self.writer_addr.as_str(), self.vault_addr.as_str()],
            PREFIX,
        )
    }

    pub fn load(store: &dyn Storage, option_id: &OptionId) -> StdResult<CallOption> {
        CALL_OPTIONS.load(store, option_id)
    }

    pub fn save(&self, store: &mut dyn Storage, option_id: &OptionId) -> StdResult<()> {
        CALL_OPTIONS.save(store, option_id, self)
    }

    pub fn update(&self, store: &mut dyn Storage, option_id: &OptionId) -> StdResult<()> {
        CALL_OPTIONS.update(store, option_id, |option| match option {
            Some(_) => Ok(self.to_owned()),
            None => Err(StdError::generic_err("updated option not found")),
        })?;
        Ok(())
    }

    pub fn count(storage: &dyn Storage) -> StdResult<u64> {
        Ok(CALL_OPTIONS_COUNT.may_load(storage)?.unwrap_or_default())
    }

    pub fn inc(storage: &mut dyn Storage) -> StdResult<OptionId> {
        let count = Self::count(storage)? + 1;
        CALL_OPTIONS_COUNT.save(storage, &count)?;
        Ok(count)
    }

    pub fn dec(storage: &mut dyn Storage) -> StdResult<OptionId> {
        let count = Self::count(storage)? - 1;
        CALL_OPTIONS_COUNT.save(storage, &count)?;
        Ok(count)
    }
}

/// Mapping to store the amount of eth in wei that may be claimed by the current owner the option nft.
pub const OPTION_CLAIMS: Map<&OptionId, Uint128> = Map::new("option_claims");

/// Flag that can be set to pause this call option contract.
pub const PAUSED: Item<bool> = Item::new("paused");

/// Storage of current call active call option for a specific asset.
/// The call option is is referenced via the option_id.
pub const VAULT_ASSET_OPTION: Map<(&Addr, &AssetId), OptionId> = Map::new("vaultassets2options");

pub fn update_vault_asset_option(
    store: &mut dyn Storage,
    vault_addr: &Addr,
    asset_id: &AssetId,
    option_id: OptionId,
) -> StdResult<OptionId> {
    VAULT_ASSET_OPTION.update(store, (vault_addr, asset_id), |_| Ok(option_id))
}

// TODO use IndexedMap
