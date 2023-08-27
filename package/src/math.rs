use std::{
    fmt::{Debug, Display},
    ops::{Add, Div, Mul, Sub, Neg},
    str::FromStr,
};

use cosmwasm_std::{Decimal, Decimal256, StdError, StdResult, Uint128, Uint256};
use forward_ref::forward_ref_binop;

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

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct SignedDecimal {
    value: Decimal,
    is_positive: bool,
}

impl SignedDecimal {
    pub fn from_decimal(value: Decimal) -> SignedDecimal {
        SignedDecimal {
            value,
            is_positive: true,
        }
    }

    pub fn from_str(value: impl Into<String>) -> StdResult<SignedDecimal> {
        let mut value: String = value.into();
        let mut is_positive = true;
        if value.chars().next().unwrap() == '-' {
            value.remove(0);
            is_positive = false;
        }

        Ok(SignedDecimal {
            value: Decimal::from_str(&value)?,
            is_positive,
        })
    }

    pub fn into_decimal(self) -> StdResult<Decimal> {
        if self.is_positive {
            Ok(self.value)
        } else {
            Err(StdError::generic_err(format!(
                "Invalid conversion from negative SignedDecimal to Decimal, value: {}",
                self.to_string()
            )))
        }
    }
}

impl Display for SignedDecimal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if !self.is_positive {
            f.write_str("-")?;
        }
        f.write_str(&format!("{}", self.value))?;

        Ok(())
    }
}

impl Add for SignedDecimal {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        if self.is_positive == other.is_positive {
            SignedDecimal {
                value: self.value + other.value,
                is_positive: self.is_positive,
            }
        } else if self.value > other.value {
            SignedDecimal {
                value: self.value - other.value,
                is_positive: self.is_positive,
            }
        } else {
            SignedDecimal {
                value: other.value - self.value,
                is_positive: other.is_positive,
            }
        }
    }
}
forward_ref_binop!(impl Add, add for SignedDecimal, SignedDecimal);

impl Sub for SignedDecimal {
    type Output = Self;

    fn sub(self, other: Self) -> Self::Output {
        if self.is_positive != other.is_positive {
            SignedDecimal {
                value: self.value + other.value,
                is_positive: self.is_positive,
            }
        } else {
            if self.value > other.value {
                SignedDecimal {
                    value: self.value - other.value,
                    is_positive: self.is_positive,
                }
            } else {
                SignedDecimal {
                    value: other.value - self.value,
                    is_positive: !self.is_positive,
                }
            }
        }
    }
}
forward_ref_binop!(impl Sub, sub for SignedDecimal, SignedDecimal);

impl Mul for SignedDecimal {
    type Output = Self;

    fn mul(self, other: Self) -> Self::Output {
        SignedDecimal {
            value: self.value * other.value,
            is_positive: self.is_positive == other.is_positive,
        }
    }
}
forward_ref_binop!(impl Mul, mul for SignedDecimal, SignedDecimal);

impl Mul<Decimal> for SignedDecimal {
    type Output = Self;

    fn mul(self, other: Decimal) -> Self::Output {
        SignedDecimal {
            value: self.value * other,
            is_positive: self.is_positive,
        }
    }
}

impl Mul<SignedDecimal> for Decimal {
    type Output = SignedDecimal;

    fn mul(self, other: SignedDecimal) -> Self::Output {
        SignedDecimal {
            value: self * other.value,
            is_positive: other.is_positive,
        }
    }
}

impl Div for SignedDecimal {
    type Output = Self;

    fn div(self, other: Self) -> Self::Output {
        SignedDecimal {
            value: self.value / other.value,
            is_positive: self.is_positive == other.is_positive,
        }
    }
}
forward_ref_binop!(impl Div, div for SignedDecimal, SignedDecimal);

impl Div<Decimal> for SignedDecimal {
    type Output = Self;

    fn div(self, other: Decimal) -> Self::Output {
        SignedDecimal {
            value: self.value / other,
            is_positive: self.is_positive,
        }
    }
}

impl Div<SignedDecimal> for Decimal {
    type Output = SignedDecimal;

    fn div(self, other: SignedDecimal) -> Self::Output {
        SignedDecimal {
            value: self / other.value,
            is_positive: other.is_positive,
        }
    }
}

impl Neg for SignedDecimal {
    type Output = Self;

    fn neg(self) -> Self::Output {
        SignedDecimal {
            value: self.value,
            is_positive: !self.is_positive
        }
    }
}

pub trait IntoSignedDeciaml {
    fn as_signed_decimal(&self) -> SignedDecimal;
}

impl IntoSignedDeciaml for Decimal {
    fn as_signed_decimal(&self) -> SignedDecimal {
        SignedDecimal::from_decimal(self.clone())
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

#[test]
pub fn test_signed_decimal() {
    let a = SignedDecimal::from_str("10").unwrap();
    let b = SignedDecimal::from_str("100").unwrap();
    assert_eq!(a - b, SignedDecimal::from_str("-90").unwrap());
    assert_eq!(b - a, SignedDecimal::from_str("90").unwrap());
    assert_eq!(a * b, SignedDecimal::from_str("1000").unwrap());
    assert_eq!(a / b, SignedDecimal::from_str("0.1").unwrap());

    let a = SignedDecimal::from_str("-10").unwrap();
    let b = SignedDecimal::from_str("100").unwrap();
    assert_eq!(a - b, SignedDecimal::from_str("-110").unwrap());
    assert_eq!(b - a, SignedDecimal::from_str("110").unwrap());
    assert_eq!(a + b, SignedDecimal::from_str("90").unwrap());
    assert_eq!(b + a, SignedDecimal::from_str("90").unwrap());
    assert_eq!(a * b, SignedDecimal::from_str("-1000").unwrap());
    assert_eq!(a / b, SignedDecimal::from_str("-0.1").unwrap());

    let a = SignedDecimal::from_str("-10").unwrap();
    let b = SignedDecimal::from_str("-100").unwrap();
    assert_eq!(a * b, SignedDecimal::from_str("1000").unwrap());
    assert_eq!(a / b, SignedDecimal::from_str("0.1").unwrap());

    let a: Decimal = Decimal::from_str("10").unwrap();
    let b = SignedDecimal::from_str("-100").unwrap();

    assert_eq!(a * b, SignedDecimal::from_str("-1000").unwrap());
    assert_eq!(a / b, SignedDecimal::from_str("-0.1").unwrap());

    let b = -b;
    assert_eq!(b, SignedDecimal::from_str("100").unwrap());



}
