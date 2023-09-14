use cosmwasm_schema::cw_serde;

// TODO what type to use for identificators?
pub type TokenId = String;
pub type AssetId = String;
pub type OptionId = u64;

// #[cw_serde]
// pub struct OptionId(pub Uint256);

pub enum OptionType {
    CALL,
    PUT,
}

#[cw_serde]
pub struct ExpiryRange {
    pub min: u64,
    pub max: u64,
}
