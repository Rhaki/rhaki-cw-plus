use std::{collections::HashMap, fmt::Debug, hash::Hash};

use crate::encdec::{self, base64_encode};
use cosmwasm_std::{CosmosMsg, StdError, StdResult};
pub use serde_cw_value::Value;
pub use serde_json::json;
use serde_json::Value as StdValue;
pub use serde_json_wasm::{from_str as sjw_from_str, to_string as sjw_to_string};

/// `Serialize` a `serde_cw_value::Value` to `String`
pub fn value_to_string(value: &Value) -> StdResult<String> {
    match sjw_to_string(&value) {
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

pub fn value_to_b64_string(value: &Value) -> StdResult<String> {
    Ok(base64_encode(&value_to_string(value)?))
}

/// Parse a `serde_json::Value` into `serde_json_wasm::Value`
pub fn std_to_sjw_value(std_value: StdValue) -> Value {
    sjw_from_str(&std_value.to_string()).unwrap()
}

/// Parse a `Value` into a CosmosMsg
pub fn value_to_comsos_msg(value: &Value) -> StdResult<CosmosMsg> {
    match value.clone().deserialize_into() {
        Ok(msg) => Ok(msg),
        Err(err) => Err(StdError::generic_err(err.to_string())),
    }
}

pub trait SerdeMapSerializer<V> {
    fn into_json_ser_map(self) -> HashMap<String, V>;
}

impl<K, V> SerdeMapSerializer<V> for HashMap<K, V>
where
    K: Into<String> + Clone,
    V: Clone,
{
    fn into_json_ser_map(self) -> HashMap<String, V> {
        let mut map: HashMap<String, V> = HashMap::new();

        for (k, v) in self {
            map.insert(Into::<String>::into(k.clone()), v.clone());
        }

        map
    }
}

#[allow(clippy::wrong_self_convention)]
pub trait SerdeMapDeserialize<V, K: TryFrom<String>> {
    fn from_json_ser_map(self) -> Result<HashMap<K, V>, K::Error>;
}

impl<K, V> SerdeMapDeserialize<V, K> for HashMap<String, V>
where
    K: TryFrom<String> + Eq + PartialEq + Hash + Debug,
    V: Clone,
{
    fn from_json_ser_map(self) -> Result<HashMap<K, V>, K::Error> {
        let mut map: HashMap<K, V> = HashMap::new();

        for (k, v) in self {
            let b = K::try_from(Into::<String>::into(k.clone()))?;
            map.insert(b, v.clone());
        }

        Ok(map)
    }
}
