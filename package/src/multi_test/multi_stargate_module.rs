use std::{cell::RefCell, collections::BTreeMap, rc::Rc};

use anyhow::bail;
use cosmwasm_std::{
    testing::MockStorage, Addr, Api, Binary, BlockInfo, CustomQuery, Querier, Storage,
};
use cw_multi_test::{
    addons::MockApiBech32, no_init, App, AppBuilder, AppResponse, BankKeeper, CosmosRouter,
    DistributionKeeper, GovFailingModule, IbcFailingModule, StakeKeeper, Stargate,
};
use schemars::JsonSchema;
use serde::de::DeserializeOwned;
use std::fmt::Debug;

use crate::{
    multi_test::router::{RouterWrapper, UseRouter, UseRouterResponse},
    router_closure,
    storage::interfaces::ItemInterface,
};
use cw_multi_test::error::AnyResult;

use cosmwasm_std::from_json;

use super::helper::{DefaultWasmKeeper, FailingCustom};

pub fn multi_stargate_app(
    prefix: &'static str,
    apps: Vec<Box<dyn StargateApplication + 'static>>,
) -> App<
    BankKeeper,
    MockApiBech32,
    MockStorage,
    FailingCustom,
    DefaultWasmKeeper,
    StakeKeeper,
    DistributionKeeper,
    IbcFailingModule,
    GovFailingModule,
    MultiStargateModule,
> {
    multi_stargate_app_builder(prefix, apps).build(no_init)
}

pub fn multi_stargate_app_builder(
    prefix: &'static str,
    apps: Vec<Box<dyn StargateApplication + 'static>>,
) -> AppBuilder<
    BankKeeper,
    MockApiBech32,
    MockStorage,
    FailingCustom,
    DefaultWasmKeeper,
    StakeKeeper,
    DistributionKeeper,
    IbcFailingModule,
    GovFailingModule,
    MultiStargateModule,
> {
    let mut stargate_module = MultiStargateModule::default();

    for app in apps {
        stargate_module = stargate_module.with_application(app);
    }
    AppBuilder::new()
        .with_api(MockApiBech32::new(prefix))
        .with_stargate(stargate_module)
}

#[derive(Default)]
pub struct MultiStargateModule {
    pub applications: BTreeMap<String, Box<dyn StargateApplication>>,
}

impl MultiStargateModule {
    pub fn with_application(mut self, app: Box<dyn StargateApplication>) -> Self {
        self.try_add_application(app).unwrap();
        self
    }
}

impl MultiStargateModule {
    fn get_application_by_msg_type_url(
        &self,
        type_url: String,
    ) -> AnyResult<&Box<dyn StargateApplication>> {
        for application in self.applications.values() {
            if application.is_msg_type_url(type_url.clone()) {
                return Ok(application);
            }
        }
        bail!("Application not found for type_url: {type_url}")
    }

    fn get_application_by_query_type_url(
        &self,
        type_url: String,
    ) -> AnyResult<&Box<dyn StargateApplication>> {
        for application in self.applications.values() {
            if application.is_query_type_url(type_url.clone()) {
                return Ok(application);
            }
        }
        bail!("Application not found for type_url: {type_url}")
    }

    pub fn try_add_application(
        &mut self,
        application: Box<dyn StargateApplication>,
    ) -> AnyResult<()> {
        for type_url in application.type_urls() {
            for existing_application in self.applications.values() {
                if existing_application.is_msg_type_url(type_url.clone())
                    || existing_application.is_query_type_url(type_url.clone())
                {
                    bail!("Dupplicated type_url among applications: {}", type_url)
                }
            }
        }

        let name = application.stargate_name();

        self.applications.insert(name, application);

        Ok(())
    }
}

impl Stargate for MultiStargateModule {
    fn execute<ExecC, QueryC>(
        &self,
        api: &dyn Api,
        storage: &mut dyn Storage,
        router: &dyn CosmosRouter<ExecC = ExecC, QueryC = QueryC>,
        block: &BlockInfo,
        sender: Addr,
        type_url: String,
        value: Binary,
    ) -> AnyResult<AppResponse>
    where
        ExecC: Debug + Clone + PartialEq + JsonSchema + DeserializeOwned + 'static,
        QueryC: CustomQuery + DeserializeOwned + 'static,
    {
        let application = self.get_application_by_msg_type_url(type_url.clone())?;

        let mut loaded = application.load(storage)?;

        let rc_storage = Rc::new(RefCell::new(storage));

        let res = loaded.stargate_msg(
            api,
            rc_storage.clone(),
            &RouterWrapper::new(&router_closure!(router, api, rc_storage, block)),
            block,
            sender,
            type_url,
            value,
        )?;

        loaded.save(*rc_storage.borrow_mut())?;

        Ok(res)
    }

    fn query(
        &self,
        api: &dyn Api,
        storage: &dyn Storage,
        querier: &dyn Querier,
        block: &BlockInfo,
        path: String,
        data: Binary,
    ) -> AnyResult<Binary> {
        let application = self.get_application_by_query_type_url(path.clone())?;

        let loaded = application.load(storage)?;

        loaded.stargate_query(api, storage, querier, block, path, data)
    }
}
pub trait StargateApplication: StargateUrls + Itemable {
    fn stargate_msg(
        &mut self,
        api: &dyn Api,
        storage: Rc<RefCell<&mut dyn Storage>>,
        router: &RouterWrapper,
        block: &BlockInfo,
        sender: Addr,
        type_url: String,
        data: Binary,
    ) -> AnyResult<AppResponse>;

    fn stargate_query(
        &self,
        api: &dyn Api,
        storage: &dyn Storage,
        querier: &dyn Querier,
        block: &BlockInfo,
        type_url: String,
        data: Binary,
    ) -> AnyResult<Binary>;
}
pub trait StargateUrls {
    fn stargate_name(&self) -> String;

    fn is_query_type_url(&self, type_url: String) -> bool;

    fn is_msg_type_url(&self, type_url: String) -> bool;

    fn type_urls(&self) -> Vec<String>;
}
pub trait Itemable {
    fn load(&self, storage: &dyn Storage) -> AnyResult<Box<dyn StargateApplication>>;
    fn save(&self, storage: &mut dyn Storage) -> AnyResult<()>;
}

pub trait ModuleDb: Default + ItemInterface {
    fn use_db<R, F: FnOnce(&mut Self, &mut dyn Storage) -> R>(
        storage: &mut dyn Storage,
        fnn: F,
    ) -> anyhow::Result<R> {
        let mut data = Self::load(storage).unwrap_or_default();
        data.as_db(storage, fnn)
    }

    fn as_db<R, F: FnOnce(&mut Self, &mut dyn Storage) -> R>(
        &mut self,
        storage: &mut dyn Storage,
        fnn: F,
    ) -> anyhow::Result<R> {
        let res = fnn(self, storage);
        self.save(storage)?;
        Ok(res)
    }
}

impl<T> ModuleDb for T where T: ItemInterface + Default {}
