use cosmwasm_std::{Addr, StdError, StdResult, Storage};
use cw_storage_plus::Item;

const OWENR_KEY: &str = "rhaki-cw-plus-owner";

pub fn set_owner(storage: &mut dyn Storage, owner: &Addr) -> StdResult<()> {
    Item::<Addr>::new(OWENR_KEY).save(storage, owner)?;
    Ok(())
}

pub fn get_owner(storage: &mut dyn Storage) -> Option<Addr> {
    match Item::<Addr>::new(OWENR_KEY).load(storage) {
        Ok(owner) => Some(owner),
        Err(_) => None,
    }
}

pub fn assert_owner(storage: &mut dyn Storage, owner: &Addr) -> StdResult<()> {
    match Item::<Addr>::new(OWENR_KEY).load(storage) {
        Ok(saved) => {
            if saved == owner {
                Ok(())
            } else {
                Err(StdError::generic_err(format!(
                    "Owner not match: found: {}, expected: {}",
                    owner, saved
                )))
            }
        }
        Err(_) => Err(StdError::generic_err("Owner never setted")),
    }
}
