use cosmwasm_schema::cw_serde;
use cosmwasm_std::{BankMsg, Coin, CosmosMsg, StdResult};
use rhaki_cw_plus::serde::{
    std_to_sjw_value, value_to_string, DoubleDeserialize,
    DoubleValueDeserializeResult, SerdeValue, ToCwJson,
};
use serde_json::json;

#[test]
pub fn main() {
    let value = json!({"bank":{"send":{
        "to_address": "addr_1",
        "amount": [
            {
                "denom": "ustake",
                "amount": "0"
            }
        ]
    }}});

    let res = std_to_sjw_value(value).unwrap();

    value_to_string(&res).unwrap();

    let v = json!({"bank":{"send":{
        "to_address": "addr_1",
        "amount": [
            {
                "denom": "ustake",
                "amount": "0"
            }
        ]
    }}})
    .into_cw()
    .unwrap();

    v.as_string().unwrap();

    let res = v.to_cosmos_msg().unwrap();

    assert_eq!(
        res,
        CosmosMsg::Bank(BankMsg::Send {
            to_address: "addr_1".to_string(),
            amount: vec![Coin::new(0, "ustake")]
        })
    )
}

#[cw_serde]
pub enum First {
    One {},
    Two {},
}

#[cw_serde]
pub enum Second {
    Three {},
    Four {},
}

#[test]
pub fn double_deserialize() {
    let res: DoubleValueDeserializeResult<First, Second> = json!({"one":{}})
        .into_cw()
        .unwrap()
        .double_deserialize()
        .unwrap();

    match res {
        DoubleValueDeserializeResult::First(first) => is_first(first).unwrap(),
        DoubleValueDeserializeResult::Second(_) => panic!("Shoud be first"),
    }

    let res: DoubleValueDeserializeResult<First, Second> = json!({"three":{}})
        .into_cw()
        .unwrap()
        .double_deserialize()
        .unwrap();

    match res {
        DoubleValueDeserializeResult::First(_) => panic!("Shoud be second"),
        DoubleValueDeserializeResult::Second(second) => is_second(second).unwrap(),
    }

    json!({"five":{}})
        .into_cw()
        .unwrap()
        .double_deserialize::<First, Second>()
        .unwrap_err();
}

fn is_first(_: First) -> StdResult<()> {
    Ok(())
}

fn is_second(_: Second) -> StdResult<()> {
    Ok(())
}
