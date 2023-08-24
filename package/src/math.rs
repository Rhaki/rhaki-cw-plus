use cosmwasm_std::{Decimal, Decimal256, StdError, StdResult, Uint128, Uint256};

pub trait IntoUint {
    fn as_uint128(self) -> StdResult<Uint128>;
    fn as_uint256(self) -> Uint256;
}

impl IntoUint for Decimal {
    fn as_uint128(self) -> StdResult<Uint128> {
        Ok(self * Uint128::one())
    }

    fn as_uint256(self) -> Uint256 {
        Uint256::from_uint128(self * Uint128::one())
    }
}

impl IntoUint for Decimal256 {
    fn as_uint128(self) -> StdResult<Uint128> {
        self.as_uint256().as_uint128()
    }

    fn as_uint256(self) -> Uint256 {
        self * Uint256::one()
    }
}

pub trait IntoUint128 {
    fn as_uint128(self) -> StdResult<Uint128>;
}

impl IntoUint128 for Uint256 {
    fn as_uint128(self) -> StdResult<Uint128> {
        self.try_into()
            .map_err(|err| StdError::ConversionOverflow { source: err })
    }
}

pub trait IntoUint256 {
    fn as_uint256(self) -> Uint256;
}

impl IntoUint256 for Uint128 {
    fn as_uint256(self) -> Uint256 {
        Uint256::from_uint128(self)
    }
}

pub trait IntoDecimal {
    fn as_decimal(self) -> StdResult<Decimal>;
    fn as_decimal_256(self) -> StdResult<Decimal256>;
}

impl IntoDecimal for Uint128 {
    fn as_decimal(self) -> StdResult<Decimal> {
        Decimal::checked_from_ratio(self, Uint128::one()).map_err(|_| {
            StdError::generic_err(format!(
                "Overflow converting {} into Decimal",
                self.to_string()
            ))
        })
    }

    fn as_decimal_256(self) -> StdResult<Decimal256> {
        Decimal256::checked_from_ratio(self, Uint128::one()).map_err(|_| {
            StdError::generic_err(format!(
                "Overflow converting {} into Decimal",
                self.to_string()
            ))
        })
    }
}

impl IntoDecimal for Uint256 {
    fn as_decimal(self) -> StdResult<Decimal> {
        Decimal::checked_from_ratio(self.as_uint128()?, Uint128::one()).map_err(|_| {
            StdError::generic_err(format!(
                "Overflow converting {} into Decimal",
                self.to_string()
            ))
        })
    }

    fn as_decimal_256(self) -> StdResult<Decimal256> {
        Decimal256::checked_from_ratio(self, Uint128::one()).map_err(|_| {
            StdError::generic_err(format!(
                "Overflow converting {} into Decimal",
                self.to_string()
            ))
        })
    }
}

#[test]
pub fn test_convert() {
    let a = Uint128::from(1_000_000_000_000_000_000_000_u128);
    a.as_decimal().unwrap_err();
    a.as_decimal_256().unwrap();
    let mut c = Uint128::MAX.as_uint256();

    let res = std::panic::catch_unwind(|| {
        let _ = Uint128::MAX * Uint128::from(2_u128);
    });

    assert!(res.is_err());

    c = c * Uint256::from_u128(2);

    c.as_uint128().unwrap_err();

    c = c / Uint256::from_u128(3);

    c.as_uint128().unwrap();
}
