use std::prelude::v1::*;

use ethereum_types::FromStrRadixErr;
pub use ethereum_types::{H160, H256, U256, U64};
use hex::FromHexError;
use serde::{de::Error, Deserialize, Deserializer, Serializer};
use std::ops::Deref;

lazy_static::lazy_static! {
    static ref GWEI: SU256 = "1000000000".into();
    static ref ZERO_ADDR: SH160 = "0x0000000000000000000000000000000000000000".into();
}

pub fn gwei() -> &'static SU256 {
    GWEI.deref()
}

pub fn zero_addr() -> &'static SH160 {
    ZERO_ADDR.deref()
}

macro_rules! impl_wrap_rlp {
    ($wrap:ident, $ori:ty) => {
        impl rlp::Decodable for $wrap {
            fn decode(rlp: &rlp::Rlp) -> Result<Self, rlp::DecoderError> {
                Ok($wrap(<$ori>::decode(rlp)?))
            }
        }
        impl rlp::Encodable for $wrap {
            fn rlp_append(&self, s: &mut rlp::RlpStream) {
                self.0.rlp_append(s)
            }
        }
    };
}
pub(crate) use impl_wrap_rlp;

macro_rules! impl_wrap_add {
    ($wrap:ident, $ori:ty) => {
        impl std::ops::SubAssign for $wrap {
            fn sub_assign(&mut self, rhs: Self) {
                self.0.sub_assign(rhs.0)
            }
        }

        impl std::ops::SubAssign<&$wrap> for $wrap {
            fn sub_assign(&mut self, rhs: &Self) {
                self.0.sub_assign(rhs.0)
            }
        }

        impl std::ops::AddAssign for $wrap {
            fn add_assign(&mut self, rhs: Self) {
                self.0.add_assign(rhs.0)
            }
        }

        impl std::ops::AddAssign<&$wrap> for $wrap {
            fn add_assign(&mut self, rhs: &Self) {
                self.0.add_assign(rhs.0)
            }
        }

        impl std::ops::Div for $wrap {
            type Output = $wrap;
            fn div(self, rhs: Self) -> Self::Output {
                $wrap(self.0.div(rhs.0))
            }
        }

        impl std::ops::Div for &$wrap {
            type Output = $wrap;
            fn div(self, rhs: Self) -> Self::Output {
                $wrap(self.0.div(rhs.0))
            }
        }

        impl std::ops::Mul for $wrap {
            type Output = Self;
            fn mul(self, rhs: Self) -> Self::Output {
                Self(self.0 * rhs.0)
            }
        }

        impl std::ops::Mul for &$wrap {
            type Output = $wrap;
            fn mul(self, rhs: Self) -> Self::Output {
                $wrap(self.0 * rhs.0)
            }
        }

        impl std::ops::Mul<&$wrap> for $wrap {
            type Output = Self;
            fn mul(self, rhs: &$wrap) -> Self::Output {
                Self(self.0 * rhs.0)
            }
        }

        impl std::ops::Add for $wrap {
            type Output = Self;
            fn add(self, rhs: Self) -> Self::Output {
                Self(self.0 + rhs.0)
            }
        }

        impl std::ops::Add for &$wrap {
            type Output = $wrap;
            fn add(self, rhs: Self) -> Self::Output {
                $wrap(self.0 + rhs.0)
            }
        }

        impl std::ops::Add<&$wrap> for $wrap {
            type Output = Self;
            fn add(self, rhs: &$wrap) -> Self::Output {
                Self(self.0 + rhs.0)
            }
        }

        impl std::ops::Sub for $wrap {
            type Output = Self;
            fn sub(self, rhs: Self) -> Self::Output {
                Self(self.0 - rhs.0)
            }
        }

        impl std::ops::Sub for &$wrap {
            type Output = $wrap;
            fn sub(self, rhs: &$wrap) -> Self::Output {
                $wrap(self.0 - rhs.0)
            }
        }

        impl std::ops::Sub<&$wrap> for $wrap {
            type Output = Self;
            fn sub(self, rhs: &$wrap) -> Self::Output {
                Self(self.0 - rhs.0)
            }
        }
    };
}

macro_rules! impl_wrap_cmp {
    ($wrap:ident, $ori:ty) => {
        impl std::cmp::PartialOrd<$wrap> for $wrap {
            fn partial_cmp(&self, other: &$wrap) -> Option<core::cmp::Ordering> {
                self.raw().partial_cmp(other.raw())
            }
        }

        impl std::cmp::Ord for $wrap {
            fn cmp(&self, other: &Self) -> core::cmp::Ordering {
                self.partial_cmp(other).unwrap()
            }
        }
    };
}

macro_rules! impl_type_from {
    ($wrap:ident, $raw:ty) => {
        impl From<$raw> for $wrap {
            fn from(raw: $raw) -> Self {
                Self(raw.into())
            }
        }
    };
}

