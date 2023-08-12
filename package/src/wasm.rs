use cosmwasm_std::{instantiate2_address, Addr, Api, Binary, QuerierWrapper, StdError, StdResult};

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
