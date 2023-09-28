use std::{
    fmt::{Debug, Display},
    ops::{Add, Div, Mul, Neg, Sub},
    str::FromStr,
};

use cosmwasm_std::{Decimal, Decimal256, StdError, StdResult, Uint128, Uint256};
use forward_ref::forward_ref_binop;
use pyth_sdk_cw::PriceFeedResponse;

use crate::traits::IntoStdResult;

pub trait IntoUint {
    fn into_uint128(self) -> Uint128;
    fn into_uint256(self) -> Uint256;
    fn try_into_uint128(self) -> StdResult<Uint128>;
    fn try_into_uint256(self) -> StdResult<Uint256>;
}

impl IntoUint for Decimal {
    fn into_uint128(self) -> Uint128 {
        self * Uint128::one()
    }

    fn into_uint256(self) -> Uint256 {
        Uint256::from_uint128(self * Uint128::one())
    }
    fn try_into_uint128(self) -> StdResult<Uint128> {
        Ok(self * Uint128::one())
    }

    fn try_into_uint256(self) -> StdResult<Uint256> {
        Ok(Uint256::from_uint128(self * Uint128::one()))
    }
}

impl IntoUint for Decimal256 {
    fn into_uint128(self) -> Uint128 {
        self.into_uint256().into_uint128()
    }

    fn into_uint256(self) -> Uint256 {
        self * Uint256::one()
    }

    fn try_into_uint128(self) -> StdResult<Uint128> {
        self.try_into_uint256()?.try_into_uint128()
    }

    fn try_into_uint256(self) -> StdResult<Uint256> {
        Ok(self * Uint256::one())
    }
}

impl IntoUint for &str {
    fn into_uint128(self) -> Uint128 {
        Uint128::from_str(self).unwrap()
    }

    fn into_uint256(self) -> Uint256 {
        Uint256::from_str(self).unwrap()
    }
    fn try_into_uint128(self) -> StdResult<Uint128> {
        Uint128::from_str(self)
    }

    fn try_into_uint256(self) -> StdResult<Uint256> {
        Uint256::from_str(self)
    }
}

impl IntoUint for u128 {
    fn into_uint128(self) -> Uint128 {
        Uint128::from(self)
    }

    fn into_uint256(self) -> Uint256 {
        Uint256::from(self)
    }

    fn try_into_uint128(self) -> StdResult<Uint128> {
        Ok(Uint128::from(self))
    }

    fn try_into_uint256(self) -> StdResult<Uint256> {
        Ok(Uint256::from(self))
    }
}

impl IntoUint for u64 {
    fn into_uint128(self) -> Uint128 {
        Uint128::from(self)
    }

    fn into_uint256(self) -> Uint256 {
        Uint256::from(self)
    }

    fn try_into_uint128(self) -> StdResult<Uint128> {
        Ok(Uint128::from(self))
    }

    fn try_into_uint256(self) -> StdResult<Uint256> {
        Ok(Uint256::from(self))
    }
}

impl IntoUint for Uint128 {
    fn into_uint128(self) -> Uint128 {
        self
    }

    fn into_uint256(self) -> Uint256 {
        Uint256::from_uint128(self)
    }

    fn try_into_uint128(self) -> StdResult<Uint128> {
        Ok(self)
    }

    fn try_into_uint256(self) -> StdResult<Uint256> {
        Ok(Uint256::from_uint128(self))
    }
}

impl IntoUint for Uint256 {
    fn into_uint128(self) -> Uint128 {
        self.try_into().unwrap()
    }

    fn into_uint256(self) -> Uint256 {
        self
    }

    fn try_into_uint128(self) -> StdResult<Uint128> {
        self.try_into().into_std_result()
    }

    fn try_into_uint256(self) -> StdResult<Uint256> {
        Ok(self)
    }
}

pub trait IntoDecimal {
    fn into_decimal(self) -> Decimal;
    fn into_decimal_256(self) -> Decimal256;
    fn try_into_decimal(self) -> StdResult<Decimal>;
    fn try_into_decimal_256(self) -> StdResult<Decimal256>;
}

