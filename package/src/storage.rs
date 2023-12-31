use std::{cmp::min, fmt::Debug};

use cosmwasm_schema::serde::{de::DeserializeOwned, Serialize};
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
    use cw_storage_plus::{IndexList, IndexedMap};

    use super::*;

    /// Load the values of a `cw_storage_plus::IndexMap`, ordered by `Order::Ascending` or `Order::Descending`
    pub fn get_items<
        'a,
        T: Serialize + DeserializeOwned + Clone,
        K: PrimaryKey<'a> + KeyDeserialize + 'static,
        I: IndexList<T>,
    >(
        storage: &dyn Storage,
        index: IndexedMap<'a, K, T, I>,
        order: Order,
        limit: Option<u32>,
        start_after: Option<K>,
    ) -> StdResult<Vec<(K::Output, T)>> {
        let (min_b, max_b) = min_max_from_order(start_after, &order);

        Ok(index
            .range(storage, min_b, max_b, order)
            .take(min(MAX_LIMIT, limit.unwrap_or(DEFAULT_LIMIT)) as usize)
            .map(|item| item.unwrap())
            .collect())
    }

    /// Load the value linked to a `UniqueIndex`.
    ///
    /// Return `(main_key, value)`
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
    ///     index_map().idx.unique_index
    ///     )?;
    /// ```
    pub fn get_unique_value<
        'a,
        IK: PrimaryKey<'a> + Debug,
        T: Serialize + DeserializeOwned + Clone,
        PK: KeyDeserialize,
    >(
        storage: &dyn Storage,
        key: IK,
        index: UniqueIndex<'a, IK, T, PK>,
    ) -> StdResult<(PK::Output, T)> {
        match index.item(storage, key.clone())? {
            Some((k, v)) => Ok((PK::from_vec(k)?, v)),
            None => Err(StdError::generic_err(format!("Key not found {key:?}"))),
        }
    }

    /// Load the values of a `cw_storage_plus::IndexMap` of a sub `MultiIndex`, ordered by `Order::Ascending` or `Order::Descending`
    pub fn get_multi_index_values<
        'a,
        IK: PrimaryKey<'a> + Prefixer<'a>,
        T: Serialize + DeserializeOwned + Clone,
        PK: PrimaryKey<'a> + KeyDeserialize + 'static,
    >(
        storage: &dyn Storage,
        key: IK,
        index: MultiIndex<'a, IK, T, PK>,
        order: Order,
        start_after: Option<PK>,
        limit: Option<u32>,
    ) -> StdResult<Vec<(PK::Output, T)>> {
        let (min_b, max_b) = min_max_from_order(start_after, &order);

        Ok(index
            .prefix(key)
            .range(storage, min_b, max_b, order)
            .take((min(MAX_LIMIT, limit.unwrap_or(DEFAULT_LIMIT))) as usize)
            .map(|item| item.unwrap())
            .collect())
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

#[cfg(test)]
mod test {
    use cosmwasm_std::{testing::mock_dependencies, Order, StdResult};
    use cw_storage_plus::Map;

    #[test]
    pub fn test_prefix() {
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
}
