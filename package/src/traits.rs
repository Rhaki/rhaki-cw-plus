use std::cmp::min;

use cosmwasm_std::{to_binary, Addr, Api, Binary, StdError, StdResult};
use serde::Serialize;

pub trait IntoAddr: Into<String> + Clone {
    fn into_addr(self, api: &dyn Api) -> StdResult<Addr> {
        api.addr_validate(&self.into())
    }
    fn into_unchecked_addr(&self) -> Addr {
        Addr::unchecked(self.clone())
    }
}

impl IntoAddr for String {}
impl IntoAddr for &str {}

pub trait IntoBinary {
    fn into_binary(self) -> StdResult<Binary>;
}

impl<T> IntoBinary for StdResult<T>
where
    T: Serialize,
{
    fn into_binary(self) -> StdResult<Binary> {
        to_binary(&self?)
    }
}

pub trait IntoStdResult<T> {
    fn into_std_result(self) -> StdResult<T>;
}
impl<T, E> IntoStdResult<T> for Result<T, E>
where
    E: std::error::Error,
{
    fn into_std_result(self) -> StdResult<T> {
        self.map_err(|err| StdError::generic_err(err.to_string()))
    }
}

pub trait MapMin {
    type Output;
    type Input;
    fn map_min(&self, with: Self::Input) -> Self::Output;
}

impl<T: Ord + Copy> MapMin for Option<T> {
    type Output = Option<T>;
    type Input = T;

    fn map_min(&self, with: Self::Input) -> Self::Output {
        self.map(|val| min(val, with))
    }
}
