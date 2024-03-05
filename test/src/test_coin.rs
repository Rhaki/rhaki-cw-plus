use cosmwasm_std::{testing::mock_dependencies, Coin, Uint128};
use cw_storage_plus::Map;
use rhaki_cw_plus::asset::{merge_coin, only_one_coin, AssetInfoPrecisioned};

#[test]
fn main() {
    let coin = Coin {
        denom: "stake".to_string(),
        amount: Uint128::from(100_u128),
    };

    let res = only_one_coin(&vec![coin.clone()], None).unwrap();

    assert_eq!(coin, res);

    let res = only_one_coin(&vec![coin.clone()], Some("stake".to_string())).unwrap();

    assert_eq!(coin, res);

    only_one_coin(&vec![coin.clone()], Some("rand".to_string())).unwrap_err();

    assert!(merge_coin(&None, &None).unwrap().is_none(), "{}", true);

    let res = merge_coin(&Some(coin.clone()), &None).unwrap().unwrap();

    assert_eq!(res, coin);

    let res = merge_coin(&None, &Some(coin.clone())).unwrap().unwrap();

    assert_eq!(res, coin);

    let res = merge_coin(&Some(coin.clone()), &Some(coin.clone()))
        .unwrap()
        .unwrap();

    assert_eq!(
        res,
        Coin {
            denom: coin.clone().denom,
            amount: coin.amount * Uint128::from(2_u128)
        }
    );

    merge_coin(
        &Some(coin),
        &Some(Coin {
            denom: "rand".to_string(),
            amount: Uint128::one(),
        }),
    )
    .unwrap_err();
}

#[test]
fn asset() {
    let asset_precisioned_1 = AssetInfoPrecisioned::native("denom", 6);
    let asset_precisioned_2 = AssetInfoPrecisioned::native("asd", 6);

    let asset_precisioned_3 = AssetInfoPrecisioned::native("denom", 8);

    let map: Map<AssetInfoPrecisioned, u64> = Map::new("asd");

    let mut deps = mock_dependencies();

    map.save(deps.as_mut().storage, asset_precisioned_1.clone(), &1)
        .unwrap();
    map.save(deps.as_mut().storage, asset_precisioned_2.clone(), &2)
        .unwrap();

    let one = map
        .load(deps.as_ref().storage, asset_precisioned_1)
        .unwrap();

    let two = map
        .load(deps.as_ref().storage, asset_precisioned_2)
        .unwrap();

    let three = map
        .load(deps.as_ref().storage, asset_precisioned_3)
        .unwrap();

    println!("{one}");
    println!("{two}");
    println!("{three}")
}
