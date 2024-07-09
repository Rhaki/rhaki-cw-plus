use {
    cosmwasm_std::{Addr, StdError, StdResult, Storage},
    cw_storage_plus::Item,
};

const OWNER: Item<Addr> = Item::new("owner");

pub fn set_owner(storage: &mut dyn Storage, owner: &Addr) -> StdResult<()> {
    OWNER.save(storage, owner)?;
    Ok(())
}

pub fn get_owner(storage: &mut dyn Storage) -> Option<Addr> {
    match OWNER.load(storage) {
        Ok(owner) => Some(owner),
        Err(_) => None,
    }
}

pub fn assert_owner(storage: &mut dyn Storage, owner: &Addr) -> StdResult<()> {
    match OWNER.load(storage) {
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
