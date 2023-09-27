use cosmwasm_schema::write_api;

use call_factory::*;

fn main() {
    write_api! {
        instantiate: InstantiateMsg,
        execute: ExecuteMsg,
        query: QueryMsg
    }
}
