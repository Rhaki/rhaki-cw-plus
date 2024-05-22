use anyhow::anyhow;
use std::path::PathBuf;

use async_trait::async_trait;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::Decimal;
use serde::{de::DeserializeOwned, Serialize};

use crate::math::IntoDecimal;

pub use cosmos_grpc_client;
pub use tokio;

pub type AnyResult<T> = anyhow::Result<T>;

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
    pub fn save_data(&self) -> AnyResult<()> {
        let chain_info_no_seed: ChainInfoNoSeed = self.chain_info.clone().into();
        let prefix = chain_info_no_seed.to_prefix();

        let data_container_no_seed = DataContainerNoSeed {
            chain_info: chain_info_no_seed,
            data: &self.data,
        };

        let mut path = PathBuf::from(std::env::current_dir()?);
        path.push(format!("{}/config/{}.json", T::PATH_CONFIG, prefix));
        Ok(std::fs::write(
            path,
            serde_json::to_string(&data_container_no_seed)?,
        )?)
    }
}

#[async_trait]
pub trait Deploier: Serialize + DeserializeOwned {
    const PATH_ARTIFACTS: &'static str;
    const PATH_CONFIG: &'static str;

    fn read_data_from_input() -> AnyResult<DataContainer<Self>> {
        let net = get_net_by_args();
        Self::read_data_from_args(net.0, &net.1)
    }

    fn read_data_from_args(net_type: NetType, chain_name: &str) -> AnyResult<DataContainer<Self>> {
        let net = (net_type, chain_name.to_string());
        let prefix = format!("{}-{}", Into::<&str>::into(net.0), net.1);

        let path = PathBuf::from(std::env::current_dir()?);

        let mut path_data = path.clone();
        let mut path_seed_phrase = path;

        path_data.push(format!("{}/config/{}.json", Self::PATH_CONFIG, prefix));
        let data = std::fs::read_to_string(path_data)?;
        let data = serde_json::from_str::<DataContainerNoSeed<Self>>(&data)?;

        path_seed_phrase.push(format!("{}/config/seed-{}.json", Self::PATH_CONFIG, prefix));
        let seed = std::fs::read_to_string(path_seed_phrase)?;
        let seed = serde_json::from_str::<SeedPhrase>(&seed)?;

        Ok(DataContainer {
            chain_info: data.chain_info.into_chain_info(seed.seed_phrase),
            data: data.data,
        })
    }

    fn generate(&self) -> AnyResult<()> {
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
        let mut path = PathBuf::from(std::env::current_dir()?);
        path.push(format!("{}/config", Self::PATH_CONFIG));

        if !path.exists() {
            std::fs::create_dir_all(path.clone())?;
        }

        let mut path_data = path.clone();
        let mut path_seed_phrase = path;

        path_data.push(format!("./{}.json", prefix));

        let data = serde_json::to_string(&container)?;
        std::fs::write(path_data.clone(), data)
            .map_err(|_| anyhow!("invalid path on generate: {}", path_data.to_str().unwrap()))?;

        path_seed_phrase.push(format!("./seed-{}.json", prefix));
        let data = serde_json::to_string(&seed_phrase)?;
        std::fs::write(path_seed_phrase.clone(), data).map_err(|_| {
            anyhow!(
                "invalid path on generate: {}",
                path_seed_phrase.to_str().unwrap()
            )
        })?;

        Ok(())
    }

