use {
    crate::{
        math::IntoDecimal,
        traits::{IntoBinary, IntoStdResult, Wrapper},
        wasm::WasmMsgBuilder,
    },
    cosmwasm_schema::cw_serde,
    cosmwasm_std::{
        to_json_binary, Addr, BankMsg, Coin, CosmosMsg, Decimal, StdError, StdResult, Uint128,
        WasmMsg,
    },
    cw_asset::{Asset, AssetError, AssetInfo},
    cw_storage_plus::{Key, KeyDeserialize, Prefixer, PrimaryKey},
    serde::Serialize,
    std::{any::type_name, collections::HashMap, fmt::Display},
};

/// Check if `coins` has a `len() == 1`.
/// If a `denom` is specified, assert them.
pub fn only_one_coin(coins: &Vec<Coin>, denom: Option<String>) -> StdResult<Coin> {
    if coins.len() == 1 {
        let coin = coins.first().unwrap().to_owned();
        match denom {
            Some(denom) => {
                if coin.denom == denom {
                    Ok(coin)
                } else {
                    Err(StdError::generic_err(format!(
                        "Denom not match, found: {}, expected: {}",
                        coin.denom, denom
                    )))
                }
            },
            None => Ok(coin),
        }
    } else {
        Err(StdError::generic_err("Not one coin"))
    }
}

/// Merge 2 `Coins`, checking if the `denom` is the same.
///
/// Cases:
/// - `from: None` - `with: None` -> Return `None`
/// - `from: Some` - `with: None` -> Return `from`
/// - `from: None` - `with: Some` -> Return `with`
/// - `from: Some` - `with: Some` -> Return `Coin:{denom: from.denom, amount: from.amount + with.amount}`
pub fn merge_coin(from: Option<Coin>, with: Option<Coin>) -> StdResult<Option<Coin>> {
    match (from, with) {
        (None, None) => Ok(None),
        (None, Some(with)) => Ok(Some(with)),
        (Some(from), None) => Ok(Some(from)),
        (Some(from), Some(with)) => {
            if from.denom != with.denom {
                return Err(StdError::generic_err("Coin must have same denom"));
            }
            Ok(Some(Coin {
                denom: from.denom.clone(),
                amount: from.amount + with.amount,
            }))
        },
    }
}

/// Transform a `Vec<Coin>` into `HashMap<String, Uint128>`
///
/// if merge_dupplicate is `true`, the function will sum the amount of the same denom
/// otherwise it will return an error if a denom is found multiple times
pub fn vec_coins_to_hashmap(
    coins: Vec<Coin>,
    merge_dupplicate: bool,
) -> StdResult<HashMap<String, Uint128>> {
    let mut map: HashMap<String, Uint128> = HashMap::new();

    for coin in coins {
        match map.get_mut(&coin.denom) {
            Some(alredy_inserted) => {
                if merge_dupplicate {
                    *alredy_inserted += coin.amount;
                } else {
                    return Err(StdError::generic_err(format!(
                        "multiple denom detected, {}",
                        &coin.denom
                    )));
                }
            },
            None => {
                map.insert(coin.denom, coin.amount);
            },
        }
    }

    Ok(map)
}

pub fn assets_multiple_transfer(
    assets: &[Asset],
    to: &Addr,
) -> StdResult<(Vec<CosmosMsg>, Vec<Coin>)> {
    let mut native: Vec<Coin> = vec![];
    let mut increase_allowance: Vec<CosmosMsg> = vec![];

    for asset in assets {
        match &asset.info {
            AssetInfo::Native(_) => native.push(asset.try_into().into_std_result()?),
            AssetInfo::Cw20(cw20_addr) => increase_allowance.push(
                WasmMsg::build_execute(
                    cw20_addr,
                    cw20::Cw20ExecuteMsg::IncreaseAllowance {
                        spender: to.to_string(),
                        amount: asset.amount,
                        expires: None,
                    },
                    vec![],
                )?
                .into(),
            ),
            _ => todo!(),
        }
    }

    Ok((increase_allowance, native))
}

#[allow(clippy::wrong_self_convention)]
pub trait AssetInfoExstender {
    fn into_send_msg(&self, receiver: &Addr, amount: Uint128) -> StdResult<CosmosMsg>;
}

