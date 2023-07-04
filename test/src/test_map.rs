use cosmwasm_std::testing::mock_dependencies;
use cw_storage_plus::Map;
use rhaki_cw_plus::map::{get_first_key, get_first_values, get_last_key, get_last_values};

#[test]
fn main() {
    let mut deps = mock_dependencies();

    let map: Map<u64, String> = Map::new("test_map");

    for i in 1_u64..100_u64 {
        map.save(deps.as_mut().storage, i, &i.to_string()).unwrap();
    }

    let first_values = get_first_values(deps.as_mut().storage, &map, None, None).unwrap();

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

    let first_values = get_first_values(
        deps.as_mut().storage,
        &map,
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

    let last_values = get_last_values(deps.as_mut().storage, &map, None, None).unwrap();

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

    let last_values = get_last_values(
        deps.as_mut().storage,
        &map,
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

    let first_key = get_first_key(deps.as_mut().storage, &map).unwrap();

    assert_eq!(first_key, 1);

    let last_key = get_last_key(deps.as_mut().storage, &map).unwrap();

    assert_eq!(last_key, 99);

    let map_empty: Map<u64, String> = Map::new("test_map_2");

    let first_key = get_first_key(deps.as_mut().storage, &map_empty);

    assert!(first_key.is_none(), "{}", true);

    let last_key = get_last_key(deps.as_mut().storage, &map_empty);

    assert!(last_key.is_none(), "{}", true)
}
