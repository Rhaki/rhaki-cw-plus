use cosmwasm_std::{Coin, StdError, StdResult};

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