impl AssetInfoExstender for AssetInfo {
    fn into_send_msg(&self, receiver: &Addr, amount: Uint128) -> StdResult<CosmosMsg> {
        match self {
            cw_asset::AssetInfoBase::Native(denom) => Ok(CosmosMsg::Bank(BankMsg::Send {
                to_address: receiver.to_string(),
                amount: vec![Coin::new(amount.u128(), denom)],
            })),
            cw_asset::AssetInfoBase::Cw20(contract_addr) => Ok(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: contract_addr.to_string(),
                msg: to_json_binary(&cw20::Cw20ExecuteMsg::Transfer {
                    recipient: receiver.to_string(),
                    amount,
                })?,
                funds: vec![],
            })),
            // ??
            _ => unimplemented!(),
        }
    }
}

/// Wrapper container for [AssetInfo] for inclding also token precision
#[cw_serde]
#[non_exhaustive]
pub struct AssetInfoPrecisioned {
    pub info: AssetInfo,
    pub precision: u8,
}

impl AssetInfoPrecisioned {
    pub fn new(info: AssetInfo, precision: u8) -> Self {
        Self { info, precision }
    }

    pub fn cw20(contract_addr: &Addr, precision: u8) -> Self {
        Self {
            info: AssetInfo::cw20(contract_addr.clone()),
            precision,
        }
    }

    pub fn native(denom: impl Into<String>, precision: u8) -> Self {
        Self {
            info: AssetInfo::native(denom.into()),
            precision,
        }
    }

    pub fn to_asset(&self, amount: impl Into<AssetAmount>) -> AssetPrecisioned {
        AssetPrecisioned::new(self.clone(), amount.into())
    }

    pub fn from_str(value: &str) -> StdResult<AssetInfoPrecisioned> {
        let words: Vec<&str> = value.split(':').collect();

        let info = match words[0] {
            "native" => {
                if words.len() != 2 {
                    Err(AssetError::InvalidAssetInfoFormat {
                        received: value.into(),
                        should_be: "native:{denom}".into(),
                    })
                } else {
                    AssetInfo::Native(String::from(words[1])).wrap_ok()
                }
            },
            "cw20" => {
                if words.len() != 2 {
                    Err(AssetError::InvalidAssetInfoFormat {
                        received: value.into(),
                        should_be: "cw20:{contract_addr}".into(),
                    })
                } else {
                    AssetInfo::Cw20(Addr::unchecked(words[1])).wrap_ok()
                }
            },
            ty => Err(AssetError::InvalidAssetType { ty: ty.into() }),
        }
        .into_std_result()?;

        let precision = words[3].parse::<u8>().into_std_result()?;

        AssetInfoPrecisioned::new(info, precision).wrap_ok()
    }
}

impl Display for AssetInfoPrecisioned {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} - P: {}", self.info, self.precision)
    }
}

impl Into<AssetInfo> for AssetInfoPrecisioned {
    fn into(self) -> AssetInfo {
        self.info
    }
}

impl<'a> PrimaryKey<'a> for AssetInfoPrecisioned {
    type Prefix = String;
    type SubPrefix = ();
    type Suffix = String;
    type SuperSuffix = Self;

    fn key(&self) -> Vec<Key> {
        let mut keys = vec![];
        match &self.info {
            AssetInfo::Cw20(addr) => {
                keys.extend("cw20:".key());
                keys.extend(addr.key());
            },
            AssetInfo::Native(denom) => {
                keys.extend("native:".key());
                keys.extend(denom.key());
            },
            _ => todo!(),
        };
        // keys.extend(format!(":precision:{}", self.precision).key());
        keys.extend(":precision:".key());
        keys.extend(self.precision.key());
        keys
    }
}

impl KeyDeserialize for AssetInfoPrecisioned {
    type Output = AssetInfoPrecisioned;

    #[inline(always)]
    fn from_vec(mut value: Vec<u8>) -> StdResult<Self::Output> {
        // ignore length prefix
        // we're allowed to do this because we set the key's namespace ourselves
        // in PrimaryKey (first key)
        value.drain(0..2);

        // parse the bytes into an utf8 string
        let s = String::from_utf8(value)?;

        println!("{s}");

        // cast the AssetError to StdError::ParseError
        AssetInfoPrecisioned::from_str(&s)
            .map_err(|err| StdError::parse_err(type_name::<Self::Output>(), err))
    }
}

impl<'a> Prefixer<'a> for AssetInfoPrecisioned {
    fn prefix(&self) -> Vec<Key> {
        self.key()
    }
}

