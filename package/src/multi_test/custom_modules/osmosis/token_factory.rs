use {
    crate::{
        multi_test::{
            helper::cw_multi_test::error::AnyResult,
            multi_stargate_module::{Itemable, StargateApplication, StargateUrls},
            router::RouterWrapper,
        },
        storage::interfaces::ItemInterface,
        traits::{IntoAddr, IntoBinary},
    },
    anyhow::{anyhow, bail},
    cosmwasm_schema::cw_serde,
    cosmwasm_std::{
        Addr, Api, BankMsg, Binary, BlockInfo, Coin, CosmosMsg, Empty, Querier, Storage, Uint128,
    },
    cw_multi_test::{AppResponse, BankSudo, SudoMsg},
    osmosis_std::types::{
        cosmos::bank::v1beta1::Metadata,
        osmosis::tokenfactory::v1beta1::{
            MsgBurn, MsgChangeAdmin, MsgCreateDenom, MsgCreateDenomResponse, MsgMint,
            MsgSetDenomMetadata, Params, QueryParamsResponse,
        },
    },
    prost::Message,
    rhaki_cw_plus_macro::{urls, Stargate},
    std::{cell::RefCell, collections::BTreeMap, rc::Rc, str::FromStr},
};

#[cw_serde]
pub struct TokenFactoryFee {
    pub fee: Vec<Coin>,
    pub fee_collector: Addr,
}

#[derive(Stargate, Default)]
#[cw_serde]
#[stargate(name = "token_factory", query_urls = TokenFactoryQueryUrls, msgs_urls = TokenFactoryMsgUrls)]
pub struct TokenFactoryModule {
    pub fee_creation: Option<TokenFactoryFee>,
    pub token_precisions: BTreeMap<String, u8>,
    pub supplies: BTreeMap<String, Uint128>,
    pub metadata: BTreeMap<String, Metadata>,
    pub admin: BTreeMap<String, Addr>,
}

#[urls]
pub enum TokenFactoryMsgUrls {
    #[strum(serialize = "/osmosis.tokenfactory.v1beta1.MsgMint")]
    MsgMint,
    #[strum(serialize = "/osmosis.tokenfactory.v1beta1.MsgCreateDenom")]
    MsgCreateDenom,
    #[strum(serialize = "/osmosis.tokenfactory.v1beta1.MsgBurn")]
    MsgBurn,
    #[strum(serialize = "/osmosis.tokenfactory.v1beta1.MsgSetDenomMetadata")]
    MsgSetDenomMetadata,
    #[strum(serialize = "/osmosis.tokenfactory.v1beta1.MsgChangeAdmin")]
    MsgChangeAdmin,
}

#[urls]
pub enum TokenFactoryQueryUrls {
    #[strum(serialize = "/osmosis.tokenfactory.v1beta1.Query/Params")]
    Params,
}

impl StargateApplication for TokenFactoryModule {
    fn stargate_msg(
        &mut self,
        api: &dyn Api,
        _storage: Rc<RefCell<&mut dyn Storage>>,
        router: &RouterWrapper,
        _block: &BlockInfo,
        sender: Addr,
        type_url: String,
        data: Binary,
    ) -> AnyResult<AppResponse> {
        match TokenFactoryMsgUrls::from_str(&type_url)? {
            TokenFactoryMsgUrls::MsgMint => {
                let msg = MsgMint::decode(data.as_slice())?;
                let coin = msg.amount.ok_or(anyhow!("amount not found"))?;

                self.run_msg_mint(
                    router,
                    sender,
                    coin.denom,
                    Uint128::from_str(&coin.amount)?,
                    msg.mint_to_address,
                )
            },
            TokenFactoryMsgUrls::MsgCreateDenom => {
                let msg = MsgCreateDenom::decode(data.as_slice())?;
                self.run_create_denom(router, sender, msg)
            },
            TokenFactoryMsgUrls::MsgBurn => {
                let msg = MsgBurn::decode(data.as_slice())?;
                let coin = msg.amount.ok_or(anyhow!("amount not found"))?;

                self.run_burn_denom(
                    api,
                    router,
                    sender,
                    coin.denom,
                    Uint128::from_str(&coin.amount)?,
                    msg.burn_from_address,
                )
            },
            TokenFactoryMsgUrls::MsgSetDenomMetadata => {
                let msg = MsgSetDenomMetadata::decode(data.as_slice())?;
                self.run_set_denom_metadata(sender, msg)
            },
            TokenFactoryMsgUrls::MsgChangeAdmin => {
                let msg = MsgChangeAdmin::decode(data.as_slice())?;
                self.run_change_admin(api, sender, msg)
            },
        }
    }

    fn stargate_query(
        &self,
        _api: &dyn Api,
        _storage: &dyn Storage,
        _querier: &dyn Querier,
        _block: &BlockInfo,
        type_url: String,
        _data: Binary,
    ) -> AnyResult<Binary> {
        match TokenFactoryQueryUrls::from_str(&type_url)? {
            TokenFactoryQueryUrls::Params => self.qy_params(),
        }
    }
}

// Msgs
impl TokenFactoryModule {
    pub fn run_msg_mint(
        &mut self,
        router: &RouterWrapper,
        sender: Addr,
        denom: String,
        amount: Uint128,
        to: String,
    ) -> AnyResult<AppResponse> {
        self.assert_owner(&sender, &denom)?;
        let mut supply = self
            .supplies
            .get(&denom)
            .cloned()
            .ok_or(anyhow!("Denom not existing denom: {denom}"))?;

        supply += amount;

        self.supplies.insert(denom.clone(), supply);

        router.sudo(SudoMsg::Bank(BankSudo::Mint {
            to_address: to.to_string(),
            amount: vec![Coin::new(amount.u128(), denom)],
        }))
    }

