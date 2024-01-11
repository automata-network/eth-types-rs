use std::prelude::v1::*;

use super::{SH160, SH256, SU256, SU64};
use hex::HexBytes;
use rlp_derive::RlpEncodable;
use serde::{Deserialize, Serialize, de::DeserializeOwned};

#[derive(Default, Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Receipt {
    // Consensus fields: These fields are defined by the Yellow Paper
    pub r#type: Option<SU64>,      //
    pub root: Option<HexBytes>,    //
    pub status: SU64,              //
    pub cumulative_gas_used: SU64, // gencodec:"required"`
    pub logs_bloom: HexBytes,      // gencodec:"required"`
    pub logs: Vec<Log>,            // gencodec:"required"`

    // Implementation fields: These fields are added by geth when processing a transaction.
    // They are stored in the chain database.
    pub transaction_hash: SH256,         //  gencodec:"required"`
    pub contract_address: Option<SH160>, //
    pub gas_used: SU64,                  //  gencodec:"required"`

    // Inclusion information: These fields provide information about the inclusion of the
    // transaction corresponding to this receipt.
    pub block_hash: Option<SH256>,   // common.Hash `:",omitempty"`
    pub block_number: Option<SU256>, // *big.Int    `:",omitempty"`
    pub transaction_index: SU64,     // uint        `:""`
}

pub trait ReceiptTrait: Clone + DeserializeOwned {
    fn status(&self) -> SU64;
    fn gas_used(&self) -> SU64;
    fn transaction_hash(&self) -> &SH256;
}

impl ReceiptTrait for Receipt {
    fn gas_used(&self) -> SU64 {
        self.gas_used
    }
    fn status(&self) -> SU64 {
        self.status
    }
    fn transaction_hash(&self) -> &SH256 {
        &self.transaction_hash
    }
}

impl Receipt {
    pub fn status_encoding(&self) -> HexBytes {
        match &self.root {
            Some(n) => n.clone(),
            None => {
                if self.status == SU64::from(0) {
                    //fail
                    HexBytes::new()
                } else {
                    vec![1_u8; 1].into()
                }
            }
        }
    }

    pub fn succ(&self) -> bool {
        self.status.as_u64() == 1
    }

    pub fn rlp_encode(&self) -> ReceiptRLP {
        ReceiptRLP {
            post_state_or_status: self.status_encoding(),
            cumulative_gas_used: self.cumulative_gas_used.as_u64(),
            bloom: self.logs_bloom.clone(),
            logs: self.logs.clone(),
        }
    }

    pub fn rlp_bytes(&self) -> Vec<u8> {
        let encode = self.rlp_encode();
        let ty = self.r#type.clone().unwrap_or(SU64::from(0)).as_u64();

        let mut stream = rlp::RlpStream::new();
        match ty {
            0 => {}
            1 => {
                const PREFIX: [u8; 1] = [1];
                stream.append_raw(&PREFIX, 0);
            }
            2 => {
                const PREFIX: [u8; 1] = [2];
                stream.append_raw(&PREFIX, 0);
            }
            _ => unreachable!(),
        }
        stream.append(&encode);
        stream.out().to_vec()
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, RlpEncodable, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ReceiptRLP {
    post_state_or_status: HexBytes,
    cumulative_gas_used: u64,
    bloom: HexBytes,
    logs: Vec<Log>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Log {
    // Consensus fields:
    // address of the contract that generated the event
    pub address: SH160,
    // list of topics provided by the contract.
    pub topics: Vec<SH256>,
    // supplied by the contract, usually ABI-encoded
    pub data: HexBytes,

    // Derived fields. These fields are filled in by the node
    // but not secured by consensus.
    // block in which the transaction was included
    pub block_number: SU64,
    // hash of the transaction
    pub transaction_hash: SH256,
    // index of the transaction in the block
    pub transaction_index: SU64,
    // hash of the block in which the transaction was included
    pub block_hash: SH256,
    // index of the log in the block
    pub log_index: SU64,

    // The Removed field is true if this log was reverted due to a chain reorganisation.
    // You must pay attention to this field if you receive logs through a filter query.
    pub removed: bool,
}

impl rlp::Encodable for Log {
    fn rlp_append(&self, s: &mut rlp::RlpStream) {
        #[derive(RlpEncodable, Serialize)]
        #[serde(rename_all = "camelCase")]
        struct RlpLog {
            address: SH160,
            topics: Vec<SH256>,
            data: HexBytes,
        }
        RlpLog {
            address: self.address.clone(),
            topics: self.topics.clone(),
            data: self.data.clone(),
        }
        .rlp_append(s)
    }
}