/// Similar to [Asset] but with [AssetInfoPrecisioned] instead of [AssetInfo]
#[cw_serde]
#[non_exhaustive]
pub struct AssetPrecisioned {
    info: AssetInfoPrecisioned,
    amount: Uint128,
}

impl AssetPrecisioned {
    pub fn new(info: AssetInfoPrecisioned, amount: impl Into<AssetAmount>) -> Self {
        Self {
            amount: amount.into().as_precisionless(info.precision),
            info,
        }
    }

    pub fn new_super(info: AssetInfo, precision: u8, amount: impl Into<AssetAmount>) -> Self {
        Self {
            info: AssetInfoPrecisioned::new(info, precision),
            amount: amount.into().as_precisionless(precision),
        }
    }

    pub fn amount_precisioned(&self) -> StdResult<Decimal> {
        Decimal::from_atomics(self.amount, self.info.precision as u32).into_std_result()
    }

    pub fn amount_raw(&self) -> Uint128 {
        self.amount
    }

    pub fn precision(&self) -> u8 {
        self.info.precision
    }

    pub fn info(&self) -> &AssetInfo {
        &self.info.info
    }

    pub fn info_precisioned(&self) -> &AssetInfoPrecisioned {
        &self.info
    }

    pub fn set_amount(&mut self, new_amount: impl Into<AssetAmount>) {
        self.amount = new_amount.into().as_precisionless(self.precision())
    }

    pub fn transfer_msg(&self, to: &Addr) -> StdResult<CosmosMsg> {
        Into::<Asset>::into(self.clone())
            .transfer_msg(to)
            .into_std_result()
    }

    pub fn send_msg<N: Serialize, C: Serialize>(
        &self,
        contract_addr: &Addr,
        native_msg: N,
        cw20_msg: C,
    ) -> StdResult<CosmosMsg> {
        match self.info() {
            AssetInfo::Native(_) => {
                WasmMsg::build_execute(contract_addr, native_msg, vec![self.clone().try_into()?])
            },
            AssetInfo::Cw20(addr) => WasmMsg::build_execute(
                addr,
                cw20::Cw20ExecuteMsg::Send {
                    contract: contract_addr.to_string(),
                    amount: self.amount,
                    msg: cw20_msg.into_binary()?,
                },
                vec![],
            ),
            _ => todo!(),
        }
        .map(|msg| msg.into())
    }

    pub fn clone_with_amount(&self, amount: impl Into<AssetAmount>) -> Self {
        Self {
            amount: amount.into().as_precisionless(self.precision()),
            info: self.info.clone(),
        }
    }

    pub fn as_asset(&self) -> Asset {
        self.clone().into()
    }

    pub fn compute_value_raw(
        &self,
        humanized_price: Decimal,
        precision_modifier: Option<u8>,
    ) -> Uint128 {
        self.amount.mul_floor(
            humanized_price
                / 10_u128
                    .pow((self.precision() - precision_modifier.unwrap_or(0)) as u32)
                    .into_decimal(),
        )
    }

    pub fn compute_humanized_value(&self, humanized_price: Decimal) -> Decimal {
        self.amount_precisioned().unwrap() * humanized_price
    }
}

impl Display for AssetPrecisioned {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", self.info)?;
        writeln!(f, "humanized: {}", self.amount_precisioned().unwrap())?;
        write!(f, "precisionless: {}", self.amount_raw())
    }
}

impl Into<Asset> for AssetPrecisioned {
    fn into(self) -> Asset {
        Asset::new(self.info().clone(), self.amount_raw())
    }
}

impl TryInto<Coin> for AssetPrecisioned {
    type Error = StdError;

    fn try_into(self) -> Result<Coin, Self::Error> {
        match self.info() {
            cw_asset::AssetInfoBase::Native(denom) => {
                Coin::new(self.amount_raw().u128(), denom).wrap_ok()
            },
            cw_asset::AssetInfoBase::Cw20(addr) => Err(StdError::generic_err(format!(
                "Cannot convert {} into Coin",
                addr
            ))),
            _ => todo!(),
        }
    }
}

impl PartialOrd for AssetPrecisioned {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        if self.amount_raw() > other.amount_raw() {
            Some(std::cmp::Ordering::Greater)
        } else if self.amount_raw() < other.amount_raw() {
            Some(std::cmp::Ordering::Less)
        } else {
            Some(std::cmp::Ordering::Equal)
        }
    }
}

/// Input type for [AssetPrecisioned]. Implement [Into] and [From] for different data type
#[cw_serde]
pub enum AssetAmount {
    Precisioned(Decimal),
    Precisionless(Uint128),
}