    pub fn run_create_denom(
        &mut self,
        router: &RouterWrapper,
        sender: Addr,
        msg: MsgCreateDenom,
    ) -> AnyResult<AppResponse> {
        if sender != msg.sender {
            bail!("Sender is not the same as the sender in the message");
        }

        let denom = self.build_denom(&sender, &msg.subdenom);
        if self.supplies.get(&denom).is_some() {
            bail!("Denom already existing denom: {denom}");
        }

        self.supplies.insert(denom.clone(), Uint128::zero());

        self.admin.insert(denom.clone(), sender.clone());

        let mut response = if let Some(fee_creation) = &self.fee_creation {
            router
                .execute(
                    sender,
                    CosmosMsg::<Empty>::Bank(BankMsg::Send {
                        to_address: fee_creation.fee_collector.to_string(),
                        amount: fee_creation.fee.clone(),
                    }),
                )
                .map_err(|e| anyhow!("Error on gather fee for denom creation: {}", e))?
        } else {
            AppResponse::default()
        };

        response.data = Some(Binary::from(
            MsgCreateDenomResponse {
                new_token_denom: denom,
            }
            .encode_to_vec(),
        ));

        Ok(response)
    }

    pub fn run_burn_denom(
        &mut self,
        api: &dyn Api,
        router: &RouterWrapper,
        sender: Addr,
        denom: String,
        amount: Uint128,
        burn_from_address: String,
    ) -> AnyResult<AppResponse> {
        self.assert_owner(&sender, &denom)?;
        let mut supply = self
            .supplies
            .get(&denom)
            .cloned()
            .ok_or(anyhow!("Denom not existing: {denom}"))?;

        supply -= amount;
        self.supplies.insert(denom.clone(), supply);

        let burn_from_address = api.addr_validate(&burn_from_address)?;

        router.execute(
            burn_from_address,
            CosmosMsg::<Empty>::Bank(BankMsg::Burn {
                amount: vec![Coin::new(amount.u128(), denom)],
            }),
        )
    }

    pub fn run_set_denom_metadata(
        &mut self,

        sender: Addr,
        msg: MsgSetDenomMetadata,
    ) -> AnyResult<AppResponse> {
        if let Some(metadata) = msg.metadata {
            let denom = metadata.base.clone();

            self.assert_owner(&sender, &denom)?;

            self.metadata.insert(denom, metadata);
        }
        AnyResult::Ok(AppResponse::default())
    }

    pub fn run_change_admin(
        &mut self,
        api: &dyn Api,

        sender: Addr,
        msg: MsgChangeAdmin,
    ) -> AnyResult<AppResponse> {
        self.assert_owner(&sender, &msg.denom)?;

        self.admin.insert(msg.denom, msg.new_admin.into_addr(api)?);

        Ok(AppResponse::default())
    }
}

// Queries
impl TokenFactoryModule {
    fn qy_params(&self) -> AnyResult<Binary> {
        Ok(QueryParamsResponse {
            params: Some(Params {
                denom_creation_fee: self
                    .fee_creation
                    .clone()
                    .map(|val| osmosis_std::cosmwasm_to_proto_coins(val.fee))
                    .unwrap_or_default(),
                denom_creation_gas_consume: 200_000,
            }),
        }
        .into_binary()?)
    }
}

impl TokenFactoryModule {
    fn assert_owner(&self, sender: &Addr, denom: &str) -> AnyResult<()> {
        let owner = self
            .admin
            .get(denom)
            .ok_or(anyhow!("Denom not existing: {denom}"))?;

        if owner != sender {
            bail!("Sender is not the owner of the denom")
        } else {
            Ok(())
        }
    }

    fn build_denom(&self, sender: &Addr, subdenom: &str) -> String {
        format!("factory/{}/{}", sender, subdenom)
    }
}

#[cfg(test)]
mod test {
    use {
        crate::{
            asset::AssetInfoPrecisioned,
            math::IntoDecimal,
            multi_test::{
                helper::{AppExt, Bench32AppExt, UnwrapError},
                multi_stargate_module::{multi_stargate_app, ModuleDb},
            },
        },
        cosmwasm_std::Coin,
        cw_multi_test::Executor,
        osmosis_std::types::osmosis::tokenfactory::v1beta1::MsgCreateDenom,
    };

    use super::{TokenFactoryFee, TokenFactoryModule};

    #[test]
    fn test() {
        let mut app = multi_stargate_app("osmo", vec![Box::new(TokenFactoryModule::default())]);

        let fee_collector = app.generate_addr("fee_collector");

        TokenFactoryModule::use_db(app.storage_mut(), |token_factory, _| {
            token_factory.fee_creation = Some(TokenFactoryFee {
                fee: vec![Coin::new(100_000_000, "uosmo")],
                fee_collector,
            })
        })
        .unwrap();

        let sender = app.generate_addr("sender");

        let msg = MsgCreateDenom {
            sender: sender.to_string(),
            subdenom: "test".to_string(),
        };

        app.execute(sender.clone(), msg.clone().into())
            .unwrap_err_contains("Error on gather fee for denom creation");

        app.mint(
            sender.clone(),
            AssetInfoPrecisioned::native("uosmo", 6).to_asset(100_u128.into_decimal()),
        );

        app.execute(sender, msg.into()).unwrap();
    }
}
