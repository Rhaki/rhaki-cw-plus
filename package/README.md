# **rhaki-cw-plus**: util packet for CosmWasm smart contract

Modules | Description
|-|-|
`map`  | Simplify data gather for `cw_storage_plus::Map`.
`coin` | Helps to assert `cosmwasm_std::Coin`, check `funds` received, and merge coins with the same denom etc...
`auth` | Create, get and assert an `cw_storage_plus::Item::<Addr>` for owner.