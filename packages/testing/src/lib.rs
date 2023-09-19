use cosmwasm_std::Uint128;
use mock_env::WRITER;

pub mod helpers;
pub mod mock_contracts;
pub mod mock_env;

#[derive(Debug)]
pub struct Params<'a> {
    pub token_id: &'a str,
    pub token_owner: &'a str,
    pub strike: Uint128,
    pub expired_after: u64, // in days
    pub denom: &'a str,
}

impl<'a> Default for Params<'a> {
    fn default() -> Self {
        Self {
            token_id: "id001",
            token_owner: WRITER,
            strike: Uint128::new(5),
            expired_after: 5,
            denom: "ATOM",
        }
    }
}