impl AssetAmount {
    pub fn as_precisioned(&self, precision: u8) -> StdResult<Decimal> {
        match self {
            AssetAmount::Precisioned(amount) => Ok(*amount),
            AssetAmount::Precisionless(amount) => {
                Decimal::from_atomics(*amount, precision as u32).into_std_result()
            },
        }
    }

    pub fn as_precisionless(&self, precision: u8) -> Uint128 {
        match self {
            AssetAmount::Precisioned(amount) => Uint128::from(10 as u32)
                .pow(precision as u32)
                .mul_floor(*amount),
            AssetAmount::Precisionless(amount) => *amount,
        }
    }
}

impl From<Uint128> for AssetAmount {
    fn from(value: Uint128) -> Self {
        Self::Precisionless(value)
    }
}

impl From<u128> for AssetAmount {
    fn from(value: u128) -> Self {
        Self::Precisionless(value.into())
    }
}

impl From<Decimal> for AssetAmount {
    fn from(value: Decimal) -> Self {
        Self::Precisioned(value)
    }
}

mod math {
    use std::ops::{Add, Div, Mul, Sub};

    use cosmwasm_std::{Decimal, StdError, StdResult};

    use super::{AssetAmount, AssetInfoPrecisioned, AssetPrecisioned};

    macro_rules! forward_ref_binop_clone {
        (impl $imp:ident, $method:ident for $t:ty, $u:ty) => {
            impl<'a> $imp<$u> for &'a $t {
                type Output = <$t as $imp<$u>>::Output;

                #[inline]
                fn $method(self, other: $u) -> <$t as $imp<$u>>::Output {
                    $imp::$method(self.clone(), other)
                }
            }

            impl $imp<&$u> for $t {
                type Output = <$t as $imp<$u>>::Output;

                #[inline]
                fn $method(self, other: &$u) -> <$t as $imp<$u>>::Output {
                    $imp::$method(self, other.clone())
                }
            }

            impl $imp<&$u> for &$t {
                type Output = <$t as $imp<$u>>::Output;

                #[inline]
                fn $method(self, other: &$u) -> <$t as $imp<$u>>::Output {
                    $imp::$method(self.clone(), other.clone())
                }
            }
        };
    }

    fn validate_asset(
        info: &AssetInfoPrecisioned,
        rhs: &AssetInfoPrecisioned,
        operation: impl Into<String>,
    ) -> StdResult<()> {
        if &info != &rhs {
            Err(StdError::generic_err(format!(
                "{} cannot be performed between {} and {}",
                operation.into(),
                info,
                rhs
            )))
        } else {
            Ok(())
        }
    }

    impl Add for AssetPrecisioned {
        type Output = Self;

        fn add(self, rhs: Self) -> Self::Output {
            validate_asset(&self.info, &rhs.info, "Add").unwrap();
            let amount = self.amount_raw();
            AssetPrecisioned::new(self.info, amount + rhs.amount_raw())
        }
    }

    impl<T> Add<T> for AssetPrecisioned
    where
        T: Into<AssetAmount>,
    {
        type Output = Self;

        fn add(self, rhs: T) -> Self::Output {
            self.clone_with_amount(
                self.amount_raw() + rhs.into().as_precisionless(self.precision()),
            )
        }
    }

    impl<T> Add<T> for &AssetPrecisioned
    where
        T: Into<AssetAmount>,
    {
        type Output = AssetPrecisioned;

        fn add(self, rhs: T) -> Self::Output {
            self.clone_with_amount(
                self.amount_raw() + rhs.into().as_precisionless(self.precision()),
            )
        }
    }

    forward_ref_binop_clone!(impl Add, add for AssetPrecisioned, AssetPrecisioned);

    impl Sub for AssetPrecisioned {
        type Output = Self;

        fn sub(self, rhs: Self) -> Self::Output {
            validate_asset(&self.info, &rhs.info, "Sub").unwrap();
            let amount = self.amount_raw();
            AssetPrecisioned::new(self.info, amount - rhs.amount_raw())
        }
    }

    impl<T> Sub<T> for AssetPrecisioned
    where
        T: Into<AssetAmount>,
    {
        type Output = Self;

        fn sub(self, rhs: T) -> Self::Output {
            self.clone_with_amount(
                self.amount_raw() - rhs.into().as_precisionless(self.precision()),
            )
        }
    }

