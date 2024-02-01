use std::collections::HashMap;

use cosmwasm_schema::cw_serde;
use cosmwasm_std::{
    to_json_binary, Addr, BankMsg, Coin, CosmosMsg, Decimal, StdError, StdResult, Uint128, WasmMsg,
};
use cw_asset::{Asset, AssetInfo};
use serde::Serialize;

use crate::{
    traits::{IntoBinary, IntoStdResult},
    utils::WrapOk,
    wasm::WasmMsgBuilder,
};

/// Check if `coins` has a `len() == 1`.
/// If a `denom` is specified, assert them.
pub fn only_one_coin(coins: &Vec<Coin>, denom: Option<String>) -> StdResult<Coin> {
    if coins.len() == 1 {
        let coin = coins.first().unwrap().to_owned();
        match denom {
            Some(denom) => {
                if coin.denom == denom {
                    Ok(coin)
                } else {
                    Err(StdError::generic_err(format!(
                        "Denom not match, found: {}, expected: {}",
                        coin.denom, denom
                    )))
                }
            }
            None => Ok(coin),
        }
    } else {
        Err(StdError::generic_err("Not one coin"))
    }
}

/// Merge 2 `Coins`, checking if the `denom` is the same.
///
/// Cases:
/// - `from: None` - `with: None` -> Return `None`
/// - `from: Some` - `with: None` -> Return `from`
/// - `from: None` - `with: Some` -> Return `with`
/// - `from: Some` - `with: Some` -> Return `Coin:{denom: from.denom, amount: from.amount + with.amount}`
pub fn merge_coin(from: &Option<Coin>, with: &Option<Coin>) -> StdResult<Option<Coin>> {
    match from {
        Some(from) => match with {
            Some(with) => {
                if from.denom != with.denom {
                    return Err(StdError::generic_err("Coin must have same denom"));
                }
                Ok(Some(Coin {
                    denom: from.denom.clone(),
                    amount: from.amount + with.amount,
                }))
            }
            None => Ok(Some(from.to_owned())),
        },
        None => Ok(with.to_owned()),
    }
}

/// Transform a `Vec<Coin>` into `HashMap<String, Uint128>`
pub fn vec_coins_to_hashmap(coins: Vec<Coin>) -> StdResult<HashMap<String, Uint128>> {
    let mut map: HashMap<String, Uint128> = HashMap::new();

    for coin in coins {
        if map.contains_key(&coin.denom) {
            return Err(StdError::generic_err(format!(
                "multiple denom detected, {}",
                &coin.denom
            )));
        }
        map.insert(coin.denom, coin.amount);
    }

    Ok(map)
}

pub fn assets_multiple_transfer(
    assets: &[Asset],
    to: &Addr,
) -> StdResult<(Vec<CosmosMsg>, Vec<Coin>)> {
    let mut native: Vec<Coin> = vec![];
    let mut increase_allowance: Vec<CosmosMsg> = vec![];

    for asset in assets {
        match &asset.info {
            AssetInfo::Native(_) => native.push(asset.try_into().into_std_result()?),
            AssetInfo::Cw20(cw20_addr) => increase_allowance.push(
                WasmMsg::build_execute(
                    cw20_addr,
                    cw20::Cw20ExecuteMsg::IncreaseAllowance {
                        spender: to.to_string(),
                        amount: asset.amount,
                        expires: None,
                    },
                    vec![],
                )?
                .into(),
            ),
            _ => todo!(),
        }
    }

    Ok((increase_allowance, native))
}

#[allow(clippy::wrong_self_convention)]
pub trait AssetInfoExstender {
    fn into_send_msg(&self, receiver: &Addr, amount: Uint128) -> StdResult<CosmosMsg>;
}

impl AssetInfoExstender for AssetInfo {
    fn into_send_msg(&self, receiver: &Addr, amount: Uint128) -> StdResult<CosmosMsg> {
        match self {
            cw_asset::AssetInfoBase::Native(denom) => Ok(CosmosMsg::Bank(BankMsg::Send {
                to_address: receiver.to_string(),
                amount: vec![Coin::new(amount.u128(), denom)],
            })),
            cw_asset::AssetInfoBase::Cw20(contract_addr) => Ok(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: contract_addr.to_string(),
                msg: to_json_binary(&cw20::Cw20ExecuteMsg::Transfer {
                    recipient: receiver.to_string(),
                    amount,
                })?,
                funds: vec![],
            })),
            // ??
            _ => unimplemented!(),
        }
    }
}

#[cw_serde]
#[non_exhaustive]
pub struct AssetInfoPrecisioned {
    pub info: AssetInfo,
    pub precision: u8,
}

