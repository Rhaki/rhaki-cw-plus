use std::cmp::min;

use cosmwasm_std::{from_json, to_json_binary, Addr, Api, Binary, StdError, StdResult};
use cw_asset::AssetInfo;
use serde::{de::DeserializeOwned, Serialize};

pub trait IntoAddr: Into<String> + Clone {
    fn into_addr(self, api: &dyn Api) -> StdResult<Addr> {
        api.addr_validate(&self.into())
    }
    fn into_unchecked_addr(&self) -> Addr {
        Addr::unchecked(self.clone())
    }
}

impl<T> IntoAddr for T where T: Into<String> + Clone {}

pub trait IntoBinaryResult {
    /// `Serialize` into `Binary`
    fn into_binary(self) -> StdResult<Binary>;
}

impl<T> IntoBinaryResult for StdResult<T>
where
    T: Serialize,
{
    fn into_binary(self) -> StdResult<Binary> {
        to_json_binary(&self?)
    }
}

pub trait IntoBinary {
    /// `Serialize` into `Binary`
    fn into_binary(self) -> StdResult<Binary>;
}

impl<T> IntoBinary for T
where
    T: Serialize,
{
    fn into_binary(self) -> StdResult<Binary> {
        to_json_binary(&self)
    }
}

pub trait FromBinaryResult {
    /// `Deserialize` `StdResult<Binary>` into specified `Struct`/`Enum`. It must to implement `DeserializeOwned`
    fn des_into<T: DeserializeOwned>(self) -> StdResult<T>;
}

impl FromBinaryResult for StdResult<Binary> {
    fn des_into<T: DeserializeOwned>(self) -> StdResult<T> {
        from_json(&self?)
    }
}

pub trait FromBinary {
    /// `Deserialize` `Binary` into specified `Struct`/`Enum`. It must to implement `DeserializeOwned`
    fn des_into<T: DeserializeOwned>(self) -> StdResult<T>;
}

impl FromBinary for Binary {
    fn des_into<T: DeserializeOwned>(self) -> StdResult<T> {
        from_json(&self)
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

pub trait AssertOwner {
    fn get_admin(&self) -> Addr;

    fn assert_admin(&self, address: Addr) -> StdResult<()> {
        if self.get_admin() != address {
            return Err(StdError::generic_err("Unauthorized"));
        }

        Ok(())
    }
}

pub trait IntoInner {
    type Inner;
    fn inner(&self) -> Self::Inner;
}

impl IntoInner for AssetInfo {
    type Inner = String;
    /// Return the denom or address
    fn inner(&self) -> Self::Inner {
        match self {
            cw_asset::AssetInfoBase::Native(denom) => denom.clone(),
            cw_asset::AssetInfoBase::Cw20(addr) => addr.to_string(),
            _ => todo!(),
        }
    }
}

pub trait Unclone {
    type Output;
    fn unclone(&self) -> Self::Output;
}

impl<T> Unclone for Option<T>
where
    T: Clone,
{
    type Output = T;

    fn unclone(&self) -> Self::Output {
        self.clone().unwrap()
    }
}

#[cfg(test)]
mod test {
    use cosmwasm_std::{testing::mock_dependencies, Addr, Coin, StdError};

    use crate::traits::{FromBinary, FromBinaryResult, IntoAddr, IntoBinary, IntoBinaryResult};

    #[test]
    fn test() {
        let coin = Coin::new(1, "asd");

        let coin_binary = coin.clone().into_binary().unwrap();

        let res_binary = Ok::<_, StdError>(coin.clone()).into_binary().unwrap();

        let coin_std: Coin = coin_binary.des_into().unwrap();
        let coin_res = Ok(res_binary).des_into::<Coin>().unwrap();

        assert_eq!(coin_std, coin);
        assert_eq!(coin_res, coin);

        assert_eq!(
            Addr::unchecked("terra123"),
            "terra123".into_unchecked_addr()
        );
        assert_eq!(
            Addr::unchecked("terra123"),
            "terra123".to_string().into_unchecked_addr()
        );

        let deps = mock_dependencies();

        assert_eq!(
            Addr::unchecked("terra123"),
            "terra123".into_addr(&deps.api).unwrap()
        );
        assert_eq!(
            Addr::unchecked("terra123"),
            "terra123".to_string().into_addr(&deps.api).unwrap()
        );
    }
}