    fn read_wasm_bytecode(&self, file_name: &str) -> AnyResult<Vec<u8>> {
        std::fs::read(format!("{}/{file_name}.wasm", Self::PATH_ARTIFACTS)).map_err(|_| {
            anyhow!(
                " {} not found in {}{}",
                file_name,
                std::env::current_dir().unwrap().to_str().unwrap(),
                Self::PATH_ARTIFACTS
            )
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
    use std::str::from_utf8;

    use anyhow::{anyhow, bail};
    use cosmos_grpc_client::{
        cosmos_sdk_proto::{
            cosmos::{
                base::v1beta1::Coin,
                tx::v1beta1::{GetTxRequest, GetTxResponse},
            },
            cosmwasm::wasm::v1::{AccessConfig, MsgInstantiateContract, MsgStoreCode},
            prost::Name,
        },
        AnyBuilder, BroadcastMode, GrpcClient, Wallet,
    };
    use cosmwasm_std::{to_json_binary, Coin as StdCoin, StdError, StdResult};
    use serde::Serialize;

    use crate::traits::IntoStdResult;

    use super::{AnyResult, ChainInfo, Deploier, NetType};

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
        wallet: &mut Wallet,
        data: &impl Deploier,
        file_name: &str,
        instantiate_permission: Option<AccessConfig>,
    ) -> AnyResult<u64> {
        print!("Storing {file_name}...");
        let bytes = data.read_wasm_bytecode(file_name)?;

        let msg = MsgStoreCode {
            sender: wallet.account_address()?,
            wasm_byte_code: bytes,
            instantiate_permission,
        }
        .build_any(MsgStoreCode::type_url());

        let res = wallet
            .broadcast_tx(vec![msg], None, None, BroadcastMode::Sync)
            .await?;

        let response = search_tx(&wallet.client, res.tx_response.unwrap().txhash, Some(10))
            .await
            .into_std_result()?;

        let code_id = get_code_id_from_init_response(response)?;

        print!(" {code_id}\n");

        Ok(code_id)
    }

    pub async fn instantiate_contract<T: Serialize>(
        wallet: &mut Wallet,
        admin: Option<String>,
        code_id: u64,
        label: impl Into<String>,
        msg: T,
        funds: Vec<StdCoin>,
        contract_name: Option<&str>,
    ) -> AnyResult<String> {
        print!(
            "Instaniate {}...",
            contract_name.unwrap_or(code_id.to_string().as_str())
        );

        let msg = MsgInstantiateContract {
            sender: wallet.account_address()?,
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
        .build_any(MsgInstantiateContract::type_url());

        let res = wallet
            .broadcast_tx(vec![msg], None, None, BroadcastMode::Sync)
            .await
            .unwrap();

        let response = search_tx(&wallet.client, res.tx_response.unwrap().txhash, None)
            .await
            .unwrap();

        let address = get_address_from_init_response(response).unwrap();

        print!(" {address}\n");

        Ok(address)
    }

    pub async fn search_tx(
        client: &GrpcClient,
        hash: String,
        max_timeout: Option<u64>,
    ) -> StdResult<GetTxResponse> {
        let timeout = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
            + max_timeout.unwrap_or(30);

        loop {
            let res = client
                .clone()
                .clients
                .tx
                .get_tx(GetTxRequest { hash: hash.clone() })
                .await;

            if let Ok(response) = res {
                return Ok(response.into_inner());
            }

            if std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs()
                > timeout
            {
                return Err(StdError::generic_err("timeout"));
            }
        }
    }

    pub fn get_code_id_from_init_response(response: GetTxResponse) -> AnyResult<u64> {
        for event in response
            .tx_response
            .ok_or(anyhow!("Empty tx_response"))?
            .events
        {
            // if event.r#type == "store_code".to_string() {
            for attribute in event.attributes {
                if attribute.key == "code_id".to_string() {
                    let val = from_utf8(&attribute.value)?;
                    return Ok(val.replace('"', "").parse().unwrap());
                }
            }
            // }
        }
        bail!("not found")
    }

    pub fn get_address_from_init_response(response: GetTxResponse) -> StdResult<String> {
        for event in response.tx_response.unwrap().events {
            // if event.r#type == "instantiate".to_string() {
            for attribute in event.attributes {
                if attribute.key == "_contract_address".to_string() {
                    return Ok(from_utf8(&attribute.value)?.to_string());
                }
            }
            // }
        }
        Err(StdError::generic_err("not found"))
    }

    pub async fn deploy_create_wallet(
        client: &GrpcClient,
        chain_info: &ChainInfo,
    ) -> AnyResult<Wallet> {
        Wallet::from_seed_phrase(
            client.clone(),
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