impl AssetInfoPrecisioned {
    pub fn new(info: AssetInfo, precision: u8) -> Self {
        Self { info, precision }
    }

    pub fn cw20(contract_addr: &Addr, precision: u8) -> Self {
        Self {
            info: AssetInfo::cw20(contract_addr.clone()),
            precision,
        }
    }

    pub fn native(denom: impl Into<String>, precision: u8) -> Self {
        Self {
            info: AssetInfo::native(denom.into()),
            precision,
        }
    }

    pub fn to_asset(&self, amount: AssetAmount) -> AssetPrecisioned {
        AssetPrecisioned::new(self.clone(), amount)
    }
}

impl Into<AssetInfo> for AssetInfoPrecisioned {
    fn into(self) -> AssetInfo {
        self.info
    }
}

#[cw_serde]
#[non_exhaustive]
pub struct AssetPrecisioned {
    info: AssetInfoPrecisioned,
    amount: Uint128,
}

impl AssetPrecisioned {
    pub fn new(info: AssetInfoPrecisioned, amount: AssetAmount) -> Self {
        Self {
            amount: amount.as_precisionless(info.precision),
            info,
        }
    }

    pub fn new_super(info: AssetInfo, precision: u8, amount: AssetAmount) -> Self {
        Self {
            info: AssetInfoPrecisioned::new(info, precision),
            amount: amount.as_precisionless(precision),
        }
    }

    pub fn amount_precisioned(&self) -> StdResult<Decimal> {
        Decimal::from_atomics(self.amount, self.info.precision as u32).into_std_result()
    }

    pub fn amount_raw(&self) -> Uint128 {
        self.amount
    }

    pub fn precision(&self) -> u8 {
        self.info.precision
    }

    pub fn info(&self) -> &AssetInfo {
        &self.info.info
    }

    pub fn info_precisioned(&self) -> &AssetInfoPrecisioned {
        &self.info
    }

    pub fn set_amount(&mut self, new_amount: AssetAmount) {
        self.amount = new_amount.as_precisionless(self.precision())
    }

    pub fn transfer_msg(&self, to: &Addr) -> StdResult<CosmosMsg> {
        Into::<Asset>::into(self.clone())
            .transfer_msg(to)
            .into_std_result()
    }

    pub fn send_msg<T: Serialize>(&self, contract_addr: &Addr, msg: T) -> StdResult<CosmosMsg> {
        match self.info() {
            AssetInfo::Native(_) => {
                WasmMsg::build_execute(contract_addr, msg, vec![self.clone().try_into()?])
            }
            AssetInfo::Cw20(addr) => WasmMsg::build_execute(
                addr,
                cw20::Cw20ExecuteMsg::Send {
                    contract: contract_addr.to_string(),
                    amount: self.amount,
                    msg: msg.into_binary()?,
                },
                vec![],
            ),
            _ => todo!(),
        }
        .map(|msg| msg.into())
    }
}

impl Into<Asset> for AssetPrecisioned {
    fn into(self) -> Asset {
        Asset::new(self.info().clone(), self.amount_raw())
    }
}

impl TryInto<Coin> for AssetPrecisioned {
    type Error = StdError;

    fn try_into(self) -> Result<Coin, Self::Error> {
        match self.info() {
            cw_asset::AssetInfoBase::Native(denom) => {
                Coin::new(self.amount_raw().u128(), denom).wrap_ok()
            }
            cw_asset::AssetInfoBase::Cw20(addr) => Err(StdError::generic_err(format!(
                "Cannot convert {} into Coin",
                addr
            ))),
            _ => todo!(),
        }
    }
}

#[cw_serde]
pub enum AssetAmount {
    Precisioned(Decimal),
    Precisionless(Uint128),
}

impl From<Uint128> for AssetAmount {
    fn from(value: Uint128) -> Self {
        Self::Precisionless(value)
    }
}

impl From<u128> for AssetAmount {
    fn from(value: u128) -> Self {
        Self::Precisionless(value.into())
    }
}

impl From<Decimal> for AssetAmount {
    fn from(value: Decimal) -> Self {
        Self::Precisioned(value)
    }
}

impl AssetAmount {
    pub fn as_precisioned(&self, precision: u8) -> StdResult<Decimal> {
        match self {
            AssetAmount::Precisioned(amount) => Ok(*amount),
            AssetAmount::Precisionless(amount) => {
                Decimal::from_atomics(*amount, precision as u32).into_std_result()
            }
        }
    }

    pub fn as_precisionless(&self, precision: u8) -> Uint128 {
        match self {
            AssetAmount::Precisioned(amount) => Uint128::from(10 as u32)
                .pow(precision as u32)
                .mul_floor(*amount),
            AssetAmount::Precisionless(amount) => *amount,
        }
    }
}