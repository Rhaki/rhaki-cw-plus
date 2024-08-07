use {
    crate::{
        encdec::{self, base64_encode},
        traits::IntoStdResult,
    },
    cosmwasm_schema::cw_serde,
    cosmwasm_std::{CosmosMsg, StdError, StdResult},
    serde::{de::DeserializeOwned, Deserialize, Serialize},
    std::{collections::HashMap, fmt::Debug, hash::Hash},
};

pub use {
    serde_cw_value::Value,
    serde_json::{json, Value as StdValue},
    serde_json_wasm::{from_str as sjw_from_str, to_string as sjw_to_string, to_vec as sjw_to_vec},
};

#[cw_serde]
pub enum PathKey {
    Index(u64),
    Key(String),
}

/// `Serialize` a `serde_cw_value::Value` to `String`
pub fn value_to_string(value: &Value) -> StdResult<String> {
    sjw_to_string(&value).into_std_result()
}

/// `Deserialize` a `String` into `serde_cw_value::Value`
pub fn value_from_string(string: &str) -> StdResult<Value> {
    serde_json_wasm::from_slice(
        serde_json_wasm::to_vec(string)
            .into_std_result()?
            .as_slice(),
    )
    .into_std_result()
}

/// `Deserialize` a `String` in `base64` into `serde_cw_value::Value`
pub fn value_from_b64(b64_string: &str) -> StdResult<Value> {
    value_from_string(&encdec::base64_decode_as_string(b64_string)?)
}

pub fn value_to_b64_string(value: &Value) -> StdResult<String> {
    Ok(base64_encode(&value_to_string(value)?))
}

/// Parse a `serde_json::Value` into `serde_json_wasm::Value`
pub fn std_to_sjw_value(std_value: StdValue) -> StdResult<Value> {
    sjw_from_str::<Value>(&std_value.to_string()).into_std_result()
}

/// Parse a `Value` into a CosmosMsg
pub fn value_to_comsos_msg(value: &Value) -> StdResult<CosmosMsg> {
    value.clone().deserialize_into().into_std_result()
}

pub trait SerdeValue {
    fn from_b64(encoded_b64: impl Into<String>) -> StdResult<Value>;
    fn from_string(string: impl Into<String>) -> StdResult<Value>;
    fn as_string(&self) -> StdResult<String>;
    fn to_cosmos_msg(&self) -> StdResult<CosmosMsg>;
    fn to_b64_encoded(&self) -> StdResult<String>;
    fn get_value_by_path<C: DeserializeOwned>(&self, path_key: Vec<PathKey>) -> StdResult<C>;
    fn get_array_index(&self, index: impl Into<usize>) -> StdResult<Value>;
    fn get_map_value(&self, value: impl Into<Value> + Clone) -> StdResult<Value>;
}

impl SerdeValue for Value {
    fn from_b64(encoded_b64: impl Into<String>) -> StdResult<Value> {
        value_from_b64(&encoded_b64.into())
    }

    fn from_string(string: impl Into<String>) -> StdResult<Value> {
        value_from_string(&string.into())
    }

    fn as_string(&self) -> StdResult<String> {
        value_to_string(self)
    }

    fn to_cosmos_msg(&self) -> StdResult<CosmosMsg> {
        value_to_comsos_msg(self)
    }

    fn to_b64_encoded(&self) -> StdResult<String> {
        value_to_b64_string(self)
    }

    fn get_value_by_path<C: DeserializeOwned>(&self, path_key: Vec<PathKey>) -> StdResult<C> {
        let mut value = self.clone();
        for k in path_key {
            match k {
                PathKey::Index(index) => match value {
                    Value::Seq(val) => value = val[index as usize].clone(),
                    _ => panic!(),
                },
                PathKey::Key(key) => match value {
                    Value::Map(val) => value = val[&Value::from_string(key).unwrap()].clone(),
                    _ => panic!(),
                },
            }
        }
        serde_json_wasm::from_slice(
            serde_json_wasm::to_vec(&value)
                .into_std_result()?
                .as_slice(),
        )
        .into_std_result()
        // Ok(value)
    }

    fn get_array_index(&self, index: impl Into<usize>) -> StdResult<Value> {
        if let Value::Seq(array) = self {
            Ok(array[index.into()].clone())
        } else {
            Err(StdError::generic_err(format!(
                "Value is not a Seq: {:?}",
                self
            )))
        }
    }

    fn get_map_value(&self, value: impl Into<Value> + Clone) -> StdResult<Value> {
        if let Value::Map(map) = self {
            map.get(&value.clone().into())
                .map(|val| val.clone())
                .ok_or(StdError::generic_err(format!(
                    "map key not found: {:?}",
                    value.into()
                )))
        } else {
            Err(StdError::generic_err(format!(
                "Value is not a Map: {:?}",
                self
            )))
        }
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

pub trait ToCwJson {
    fn into_cw(&self) -> StdResult<Value>;
}

impl ToCwJson for StdValue {
    fn into_cw(&self) -> StdResult<Value> {
        std_to_sjw_value(self.clone())
    }
}

pub trait DoubleDeserialize {
    fn double_deserialize<'de, F: Deserialize<'de>, S: Deserialize<'de>>(
        &self,
    ) -> StdResult<DoubleValueDeserializeResult<F, S>>;
}

impl DoubleDeserialize for Value {
    fn double_deserialize<'de, F: Deserialize<'de>, S: Deserialize<'de>>(
        &self,
    ) -> StdResult<DoubleValueDeserializeResult<F, S>> {
        if let Ok(res) = self.clone().deserialize_into() {
            return Ok(DoubleValueDeserializeResult::First(res));
        }

        if let Ok(res) = self.clone().deserialize_into() {
            return Ok(DoubleValueDeserializeResult::Second(res));
        }

        Err(StdError::generic_err("Deserialize failed"))
    }
}

#[cw_serde]
pub enum DoubleValueDeserializeResult<F, S> {
    First(F),
    Second(S),
}

impl<'de, F, S> DoubleValueDeserializeResult<F, S>
where
    F: Deserialize<'de>,
    S: Deserialize<'de>,
{
}

#[allow(clippy::wrong_self_convention)]
pub trait IntoSerdeJsonString: Serialize {
    fn into_json_string(&self) -> StdResult<String> {
        serde_json_wasm::to_string(self).into_std_result()
    }
}

impl<T: Serialize> IntoSerdeJsonString for Option<T> {}
