use std::collections::HashMap;

use cosmwasm_std::{Addr, Attribute, Coin, StdError, StdResult, Uint128};
use cw_multi_test::{App, AppBuilder, AppResponse, Executor};

/// Create a `cw_mutli_test::App` with starting balances
pub fn mock_app(users: Vec<(&str, Vec<Coin>)>) -> App {
    if !users.is_empty() {
        let mut map: HashMap<String, Uint128> = HashMap::new();

        for (_, coins) in users.clone() {
            for coin in coins {
                match map.get_mut(&coin.denom) {
                    Some(amount) => {
                        *amount += coin.amount;
                    }
                    None => {
                        map.insert(coin.denom, coin.amount);
                    }
                };
            }
        }

        let coin: Vec<Coin> = map
            .into_iter()
            .map(|(k, v)| Coin::new(v.into(), k))
            .collect();

        let first_user = users.first().unwrap().0;

        let mut app = AppBuilder::new().build(|router, _, storage| {
            router
                .bank
                .init_balance(storage, &Addr::unchecked(first_user), coin)
                .unwrap()
        });

        for (user, coins) in users {
            if user != first_user {
                app.send_tokens(Addr::unchecked(first_user), Addr::unchecked(user), &coins)
                    .unwrap();
            }
        }

        app
    } else {
        App::default()
    }
}

/// Assert that every `Attribute` in a `Vec<Attribute> `are presents inside any `Response.events`
pub fn app_assert_attributes_response(
    response: AppResponse,
    attributes: Vec<Attribute>,
) -> StdResult<()> {
    let not_found_events: Vec<Attribute> = attributes
        .into_iter()
        .filter(|attr| {
            for i in response.events.clone() {
                if i.attributes.contains(attr) {
                    return false;
                }
            }
            true
        })
        .collect();

    if not_found_events.is_empty() {
        Ok(())
    } else {
        Err(StdError::generic_err(format!(
            "Some events not found: {not_found_events:?}"
        )))
    }
}
