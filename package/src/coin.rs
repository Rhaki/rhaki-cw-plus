use std::collections::HashMap;

use cosmwasm_std::{
    to_binary, Addr, BankMsg, Coin, CosmosMsg, StdError, StdResult, Uint128, WasmMsg,
};
use cw_asset::AssetInfo;

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
}
