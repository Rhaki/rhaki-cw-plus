mod map {
    use cosmwasm_std::{testing::mock_dependencies, Order};
    use cw_storage_plus::Map;
    use rhaki_cw_plus::storage::map::get_items;

    #[test]
    fn main() {
        let mut deps = mock_dependencies();

        let map: Map<u64, String> = Map::new("test_map");

        for i in 1_u64..100_u64 {
            map.save(deps.as_mut().storage, i, &i.to_string()).unwrap();
        }

        let first_values =
            get_items(deps.as_mut().storage, &map, Order::Ascending, None, None).unwrap();

        assert_eq!(
            first_values,
            vec![
                (1, "1".to_string()),
                (2, "2".to_string()),
                (3, "3".to_string()),
                (4, "4".to_string()),
                (5, "5".to_string()),
                (6, "6".to_string()),
                (7, "7".to_string()),
                (8, "8".to_string()),
                (9, "9".to_string()),
                (10, "10".to_string()),
            ]
        );

        let first_values = get_items(
            deps.as_mut().storage,
            &map,
            Order::Ascending,
            Some(5),
            Some(first_values.last().unwrap().0),
        )
        .unwrap();

        assert_eq!(
            first_values,
            vec![
                (11, "11".to_string()),
                (12, "12".to_string()),
                (13, "13".to_string()),
                (14, "14".to_string()),
                (15, "15".to_string()),
            ]
        );

        let last_values =
            get_items(deps.as_mut().storage, &map, Order::Descending, None, None).unwrap();

        assert_eq!(
            last_values,
            vec![
                (99, "99".to_string()),
                (98, "98".to_string()),
                (97, "97".to_string()),
                (96, "96".to_string()),
                (95, "95".to_string()),
                (94, "94".to_string()),
                (93, "93".to_string()),
                (92, "92".to_string()),
                (91, "91".to_string()),
                (90, "90".to_string()),
            ]
        );

        let last_values = get_items(
            deps.as_mut().storage,
            &map,
            Order::Descending,
            Some(3),
            Some(last_values.last().unwrap().0),
        )
        .unwrap();

        assert_eq!(
            last_values,
            vec![
                (89, "89".to_string()),
                (88, "88".to_string()),
                (87, "87".to_string()),
            ]
        );
    }
}
mod test_multi_index {
    use cosmwasm_schema::cw_serde;
    use cosmwasm_std::{testing::mock_dependencies, Order};
    use cw_storage_plus::{IndexedMap, MultiIndex, UniqueIndex, index_list};
    use rhaki_cw_plus::storage::multi_index::{get_multi_index_values, get_unique_value};

    #[cw_serde]
    pub struct ChainInfo {
        pub src_channel: String,
        pub dest_channel: String,
        pub enable: u64,
    }

