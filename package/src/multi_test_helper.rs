pub use cw_multi_test;

use std::fmt::{self, Debug, Display};

use cosmwasm_std::{
    Addr, Binary, Coin, CustomQuery, Deps, DepsMut, Empty, Env, MessageInfo, Reply, Response,
    StdResult,
};
use cw_multi_test::{
    addons::{MockAddressGenerator, MockApiBech32},
    error::AnyResult,
    no_init, App, AppBuilder, AppResponse, BankKeeper, BankSudo, ContractWrapper, Executor,
    SudoMsg, WasmKeeper,
};
use schemars::JsonSchema;
use serde::de::DeserializeOwned;

use crate::{
    token::{AssetAmount, AssetInfoPrecisioned, AssetPrecisioned},
    traits::{IntoAddr, IntoStdResult},
    utils::WrapOk,
};

const BENCH32_PREFIX: &str = "cosmos";

pub type Bech32App = App<BankKeeper, MockApiBech32>;

pub type AppResult = AnyResult<AppResponse>;

fn build_api() -> MockApiBech32 {
    MockApiBech32::new(BENCH32_PREFIX)
}

fn build_wasm_keeper() -> WasmKeeper<Empty, Empty> {
    WasmKeeper::default().with_address_generator(MockAddressGenerator)
}

pub fn generate_addr(name: &str) -> Addr {
    build_api().addr_make(name)
}

pub fn build_bech32_app() -> Bech32App {
    AppBuilder::new()
        .with_api(build_api())
        .with_wasm(build_wasm_keeper())
        .build(no_init)
}

type ContractFn<T, C, E, Q> =
    fn(deps: DepsMut<Q>, env: Env, info: MessageInfo, msg: T) -> Result<Response<C>, E>;

type QueryFn<T, E, Q> = fn(deps: Deps<Q>, env: Env, msg: T) -> Result<Binary, E>;

type ReplyFn<C, E, Q> = fn(deps: DepsMut<Q>, env: Env, msg: Reply) -> Result<Response<C>, E>;

#[allow(clippy::type_complexity)]
pub fn create_code<
    T1: DeserializeOwned + fmt::Debug + 'static,
    T2: DeserializeOwned + 'static,
    T3: DeserializeOwned + 'static,
    C: Clone + fmt::Debug + PartialEq + JsonSchema + 'static,
    E1: Display + fmt::Debug + Send + Sync + 'static,
    E2: Display + fmt::Debug + Send + Sync + 'static,
    E3: Display + fmt::Debug + Send + Sync + 'static,
    Q: CustomQuery + DeserializeOwned + 'static,
>(
    instantiate: ContractFn<T2, C, E2, Q>,
    execute: ContractFn<T1, C, E1, Q>,
    query: QueryFn<T3, E3, Q>,
) -> Box<ContractWrapper<T1, T2, T3, E1, E2, E3, C, Q>> {
    let contract: ContractWrapper<T1, T2, T3, E1, E2, E3, C, Q> =
        ContractWrapper::new(execute, instantiate, query);

    Box::new(contract)
}

pub fn create_code_with_reply<
    T1: DeserializeOwned + fmt::Debug + 'static,
    T2: DeserializeOwned + 'static,
    T3: DeserializeOwned + 'static,
    C: Clone + fmt::Debug + PartialEq + JsonSchema + 'static,
    E1: Display + fmt::Debug + Send + Sync + 'static,
    E2: Display + fmt::Debug + Send + Sync + 'static,
    E3: Display + fmt::Debug + Send + Sync + 'static,
    E4: Display + fmt::Debug + Send + Sync + 'static,
    Q: CustomQuery + DeserializeOwned + 'static,
>(
    instantiate: ContractFn<T2, C, E2, Q>,
    execute: ContractFn<T1, C, E1, Q>,
    query: QueryFn<T3, E3, Q>,
    reply: ReplyFn<C, E4, Q>,
) -> Box<ContractWrapper<T1, T2, T3, E1, E2, E3, C, Q, cosmwasm_std::Empty, anyhow::Error, E4>> {
    let contract: ContractWrapper<T1, T2, T3, E1, E2, E3, C, Q> =
        ContractWrapper::new(execute, instantiate, query);

    let contract: ContractWrapper<
        T1,
        T2,
        T3,
        E1,
        E2,
        E3,
        C,
        Q,
        cosmwasm_std::Empty,
        anyhow::Error,
        E4,
    > = contract.with_reply(reply);

    Box::new(contract)
}

pub trait Bech32AppExt {
    fn increase_time(&mut self, seconds: u64);
    fn mint(&mut self, minter: impl Into<String>, to: impl Into<String>, amount: impl MintAmount);
    fn qy_balance(
        &mut self,
        address: &Addr,
        asset: &AssetInfoPrecisioned,
    ) -> StdResult<AssetPrecisioned>;
}

impl Bech32AppExt for Bech32App {
    fn increase_time(&mut self, seconds: u64) {
        self.update_block(|block_info| block_info.time = block_info.time.plus_seconds(seconds))
    }

    fn mint(&mut self, minter: impl Into<String>, to: impl Into<String>, amount: impl MintAmount) {
        let asset = amount.get_asset();
        let amount = asset.amount_raw();
        match &asset.info() {
            cw_asset::AssetInfoBase::Native(denom) => {
                self.sudo(SudoMsg::Bank(BankSudo::Mint {
                    to_address: to.into(),
                    amount: vec![Coin::new(amount.u128(), denom)],
                }))
                .unwrap();
            }
            cw_asset::AssetInfoBase::Cw20(cw20) => {
                self.execute_contract(
                    Into::<String>::into(minter).into_unchecked_addr(),
                    cw20.clone(),
                    &cw20::Cw20ExecuteMsg::Mint {
                        recipient: to.into(),
                        amount,
                    },
                    &[],
                )
                .unwrap();
            }
            _ => todo!(),
        }
    }

    fn qy_balance(
        &mut self,
        address: &Addr,
        asset: &AssetInfoPrecisioned,
    ) -> StdResult<AssetPrecisioned> {
        let amount = asset
            .info
            .query_balance(&self.wrap(), address)
            .into_std_result()?;

        AssetPrecisioned::new(asset.clone(), amount.into()).wrap_ok()
    }
}

pub trait UnwrapError {
    type Error;
    fn unwrap_err_contains(self, text: impl Into<String>) -> Self::Error;
}

impl<T: Debug, E: Display> UnwrapError for Result<T, E> {
    type Error = E;

    fn unwrap_err_contains(self, text: impl Into<String>) -> Self::Error {
        let text: String = text.into();

        let err = match self {
            Ok(_) => panic!("Result is not error, error {text} not found"),
            Err(e) => e,
        };

        if format!("{:#}", err).contains(&text) {
            err
        } else {
            panic!("{text} not contained in {err:#}")
        }
    }
}

pub trait MintAmount {
    fn get_asset(&self) -> AssetPrecisioned;
}

impl MintAmount for AssetPrecisioned {
    fn get_asset(&self) -> AssetPrecisioned {
        self.clone()
    }
}

#[allow(suspicious_double_ref_op)]
impl MintAmount for &AssetPrecisioned {
    fn get_asset(&self) -> AssetPrecisioned {
        self.clone().clone()
    }
}

impl MintAmount for (AssetInfoPrecisioned, AssetAmount) {
    fn get_asset(&self) -> AssetPrecisioned {
        AssetPrecisioned::new(self.0.clone(), self.1.clone())
    }
}
