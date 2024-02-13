use std::{cmp::min, fmt::Debug};

use cosmwasm_schema::{
    cw_serde,
    serde::{de::DeserializeOwned, Serialize},
};
use cosmwasm_std::{Order, StdError, StdResult, Storage};
use cw_storage_plus::{Bound, KeyDeserialize, Map, MultiIndex, Prefixer, PrimaryKey, UniqueIndex};

const DEFAULT_LIMIT: u32 = 10;
const MAX_LIMIT: u32 = 30;

pub mod map {

    use super::*;

    /// Load the values of a `cw_storage_plus::Map`, ordered by `Order::Ascending` or `Order::Descending`
    pub fn get_items<
        'a,
        T: Serialize + DeserializeOwned,
        K: PrimaryKey<'a> + KeyDeserialize + 'static,
    >(
        storage: &dyn Storage,
        map: &Map<'a, K, T>,
        order: Order,
        limit: Option<u32>,
        start_after: Option<K>,
    ) -> StdResult<Vec<(K::Output, T)>> {
        let (min_b, max_b) = min_max_from_order(start_after, &order);

        Ok(map
            .range(storage, min_b, max_b, order)
            .take(min(MAX_LIMIT, limit.unwrap_or(DEFAULT_LIMIT)) as usize)
            .map(|item| item.unwrap())
            .collect())
    }
}

pub mod multi_index {
    use std::marker::PhantomData;

    use cw_storage_plus::{IndexList, IndexedMap};

    use super::*;

    /// Load the values of a `cw_storage_plus::IndexMap`, ordered by `Order::Ascending` or `Order::Descending`
    pub fn get_items<
        'a,
        T: Serialize + DeserializeOwned + Clone,
        K: PrimaryKey<'a> + KeyDeserialize + 'static,
        I: IndexList<T>,
        R,
        F: Fn(<K as KeyDeserialize>::Output, T, PhantomData<K>) -> R,
    >(
        storage: &dyn Storage,
        index: IndexedMap<'a, K, T, I>,
        order: Order,
        limit: Option<u32>,
        start_after: Option<K>,
        map_fn: F,
    ) -> StdResult<Vec<R>> {
        let (min_b, max_b) = min_max_from_order(start_after, &order);

        index
            .range(storage, min_b, max_b, order)
            .take(min(MAX_LIMIT, limit.unwrap_or(DEFAULT_LIMIT)) as usize)
            .map(|item| item.map(|val| map_fn(val.0, val.1, PhantomData)))
            .collect()
    }

    // --- Unique ---

    /// Load the value linked to a `UniqueIndex`.
    ///
    /// The variable `map_fn` map the result before returning it. If no custom mapping is require use the default provided:
    ///
    /// - [unique_map_key] for only get only the Primary key
    /// - [unique_map_value] for get only the value
    /// - [unique_map_default] for get both `(primary_key, value)`
    ///  
    ///
    /// ## Example:
    /// ```ignore
    /// #[cw_serde]
    /// pub struct ChainInfo{
    ///     pub src_channel: String,
    ///     pub dest_channel: String
    /// }
    ///
    /// #[index_list(ChainInfo)]
    /// pub struct ChainInfoIndexes<'a>{
    ///     pub src_channel: UniqueIndex<'a, String, ChainInfo, String>,
    /// }
    ///
    /// pub fn chains<'a>() -> IndexedMap<'a, String, ChainInfo, ChainInfoIndexes<'a>> {
    ///     let indexes = ChainInfoIndexes {
    ///         src_channel: UniqueIndex::new(
    ///             |chain| {chain.src_channel.clone()},
    ///             "ns_chains_src_channel"),
    ///     }
    ///     IndexedMap::new("ns_chains", indexes)
    /// }
    ///
    /// let (main_key: String, value: ChainInfo) = get_unique_value(
    ///     storage,
    ///     "channel_0",
    ///     index_map().idx.unique_index,
    ///     uv_default
    ///     )?;
    /// ```
    pub fn get_unique_value<
        'a,
        IK: PrimaryKey<'a> + Debug,
        T: Serialize + DeserializeOwned + Clone,
        PK: KeyDeserialize,
        R,
        F: Fn(Vec<u8>, T, PhantomData<PK>) -> R,
    >(
        storage: &dyn Storage,
        key: IK,
        index: UniqueIndex<'a, IK, T, PK>,
        map_fn: F,
    ) -> StdResult<R> {
        match index.item(storage, key.clone())? {
            Some((k, v)) => Ok(map_fn(k, v, PhantomData::<PK>)),
            None => Err(StdError::generic_err(format!("Key not found {key:?}"))),
        }
    }

