use cosmwasm_std::CosmosMsg;
use rhaki_cw_plus::serde::{sjw_from_str, std_to_sjw_value, value_to_string};
use serde_json::json;

#[test]
pub fn main() {
    let v = json!({"bank":{"send":{
        "to_address": "addr_1",
        "amount": [
            {
                "denom": "ustake",
                "amount": "0"
            }
        ]
    }}});

    let res = std_to_sjw_value(v);

    let res = value_to_string(&res).unwrap();

    sjw_from_str::<CosmosMsg>(&res).unwrap();
}
