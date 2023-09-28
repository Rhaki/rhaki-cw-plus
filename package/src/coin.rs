use std::collections::HashMap;

use cosmwasm_std::{Coin, StdError, StdResult, Uint128, Env, CosmosMsg, to_binary, WasmMsg, Addr, QuerierWrapper, BankMsg};
use cw20::TokenInfoResponse;
use cw_asset::AssetInfo;
use osmosis_std::types::{osmosis::tokenfactory::v1beta1::{MsgMint, MsgBurn}, cosmos::bank::v1beta1::BankQuerier};

use crate::{traits::IntoStdResult, math::IntoUint};

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


pub trait UnwrapBase {
    type Output;
    fn unwrap_base(&self) -> Self::Output;
}

impl UnwrapBase for AssetInfo {
    type Output = String;
    fn unwrap_base(&self) -> String {
        match self {
            cw_asset::AssetInfoBase::Native(denom) => denom.clone(),
            cw_asset::AssetInfoBase::Cw20(addr) => addr.to_string(),
            _ => todo!(),
        }
    }
}

#[allow(clippy::wrong_self_convention)]
pub trait AssetInfoExstender {
    fn into_send_msg(&self, receiver: &Addr, amount: Uint128) -> StdResult<CosmosMsg>;
    fn into_mint_msg(&self, receiver: &Addr, env: &Env, amount: Uint128) -> StdResult<CosmosMsg>;
    fn into_burn_msg(&self, env: &Env, amount: Uint128) -> StdResult<CosmosMsg>;
    fn get_supply(&self, query: &QuerierWrapper) -> StdResult<Uint128>;
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
                msg: to_binary(&cw20::Cw20ExecuteMsg::Transfer {
                    recipient: receiver.to_string(),
                    amount,
                })?,
                funds: vec![],
            })),
            // ??
            _ => unimplemented!(),
        }
    }

    fn into_mint_msg(&self, receiver: &Addr, env: &Env, amount: Uint128) -> StdResult<CosmosMsg> {
        match self {
            cw_asset::AssetInfoBase::Native(denom) => Ok(MsgMint {
                sender: env.contract.address.to_string(),
                amount: Some(Coin::new(amount.u128(), denom).into()),
                mint_to_address: receiver.to_string(),
            }
            .into()),
            cw_asset::AssetInfoBase::Cw20(contract_addr) => Ok(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: contract_addr.to_string(),
                msg: to_binary(&cw20::Cw20ExecuteMsg::Mint {
                    recipient: receiver.to_string(),
                    amount,
                })?,
                funds: vec![],
            })),
            // ??
            _ => unimplemented!(),
        }
    }

    fn into_burn_msg(&self, env: &Env, amount: Uint128) -> StdResult<CosmosMsg> {
        match self {
            cw_asset::AssetInfoBase::Native(denom) => Ok(MsgBurn {
                sender: env.contract.address.to_string(),
                amount: Some(Coin::new(amount.u128(), denom).into()),
                burn_from_address: env.contract.address.to_string(),
            }
            .into()),
            cw_asset::AssetInfoBase::Cw20(contract_addr) => Ok(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: contract_addr.to_string(),
                msg: to_binary(&cw20::Cw20ExecuteMsg::Burn { amount })?,
                funds: vec![],
            })),
            // ??
            _ => unimplemented!(),
        }
    }

    fn get_supply(&self, querier: &QuerierWrapper) -> StdResult<Uint128> {
        match self {
            cw_asset::AssetInfoBase::Native(denom) => {
                let bank_querier = BankQuerier::new(querier);
                let supply = bank_querier
                    .supply_of(denom.clone())
                    .into_std_result()?
                    .amount
                    .unwrap()
                    .amount;

                supply.try_into_uint128()
            }
            cw_asset::AssetInfoBase::Cw20(contract_token) => Ok(querier
                .query_wasm_smart::<TokenInfoResponse>(
                    contract_token,
                    &cw20::Cw20QueryMsg::TokenInfo {},
                )?
                .total_supply),
            // ??
            _ => unimplemented!(),
        }
    }
}
