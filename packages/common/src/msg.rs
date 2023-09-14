use cosmwasm_std::{BankMsg, Coin, CosmosMsg};

pub fn bank_send_msg<T>(to_address: String, amount: Vec<Coin>) -> CosmosMsg<T> {
    CosmosMsg::Bank(BankMsg::Send { to_address, amount })
}
