use std::{cell::RefCell, rc::Rc};

use super::custom_modules::token_factory::CTokenFactory;

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
