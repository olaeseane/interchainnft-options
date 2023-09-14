use std::marker::PhantomData;

use cosmwasm_std::{
    to_binary, Addr, Empty, MessageInfo, QuerierWrapper, StdError, StdResult, SubMsg, WasmMsg,
};
use cw721::{Approval, ApprovalResponse, ApprovalsResponse, Cw721ExecuteMsg, OwnerOfResponse};
use cw721_base::helpers::Cw721Contract;

/// Invoke `transfer_nft` to build a `SubMsg` to transfer an NFT to an address.
pub fn transfer_nft(collection: &Addr, token_id: &str, recipient: &Addr) -> StdResult<SubMsg> {
    let cw721_transfer_msg = Cw721ExecuteMsg::TransferNft {
        token_id: token_id.to_string(),
        recipient: recipient.to_string(),
    };

    let exec_cw721_transfer = WasmMsg::Execute {
        contract_addr: collection.to_string(),
        msg: to_binary(&cw721_transfer_msg)?,
        funds: vec![],
    };

    Ok(SubMsg::new(exec_cw721_transfer))
}

/// Invoke `owner_of` to get the owner of an NFT.
pub fn owner_of(
    querier: &QuerierWrapper,
    collection: &Addr,
    token_id: &str,
) -> StdResult<OwnerOfResponse> {
    Cw721Contract::<Empty, Empty>(collection.clone(), PhantomData, PhantomData)
        .owner_of(querier, token_id, false)
}

/// Invoke `only_owner` to check that the sender is the owner of the NFT.
pub fn only_owner(
    querier: &QuerierWrapper,
    info: &MessageInfo,
    collection: &Addr,
    token_id: &str,
) -> StdResult<()> {
    let owner_of_response = owner_of(querier, collection, token_id)?;
    if owner_of_response.owner != info.sender {
        return Err(StdError::generic_err("Unauthorized"));
    }
    Ok(())
}

/// Invoke `has_approval` to check that the spender has approval for the NFT.
pub fn approval(
    querier: &QuerierWrapper,
    spender: &Addr,
    collection: &Addr,
    token_id: &str,
    include_expired: Option<bool>,
) -> StdResult<ApprovalResponse> {
    Cw721Contract::<Empty, Empty>(collection.clone(), PhantomData, PhantomData).approval(
        querier,
        token_id,
        spender.as_str(),
        include_expired,
    )
}

/// Invoke `approvals` to check that the spender has approval for the NFT.
pub fn approvals(
    querier: &QuerierWrapper,
    collection: &Addr,
    token_id: &str,
    include_expired: Option<bool>,
) -> StdResult<ApprovalsResponse> {
    Cw721Contract::<Empty, Empty>(collection.clone(), PhantomData, PhantomData).approvals(
        querier,
        token_id,
        include_expired,
    )
}

/// Invoke `all_operators` to check that the spender has approval for the NFT.
pub fn all_operators(
    querier: &QuerierWrapper,
    owner: &String,
    collection: &Addr,
    start_after: Option<String>,
    limit: Option<u32>,
    include_expired: bool,
) -> StdResult<Vec<Approval>> {
    Cw721Contract::<Empty, Empty>(collection.clone(), PhantomData, PhantomData).all_operators(
        querier,
        owner,
        include_expired,
        start_after,
        limit,
    )
}

/// Invoke `operator` to check that the spender is operator of owner this NFT.
pub fn operator<T: Into<String>>(
    querier: &QuerierWrapper,
    owner: T,
    operator: T,
    collection: &Addr,
    include_expired: Option<bool>,
) -> StdResult<Approval> {
    let req = cw721_base::QueryMsg::Operator {
        owner: owner.into(),
        operator: operator.into(),
        include_expired,
    };
    let res: cw721::OperatorResponse =
        Cw721Contract::<Empty, Empty>(collection.clone(), PhantomData, PhantomData)
            .query(querier, req)?;
    Ok(res.approval)
}
