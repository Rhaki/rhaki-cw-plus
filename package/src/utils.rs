use std::{collections::HashMap, fmt::Debug, hash::Hash};

use cosmwasm_std::{StdError, StdResult};

/// Transform a `Vec<T>` into `HashMap<uszie, T>`, where `key` is the index of `T`
pub fn vec_to_i_hashmap<T>(vec: Vec<T>) -> HashMap<usize, T> {
    let mut map: HashMap<usize, T> = HashMap::new();

    for (i, v) in vec.into_iter().enumerate() {
        map.insert(i, v);
    }

    map
}

/// Transform a `Vec<(K, V)>` into `HashMap<K, V>`
pub fn vec_tuple_to_hashmap<K: Eq + Hash + Debug + Clone, V>(
    vec: Vec<(K, V)>,
) -> StdResult<HashMap<K, V>> {
    let mut map: HashMap<K, V> = HashMap::new();

    for (k, v) in vec {
        if map.insert(k.clone(), v).is_some() {
            return Err(StdError::generic_err(format!("Key alredy inserted, {k:?}")));
        };
    }

    Ok(map)
}
