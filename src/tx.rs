use std::prelude::v1::*;

use crate::Hasher;

use super::{BlockHeader, Nilable, Signer, SH160, SH256, SU256, SU64};
use crypto::{
    keccak_hash, secp256k1_rec_sign_bytes, Secp256k1PrivateKey, Secp256k1RecoverableSignature,
};
use hex::HexBytes;
use rlp_derive::{RlpDecodable, RlpEncodable};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::sync::Arc;

#[derive(
    Default, Clone, Debug, Deserialize, Serialize, RlpEncodable, RlpDecodable, PartialEq, Eq,
)]
#[serde(rename_all = "camelCase")]
pub struct LegacyTx {
    pub nonce: SU64,        // nonce of sender account
    pub gas_price: SU256,   // wei per gas
    pub gas: SU64,          // gas limit
    pub to: Nilable<SH160>, // nil means contract creation
    pub value: SU256,       // wei amount
    pub data: HexBytes,     // contract invocation input data
    pub v: SU256,
    pub r: SU256,
    pub s: SU256,
}

#[derive(Clone, Debug, Deserialize, Serialize, RlpEncodable, RlpDecodable, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AccessListTx {
    pub chain_id: SU256,                          // destination chain ID
    pub nonce: SU64,                              // nonce of sender account
    pub gas_price: SU256,                         // wei per gas
    pub gas: SU64,                                // gas limit
    pub to: Nilable<SH160>,                       // nil means contract creation
    pub value: SU256,                             // wei amount
    pub data: HexBytes,                           // contract invocation input data
    pub access_list: Vec<TransactionAccessTuple>, // EIP-2930 access list
    pub v: SU256,
    pub r: SU256,
    pub s: SU256,
}

#[derive(Clone, Debug, Deserialize, Serialize, RlpEncodable, RlpDecodable, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DynamicFeeTx {
    pub chain_id: SU256,
    pub nonce: SU64,
    pub max_priority_fee_per_gas: SU256,
    pub max_fee_per_gas: SU256,
    pub gas: SU64,
    pub to: Nilable<SH160>,
    pub value: SU256,
    pub data: HexBytes,
    pub access_list: Vec<TransactionAccessTuple>,
    pub v: SU256,
    pub r: SU256,
    pub s: SU256,
}

#[derive(Clone, Debug, Deserialize, Serialize, RlpEncodable, RlpDecodable, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct TransactionAccessTuple {
    pub address: SH160,
    pub storage_keys: Vec<SH256>,
}

impl TransactionAccessTuple {
    pub fn new(acc: SH160) -> Self {
        Self {
            address: acc,
            storage_keys: vec![],
        }
    }
}

