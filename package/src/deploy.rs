use std::path::PathBuf;

use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Decimal, StdError, StdResult};
use serde::{de::DeserializeOwned, Serialize};

use crate::traits::IntoStdResult;

#[cw_serde]
pub struct ChainInfo {
    pub grpc: String,
    pub seed_phrase: String,
    pub chain_prefix: String,
    pub coin_type: u64,
    pub gas_price: Decimal,
    pub gas_denom: String,
    pub gas_adjustment: Decimal,
}

#[non_exhaustive]
pub struct DeployHelper {
    path_artifacts: String,
    path_config: String,
}

impl DeployHelper {
    pub fn new(path_artifacts: &str, path_conifg: &str) -> Self {
        Self {
            path_artifacts: path_artifacts.to_string(),
            path_config: path_conifg.to_string(),
        }
    }
    pub fn read_data<T: DeserializeOwned>(&self) -> StdResult<T> {
        let mut complete_path = PathBuf::from(std::env::current_dir().unwrap());
        complete_path.push(format!("{}", self.path_config));
        let config = std::fs::read_to_string(complete_path).into_std_result()?;
        serde_json::from_str(&config).into_std_result()
    }

    pub fn save_data<T: Serialize>(&self, data: &T) {
        let mut path = PathBuf::from(std::env::current_dir().unwrap());
        path.push(format!("{}", self.path_config));
        std::fs::write(path, serde_json::to_string(&data).unwrap()).unwrap();
    }

    pub fn read_wasm_bytecode(&self, file_name: &str) -> Vec<u8> {
        std::fs::read(format!("{}/{file_name}.wasm", self.path_artifacts))
            .map_err(|_| {
                StdError::generic_err(format!("path not found: {:?}", std::env::current_dir()))
            })
            .unwrap()
    }
}
