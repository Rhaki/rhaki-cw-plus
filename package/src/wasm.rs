use cosmwasm_std::{
    instantiate2_address, to_binary, Addr, Api, Binary, Coin, CosmosMsg, QuerierWrapper, ReplyOn,
    StdError, StdResult, SubMsg, WasmMsg,
};
use serde::Serialize;

pub fn generate_instantiate_2_addr(
    querier: &QuerierWrapper,
    api: &dyn Api,
    code_id: u64,
    creator: &Addr,
    salt: &Binary,
) -> StdResult<Addr> {
    let res = querier.query_wasm_code_info(code_id)?;

    let addr = match instantiate2_address(
        &res.checksum,
        &api.addr_canonicalize(creator.as_ref())?,
        salt,
    ) {
        Ok(addr) => addr,
        Err(err) => return Err(StdError::generic_err(err.to_string())),
    };

    api.addr_humanize(&addr)
}

pub trait CosmosMsgBuilder {
    fn into_cosmos_msg(self) -> CosmosMsg;
}

pub trait WasmMsgBuilder {
    fn build_execute<T: Serialize>(
        contract: impl Into<String>,
        msg: T,
        funds: Vec<Coin>,
    ) -> StdResult<WasmMsg> {
        Ok(WasmMsg::Execute {
            contract_addr: contract.into(),
            msg: to_binary(&msg)?,
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
            msg: to_binary(&msg)?,
            funds,
            label,
        })
    }
}

impl WasmMsgBuilder for WasmMsg {}

impl CosmosMsgBuilder for WasmMsg {
    fn into_cosmos_msg(self) -> CosmosMsg {
        CosmosMsg::Wasm(self)
    }
}

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
