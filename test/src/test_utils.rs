use std::collections::HashMap;

use cosmwasm_schema::cw_serde;
use rhaki_cw_plus::serde_value::Value;
use rhaki_cw_plus::utils::{vec_to_i_hashmap, vec_tuple_to_hashmap};
use rhaki_cw_plus::{cw_serde_value, Optionable, SmallerTwin};

#[cw_serde_value]
pub struct WithCwSerdeStruct {
    pub val: Value,
}

#[cw_serde]
pub struct WithoutSerdeStruct {}

#[cw_serde]
#[derive(Optionable, SmallerTwin)]
#[optionable(name = MsgOwnable, attributes(cw_serde))]
#[smaller_twin(name = ConfigInit, attributes(cw_serde))]
pub struct Config {
    pub foo: String,
    pub bar: u64,
    #[optionable(skip)]
    pub skip_field: String,
    #[smaller_twin(skip)]
    pub option_field: Option<String>,
}

#[test]
fn main() {
    let mut vec = vec![
        ("first".to_string(), 1_u128),
        ("second".to_string(), 2_u128),
        ("third".to_string(), 3_u128),
    ];

    let res = vec_tuple_to_hashmap(vec.clone()).unwrap();

    let mut map: HashMap<String, u128> = HashMap::new();

    map.insert("first".to_string(), 1_u128);
    map.insert("second".to_string(), 2_u128);
    map.insert("third".to_string(), 3_u128);

    assert_eq!(res, map);

    vec.push(("third".to_string(), 3_u128));

    vec_tuple_to_hashmap(vec.clone()).unwrap_err();

    let vec = vec![
        "first".to_string(),
        "second".to_string(),
        "third".to_string(),
    ];

    let res = vec_to_i_hashmap(vec);

    let mut map: HashMap<usize, String> = HashMap::new();

    map.insert(0, "first".to_string());
    map.insert(1, "second".to_string());
    map.insert(2, "third".to_string());

    assert_eq!(res, map);
}