impl IntoDecimal for Uint128 {
    fn into_decimal(self) -> Decimal {
        Decimal::from_ratio(self, Uint128::one())
    }

    fn into_decimal_256(self) -> Decimal256 {
        Decimal256::from_ratio(self, Uint256::one())
    }

    fn try_into_decimal(self) -> StdResult<Decimal> {
        Decimal::checked_from_ratio(self, Uint128::one()).into_std_result()
    }

    fn try_into_decimal_256(self) -> StdResult<Decimal256> {
        Decimal256::checked_from_ratio(self, Uint128::one()).into_std_result()
    }
}

impl IntoDecimal for Uint256 {
    fn into_decimal(self) -> Decimal {
        Decimal::from_ratio(self.into_uint128(), Uint128::one())
    }

    fn into_decimal_256(self) -> Decimal256 {
        Decimal256::from_ratio(self, Uint128::one())
    }

    fn try_into_decimal(self) -> StdResult<Decimal> {
        Decimal::checked_from_ratio(self.into_uint128(), Uint128::one()).into_std_result()
    }

    fn try_into_decimal_256(self) -> StdResult<Decimal256> {
        Decimal256::checked_from_ratio(self, Uint128::one()).into_std_result()
    }
}

impl IntoDecimal for &str {
    fn into_decimal(self) -> Decimal {
        Decimal::from_str(self).unwrap()
    }

    fn into_decimal_256(self) -> Decimal256 {
        Decimal256::from_str(self).unwrap()
    }

    fn try_into_decimal(self) -> StdResult<Decimal> {
        Decimal::from_str(self)
    }

    fn try_into_decimal_256(self) -> StdResult<Decimal256> {
        Decimal256::from_str(self)
    }
}

impl IntoDecimal for Decimal {
    fn into_decimal(self) -> Decimal {
        self
    }

    fn into_decimal_256(self) -> Decimal256 {
        Decimal256::from_atomics(self.atomics().into_uint128(), self.decimal_places()).unwrap()
    }

    fn try_into_decimal(self) -> StdResult<Decimal> {
        Ok(self)
    }

    fn try_into_decimal_256(self) -> StdResult<Decimal256> {
        Decimal256::from_atomics(self.atomics().try_into_uint128()?, self.decimal_places())
            .into_std_result()
    }
}

impl IntoDecimal for Decimal256 {
    fn into_decimal(self) -> Decimal {
        Decimal::from_atomics(self.atomics().into_uint128(), self.decimal_places()).unwrap()
    }

    fn into_decimal_256(self) -> Decimal256 {
        self
    }

    fn try_into_decimal(self) -> StdResult<Decimal> {
        Decimal::from_atomics(self.atomics().try_into_uint128()?, self.decimal_places())
            .into_std_result()
    }

    fn try_into_decimal_256(self) -> StdResult<Decimal256> {
        Ok(self)
    }
}

impl IntoDecimal for u128 {
    fn into_decimal(self) -> Decimal {
        Decimal::from_ratio(self, Uint128::one())
    }

    fn into_decimal_256(self) -> Decimal256 {
        Decimal256::from_ratio(self, Uint256::one())
    }

    fn try_into_decimal(self) -> StdResult<Decimal> {
        Decimal::checked_from_ratio(self, Uint128::one()).into_std_result()
    }

    fn try_into_decimal_256(self) -> StdResult<Decimal256> {
        Decimal256::checked_from_ratio(self, Uint128::one()).into_std_result()
    }
}

impl IntoDecimal for u64 {
    fn into_decimal(self) -> Decimal {
        Decimal::from_ratio(self, Uint128::one())
    }

    fn into_decimal_256(self) -> Decimal256 {
        Decimal256::from_ratio(self, Uint256::one())
    }

    fn try_into_decimal(self) -> StdResult<Decimal> {
        Decimal::checked_from_ratio(self, Uint128::one()).into_std_result()
    }

    fn try_into_decimal_256(self) -> StdResult<Decimal256> {
        Decimal256::checked_from_ratio(self, Uint128::one()).into_std_result()
    }
}

impl IntoDecimal for PriceFeedResponse {
    fn into_decimal(self) -> Decimal {
        self.try_into_decimal().unwrap()
    }

