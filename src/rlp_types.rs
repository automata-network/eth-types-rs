use std::prelude::v1::*;

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use core::ops::Deref;

#[derive(Default, Clone, Debug, PartialEq, Eq)]
pub struct Nilable<T>(pub Option<T>);

impl<T> From<Option<T>> for Nilable<T> {
    fn from(opt: Option<T>) -> Self {
        Self(opt)
    }
}

impl<T> From<Nilable<T>> for Option<T> {
    fn from(opt: Nilable<T>) -> Self {
        opt.0
    }
}

impl<T> From<T> for Nilable<T> {
    fn from(n: T) -> Self {
        Self(Some(n))
    }
}

impl<T> Deref for Nilable<T> {
    type Target = Option<T>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> rlp::Encodable for Nilable<T>
where
    T: rlp::Encodable,
{
    fn rlp_append(&self, s: &mut rlp::RlpStream) {
        match &self.0 {
            Some(n) => n.rlp_append(s),
            None => {
                s.append_raw(&rlp::NULL_RLP, 0);
            }
        }
    }
}

impl<T> rlp::Decodable for Nilable<T>
where
    T: rlp::Decodable,
{
    fn decode(rlp: &rlp::Rlp) -> Result<Self, rlp::DecoderError> {
        if rlp.is_empty() {
            return Ok(Self(None));
        }
        Ok(Self(Some(T::decode(rlp)?)))
    }
}

impl<'de, T> Deserialize<'de> for Nilable<T>
where
    T: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(Self(<Option<T>>::deserialize(deserializer)?))
    }
}

impl<T> Serialize for Nilable<T>
where
    T: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.0.serialize(serializer)
    }
}
