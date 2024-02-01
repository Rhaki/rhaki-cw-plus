use std::{collections::HashMap, error::Error, fmt::Debug, hash::Hash};

use cosmwasm_schema::cw_serde;
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

#[cw_serde]
pub enum UpdateOption<T> {
    ToNone,
    Some(T),
}

impl<T: Clone> UpdateOption<T> {
    pub fn into_option(&self) -> Option<T> {
        match self {
            UpdateOption::ToNone => None,
            UpdateOption::Some(t) => Some(t.clone()),
        }
    }

    pub fn unwrap(self) -> T {
        match self {
            UpdateOption::ToNone => panic!("Unwrap a None value"),
            UpdateOption::Some(val) => val,
        }
    }
}

#[allow(clippy::from_over_into)]
impl<T> Into<Option<T>> for UpdateOption<T> {
    fn into(self) -> Option<T> {
        match self {
            UpdateOption::ToNone => None,
            UpdateOption::Some(val) => Some(val),
        }
    }
}

pub trait WrapOk: Sized {
    /// Wrap `self` into `Ok(self)`
    fn wrap_ok<E: Error>(self) -> Result<Self, E> {
        Ok(self)
    }
}

impl<T> WrapOk for T {}

pub trait WrapOption: Sized {
    fn wrap_some(self) -> Option<Self> {
        Some(self)
    }
}

impl <T> WrapOption for T {}