    fn into_decimal_256(self) -> Decimal256 {
        self.try_into_decimal_256().unwrap()
    }

    fn try_into_decimal(self) -> StdResult<Decimal> {
        let price = self.price_feed.get_price_unchecked();

        Decimal::from_atomics(price.price as u128, price.expo as u32).into_std_result()
    }

    fn try_into_decimal_256(self) -> StdResult<Decimal256> {
        let price = self.price_feed.get_price_unchecked();

        Decimal256::from_atomics(price.price as u128, price.expo as u32).into_std_result()
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

impl Add<Decimal> for SignedDecimal {
    type Output = Self;

    fn add(self, other: Decimal) -> Self::Output {
        if self.is_positive {
            SignedDecimal {
                value: self.value + other,
                is_positive: true,
            }
        } else if self.value > other {
            SignedDecimal {
                value: self.value - other,
                is_positive: false,
            }
        } else {
            SignedDecimal {
                value: other - self.value,
                is_positive: true,
            }
        }
    }
}

impl Add<SignedDecimal> for Decimal {
    type Output = SignedDecimal;

    fn add(self, other: SignedDecimal) -> Self::Output {
        if other.is_positive {
            SignedDecimal {
                value: other.value + self,
                is_positive: true,
            }
        } else if self > other.value {
            SignedDecimal {
                value: self - other.value,
                is_positive: true,
            }
        } else {
            SignedDecimal {
                value: other.value - self,
                is_positive: false,
            }
        }
    }
}

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

impl Sub<Decimal> for SignedDecimal {
    type Output = Self;

    fn sub(self, other: Decimal) -> Self::Output {
        if !self.is_positive {
            SignedDecimal {
                value: self.value + other,
                is_positive: false,
            }
        } else {
            if self.value > other {
                SignedDecimal {
                    value: self.value - other,
                    is_positive: true,
                }
            } else {
                SignedDecimal {
                    value: other - self.value,
                    is_positive: false,
                }
            }
        }
    }
}

impl Sub<SignedDecimal> for Decimal {
    type Output = SignedDecimal;

    fn sub(self, other: SignedDecimal) -> Self::Output {
        if !other.is_positive {
            SignedDecimal {
                value: self + other.value,
                is_positive: true,
            }
        } else {
            if self > other.value {
                SignedDecimal {
                    value: self - other.value,
                    is_positive: true,
                }
            } else {
                SignedDecimal {
                    value: other.value - self,
                    is_positive: false,
                }
            }
        }
    }
}

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
            is_positive: !self.is_positive,
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
    assert_eq!("0.01", format!("{}", "0.01".into_decimal()));
    assert_eq!("0.01", format!("{}", "0.01".into_decimal_256()));
    assert_eq!(Uint128::one(), "1".into_uint128());
    assert_eq!(Uint256::one(), "1".into_uint256());
    assert_eq!(Uint128::one(), Uint256::one().into_uint128());
    assert_eq!(Uint256::one(), Uint128::one().into_uint256());
    assert_eq!(Decimal::one(), Uint128::one().into_decimal());
    assert_eq!(Decimal256::one(), Uint128::one().into_decimal_256());
    assert_eq!(Decimal::one(), Uint256::one().into_decimal());
    assert_eq!(Decimal256::one(), Uint256::one().into_decimal_256());
    assert_eq!(Uint128::one(), Decimal::one().into_uint128());
    assert_eq!(Uint256::one(), Decimal::one().into_uint256());
    assert_eq!(Uint128::one(), Decimal256::one().into_uint128());
    assert_eq!(Uint256::one(), Decimal256::one().into_uint256());
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
    assert_eq!(a + b, SignedDecimal::from_str("-90").unwrap());
    assert_eq!(a - b, SignedDecimal::from_str("110").unwrap());
    assert_eq!(b - a, SignedDecimal::from_str("-110").unwrap());
    assert_eq!(a * b, SignedDecimal::from_str("-1000").unwrap());
    assert_eq!(a / b, SignedDecimal::from_str("-0.1").unwrap());

    let b = -b;
    assert_eq!(b, SignedDecimal::from_str("100").unwrap());
}
