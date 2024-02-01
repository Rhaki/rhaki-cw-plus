use cosmwasm_std::{
    instantiate2_address, Addr, Binary, Coin, CosmosMsg, Deps, ReplyOn, StdError,
    StdResult, SubMsg, WasmMsg, to_json_binary,
};
use serde::Serialize;

use crate::traits::IntoBinary;

pub fn generate_instantiate_2_addr(
    deps: Deps,
    code_id: u64,
    creator: &Addr,
    salt: &Binary,
) -> StdResult<Addr> {
    let res = deps.querier.query_wasm_code_info(code_id)?;

    let addr = match instantiate2_address(
        &res.checksum,
        &deps.api.addr_canonicalize(creator.as_ref())?,
        salt,
    ) {
        Ok(addr) => addr,
        Err(err) => return Err(StdError::generic_err(err.to_string())),
    };

    deps.api.addr_humanize(&addr)
}

pub fn build_instantiate_2<T: Serialize>(
    deps: Deps,
    creator: &Addr,
    salt: Binary,
    admin: Option<String>,
    code_id: u64,
    msg: T,
    funds: Vec<Coin>,
    label: String,
) -> StdResult<(CosmosMsg, Addr)> {
    let addr = generate_instantiate_2_addr(deps, code_id, creator, &salt)?;

    Ok((
        WasmMsg::build_init2(admin, code_id, msg, funds, label, salt)?.into(),
        addr,
    ))
}

pub trait WasmMsgBuilder {
    fn build_execute<T: Serialize>(
        contract: impl Into<String>,
        msg: T,
        funds: Vec<Coin>,
    ) -> StdResult<WasmMsg> {
        Ok(WasmMsg::Execute {
            contract_addr: contract.into(),
            msg: to_json_binary(&msg)?,
            funds,
        })
    }

    fn build_init<T: Serialize>(
        admin: Option<String>,
        code_id: u64,
        msg: T,
        funds: Vec<Coin>,
        label: String,
    ) -> StdResult<WasmMsg> {
        Ok(WasmMsg::Instantiate {
            admin,
            code_id,
            msg: msg.into_binary()?,
            funds,
            label,
        })
    }

    fn build_init2<T: Serialize>(
        admin: Option<String>,
        code_id: u64,
        msg: T,
        funds: Vec<Coin>,
        label: String,
        salt: Binary,
    ) -> StdResult<WasmMsg> {
        Ok(WasmMsg::Instantiate2 {
            admin,
            code_id,
            msg: msg.into_binary()?,
            funds,
            label,
            salt,
        })
    }
}

impl WasmMsgBuilder for WasmMsg {}

pub trait CosmosMsgExt {
    fn into_submsg_always(self, reply_id: u64, gas_limit: Option<u64>) -> SubMsg;
    fn into_submsg_on_error(self, reply_id: u64, gas_limit: Option<u64>) -> SubMsg;
    fn into_submsg_on_success(self, reply_id: u64, gas_limit: Option<u64>) -> SubMsg;
    fn into_submsg_never(self) -> SubMsg;
}

impl CosmosMsgExt for CosmosMsg {
    fn into_submsg_always(self, reply_id: u64, gas_limit: Option<u64>) -> SubMsg {
        SubMsg {
            id: reply_id,
            msg: self,
            gas_limit,
            reply_on: ReplyOn::Always,
        }
    }

    fn into_submsg_on_error(self, reply_id: u64, gas_limit: Option<u64>) -> SubMsg {
        SubMsg {
            id: reply_id,
            msg: self,
            gas_limit,
            reply_on: ReplyOn::Error,
        }
    }

    fn into_submsg_on_success(self, reply_id: u64, gas_limit: Option<u64>) -> SubMsg {
        SubMsg {
            id: reply_id,
            msg: self,
            gas_limit,
            reply_on: ReplyOn::Success,
        }
    }

    fn into_submsg_never(self) -> SubMsg {
        SubMsg::new(self)
    }
}
