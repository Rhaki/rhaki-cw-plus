pub use anyhow;
use cw20::MinterResponse;
pub use cw_multi_test;

use std::{
    fmt::{self, Debug, Display},
    ops::Sub,
};

use cosmwasm_std::{
    testing::MockStorage, Addr, Api, Binary, Coin, CustomQuery, Deps, DepsMut, Empty, Env,
    MessageInfo, Reply, Response, StdResult, Storage,
};
use cw_multi_test::{
    addons::{MockAddressGenerator, MockApiBech32},
    error::AnyResult,
    no_init, App, AppBuilder, AppResponse, Bank, BankKeeper, BankSudo, ContractWrapper,
    Distribution, DistributionKeeper, Executor, FailingModule, Gov, GovFailingModule, Ibc,
    IbcFailingModule, Module, StakeKeeper, Staking, Stargate, StargateFailing, SudoMsg, Wasm,
    WasmKeeper,
};
use schemars::JsonSchema;
use serde::de::DeserializeOwned;

// use anyhow::Result as AnyResult;

use crate::{
    asset::{AssetInfoPrecisioned, AssetPrecisioned},
    math::IntoDecimal,
    traits::{IntoAddr, IntoStdResult, Wrapper},
};

pub type Bech32App = App<BankKeeper, MockApiBech32>;

pub type AppResult = AnyResult<AppResponse>;

pub type FailingCustom = FailingModule<Empty, Empty, Empty>;

pub type DefaultWasmKeeper = WasmKeeper<Empty, Empty>;

fn build_api(chain_prefix: &'static str) -> MockApiBech32 {
    MockApiBech32::new(chain_prefix)
}

fn build_wasm_keeper() -> WasmKeeper<Empty, Empty> {
    WasmKeeper::default().with_address_generator(MockAddressGenerator)
}

pub fn build_bech32_app(chain_prefix: &'static str) -> Bech32App {
    bench32_app_builder(chain_prefix).build(no_init)
}

pub fn bench32_app_builder(
    chain_prefix: &'static str,
) -> AppBuilder<
    BankKeeper,
    MockApiBech32,
    MockStorage,
    FailingCustom,
    DefaultWasmKeeper,
    StakeKeeper,
    DistributionKeeper,
    IbcFailingModule,
    GovFailingModule,
    StargateFailing,
> {
    AppBuilder::new()
        .with_api(build_api(chain_prefix))
        .with_wasm(build_wasm_keeper())
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

pub trait AppExt {
    fn increase_time(&mut self, seconds: u64);
    fn mint<A: Into<AssetPrecisioned>>(&mut self, to: impl Into<String>, amount: A);
    fn qy_balance(
        &mut self,
        address: &Addr,
        asset: &AssetInfoPrecisioned,
    ) -> StdResult<AssetPrecisioned>;
}

pub trait Bench32AppExt {
    fn generate_addr(&self, name: &str) -> Addr;
}

impl<BankT, ApiT, StorageT, CustomT, WasmT, StakingT, DistrT, IbcT, GovT, StargateT> AppExt
    for App<BankT, ApiT, StorageT, CustomT, WasmT, StakingT, DistrT, IbcT, GovT, StargateT>
where
    CustomT::ExecT: Debug + PartialEq + Clone + JsonSchema + DeserializeOwned + 'static,
    CustomT::QueryT: CustomQuery + DeserializeOwned + 'static,
    WasmT: Wasm<CustomT::ExecT, CustomT::QueryT>,
    BankT: Bank,
    ApiT: Api,
    StorageT: Storage,
    CustomT: Module,
    StakingT: Staking,
    DistrT: Distribution,
    IbcT: Ibc,
    GovT: Gov,
    StargateT: Stargate,
{
    fn increase_time(&mut self, seconds: u64) {
        self.update_block(|block_info| block_info.time = block_info.time.plus_seconds(seconds))
    }

    fn mint<A: Into<AssetPrecisioned>>(&mut self, to: impl Into<String>, asset: A) {
        let asset: AssetPrecisioned = asset.into();
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
                let minter = self
                    .wrap()
                    .query_wasm_smart::<MinterResponse>(cw20, &cw20::Cw20QueryMsg::Minter {})
                    .unwrap()
                    .minter;

                self.execute_contract(
                    minter.into_unchecked_addr(),
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
        match &asset.info {
            cw_asset::AssetInfoBase::Native(denom) => {
                let amount = self
                    .wrap()
                    .query_balance(address, denom)
                    .into_std_result()?;
                AssetPrecisioned::new(asset.clone(), amount.amount).wrap_ok()
            }
            cw_asset::AssetInfoBase::Cw20(cw20) => {
                let amount = self
                    .wrap()
                    .query_wasm_smart::<cw20::BalanceResponse>(
                        cw20,
                        &cw20::Cw20QueryMsg::Balance {
                            address: address.into(),
                        },
                    )
                    .into_std_result()?;

                AssetPrecisioned::new(asset.clone(), amount.balance).wrap_ok()
            }
            _ => todo!(),
        }
    }
}

impl<BankT, StorageT, CustomT, WasmT, StakingT, DistrT, IbcT, GovT, StargateT> Bench32AppExt
    for App<BankT, MockApiBech32, StorageT, CustomT, WasmT, StakingT, DistrT, IbcT, GovT, StargateT>
where
    CustomT::ExecT: Debug + PartialEq + Clone + JsonSchema + DeserializeOwned + 'static,
    CustomT::QueryT: CustomQuery + DeserializeOwned + 'static,
    WasmT: Wasm<CustomT::ExecT, CustomT::QueryT>,
    BankT: Bank,
    StorageT: Storage,
    CustomT: Module,
    StakingT: Staking,
    DistrT: Distribution,
    IbcT: Ibc,
    GovT: Gov,
    StargateT: Stargate,
{
    fn generate_addr(&self, name: &str) -> Addr {
        self.api().addr_make(name)
    }
}

pub fn assert_with_tollerance<T>(val1: T, val2: T, delta: T)
where
    T: PartialEq + PartialOrd + Sub + Display + Clone,
    <T as std::ops::Sub>::Output: PartialOrd<T> + Display,
{
    if val1 > val2 {
        assert!(val1.clone() - val2.clone() <= delta, "{} - {} <= {}", val1, val2, delta);
    } else {
        assert!(val2.clone() - val1.clone() <= delta, "{} - {} <= {}", val2, val1, delta);
    }
}

#[test]
fn test() {
    let a = "10".into_decimal();
    let b = "10.1".into_decimal();
    assert_with_tollerance(a, b, "2".into_decimal());
}
