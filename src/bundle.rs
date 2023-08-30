use std::prelude::v1::*;

use crate::{PoolTx, Signer, SH160, SH256, SU64};
use crypto::keccak_hash;
use hex::HexBytes;
use rlp_derive::{RlpDecodable, RlpEncodable};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct Bundle {
    pub txs: Vec<PoolTx>,
    pub block_number: SU64,
    pub min_timestamp: Option<u64>,
    pub max_timestamp: Option<u64>,
    pub uuid: String,
    pub refund_percent: u64,
    pub refund_recipient: SH160,
}

impl Bundle {
    pub fn hash(&self) -> SH256 {
        let mut hash_bytes = Vec::with_capacity(self.txs.len() * 32);
        for tx in &self.txs {
            hash_bytes.extend_from_slice(tx.hash.as_bytes());
        }
        keccak_hash(&hash_bytes).into()
    }

    pub fn to_rlp(&self) -> BundleRlp {
        let txs = self.txs.iter().map(|n| n.to_bytes()).collect();
        BundleRlp {
            txs,
            block_number: self.block_number,
            uuid: self.uuid.clone(),
            refund_percent: self.refund_percent,
            refund_recipient: self.refund_recipient,
        }
    }

    pub fn to_bytes(&self) -> HexBytes {
        let data: Vec<u8> = rlp::encode(&self.to_rlp()).into();
        data.into()
    }

    pub fn from_bytes(signer: &Signer, data: &[u8]) -> Result<Self, rlp::DecoderError> {
        let val: BundleRlp = rlp::decode(data)?;
        Self::from_rlp(signer, val)
    }

    pub fn from_rlp(signer: &Signer, val: BundleRlp) -> Result<Self, rlp::DecoderError> {
        let mut txs = Vec::with_capacity(val.txs.len());
        for tx in val.txs {
            txs.push(PoolTx::from_bytes(signer, &tx)?);
        }
        Ok(Bundle {
            txs,
            block_number: val.block_number,
            uuid: val.uuid,
            min_timestamp: None,
            max_timestamp: None,
            refund_percent: val.refund_percent,
            refund_recipient: val.refund_recipient,
        })
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, RlpEncodable, RlpDecodable, PartialEq, Eq)]
pub struct BundleRlp {
    pub txs: Vec<HexBytes>,
    pub block_number: SU64,
    pub uuid: String,
    pub refund_percent: u64,
    pub refund_recipient: SH160,
}

pub enum PoolItemType {
    Tx,
    Bundle,
}

#[derive(Debug)]
pub enum PoolItem {
    Tx(PoolTx),
    Bundle(Bundle),
}

impl PoolItem {
    pub fn from_bytes(signer: &Signer, data: &[u8]) -> Result<Self, rlp::DecoderError> {
        if data.len() < 1 {
            return Err(rlp::DecoderError::RlpIsTooShort);
        }
        Ok(match data[0] {
            1 => PoolItem::Tx(PoolTx::from_bytes(signer, &data[1..])?),
            2 => PoolItem::Bundle(Bundle::from_bytes(signer, &data[1..])?),
            _ => return Err(rlp::DecoderError::Custom("unknown tx prefix")),
        })
    }
}