    pub fn unique_map_key<PK: KeyDeserialize, T: Serialize + DeserializeOwned + Clone>(
        key: Vec<u8>,
        _value: T,
        _: PhantomData<PK>,
    ) -> StdResult<PK::Output> {
        PK::from_vec(key)
    }

    pub fn unique_map_value<T: Serialize + DeserializeOwned + Clone, PK: KeyDeserialize>(
        _key: Vec<u8>,
        value: T,
        _: PhantomData<PK>,
    ) -> T {
        value
    }

    pub fn unique_map_default<PK: KeyDeserialize, T: Serialize + DeserializeOwned + Clone>(
        key: Vec<u8>,
        value: T,
        _: PhantomData<PK>,
    ) -> StdResult<(PK::Output, T)> {
        PK::from_vec(key).map(|key| (key, value))
    }

    // --- MULTI ---

    /// Load the values of a `cw_storage_plus::IndexMap` of a sub `MultiIndex`, ordered by `Order::Ascending` or `Order::Descending`
    /// The variable `map_fn` map the result before returning it. If no custom mapping is require use the default provided:
    ///
    /// - [multi_map_key] for only get only the Primary key
    /// - [multi_map_value] for get only the value
    /// - [multi_map_default] for get both `(primary_key, value)
    pub fn get_multi_index_values<
        'a,
        IK: PrimaryKey<'a> + Prefixer<'a>,
        T: Serialize + DeserializeOwned + Clone,
        PK: PrimaryKey<'a> + KeyDeserialize + 'static,
        R,
        F: Fn(<PK as KeyDeserialize>::Output, T, PhantomData<PK>) -> R,
    >(
        storage: &dyn Storage,
        key: IK,
        index: MultiIndex<'a, IK, T, PK>,
        order: Order,
        start_after: Option<PK>,
        limit: Option<u32>,
        map_fn: F,
    ) -> StdResult<Vec<R>> {
        let (min_b, max_b) = min_max_from_order(start_after, &order);

        index
            .prefix(key)
            .range(storage, min_b, max_b, order)
            .take((min(MAX_LIMIT, limit.unwrap_or(DEFAULT_LIMIT))) as usize)
            .map(|item| item.map(|val| map_fn(val.0, val.1, PhantomData)))
            .collect()
    }

    pub fn multi_map_value<T: Serialize + DeserializeOwned + Clone, PK: KeyDeserialize>(
        _key: PK::Output,
        value: T,
        _phanton: PhantomData<PK>,
    ) -> T {
        value
    }

    pub fn multi_map_key<T: Serialize + DeserializeOwned + Clone, PK: KeyDeserialize>(
        key: PK::Output,
        _value: T,
        _phanton: PhantomData<PK>,
    ) -> PK::Output {
        key
    }

    pub fn multi_map_default<T: Serialize + DeserializeOwned + Clone, PK: KeyDeserialize>(
        key: PK::Output,
        value: T,
        _phanton: PhantomData<PK>,
    ) -> (PK::Output, T) {
        (key, value)
    }
}

pub mod interfaces {
    use std::fmt::Display;

    use cosmwasm_std::{StdError, StdResult, Storage};
    use cw_storage_plus::{Item, Map, PrimaryKey};
    use serde::{de::DeserializeOwned, Serialize};

    pub trait ItemInterface: Sized + Serialize + DeserializeOwned {
        const NAMESPACE: &'static str;
        const CONTRACT_NAME: &'static str;

        fn load(storage: &dyn Storage) -> StdResult<Self> {
            Self::item().load(storage).map_err(|_| {
                StdError::generic_err(format!(
                    "Item {} on contract {} can't be loaded",
                    Self::NAMESPACE,
                    Self::CONTRACT_NAME
                ))
            })
        }

        fn save(&self, storage: &mut dyn Storage) -> StdResult<()> {
            Self::item().save(storage, &self)
        }

