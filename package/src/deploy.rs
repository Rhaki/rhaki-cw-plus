use std::path::PathBuf;

use async_trait::async_trait;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Decimal, StdError, StdResult};
use serde::{de::DeserializeOwned, Serialize};

use crate::{math::IntoDecimal, traits::IntoStdResult};

pub use cosmos_grpc_client;
pub use tokio;

use self::functions::get_net_by_args;

#[non_exhaustive]
#[cw_serde]
pub struct ChainInfo {
    pub chain_name: String,
    pub net: NetType,
    pub grpc: String,
    pub seed_phrase: String,
    pub chain_prefix: String,
    pub account_index: u64,
    pub coin_type: u64,
    pub gas_price: Decimal,
    pub gas_denom: String,
    pub gas_adjustment: Decimal,
}

impl Into<ChainInfoNoSeed> for ChainInfo {
    fn into(self) -> ChainInfoNoSeed {
        ChainInfoNoSeed {
            chain_name: self.chain_name,
            net: self.net,
            grpc: self.grpc,
            chain_prefix: self.chain_prefix,
            account_index: self.account_index,
            coin_type: self.coin_type,
            gas_price: self.gas_price,
            gas_denom: self.gas_denom,
            gas_adjustment: self.gas_adjustment,
        }
    }
}
#[cw_serde]
pub struct DataContainer<T> {
    pub chain_info: ChainInfo,
    pub data: T,
}

#[non_exhaustive]
#[cw_serde]
struct ChainInfoNoSeed {
    pub chain_name: String,
    pub net: NetType,
    pub grpc: String,
    pub chain_prefix: String,
    pub account_index: u64,
    pub coin_type: u64,
    pub gas_price: Decimal,
    pub gas_denom: String,
    pub gas_adjustment: Decimal,
}

impl ChainInfoNoSeed {
    pub fn new(
        chain_name: &str,
        grpc: &str,
        net_type: NetType,
        chain_prefix: &str,
        coin_type: u64,
        gas_price: Decimal,
        gas_denom: &str,
        gas_adjustment: Decimal,
    ) -> Self {
        Self {
            chain_name: chain_name.to_string(),
            net: net_type,
            grpc: grpc.to_string(),
            chain_prefix: chain_prefix.to_string(),
            account_index: 0,
            coin_type,
            gas_price,
            gas_denom: gas_denom.to_string(),
            gas_adjustment,
        }
    }

    pub fn to_prefix(&self) -> String {
        format!(
            "{}-{}",
            Into::<&str>::into(self.net.clone()),
            self.chain_name
        )
    }
    pub fn into_chain_info(self, seed_phrase: String) -> ChainInfo {
        ChainInfo {
            seed_phrase,
            chain_name: self.chain_name,
            net: self.net,
            grpc: self.grpc,
            chain_prefix: self.chain_prefix,
            account_index: self.account_index,
            coin_type: self.coin_type,
            gas_price: self.gas_price,
            gas_denom: self.gas_denom,
            gas_adjustment: self.gas_adjustment,
        }
    }
}

#[cw_serde]
struct DataContainerNoSeed<T> {
    pub chain_info: ChainInfoNoSeed,
    pub data: T,
}

#[cw_serde]
struct SeedPhrase {
    pub seed_phrase: String,
}

impl<T> DataContainer<T>
where
    T: Deploier,
{
    pub fn save_data(&self) -> StdResult<()> {
        let chain_info_no_seed: ChainInfoNoSeed = self.chain_info.clone().into();
        let prefix = chain_info_no_seed.to_prefix();

        let data_container_no_seed = DataContainerNoSeed {
            chain_info: chain_info_no_seed,
            data: &self.data,
        };

        let mut path = PathBuf::from(std::env::current_dir().into_std_result()?);
        path.push(format!("{}/config/{}.json", T::PATH_CONFIG, prefix));
        std::fs::write(
            path,
            serde_json::to_string(&data_container_no_seed).into_std_result()?,
        )
        .into_std_result()
    }
}

#[async_trait]
pub trait Deploier: Serialize + DeserializeOwned {
    const PATH_ARTIFACTS: &'static str;
    const PATH_CONFIG: &'static str;

    fn read_data() -> StdResult<DataContainer<Self>> {
        let net = get_net_by_args();
        let prefix = format!("{}-{}", Into::<&str>::into(net.0), net.1);

        let path = PathBuf::from(std::env::current_dir().into_std_result()?);

        let mut path_data = path.clone();
        let mut path_seed_phrase = path;

        path_data.push(format!("{}/config/{}.json", Self::PATH_CONFIG, prefix));
        let data = std::fs::read_to_string(path_data).into_std_result()?;
        let data = serde_json::from_str::<DataContainerNoSeed<Self>>(&data).into_std_result()?;

        path_seed_phrase.push(format!("{}/config/seed-{}.json", Self::PATH_CONFIG, prefix));
        let seed = std::fs::read_to_string(path_seed_phrase).into_std_result()?;
        let seed = serde_json::from_str::<SeedPhrase>(&seed).into_std_result()?;

        Ok(DataContainer {
            chain_info: data.chain_info.into_chain_info(seed.seed_phrase),
            data: data.data,
        })
    }

