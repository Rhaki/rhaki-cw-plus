use std::str::FromStr;

use crate::multi_test::custom_app::{CModuleWrapper, ModuleDb};
use crate::multi_test::helper::{bench32_app_builder, DefaultWasmKeeper, FailingCustom};

use cosmwasm_std::testing::MockStorage;
use cosmwasm_std::{Addr, Api, Binary, BlockInfo, Empty, Querier, Storage, Uint128};
use cw_multi_test::addons::MockApiBech32;
use cw_multi_test::error::anyhow;
use cw_multi_test::{
    no_init, App, AppResponse, BankKeeper, CosmosRouter, DistributionKeeper, FailingModule,
    GovFailingModule, IbcFailingModule, StakeKeeper, Stargate,
};
use osmosis_std::types::osmosis::tokenfactory::v1beta1::{MsgBurn, MsgCreateDenom, MsgMint};
use prost::Message;
use strum_macros::EnumString;

#[derive(Default)]
pub struct OsmosisStargateModule {}

#[derive(EnumString)]
#[strum(ascii_case_insensitive)]
enum OsmosisStargateQueryUrls {
    #[strum(serialize = "/osmosis.tokenfactory.v1beta1.Query/DenomAuthorityMetadata")]
    QueryDenomAuthorityMetadataRequest,
}

#[derive(EnumString)]
#[strum(ascii_case_insensitive)]
enum OsmosisStargateExecuteUrls {
    #[strum(serialize = "/osmosis.tokenfactory.v1beta1.MsgMint")]
    MsgMint,
    #[strum(serialize = "/osmosis.tokenfactory.v1beta1.MsgCreateDenom")]
    MsgCreateDenom,
    #[strum(serialize = "/osmosis.tokenfactory.v1beta1.MsgBurn")]
    MsgBrun,
}

/// returns:
/// - `custom module` (no custom is provided for osmosis, so `FailingModule` is used)
/// - `stargate module`
/// - `db`
pub fn build_osmosis_modules() -> (
    FailingModule<Empty, Empty, Empty>,
    OsmosisStargateModule,
    CModuleWrapper,
) {
    let custom_module = FailingModule::default();
    let ibc_module = OsmosisStargateModule::default();
    (custom_module, ibc_module, CModuleWrapper::default())
}

pub fn build_osmosis_app() -> (
    App<
        BankKeeper,
        MockApiBech32,
        MockStorage,
        FailingCustom,
        DefaultWasmKeeper,
        StakeKeeper,
        DistributionKeeper,
        IbcFailingModule,
        GovFailingModule,
        OsmosisStargateModule,
    >,
    CModuleWrapper,
) {
    let (_, ibc_module, db) = build_osmosis_modules();
    (
        bench32_app_builder("osmo")
            .with_stargate(ibc_module)
            .build(no_init),
        db,
    )
}

// Stargate

impl Stargate for OsmosisStargateModule {
    fn execute<ExecC, QueryC>(
        &self,
        api: &dyn Api,
        storage: &mut dyn Storage,
        router: &dyn CosmosRouter<ExecC = ExecC, QueryC = QueryC>,
        block: &BlockInfo,
        sender: Addr,
        type_url: String,
        value: Binary,
    ) -> anyhow::Result<AppResponse>
    where
        ExecC: std::fmt::Debug
            + Clone
            + PartialEq
            + schemars::JsonSchema
            + serde::de::DeserializeOwned
            + 'static,
        QueryC: cosmwasm_std::CustomQuery + serde::de::DeserializeOwned + 'static,
    {
        match OsmosisStargateExecuteUrls::from_str(&type_url)? {
            OsmosisStargateExecuteUrls::MsgMint => {
                let msg = MsgMint::decode(value.as_slice())?;
                let coin = msg.amount.ok_or(anyhow!("amount not found"))?;

                CModuleWrapper::use_db(storage, |db, storage| {
                    db.token_factory.run_msg_mint(
                        api,
                        storage,
                        router,
                        block,
                        sender,
                        coin.denom,
                        Uint128::from_str(&coin.amount)?,
                        msg.mint_to_address,
                    )
                })?
            }
            OsmosisStargateExecuteUrls::MsgCreateDenom => {
                let msg = MsgCreateDenom::decode(value.as_slice())?;

                CModuleWrapper::use_db(storage, |db, storage| {
                    db.token_factory.run_create_denom(
                        api,
                        storage,
                        router,
                        block,
                        sender,
                        msg.subdenom,
                    )
                })?
            }
            OsmosisStargateExecuteUrls::MsgBrun => {
                let msg = MsgBurn::decode(value.as_slice())?;
                let coin = msg.amount.ok_or(anyhow!("amount not found"))?;

                CModuleWrapper::use_db(storage, |db, storage| {
                    db.token_factory.run_burn_denom(
                        api,
                        storage,
                        router,
                        block,
                        sender,
                        coin.denom,
                        Uint128::from_str(&coin.amount)?,
                        msg.burn_from_address,
                    )
                })?
            }
        }
    }

