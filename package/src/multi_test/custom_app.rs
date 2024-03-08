use std::{cell::RefCell, rc::Rc};

use anyhow::Ok;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::Storage;

use crate::storage::interfaces::ItemInterface;

use super::custom_modules::token_factory::CTokenFactory;

#[cw_serde]
#[derive(Default)]
pub struct CModuleWrapper {
    pub token_factory: CTokenFactory,
}

impl CModuleWrapper {
    pub fn new() -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(CModuleWrapper {
            token_factory: CTokenFactory::default(),
        }))
    }
}

impl ItemInterface for CModuleWrapper {
    const NAMESPACE: &'static str = "c_module_wrapper_db";

    const CONTRACT_NAME: &'static str = "c_module_wrapper_db";
}

pub trait ModuleDb: ItemInterface + Default {
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

impl ModuleDb for CModuleWrapper {}