macro_rules! impl_wrap_type {
    ($wrap:ident, $ori:ty, $array:ty, $deser:ident, $ser:ident) => {
        #[derive(Clone, Default, PartialEq, Eq, Copy)]
        pub struct $wrap($ori);
        impl core::ops::Deref for $wrap {
            type Target = $ori;
            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }
        impl core::ops::DerefMut for $wrap {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.0
            }
        }
        impl std::fmt::Debug for $wrap {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                self.0.fmt(f)
            }
        }
        impl std::fmt::Display for $wrap {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                self.0.fmt(f)
            }
        }
        impl From<$ori> for $wrap {
            fn from(val: $ori) -> Self {
                Self(val.into())
            }
        }
        impl From<$wrap> for $ori {
            fn from(val: $wrap) -> Self {
                val.0
            }
        }
        impl From<$array> for $wrap {
            fn from(val: $array) -> Self {
                let mut ori = <$ori>::default();
                ori.0 = val;
                Self(ori)
            }
        }
        impl $wrap {
            pub fn raw_mut(&mut self) -> &mut $ori {
                &mut self.0
            }
            pub fn raw(&self) -> &$ori {
                &self.0
            }
        }
        impl<'de> serde::Deserialize<'de> for $wrap {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                Ok(Self($deser(deserializer)?))
            }
        }
        impl serde::Serialize for $wrap {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                $ser(&self.0, serializer)
            }
        }
    };
}
pub(crate) use impl_wrap_type;

macro_rules! impl_ssz_type {
    ($name:ident, $size:ty) => {
        impl ssz::Codec for $name {
            type Size = <$size as ssz::Codec>::Size;
        }
        impl ssz::Encode for $name {
            fn encode(&self) -> Vec<u8> {
                let n: &$size = &self.raw().0;
                ssz::Encode::encode(n)
            }
        }
        impl ssz::Decode for $name {
            fn decode(value: &[u8]) -> Result<Self, ssz::Error> {
                let n: $size = ssz::Decode::decode(value)?;
                Ok(n.into())
            }
        }
    };
}

macro_rules! impl_asref {
    ($name:ident) => {
        impl AsRef<[u8]> for $name {
            fn as_ref(&self) -> &[u8] {
                self.as_bytes()
            }
        }
    };
}

impl_wrap_type!(SH256, H256, [u8; 32], deserialize_h256, serialize_h256);
impl_wrap_cmp!(SH256, H256);
impl_wrap_rlp!(SH256, H256);
impl_ssz_type!(SH256, [u8; 32]);
impl_asref!(SH256);

impl From<&str> for SH256 {
    fn from(val: &str) -> Self {
        parse_string_h256(val).unwrap().into()
    }
}

impl From<&str> for SH160 {
    fn from(val: &str) -> Self {
        parse_string_h160(val).unwrap().into()
    }
}

impl From<&str> for SU256 {
    fn from(val: &str) -> Self {
        parse_string_u256(val).unwrap().into()
    }
}

impl From<i32> for SU256 {
    fn from(val: i32) -> Self {
        let val: U256 = val.into();
        val.into()
    }
}

impl From<usize> for SU256 {
    fn from(val: usize) -> Self {
        let val: U256 = val.into();
        val.into()
    }
}

impl_wrap_type!(SH160, H160, [u8; 20], deserialize_h160, serialize_h160);
impl_wrap_rlp!(SH160, H160);
impl_ssz_type!(SH160, [u8; 20]);
impl_wrap_cmp!(SH160, H160);
impl_asref!(SH160);

impl From<&SH160> for SH256 {
    fn from(addr: &SH160) -> Self {
        let mut new = Self::default();
        new.0[12..].copy_from_slice(addr.as_bytes());
        new
    }
}

impl_wrap_type!(SU256, U256, [u64; 4], deserialize_u256, serialize_u256_hex);
impl_wrap_rlp!(SU256, U256);
impl_wrap_add!(SU256, U256);
impl_wrap_cmp!(SU256, U256);
impl_type_from!(SU256, u64);
impl_type_from!(SU256, [u8; 32]);
impl_ssz_type!(SU256, [u64; 4]);
impl From<SU256> for [u8; 32] {
    fn from(val: SU256) -> [u8; 32] {
        val.0.into()
    }
}
impl From<SU256> for SH256 {
    fn from(val: SU256) -> Self {
        let tmp: [u8; 32] = val.into();
        tmp.into()
    }
}

impl SU256 {
    pub fn one() -> SU256 {
        1u64.into()
    }
    pub fn zero() -> SU256 {
        Self::default()
    }
    pub fn from_big_endian(slice: &[u8]) -> SU256 {
        U256::from_big_endian(slice).into()
    }
    pub fn from_little_endian(slice: &[u8]) -> SU256 {
        U256::from_little_endian(slice).into()
    }
}

impl PartialEq<u64> for SU256 {
    fn eq(&self, other: &u64) -> bool {
        let val: SU256 = (*other).into();
        val.raw().eq(self.raw())
    }
}