        fn item<'a>() -> Item<'a, Self> {
            Item::new(Self::NAMESPACE)
        }
    }

    pub trait MapExt {
        type K;
        type V;
        fn better_load(&self, storage: &dyn Storage, key: Self::K) -> StdResult<Self::V>;
    }

    impl<'a, K, V> MapExt for Map<'a, K, V>
    where
        V: Serialize + DeserializeOwned + Clone,
        K: PrimaryKey<'a> + Display + Clone,
    {
        type K = K;
        type V = V;

        fn better_load(&self, storage: &dyn Storage, key: Self::K) -> StdResult<Self::V> {
            self.load(storage, key.clone()).map_err(|_| {
                StdError::generic_err(format!(
                    "Unable to load key {} on Map with namespace {}",
                    key,
                    String::from_utf8_lossy(self.namespace())
                ))
            })
        }
    }
}

fn min_max_from_order<'a, PK: PrimaryKey<'a> + KeyDeserialize + 'static>(
    start_after: Option<PK>,
    order: &Order,
) -> (Option<Bound<'a, PK>>, Option<Bound<'a, PK>>) {
    match order {
        Order::Ascending => (start_after.map(Bound::exclusive), None),
        Order::Descending => (None, start_after.map(Bound::exclusive)),
    }
}

#[cw_serde]
pub enum StorageOrder {
    Ascending,
    Descending,
}

impl Into<Order> for StorageOrder {
    fn into(self) -> Order {
        match self {
            StorageOrder::Ascending => Order::Ascending,
            StorageOrder::Descending => Order::Descending,
        }
    }
}

#[cfg(test)]
mod test {
    use cosmwasm_schema::cw_serde;
    use cosmwasm_std::{testing::mock_dependencies, Order, StdError, StdResult};
    use cw_storage_plus::Map;

    use crate::storage::interfaces::MapExt;

    use super::interfaces::ItemInterface;

    #[cw_serde]
    pub struct TestConfig {
        some_value: String,
        another_value: u64,
    }

    impl ItemInterface for TestConfig {
        const NAMESPACE: &'static str = "config";
        const CONTRACT_NAME: &'static str = "test_contract";
    }

    pub const MAP: Map<&str, u64> = Map::new("map_namespace");

    #[test]
    fn test_prefix() {
        let map: Map<(u64, u64), String> = Map::new("map");

        let mut deps = mock_dependencies();

        map.save(deps.as_mut().storage, (1, 1), &"1-1".to_string())
            .unwrap();
        map.save(deps.as_mut().storage, (1, 2), &"1-2".to_string())
            .unwrap();
        map.save(deps.as_mut().storage, (1, 3), &"1-3".to_string())
            .unwrap();
        map.save(deps.as_mut().storage, (2, 1), &"2-1".to_string())
            .unwrap();
        map.save(deps.as_mut().storage, (2, 2), &"2-2".to_string())
            .unwrap();
        map.save(deps.as_mut().storage, (2, 3), &"2-3".to_string())
            .unwrap();

        let res = map
            .prefix(1)
            .range(deps.as_ref().storage, None, None, Order::Ascending)
            .collect::<StdResult<Vec<(u64, String)>>>()
            .unwrap();

        println!("{res:#?}")
    }

    #[test]
    fn test_intefrace() {
        let mut deps = mock_dependencies();

        let config = TestConfig {
            some_value: "foo".to_string(),
            another_value: 0,
        };

        config.save(deps.as_mut().storage).unwrap();

        let mut config = TestConfig::load(deps.as_ref().storage).unwrap();

        config.some_value = "bar".to_string();
        config.another_value = 1;

        config.save(deps.as_mut().storage).unwrap();

        let config = TestConfig::load(deps.as_ref().storage).unwrap();

        assert_eq!(config.some_value, "bar");
        assert_eq!(config.another_value, 1);

        MAP.save(deps.as_mut().storage, "some_key", &1).unwrap();

        // "Unable to load key another_key on Map with namespace map_namespace"

        let err = MAP
            .better_load(deps.as_ref().storage, &"another_key")
            .unwrap_err();

        assert_eq!(
            err,
            StdError::generic_err(
                "Unable to load key another_key on Map with namespace map_namespace"
            )
        )
    }
}