    #[index_list(ChainInfo)]
    pub struct ChainInfoIndexes<'a> {
        pub src_channel: UniqueIndex<'a, String, ChainInfo, String>,
        pub dest_channel: UniqueIndex<'a, String, ChainInfo, String>,
        pub src_channel_dest_channel: UniqueIndex<'a, (String, String), ChainInfo, String>,
        pub enable: MultiIndex<'a, u64, ChainInfo, String>,
    }

    #[allow(non_snake_case)]
    pub fn CHAINS<'a>() -> IndexedMap<'a, String, ChainInfo, ChainInfoIndexes<'a>> {
        let indexes = ChainInfoIndexes {
            src_channel: UniqueIndex::new(
                |chain| chain.src_channel.clone(),
                "ns_chains_src_channel",
            ),
            dest_channel: UniqueIndex::new(
                |chain| chain.dest_channel.clone(),
                "ns_chains_dest_channel",
            ),
            src_channel_dest_channel: UniqueIndex::new(
                |chain| (chain.src_channel.clone(), chain.dest_channel.clone()),
                "ns_chains_src_channel_dest_channel",
            ),
            enable: MultiIndex::new(|_, chain| chain.enable, "ns_chains", "ns_chains_enable"),
        };
        IndexedMap::new("ns_chains", indexes)
    }

    #[test]
    fn main() {
        let mut deps = mock_dependencies();
        CHAINS()
            .save(
                deps.as_mut().storage,
                "terra".to_string(),
                &ChainInfo {
                    src_channel: "channel_0".to_string(),
                    dest_channel: "channel_1".to_string(),
                    enable: 0,
                },
            )
            .unwrap();
        CHAINS()
            .save(
                deps.as_mut().storage,
                "juno".to_string(),
                &ChainInfo {
                    src_channel: "channel_2".to_string(),
                    dest_channel: "channel_3".to_string(),
                    enable: 0,
                },
            )
            .unwrap();
        CHAINS()
            .save(
                deps.as_mut().storage,
                "injective".to_string(),
                &ChainInfo {
                    src_channel: "channel_2".to_string(),
                    dest_channel: "channel_5".to_string(),
                    enable: 0,
                },
            )
            .unwrap_err();
        CHAINS()
            .save(
                deps.as_mut().storage,
                "injective".to_string(),
                &ChainInfo {
                    src_channel: "channel_4".to_string(),
                    dest_channel: "channel_5".to_string(),
                    enable: 0,
                },
            )
            .unwrap();
        CHAINS()
            .save(
                deps.as_mut().storage,
                "osmosis".to_string(),
                &ChainInfo {
                    src_channel: "channel_6".to_string(),
                    dest_channel: "channel_7".to_string(),
                    enable: 1,
                },
            )
            .unwrap();

        let res = get_unique_value(
            deps.as_ref().storage,
            "channel_2".to_string(),
            CHAINS().idx.src_channel,
        )
        .unwrap();

        assert_eq!(
            res,
            (
                "juno".to_string(),
                ChainInfo {
                    src_channel: "channel_2".to_string(),
                    dest_channel: "channel_3".to_string(),
                    enable: 0
                }
            )
        );

        let res = get_multi_index_values(
            deps.as_ref().storage,
            0,
            CHAINS().idx.enable,
            Order::Descending,
            None,
            Some(1),
        )
        .unwrap();

        assert_eq!(
            res,
            vec![(
                "terra".to_string(),
                ChainInfo {
                    src_channel: "channel_0".to_string(),
                    dest_channel: "channel_1".to_string(),
                    enable: 0
                }
            )]
        );

        let res = get_multi_index_values(
            deps.as_ref().storage,
            0,
            CHAINS().idx.enable,
            Order::Descending,
            Some("terra".to_string()),
            Some(1),
        )
        .unwrap();

        assert_eq!(
            res,
            vec![(
                "juno".to_string(),
                ChainInfo {
                    src_channel: "channel_2".to_string(),
                    dest_channel: "channel_3".to_string(),
                    enable: 0
                }
            )]
        );

        let res = get_multi_index_values(
            deps.as_ref().storage,
            0,
            CHAINS().idx.enable,
            Order::Descending,
            Some("terra".to_string()),
            None,
        )
        .unwrap();

        assert_eq!(
            res,
            vec![
                (
                    "juno".to_string(),
                    ChainInfo {
                        src_channel: "channel_2".to_string(),
                        dest_channel: "channel_3".to_string(),
                        enable: 0
                    }
                ),
                (
                    "injective".to_string(),
                    ChainInfo {
                        src_channel: "channel_4".to_string(),
                        dest_channel: "channel_5".to_string(),
                        enable: 0
                    }
                )
            ]
        );

        let res = get_multi_index_values(
            deps.as_ref().storage,
            0,
            CHAINS().idx.enable,
            Order::Ascending,
            None,
            Some(1),
        )
        .unwrap();

        assert_eq!(
            res,
            vec![(
                "injective".to_string(),
                ChainInfo {
                    src_channel: "channel_4".to_string(),
                    dest_channel: "channel_5".to_string(),
                    enable: 0
                }
            )]
        );

        let res = get_multi_index_values(
            deps.as_ref().storage,
            0,
            CHAINS().idx.enable,
            Order::Ascending,
            Some("injective".to_string()),
            Some(1),
        )
        .unwrap();

        assert_eq!(
            res,
            vec![(
                "juno".to_string(),
                ChainInfo {
                    src_channel: "channel_2".to_string(),
                    dest_channel: "channel_3".to_string(),
                    enable: 0
                }
            )]
        );

        let res = get_multi_index_values(
            deps.as_ref().storage,
            0,
            CHAINS().idx.enable,
            Order::Ascending,
            Some("juno".to_string()),
            None,
        )
        .unwrap();

        assert_eq!(
            res,
            vec![(
                "terra".to_string(),
                ChainInfo {
                    src_channel: "channel_0".to_string(),
                    dest_channel: "channel_1".to_string(),
                    enable: 0
                }
            )]
        );
    }
}
