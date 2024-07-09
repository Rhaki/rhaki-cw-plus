use {
    crate::traits::IntoStdResult,
    base64::{engine::general_purpose::STANDARD, Engine},
    cosmwasm_std::{StdError, StdResult},
};

/// Encode a `&str` in `base64` and return `String`
pub fn base64_encode(decoded: &str) -> String {
    STANDARD.encode(decoded.as_bytes())
}

// Decode a `&str` in `base64` and return `Vec<u8>`
pub fn base64_decode(encoded: &str) -> StdResult<Vec<u8>> {
    STANDARD.decode(encoded.as_bytes()).into_std_result()
}

/// Decode a `&str` in `base64` and return `String`
pub fn base64_decode_as_string(encoded: &str) -> StdResult<String> {
    match base64_decode(encoded) {
        Ok(decoded) => match String::from_utf8(decoded) {
            Ok(decoded) => Ok(decoded),
            Err(err) => Err(StdError::generic_err(format!(
                "Error on String::from_utf8: {}",
                err
            ))),
        },
        Err(err) => Err(StdError::generic_err(format!(
            "Error on base_64_decode {}",
            err
        ))),
    }
}