    fn generate(&self) -> StdResult<()> {
        let net = get_net_by_args();

        let chain_info: ChainInfoNoSeed = net.into();
        let prefix = chain_info.to_prefix();
        let container = DataContainerNoSeed {
            chain_info,
            data: self,
        };

        let seed_phrase = SeedPhrase {
            seed_phrase: "".to_string(),
        };

        // Check if folder config exists
        let mut path = PathBuf::from(std::env::current_dir().into_std_result()?);
        path.push(format!("{}/config", Self::PATH_CONFIG));

        if !path.exists() {
            std::fs::create_dir_all(path.clone()).into_std_result()?;
        }

        let mut path_data = path.clone();
        let mut path_seed_phrase = path;

        path_data.push(format!("./{}.json", prefix));

        let data = serde_json::to_string(&container).into_std_result()?;
        std::fs::write(path_data.clone(), data)
            .into_std_result()
            .map_err(|_| {
                StdError::generic_err(format!(
                    "invalid path on generate: {}",
                    path_data.to_str().unwrap()
                ))
            })?;

        path_seed_phrase.push(format!("./seed-{}.json", prefix));
        let data = serde_json::to_string(&seed_phrase).into_std_result()?;
        std::fs::write(path_seed_phrase.clone(), data)
            .into_std_result()
            .map_err(|_| {
                StdError::generic_err(format!(
                    "invalid path on generate: {}",
                    path_seed_phrase.to_str().unwrap()
                ))
            })?;

        Ok(())
    }

    fn read_wasm_bytecode(&self, file_name: &str) -> StdResult<Vec<u8>> {
        std::fs::read(format!("{}/{file_name}.wasm", Self::PATH_ARTIFACTS)).map_err(|_| {
            StdError::generic_err(format!(
                " {} not found in {}{}",
                file_name,
                std::env::current_dir().unwrap().to_str().unwrap(),
                Self::PATH_ARTIFACTS
            ))
        })
    }
}

impl From<(NetType, String)> for ChainInfoNoSeed {
    fn from(value: (NetType, String)) -> Self {
        match value.1.as_str() {
            "terra" => ChainInfoNoSeed::new(
                value.1.as_str(),
                match value.0 {
                    NetType::Mainnet => "https:/terra-grpc.polkachu.com:11790",
                    NetType::Testnet => "https://terra-testnet-grpc.polkachu.com:11790",
                },
                value.0,
                "terra",
                330,
                "0.15".into_decimal(),
                "uluna",
                "1.3".into_decimal(),
            ),
            "osmosis" => ChainInfoNoSeed::new(
                value.1.as_str(),
                match value.0 {
                    NetType::Mainnet => "https://osmosis-grpc.polkachu.com:12590",
                    NetType::Testnet => "https://osmosis-testnet-grpc.polkachu.com:12590",
                },
                value.0,
                "osmo",
                118,
                "0.025".into_decimal(),
                "uosmo",
                "1.3".into_decimal(),
            ),
            "injective" => ChainInfoNoSeed::new(
                value.1.as_str(),
                match value.0 {
                    NetType::Mainnet => "",
                    NetType::Testnet => "https://injective-testnet-grpc.polkachu.com:14390",
                },
                value.0,
                "inj",
                118,
                "700000000".into_decimal(),
                "inj",
                "1.3".into_decimal(),
            ),
            _ => ChainInfoNoSeed::new(
                value.1.as_str(),
                "",
                value.0,
                "",
                118,
                "0".into_decimal(),
                "",
                "1.3".into_decimal(),
            ),
        }
    }
}

#[cw_serde]
pub enum NetType {
    Mainnet,
    Testnet,
}

impl Into<&str> for NetType {
    fn into(self) -> &'static str {
        match self {
            NetType::Mainnet => "mainnet",
            NetType::Testnet => "testnet",
        }
    }
}

pub mod functions {
    use cosmos_grpc_client::{
        cosmos_sdk_proto::{
            cosmos::{
                base::v1beta1::Coin,
                tx::v1beta1::{GetTxRequest, GetTxResponse},
            },
            cosmwasm::wasm::v1::{AccessConfig, MsgInstantiateContract, MsgStoreCode},
        },
        cosmrs::tx::MessageExt,
        BroadcastMode, GrpcClient, Wallet,
    };
    use cosmwasm_std::{to_json_binary, Coin as StdCoin, StdError, StdResult};
    use serde::Serialize;

    use crate::traits::IntoStdResult;

    use super::{ChainInfo, Deploier, NetType};