    fn query(
        &self,
        _api: &dyn Api,
        _storage: &dyn Storage,
        _querier: &dyn Querier,
        _block: &BlockInfo,
        _path: String,
        _data: Binary,
    ) -> anyhow::Result<Binary> {
        unimplemented!()
    }
}

#[cfg(test)]
mod test {
    use cosmwasm_std::{Binary, Coin, CosmosMsg};
    use cw_asset::AssetInfo;
    use cw_multi_test::Executor;
    use osmosis_std::types::{
        cosmos::base::v1beta1::Coin as OsmosisCoin,
        osmosis::tokenfactory::v1beta1::{MsgBurn, MsgCreateDenom, MsgMint},
    };

    use super::build_osmosis_app;
    use crate::{
        asset::{AssetInfoPrecisioned, AssetPrecisioned},
        multi_test::{
            custom_app::ModuleDb,
            helper::{AppExt, Bench32AppExt, UnwrapError},
        },
        traits::Wrapper,
    };

    #[test]
    fn t1() {
        // db is the internal state of the custom module.
        // It's a Rc<RefCell<CModuleWrapper>>, so is possilbe to safely borrow it to change it state without executing msg.
        // This is usefull to set params that can't be done with msgs (like set fee for token creation)
        let (mut app, mut db) = build_osmosis_app();

        let minter = app.generate_addr("minter");
        let to = app.generate_addr("to");
        let sub_denom = "foo";

        let denom = format!("factory/{}/{}", minter, sub_denom);
        let asset = AssetInfoPrecisioned::native(denom, 6);

        // Set fee token creation

        let fee = AssetPrecisioned::new_super(AssetInfo::native("uluna"), 6, 100_u128);
        let fee_collector = app.generate_addr("token_factory_collector");

        db.as_db(app.storage_mut(), |db, _| {
            db.token_factory
                .set_fee_creation(vec![Coin::new(100, "uluna")], fee_collector.clone())
        })
        .unwrap();

        // Create denom

        let msg_create_denom = MsgCreateDenom {
            sender: minter.to_string(),
            subdenom: sub_denom.to_string(),
        }
        .to_any();

        let msg_create_denom = CosmosMsg::Stargate {
            type_url: msg_create_denom.type_url,
            value: Binary::from(msg_create_denom.value),
        };

        app.execute(minter.clone(), msg_create_denom.clone())
            .unwrap_err_contains("Error on gather fee for denom creation");

        // Mint fee token creation to minter
        app.mint(&minter, fee.clone());

        app.execute(minter.clone(), msg_create_denom).unwrap();

        // Mint 100 tokens

        let mint_amount: OsmosisCoin = TryInto::<Coin>::try_into(asset.to_asset(100_u128))
            .unwrap()
            .into();

        let msg_mint = MsgMint {
            sender: minter.to_string(),
            amount: mint_amount.clone().wrap_some(),
            mint_to_address: to.to_string(),
        }
        .to_any();

        let msg_mint = CosmosMsg::Stargate {
            type_url: msg_mint.type_url,
            value: Binary::from(msg_mint.value),
        };

        app.execute(minter.clone(), msg_mint).unwrap();

        let balance = app.qy_balance(&to, &asset).unwrap();

        assert_eq!(balance, asset.to_asset(100_u128));

        // Burn 50 tokens

        let burn_amount: OsmosisCoin = TryInto::<Coin>::try_into(asset.to_asset(50_u128))
            .unwrap()
            .into();

        let msg_burn = MsgBurn {
            sender: minter.to_string(),
            amount: burn_amount.wrap_some(),
            burn_from_address: to.to_string(),
        }
        .to_any();

        let msg_burn = CosmosMsg::Stargate {
            type_url: msg_burn.type_url,
            value: Binary::from(msg_burn.value),
        };

        app.execute(minter.clone(), msg_burn).unwrap();

        let balance = app.qy_balance(&to, &asset).unwrap();

        assert_eq!(balance, asset.to_asset(50_u128));
    }
}
