use crate::encdec;
use cosmwasm_std::{StdError, StdResult};
pub use serde_cw_value::Value;

/// `Serialize` a `serde_cw_value::Value` to `String`
pub fn value_to_string(value: &Value) -> StdResult<String> {
    match value.clone().deserialize_into() {
        Ok(v) => Ok(v),
        Err(err) => Err(StdError::generic_err(err.to_string())),
    }
}

/// `Deserialize` a `String` into `serde_cw_value::Value`
pub fn value_from_string(string: &str) -> StdResult<Value> {
    match serde_json_wasm::from_str(string) {
        Ok(v) => Ok(v),
        Err(err) => Err(StdError::generic_err(err.to_string())),
    }
}

/// `Deserialize` a `String` in `base64` into `serde_cw_value::Value`
pub fn value_from_b64(b64_string: &str) -> StdResult<Value> {
    value_from_string(&encdec::base64_decode_as_string(b64_string)?)
}