    pub fn get_net_by_args() -> (NetType, String) {
        let args = std::env::args().collect::<Vec<String>>();

        if args.len() != 3 {
            panic!("Invalid args len: {:?}", args.len() - 1)
        };

        (
            match args[1].as_str() {
                "mainnet" => NetType::Mainnet,
                "testnet" => NetType::Testnet,
                val => panic!("args is not mainnet or testnet: {val}"),
            },
            args[2].clone(),
        )
    }

    pub async fn store_code(
        client: &mut GrpcClient,
        wallet: &mut Wallet,
        data: &impl Deploier,
        file_name: &str,
        instantiate_permission: Option<AccessConfig>,
    ) -> StdResult<u64> {
        print!("Storing {file_name}...");
        let bytes = data.read_wasm_bytecode(file_name)?;

        let msg = MsgStoreCode {
            sender: wallet.account_address(),
            wasm_byte_code: bytes,
            instantiate_permission,
        }
        .to_any()
        .into_std_result()?;

        let res = wallet
            .broadcast_tx(client, vec![msg], None, None, BroadcastMode::Sync)
            .await
            .into_std_result()?;

        let response = search_tx(client, res.tx_response.unwrap().txhash, None)
            .await
            .into_std_result()?;

        let code_id = get_code_id_from_init_response(response)?;

        print!(" {code_id}\n");

        Ok(code_id)
    }

    pub async fn instantiate_contract<T: Serialize>(
        client: &mut GrpcClient,
        wallet: &mut Wallet,
        admin: Option<String>,
        code_id: u64,
        label: impl Into<String>,
        msg: T,
        funds: Vec<StdCoin>,
        contract_name: Option<&str>,
    ) -> StdResult<String> {
        print!(
            "Instaniate {}...",
            contract_name.unwrap_or(code_id.to_string().as_str())
        );

        let msg = MsgInstantiateContract {
            sender: wallet.account_address(),
            admin: admin.unwrap_or_default(),
            code_id,
            label: label.into(),
            msg: to_json_binary(&msg).unwrap().to_vec(),
            funds: funds
                .into_iter()
                .map(|val| Coin {
                    denom: val.denom,
                    amount: val.amount.to_string(),
                })
                .collect(),
        }
        .to_any()
        .unwrap();

        let res = wallet
            .broadcast_tx(client, vec![msg], None, None, BroadcastMode::Sync)
            .await
            .unwrap();

        let response = search_tx(client, res.tx_response.unwrap().txhash, None)
            .await
            .unwrap();

        let address = get_address_from_init_response(response).unwrap();

        print!(" {address}\n");

        Ok(address)
    }

    pub async fn search_tx(
        client: &mut GrpcClient,
        hash: String,
        _max_timeout: Option<u64>,
    ) -> StdResult<GetTxResponse> {
        loop {
            let res = client
                .clients
                .tx
                .get_tx(GetTxRequest { hash: hash.clone() })
                .await;

            if let Ok(response) = res {
                return Ok(response.into_inner());
            }
        }
    }

    pub fn get_code_id_from_init_response(response: GetTxResponse) -> StdResult<u64> {
        for event in response.tx_response.unwrap().events {
            // if event.r#type == "store_code".to_string() {
            for attribute in event.attributes {
                if attribute.key == "code_id".to_string() {
                    // clear code id
                    // let a  = attribute.key.replace('"', "");
                    return Ok(attribute.value.replace('"', "").parse().unwrap());
                }
            }
            // }
        }
        Err(StdError::generic_err("not found"))
    }

    pub fn get_address_from_init_response(response: GetTxResponse) -> StdResult<String> {
        for event in response.tx_response.unwrap().events {
            // if event.r#type == "instantiate".to_string() {
            for attribute in event.attributes {
                if attribute.key == "_contract_address".to_string() {
                    return Ok(attribute.value);
                }
            }
            // }
        }
        Err(StdError::generic_err("not found"))
    }

    pub async fn deploy_create_wallet(
        client: &mut GrpcClient,
        chain_info: &ChainInfo,
    ) -> StdResult<Wallet> {
        Wallet::from_seed_phrase(
            client,
            chain_info.seed_phrase.clone(),
            chain_info.chain_prefix.clone(),
            chain_info.coin_type,
            chain_info.account_index,
            chain_info.gas_price,
            chain_info.gas_adjustment,
            chain_info.gas_denom.clone(),
        )
        .await
    }
}

// const DEFAULT_GAS_ADJUSTMENT: Decimal = "1.3".into_decimal();

// const DEFAULT_OSMOSIS_TESTNET: ChainInfoNoSeed = ChainInfoNoSeed::new(
//     "osmosis",
//     "https://osmosis-testnet-grpc.polkachu.com:12590",
//     NetType::Testnet,
//     "osmo",
//     118,
//     "0.025",
//     "uosmo",
//     DEFAULT_GAS_ADJUSTMENT,
// );