impl_wrap_type!(SU64, U64, [u64; 1], deserialize_u64, serialize_u64_hex);
impl_wrap_rlp!(SU64, U64);
impl_type_from!(SU64, u64);
impl_wrap_add!(SU64, u64);
impl_wrap_cmp!(SU64, u64);
impl_ssz_type!(SU64, [u64; 1]);

impl SU64 {
    pub fn as_u256(&self) -> SU256 {
        self.as_u64().into()
    }
}

impl From<SU64> for SU256 {
    fn from(val: SU64) -> Self {
        val.as_u64().into()
    }
}

pub fn serialize_h256<S>(item: &H256, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&encode_string_h256(item))
}

pub fn encode_string_h256(h256: &H256) -> String {
    // here we just make use of the display functionality of H256
    // the debug string prints in full form (hex)
    format!("{:?}", h256).to_owned()
}

pub fn deserialize_h256<'de, D>(deserializer: D) -> Result<H256, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    Ok(parse_string_h256(&s).map_err(Error::custom)?)
}

pub fn parse_string_h256(h256_str: &str) -> Result<H256, hex::FromHexError> {
    // hex string can be one of two forms
    // 1. 0x1123a5
    // 2.   1123a5
    // NOTE: for ethereum h256, the bytestring is represented in "big-endian" form
    // that is for an array of the form
    //   lsb [a5, 23, 11] msb
    // index: 0   1   2
    // the corresponding bytestring is of the form:
    // 0xa523110000..00
    //
    // Here, we'll strip the initial 0x and parse it using hex::decode
    // which gives us the exact representation we want.
    // 0xa5 23 11 00 .. 00
    //   a5 23 11 00 .. 00
    //  [a5,23,11,00,..,00] <- in the right endianness

    let bytes = hex::decode(h256_str.trim_start_matches("0x"))?;
    // pad the bytes to 32bytes
    let mut padded_bytes = [0_u8; 32];
    padded_bytes[32 - bytes.len()..].copy_from_slice(&bytes);

    Ok(H256::from_slice(&padded_bytes))
}

pub fn serialize_u256<S>(item: &U256, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let item_str = format!("{}", item);
    serializer.serialize_str(&item_str)
}

pub fn serialize_u256_hex<S>(item: &U256, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let item_str = format!("0x{:x}", item);
    serializer.serialize_str(&item_str)
}

pub fn deserialize_u256<'de, D>(deserializer: D) -> Result<U256, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    Ok(parse_string_u256(&s).map_err(Error::custom)?)
}

pub fn parse_string_u256(u256_str: &str) -> Result<U256, FromStrRadixErr> {
    if u256_str.starts_with("0x") {
        if u256_str.len() % 2 == 1 {
            let new_u256_str = "0x0".to_owned() + &u256_str[2..];
            U256::from_str_radix(&new_u256_str, 16)
        } else {
            U256::from_str_radix(u256_str, 16)
        }
    } else {
        U256::from_str_radix(u256_str, 10)
    }
}

pub fn deserialize_h160<'de, D>(deserializer: D) -> Result<H160, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    let addr = parse_string_h160(&s).map_err(Error::custom)?;
    Ok(addr)
}

pub fn serialize_h160<S>(item: &H160, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let item_str = encode_string_h160(&item);
    serializer.serialize_str(&item_str)
}

pub fn parse_string_h160(h160_str: &str) -> Result<H160, FromHexError> {
    let bytes = hex::decode(h160_str.trim_start_matches("0x"))?;
    let mut padded_bytes = [0_u8; 20];
    padded_bytes[20 - bytes.len()..].copy_from_slice(&bytes);
    Ok(H160::from_slice(&padded_bytes))
}

pub fn encode_string_h160(h160: &H160) -> String {
    // here we just make use of the display functionality of H160
    // the debug string prints in full form (hex)
    format!("{:?}", h160).to_owned()
}

pub fn serialize_u64<S>(item: &U64, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let item_str = format!("{}", item);
    serializer.serialize_str(&item_str)
}

pub fn serialize_u64_hex<S>(item: &U64, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let item_str = format!("0x{:x}", item);
    serializer.serialize_str(&item_str)
}

pub fn deserialize_u64<'de, D>(deserializer: D) -> Result<U64, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    Ok(parse_string_u64(&s).map_err(Error::custom)?)
}

pub fn parse_string_u64(u64_str: &str) -> Result<U64, FromStrRadixErr> {
    if u64_str.starts_with("0x") {
        U64::from_str_radix(u64_str, 16)
    } else {
        U64::from_str_radix(u64_str, 10)
    }
}

pub fn deserialize_u32<'de, D>(deserializer: D) -> Result<u32, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    let v = if s.starts_with("0x") {
        let s = s.trim_start_matches("0x");
        u32::from_str_radix(&s, 16)
    } else {
        u32::from_str_radix(&s, 10)
    }
    .map_err(Error::custom)?;
    Ok(v)
}
