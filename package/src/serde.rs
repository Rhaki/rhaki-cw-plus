use serde_cw_value::DeserializerError;
pub use serde_cw_value::Value;

/// `Serialize` a `serde_cw_value::Value` to `String`
pub fn value_to_string(value: Value) -> Result<String, DeserializerError> {
    value.deserialize_into()
}

/// `Deserialize` a `String` into `serde_cw_value::Value`
pub fn value_from_string(string: &str) -> Result<Value, serde_json_wasm::de::Error> {
    serde_json_wasm::from_str(string)
}
