use cosmwasm_std::{Coin, Uint128};
use rhaki_cw_plus::coin::{merge_coin, only_one_coin};

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
