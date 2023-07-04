use std::cmp::min;

use cosmwasm_schema::serde::{de::DeserializeOwned, Serialize};
use cosmwasm_std::{Order, StdResult, Storage};
use cw_storage_plus::{Bound, KeyDeserialize, Map, PrimaryKey};

const DEFAULT_LIMIT: u64 = 10;
const MAX_LIMIT: u64 = 30;

/// Return the `first` key of a `cw_storage_plus::map`, ordered by `Order::Ascending`
pub fn get_first_key<
    'a,
    T: Serialize + DeserializeOwned,
    K: PrimaryKey<'a> + KeyDeserialize + 'static,
>(
    storage: &dyn Storage,
    map: &Map<'a, K, T>,
) -> Option<K::Output> {
    map.range(storage, None, None, Order::Ascending)
        .take(1)
        .last()
        .map(|v| v.unwrap().0)
}

/// Return the `last` key of a `cw_storage_plus::map`, ordered by `Order::Ascending`
pub fn get_last_key<
    'a,
    T: Serialize + DeserializeOwned,
    K: PrimaryKey<'a> + KeyDeserialize + 'static,
>(
    storage: &dyn Storage,
    map: &Map<'a, K, T>,
) -> Option<K::Output> {
    map.range(storage, None, None, Order::Descending)
        .take(1)
        .last()
        .map(|v| v.unwrap().0)
}

/// Return the `first` values of a `cw_storage_plus::map`, ordered by `Order::Ascending`
pub fn get_first_values<
    'a,
    T: Serialize + DeserializeOwned,
    K: PrimaryKey<'a> + KeyDeserialize + 'static,
>(
    storage: &dyn Storage,
    map: &Map<'a, K, T>,
    limit: Option<u64>,
    start_after: Option<K>,
) -> StdResult<Vec<(K::Output, T)>> {
    Ok(map
        .range(
            storage,
            start_after.map(Bound::exclusive),
            None,
            Order::Ascending,
        )
        .take(usize::try_from(min(MAX_LIMIT, limit.unwrap_or(DEFAULT_LIMIT))).unwrap())
        .map(|item| item.unwrap())
        .collect())
}

/// Return the `last` values of a `cw_storage_plus::map`, ordered by `Order::Ascending`
pub fn get_last_values<
    'a,
    T: Serialize + DeserializeOwned,
    K: PrimaryKey<'a> + KeyDeserialize + 'static,
>(
    storage: &dyn Storage,
    map: &Map<'a, K, T>,
    limit: Option<u64>,
    start_after: Option<K>,
) -> StdResult<Vec<(K::Output, T)>> {
    Ok(map
        .range(
            storage,
            None,
            start_after.map(Bound::exclusive),
            Order::Descending,
        )
        .take(usize::try_from(min(MAX_LIMIT, limit.unwrap_or(DEFAULT_LIMIT))).unwrap())
        .map(|item| item.unwrap())
        .collect())
}
