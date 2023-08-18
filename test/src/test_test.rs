use cosmwasm_std::{attr, Coin, Event};
use cw_multi_test::AppResponse;
use rhaki_cw_plus::test::{app_assert_attributes_response, mock_app};

pub const USER_1: &str = "user_1";
pub const USER_2: &str = "user_2";
pub const USER_3: &str = "user_3";

#[test]
fn main() {
    let app = mock_app(vec![(USER_1, vec![Coin::new(100, "uluna".to_string())])]);

    assert_eq!(
        app.wrap().query_all_balances(USER_1).unwrap(),
        vec![Coin::new(100, "uluna".to_string())]
    );

    let app = mock_app(vec![
        (USER_1, vec![Coin::new(100, "uluna".to_string())]),
        (USER_2, vec![Coin::new(200, "uluna".to_string())]),
        (
            USER_3,
            vec![
                Coin::new(300, "uluna".to_string()),
                Coin::new(50, "uatom".to_string()),
            ],
        ),
    ]);

    assert_eq!(
        app.wrap().query_all_balances(USER_1).unwrap(),
        vec![Coin::new(100, "uluna".to_string())]
    );

    assert_eq!(
        app.wrap().query_all_balances(USER_2).unwrap(),
        vec![Coin::new(200, "uluna".to_string())]
    );

    assert_eq!(
        app.wrap().query_all_balances(USER_3).unwrap(),
        vec![
            Coin::new(50, "uatom".to_string()),
            Coin::new(300, "uluna".to_string())
        ]
    );

    let res = AppResponse {
        data: None,
        events: vec![
            Event::new("wasm".to_string())
                .add_attributes(vec![attr("key_1", "value_1"), attr("key_2", "value_2")]),
            Event::new("wasm".to_string())
                .add_attributes(vec![attr("key_3", "value_3"), attr("key_4", "value_4")]),
        ],
    };

    app_assert_attributes_response(
        res.clone(),
        vec![attr("key_1", "value_1"), attr("key_4", "value_4")],
    )
    .unwrap();
    app_assert_attributes_response(
        res,
        vec![attr("key_1", "value_1"), attr("key_5", "value_4")],
    )
    .unwrap_err();
}