impl From<Vec<&str>> for TransactionAccessTuple {
    fn from(list: Vec<&str>) -> Self {
        let address = list[0].into();
        let mut storage_keys = Vec::with_capacity(list.len() - 1);
        for item in list {
            storage_keys.push(item.into());
        }
        Self {
            address,
            storage_keys,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Transaction {
    pub block_hash: Option<SH256>,
    pub block_number: Option<SU64>,
    pub from: Option<SH160>,
    pub gas: SU64,
    pub gas_price: Option<SU256>,
    pub max_fee_per_gas: Option<SU256>,
    pub max_priority_fee_per_gas: Option<SU256>,
    pub hash: SH256,
    pub input: HexBytes,
    pub nonce: SU64,
    pub to: Option<SH160>,
    pub transaction_index: Option<SU64>,
    pub value: SU256,
    pub r#type: SU64,
    pub access_list: Option<Vec<TransactionAccessTuple>>,
    pub chain_id: Option<SU256>,
    pub v: SU256,
    pub r: SU256,
    pub s: SU256,
}

impl rlp::Encodable for TransactionInner {
    fn rlp_append(&self, s: &mut rlp::RlpStream) {
        match self {
            TransactionInner::Legacy(tx) => tx.rlp_append(s),
            TransactionInner::AccessList(tx) => {
                const PREFIX: [u8; 1] = [1];
                s.append_raw(&PREFIX, 0);
                tx.rlp_append(s);
            }
            TransactionInner::DynamicFee(tx) => {
                const PREFIX: [u8; 1] = [2];
                s.append_raw(&PREFIX, 0);
                tx.rlp_append(s);
            }
        }
    }
}

impl rlp::Decodable for TransactionInner {
    fn decode(rlp: &rlp::Rlp) -> Result<Self, rlp::DecoderError> {
        if rlp.is_list() {
            return Ok(Self::Legacy(LegacyTx::decode(rlp)?));
        }
        let n = rlp.as_raw();
        if n.len() < 1 {
            return Err(rlp::DecoderError::RlpIsTooShort);
        }

        match n[0] {
            1 => Ok(Self::AccessList(rlp::decode(&n[1..])?)),
            2 => Ok(Self::DynamicFee(rlp::decode(&n[1..])?)),
            _ => Err(rlp::DecoderError::Custom("unknown tx prefix")),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TransactionInner {
    Legacy(LegacyTx),
    AccessList(AccessListTx),
    DynamicFee(DynamicFeeTx),
}

impl Serialize for TransactionInner {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let tx: HexBytes = self.to_bytes().into();
        serializer.serialize_str(&format!("{}", tx))
    }
}

impl<'de> Deserialize<'de> for TransactionInner {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s: HexBytes = Deserialize::deserialize(deserializer)?;
        TransactionInner::from_bytes(&s)
            .map_err(|err| serde::de::Error::custom(format!("{:?}", err)))
    }
}

impl core::cmp::PartialOrd for TransactionInner {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl core::cmp::Ord for TransactionInner {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.to_bytes().cmp(&other.to_bytes())
    }
}

impl TransactionInner {
    pub fn to_bytes(&self) -> Vec<u8> {
        rlp::encode(self).to_vec()
    }

    pub fn from_bytes(data: &[u8]) -> Result<Self, rlp::DecoderError> {
        rlp::decode(data)
    }

    pub fn to_transaction(self, header: Option<&BlockHeader>) -> Transaction {
        let mut target = Transaction::default();
        target.hash = self.hash();
        match self {
            Self::Legacy(tx) => {
                target.r#type = 0.into();
                target.nonce = tx.nonce;
                target.gas_price = Some(tx.gas_price);
                target.gas = tx.gas;
                target.to = tx.to.into();
                target.value = tx.value;
                target.input = tx.data;
                target.v = tx.v;
                target.r = tx.r;
                target.s = tx.s;
            }
            Self::AccessList(tx) => {
                target.r#type = 1.into();
                target.chain_id = Some(tx.chain_id);
                target.nonce = tx.nonce;
                target.gas_price = Some(tx.gas_price);
                target.gas = tx.gas;
                target.to = tx.to.into();
                target.value = tx.value;
                target.input = tx.data;
                target.access_list = Some(tx.access_list);
                target.v = tx.v;
                target.r = tx.r;
                target.s = tx.s;
            }
            Self::DynamicFee(tx) => {
                target.r#type = 2.into();
                target.chain_id = Some(tx.chain_id);
                target.nonce = tx.nonce;
                target.max_priority_fee_per_gas = Some(tx.max_priority_fee_per_gas);
                target.max_fee_per_gas = Some(tx.max_fee_per_gas.clone());
                target.gas = tx.gas;
                target.gas_price = Some(tx.max_fee_per_gas.clone()); // maybe wrong if we have block info
                if let Some(header) = header {
                    let gas_tip_cap = tx.max_priority_fee_per_gas.clone();
                    let gas_fee_cap = tx.max_fee_per_gas.clone();
                    target.gas_price = Some(
                        gas_fee_cap
                            .raw()
                            .clone()
                            .min(header.base_fee_per_gas.raw().clone() + gas_tip_cap.raw())
                            .into(),
                    );
                }
                target.to = tx.to.into();
                target.value = tx.value;
                target.input = tx.data;
                target.access_list = Some(tx.access_list);
                target.v = tx.v;
                target.r = tx.r;
                target.s = tx.s;
            }
        }
        if let Some(header) = header {
            target.block_hash = Some(header.hash());
            target.block_number = Some(header.number.as_u64().into());
        }
        target
    }

    pub fn value(&self) -> SU256 {
        match self {
            Self::Legacy(tx) => tx.value.clone(),
            Self::DynamicFee(tx) => tx.value.clone(),
            Self::AccessList(tx) => tx.value.clone(),
        }
    }

    pub fn nonce(&self) -> u64 {
        match self {
            Self::Legacy(tx) => tx.nonce.as_u64(),
            Self::DynamicFee(tx) => tx.nonce.as_u64(),
            Self::AccessList(tx) => tx.nonce.as_u64(),
        }
    }

    pub fn gas_limit(&self) -> u64 {
        match self {
            Self::Legacy(tx) => tx.gas.as_u64(),
            Self::DynamicFee(tx) => tx.gas.as_u64(),
            Self::AccessList(tx) => tx.gas.as_u64(),
        }
    }

    pub fn gas_price(&self, base_fee: Option<SU256>) -> SU256 {
        match self {
            Self::Legacy(tx) => tx.gas_price,
            Self::AccessList(tx) => tx.gas_price,
            Self::DynamicFee(tx) => match base_fee {
                Some(base_fee) => tx
                    .max_fee_per_gas
                    .min(base_fee + &tx.max_priority_fee_per_gas),
                None => tx.max_fee_per_gas,
            },
        }
    }

    pub fn cost(&self, base_fee: Option<SU256>) -> SU256 {
        let gas: SU256 = self.gas().into();
        let gas_price = self.gas_price(base_fee);
        let value = self.value();
        (gas * gas_price) + value
    }

    pub fn access_list(&self) -> Option<&[TransactionAccessTuple]> {
        match self {
            Self::Legacy(_) => None,
            Self::DynamicFee(tx) => Some(&tx.access_list),
            Self::AccessList(tx) => Some(&tx.access_list),
        }
    }

    pub fn max_fee_per_gas(&self) -> &SU256 {
        match self {
            Self::Legacy(tx) => &tx.gas_price,
            Self::DynamicFee(tx) => &tx.max_fee_per_gas,
            Self::AccessList(tx) => &tx.gas_price,
        }
    }

    pub fn reward(&self, gas: u64, base_fee: Option<&SU256>) -> Option<SU256> {
        self.effective_gas_tip(base_fee)
            .map(|item| item * SU256::from(gas))
    }

    pub fn max_priority_fee_per_gas(&self) -> &SU256 {
        match self {
            Self::Legacy(tx) => &tx.gas_price,
            Self::DynamicFee(tx) => &tx.max_priority_fee_per_gas,
            Self::AccessList(tx) => &tx.gas_price,
        }
    }

    pub fn input(&self) -> &[u8] {
        match self {
            Self::Legacy(tx) => &tx.data,
            Self::DynamicFee(tx) => &tx.data,
            Self::AccessList(tx) => &tx.data,
        }
    }

    pub fn gas(&self) -> SU64 {
        match self {
            Self::Legacy(tx) => tx.gas.clone(),
            Self::DynamicFee(tx) => tx.gas.clone(),
            Self::AccessList(tx) => tx.gas.clone(),
        }
    }

    pub fn to(&self) -> Option<SH160> {
        match self {
            Self::Legacy(tx) => tx.to.clone().into(),
            Self::DynamicFee(tx) => tx.to.clone().into(),
            Self::AccessList(tx) => tx.to.clone().into(),
        }
    }

    pub fn sender(&self, signer: &Signer) -> SH160 {
        signer.sender(self)
    }

    pub fn signature(&self, chain_id: u64) -> Secp256k1RecoverableSignature {
        match self {
            Self::Legacy(tx) => {
                let v = match tx.v.as_u64() {
                    0 | 1 => tx.v.as_u64(),
                    27 | 28 => tx.v.as_u64() - 27,
                    _protected => tx.v.as_u64() - chain_id * 2 - 8 - 27,
                };
                Secp256k1RecoverableSignature {
                    v: v as _,
                    r: tx.r.clone().into(),
                    s: tx.s.clone().into(),
                }
            }
            Self::DynamicFee(tx) => Secp256k1RecoverableSignature {
                v: tx.v.as_u32() as u8,
                r: tx.r.clone().into(),
                s: tx.s.clone().into(),
            },
            Self::AccessList(tx) => Secp256k1RecoverableSignature {
                v: tx.v.as_u32() as u8,
                r: tx.r.clone().into(),
                s: tx.s.clone().into(),
            },
        }
    }

    pub fn ty(&self) -> u64 {
        match self {
            Self::Legacy(_) => 0,
            Self::AccessList(_) => 1,
            Self::DynamicFee(_) => 2,
        }
    }

    pub fn sign(&mut self, prvkey: &Secp256k1PrivateKey, chain_id: u64) {
        let mut trim_suffix = 0;
        match self {
            Self::Legacy(tx) => {
                tx.v = chain_id.into();
                tx.r = Default::default();
                tx.s = Default::default();
            }
            Self::AccessList(tx) => {
                tx.v = Default::default();
                tx.r = Default::default();
                tx.s = Default::default();
                trim_suffix = 3;
            }
            Self::DynamicFee(tx) => {
                tx.v = Default::default();
                tx.r = Default::default();
                tx.s = Default::default();
                trim_suffix = 3;
            }
        }
        let mut signed_txn_bytes = self.to_bytes();
        if trim_suffix > 0 {
            // truncate the signature
            signed_txn_bytes[1] -= trim_suffix;
            signed_txn_bytes.truncate(signed_txn_bytes.len() - trim_suffix as usize);
        }
        let rec_sig = secp256k1_rec_sign_bytes(prvkey, &signed_txn_bytes);
        match self {
            Self::Legacy(tx) => {
                tx.v = (u64::from(rec_sig.v) + chain_id * 2 + 35).into();
                tx.r = rec_sig.r.into();
                tx.s = rec_sig.s.into();
            }
            Self::DynamicFee(tx) => {
                tx.v = u64::from(rec_sig.v).into();
                tx.r = rec_sig.r.into();
                tx.s = rec_sig.s.into();
            }
            Self::AccessList(tx) => {
                tx.v = u64::from(rec_sig.v).into();
                tx.r = rec_sig.r.into();
                tx.s = rec_sig.s.into();
            }
        }
    }

    pub fn hash(&self) -> SH256 {
        Hasher::hash(self)
    }

    pub fn sign_msg(&self, chain_id: &SU256) -> Vec<u8> {
        let data = match self {
            TransactionInner::DynamicFee(tx) => {
                // stream.append_raw(bytes, item_count)
                let mut s = rlp::RlpStream::new_list(9);
                s.append(&tx.chain_id);
                s.append(&tx.nonce);
                s.append(&tx.max_priority_fee_per_gas);
                s.append(&tx.max_fee_per_gas);
                s.append(&tx.gas);
                s.append(&tx.to);
                s.append(&tx.value);
                s.append(&tx.data);
                s.append_list(&tx.access_list);
                let mut rlp = s.out().to_vec();
                let mut out = vec![2];
                out.append(&mut rlp);
                out
            }
            TransactionInner::AccessList(tx) => {
                // stream.append_raw(bytes, item_count)
                let mut s = rlp::RlpStream::new_list(8);
                s.append(&tx.chain_id);
                s.append(&tx.nonce);
                s.append(&tx.gas_price);
                s.append(&tx.gas);
                s.append(&tx.to);
                s.append(&tx.value);
                s.append(&tx.data);
                s.append_list(&tx.access_list);

                let mut rlp = s.out().to_vec();
                let mut out = vec![1];
                out.append(&mut rlp);
                out
            }
            TransactionInner::Legacy(tx) => {
                let v = tx.v.as_u64();
                let is_protected = v != 27 && v != 28 && v != 1 && v != 0;
                let mut len = 9;
                if !is_protected {
                    len = 6;
                }
                let mut s = rlp::RlpStream::new_list(len);
                s.append(&tx.nonce);
                s.append(&tx.gas_price);
                s.append(&tx.gas);
                s.append(&tx.to);
                s.append(&tx.value);
                s.append(&tx.data);

                if is_protected {
                    s.append(chain_id);
                    s.append(&0usize);
                    s.append(&0usize);
                }

                s.out().into()
            }
        };
        data
    }
}

impl Hasher for TransactionInner {
    fn hash(&self) -> SH256 {
        let data = rlp::encode(self).to_vec();
        let mut hash = SH256::default();
        hash.as_bytes_mut().copy_from_slice(&keccak_hash(&data));
        hash
    }
}

impl Transaction {
    pub fn inner(self) -> Option<TransactionInner> {
        Some(match self.r#type.as_u64() {
            0 => TransactionInner::Legacy(LegacyTx {
                nonce: self.nonce,
                gas_price: self.gas_price?,
                gas: self.gas,
                to: self.to.into(),
                value: self.value,
                data: self.input,
                v: self.v,
                r: self.r,
                s: self.s,
            }),
            1 => TransactionInner::AccessList(AccessListTx {
                chain_id: self.chain_id?,
                nonce: self.nonce,
                gas_price: self.gas_price?,
                gas: self.gas,
                to: self.to.into(),
                value: self.value,
                data: self.input,
                access_list: self.access_list?,
                v: self.v,
                r: self.r,
                s: self.s,
            }),
            2 => TransactionInner::DynamicFee(DynamicFeeTx {
                chain_id: self.chain_id?,
                nonce: self.nonce,
                max_priority_fee_per_gas: self.max_priority_fee_per_gas?,
                max_fee_per_gas: self.max_fee_per_gas?,
                access_list: self.access_list?,
                gas: self.gas,
                to: self.to.into(),
                value: self.value,
                data: self.input,
                v: self.v,
                r: self.r,
                s: self.s,
            }),
            _ => return None,
        })
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct PoolTx {
    pub caller: SH160,
    pub tx: Arc<TransactionInner>,
    pub access_list: Arc<Vec<TransactionAccessTuple>>,
    pub hash: SH256,
    pub gas: u64,
    pub allow_revert: bool,
    pub block: u64,
    pub result: String,
}

impl PoolTx {
    pub fn with_tx(signer: &Signer, tx: TransactionInner) -> Self {
        Self::with_acl(signer, tx, Vec::new(), 0, 0, "".into(), true)
    }

    pub fn with_acl(
        signer: &Signer,
        tx: TransactionInner,
        acl: Vec<TransactionAccessTuple>,
        gas: u64,
        blk: u64,
        result: String,
        allow_revert: bool,
    ) -> Self {
        let hash = tx.hash();
        let caller = signer.sender(&tx);
        Self {
            caller,
            tx: Arc::new(tx),
            access_list: Arc::new(acl),
            hash,
            gas,
            allow_revert,
            block: blk,
            result,
        }
    }

    pub fn from_rlp(signer: &Signer, tx: PoolTxRlp) -> Result<Self, rlp::DecoderError> {
        let access_list = rlp::Rlp::new(&tx.access_list).as_list()?;
        let inner = rlp::decode(&tx.tx)?;
        Ok(Self::with_acl(
            signer,
            inner,
            access_list,
            tx.gas,
            tx.blk,
            tx.result,
            tx.allow_revert,
        ))
    }

    pub fn from_bytes(signer: &Signer, data: &[u8]) -> Result<Self, rlp::DecoderError> {
        let tx_rlp: PoolTxRlp = rlp::decode(data)?;
        let tx = Self::from_rlp(signer, tx_rlp)?;
        Ok(tx)
    }

    pub fn to_rlp(&self) -> PoolTxRlp {
        let tx: Vec<u8> = rlp::encode(self.tx.as_ref()).into();
        let access_list = Arc::as_ref(&self.access_list);
        let access_list: Vec<u8> = rlp::encode_list(&access_list).into();
        let access_list = access_list.into();

        PoolTxRlp {
            tx: tx.into(),
            access_list,
            gas: self.gas,
            blk: self.block,
            result: self.result.clone(),
            allow_revert: self.allow_revert,
        }
    }

    pub fn to_bytes(&self) -> HexBytes {
        let data: Vec<u8> = rlp::encode(&self.to_rlp()).into();
        data.into()
    }
}

pub struct PoolTxRef<'a> {
    pub tx: &'a TransactionInner,
    pub access_list: &'a [TransactionAccessTuple],
    pub hash: &'a SH256,
    pub gas: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq, RlpEncodable, RlpDecodable)]
pub struct PoolTxRlp {
    pub tx: HexBytes,
    pub access_list: HexBytes,
    pub gas: u64,
    pub blk: u64,
    pub result: String,
    pub allow_revert: bool,
}

pub trait TxTrait: Clone + std::fmt::Debug + Serialize {
    fn gas_price(&self, base_fee: Option<SU256>) -> SU256;
    fn max_priority_fee_per_gas(&self) -> &SU256;
    fn gas(&self) -> SU64;
    fn hash(&self) -> SH256;
    fn max_fee_per_gas(&self) -> &SU256;
    fn to(&self) -> Option<SH160>;
    fn value(&self) -> SU256;
    fn input(&self) -> &[u8];
    fn nonce(&self) -> u64;
    fn access_list(&self) -> Option<&[TransactionAccessTuple]>;
    fn gas_limit(&self) -> u64;
    fn sender(&self, signer: &Signer) -> SH160;
    fn to_json_map(&self) -> Map<String, Value>;

    fn effective_gas_tip(&self, base_fee: Option<&SU256>) -> Option<SU256> {
        match base_fee {
            None => Some(self.max_priority_fee_per_gas().clone()),
            Some(base_fee) => {
                let gas_fee_cap = self.max_fee_per_gas();
                if gas_fee_cap < base_fee {
                    None
                } else {
                    Some(
                        self.max_priority_fee_per_gas()
                            .clone()
                            .min(gas_fee_cap - base_fee),
                    )
                }
            }
        }
    }
}

impl TxTrait for TransactionInner {
    fn gas_price(&self, base_fee: Option<SU256>) -> SU256 {
        TransactionInner::gas_price(self, base_fee)
    }
    fn max_priority_fee_per_gas(&self) -> &SU256 {
        TransactionInner::max_priority_fee_per_gas(&self)
    }
    fn gas(&self) -> SU64 {
        TransactionInner::gas(&self)
    }
    fn hash(&self) -> SH256 {
        TransactionInner::hash(&self)
    }
    fn max_fee_per_gas(&self) -> &SU256 {
        TransactionInner::max_fee_per_gas(&self)
    }
    fn to(&self) -> Option<SH160> {
        TransactionInner::to(self)
    }
    fn value(&self) -> SU256 {
        TransactionInner::value(&self)
    }
    fn input(&self) -> &[u8] {
        TransactionInner::input(&self)
    }
    fn nonce(&self) -> u64 {
        TransactionInner::nonce(&self)
    }
    fn access_list(&self) -> Option<&[TransactionAccessTuple]> {
        TransactionInner::access_list(self)
    }
    fn gas_limit(&self) -> u64 {
        TransactionInner::gas_limit(self)
    }
    fn sender(&self, signer: &Signer) -> SH160 {
        TransactionInner::sender(self, signer)
    }
    fn to_json_map(&self) -> Map<String, Value> {
        let tx = match self {
            TransactionInner::AccessList(tx) => serde_json::to_value(&tx).unwrap(),
            TransactionInner::Legacy(tx) => serde_json::to_value(&tx).unwrap(),
            TransactionInner::DynamicFee(tx) => serde_json::to_value(&tx).unwrap(),
        };
        match tx {
            Value::Object(n) => n,
            _ => unreachable!(),
        }
    }
}

// impl TxTrait for Transaction {
//     fn access_list(&self) -> Option<&[TransactionAccessTuple]> {
//         self.access_list.as_ref()
//     }

//     fn gas(&self) -> SU64 {
//         self.gas
//     }

//     fn gas_limit(&self) -> u64 {
//         self.gas
//     }

//     fn gas_price(&self, base_fee: Option<SU256>) -> SU256 {

//     }
// }