    impl<T> Sub<T> for &AssetPrecisioned
    where
        T: Into<AssetAmount>,
    {
        type Output = AssetPrecisioned;

        fn sub(self, rhs: T) -> Self::Output {
            self.clone_with_amount(
                self.amount_raw() - rhs.into().as_precisionless(self.precision()),
            )
        }
    }

    forward_ref_binop_clone!(impl Sub, sub for AssetPrecisioned, AssetPrecisioned);

    impl Mul for AssetPrecisioned {
        type Output = Self;

        fn mul(self, rhs: Self) -> Self::Output {
            validate_asset(&self.info, &rhs.info, "Mul").unwrap();
            let amount = self.amount_raw();
            AssetPrecisioned::new(self.info, amount * rhs.amount_raw())
        }
    }

    impl Mul<Decimal> for AssetPrecisioned {
        type Output = Self;

        fn mul(self, rhs: Decimal) -> Self::Output {
            let amount = self.amount_precisioned().unwrap();
            AssetPrecisioned::new(self.info, amount * rhs)
        }
    }

    forward_ref_binop_clone!(impl Mul, mul for AssetPrecisioned, AssetPrecisioned);
    forward_ref_binop_clone!(impl Mul, mul for AssetPrecisioned, Decimal);

    impl Div for AssetPrecisioned {
        type Output = Self;

        fn div(self, rhs: Self) -> Self::Output {
            validate_asset(&self.info, &rhs.info, "Div").unwrap();
            let amount = self.amount_raw();
            AssetPrecisioned::new(self.info, amount / rhs.amount_raw())
        }
    }

    impl Div<Decimal> for AssetPrecisioned {
        type Output = Self;

        fn div(self, rhs: Decimal) -> Self::Output {
            let amount = self.amount_precisioned().unwrap();
            AssetPrecisioned::new(self.info, amount / rhs)
        }
    }

    forward_ref_binop_clone!(impl Div, div for AssetPrecisioned, AssetPrecisioned);
    forward_ref_binop_clone!(impl Div, div for AssetPrecisioned, Decimal);
}

#[test]
fn t_1() {
    let asset = AssetPrecisioned::new(
        AssetInfoPrecisioned::native("uusd", 6),
        "100".into_decimal(),
    );

    assert_eq!(
        200_000_000_u128,
        asset
            .compute_value_raw("2".into_decimal(), 6.wrap_some())
            .u128()
    );

    assert_eq!(
        200_u128,
        asset.compute_value_raw("2".into_decimal(), None).u128()
    );

    assert_eq!(
        "200".into_decimal(),
        asset.compute_humanized_value("2".into_decimal())
    );

    let smaller = AssetPrecisioned::new_super(AssetInfo::native("uluna"), 6, 100);
    let greater = AssetPrecisioned::new_super(AssetInfo::native("uluna"), 6, 200);

    assert!(smaller < greater);
    assert!(greater > smaller);
    assert!(greater == greater.clone());

    let inj: AssetInfoPrecisioned = AssetInfoPrecisioned::native("inj", 18);

    // If to_asset arg type is Uint, it will be converted to AssetAmount::Precisionless
    let amount_on_wallet_from_precisionless = inj.to_asset(Uint128::new(1_000_000_000_000_000_000));
    // If to_asset arg type is Decimal, it will be converted to AssetAmount::Precisioned
    let amount_on_wallet_from_precisioned = inj.to_asset("1".into_decimal());

    assert_eq!(
        amount_on_wallet_from_precisionless,
        amount_on_wallet_from_precisioned
    );
}

#[test]
fn math() {
    let asset = AssetInfoPrecisioned::native("eth", 18);

    let a = AssetPrecisioned::new(asset.clone(), "100".into_decimal());
    let b = AssetPrecisioned::new(asset.clone(), "200".into_decimal());

    assert_eq!(
        AssetPrecisioned::new(asset.clone(), "300".into_decimal()),
        &a + &b
    );

    assert_eq!(
        AssetPrecisioned::new(asset.clone(), "100".into_decimal()),
        &b - &a
    );

    assert_eq!(
        AssetPrecisioned::new(asset.clone(), "400".into_decimal()),
        &b * "2".into_decimal()
    );

    assert_eq!(
        AssetPrecisioned::new(asset.clone(), "200.000000000000000200".into_decimal()),
        &b + 200_u128
    );

    assert_eq!(
        AssetPrecisioned::new(asset.clone(), "400.000000000000000000".into_decimal()),
        b + "200".into_decimal()
    );
}
