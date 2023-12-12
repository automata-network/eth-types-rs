use std::prelude::v1::*;

use super::{TransactionAccessTuple, SH160, SH256, SU256, SU64};
use crypto::keccak_hash;
use hex::HexBytes;
use std::borrow::Cow;

use rlp_derive::{RlpDecodable, RlpEncodable};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq, RlpEncodable, RlpDecodable)]
pub struct StateAccount {
    pub nonce: u64,
    pub balance: SU256,
    pub root: SH256,
    pub code_hash: SH256,
}

impl Default for StateAccount {
    fn default() -> Self {
        Self {
            nonce: 0,
            balance: 0.into(),
            root: Self::empty_root().into(),
            code_hash: Self::empty_code_hash().into(),
        }
    }
}

impl StateAccount {
    pub fn empty_root() -> SH256 {
        "0x56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421".into()
    }

    pub fn empty_code_hash() -> SH256 {
        "0xc5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470".into()
    }

    pub fn is_exist(&self) -> bool {
        self != &StateAccount::default()
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        if self.is_exist() {
            rlp::encode(self).into()
        } else {
            Vec::new()
        }
    }

    pub fn from_bytes(data: &[u8]) -> Result<Self, rlp::DecoderError> {
        if data.len() == 0 {
            return Ok(Self::default());
        }
        rlp::decode(data)
    }
}

pub trait StateAccountTrait:
    rlp::Encodable + rlp::Decodable + Default + Clone + std::fmt::Debug + Send + 'static
{
    fn is_exist(&self) -> bool;
    fn set_balance(&mut self, val: SU256) -> bool;
    fn balance(&self) -> SU256;
    fn set_nonce(&mut self, val: u64) -> bool;
    fn nonce(&self) -> u64;
    fn code_hash(&self) -> SH256;
    fn set_code(&mut self, code: &[u8]) -> bool;
    fn root(&self) -> SH256;
    fn update_root(&mut self, root: SH256) -> bool;
}

impl StateAccountTrait for StateAccount {
    fn is_exist(&self) -> bool {
        StateAccount::is_exist(self)
    }

    fn balance(&self) -> SU256 {
        self.balance
    }

    fn code_hash(&self) -> SH256 {
        self.code_hash
    }

    fn nonce(&self) -> u64 {
        self.nonce
    }

    fn root(&self) -> SH256 {
        self.root
    }

    fn set_balance(&mut self, val: SU256) -> bool {
        if self.balance == val {
            return false;
        }
        self.balance = val;
        true
    }

    fn set_code(&mut self, code: &[u8]) -> bool {
        let hash: SH256 = keccak_hash(code).into();
        if self.code_hash == hash {
            return false;
        }
        self.code_hash = hash;
        true
    }

    fn set_nonce(&mut self, val: u64) -> bool {
        if self.nonce == val {
            return false;
        }
        self.nonce = val;
        true
    }

    fn update_root(&mut self, root: SH256) -> bool {
        if self.root == root {
            return false;
        }
        self.root = root;
        true
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub struct AccountResult {
    pub address: SH160,
    pub account_proof: Vec<HexBytes>,
    pub balance: SU256,
    pub code_hash: SH256,
    pub nonce: SU64,
    pub storage_hash: SH256,
    pub storage_proof: Vec<StorageResult>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct StorageResult {
    pub key: HexBytes,
    pub value: SU256, // FIXME: maybe it's longer than 32bytes?
    pub proof: Vec<HexBytes>,
}

#[derive(Debug, Clone, Default)]
pub struct FetchStateResult {
    pub acc: Option<AccountResult>,
    pub code: Option<HexBytes>,
}

#[derive(Debug, Clone)]
pub struct FetchState<'a> {
    pub access_list: Option<Cow<'a, TransactionAccessTuple>>,
    pub code: Option<SH160>,
}

impl<'a> FetchState<'a> {
    pub fn is_match<'b>(&self, other: &FetchState<'b>) -> bool {
        self.get_addr() == other.get_addr()
    }

    pub fn get_addr(&self) -> Option<&SH160> {
        if let Some(addr) = &self.access_list {
            return Some(&addr.address);
        }
        if let Some(addr) = &self.code {
            return Some(addr);
        }
        return None;
    }

    pub fn merge(&mut self, other: Self) {
        if !self.is_match(&other) {
            return;
        }
        if let (None, Some(_)) = (&self.code, other.code) {
            self.code = other.code;
        }
        match (&mut self.access_list, other.access_list) {
            (None, Some(n)) => self.access_list = Some(n),
            (None, None) => {}
            (Some(_), None) => {}
            (Some(a), Some(b)) => {
                if a.as_ref() == b.as_ref() {
                    return;
                }
                for key in &b.storage_keys {
                    if a.storage_keys.contains(&key) {
                        a.to_mut().storage_keys.push(key.clone());
                    }
                }
            }
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AccessListResult {
    pub access_list: Vec<TransactionAccessTuple>,
    pub error: Option<String>,
    pub gas_used: SU64,
}

impl AccessListResult {
    pub fn ensure(&mut self, caller: &SH160, to: Option<SH160>) {
        self.get_or_insert(caller);
        if let Some(to) = to {
            self.get_or_insert(&to);
        }
    }

    pub fn get_or_insert(&mut self, addr: &SH160) -> &mut TransactionAccessTuple {
        if let Some(i) = self
            .access_list
            .iter()
            .position(|item| &item.address == addr)
        {
            &mut self.access_list[i]
        } else {
            self.access_list.push(TransactionAccessTuple {
                address: addr.clone(),
                storage_keys: Vec::new(),
            });
            self.access_list.last_mut().unwrap()
        }
    }
}
