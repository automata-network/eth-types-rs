use std::prelude::v1::*;

use super::{
    deserialize_u64, parse_string_h256, parse_string_u64, serialize_u64_hex, Nilable, Receipt,
    Transaction, TransactionInner, SH160, SH256, SU256, SU64,
};
use crypto::keccak_hash;
use ethereum_types::U64;
use hash256_std_hasher::Hash256StdHasher;
use hex::HexBytes;
use rlp_derive::{RlpDecodable, RlpEncodable};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize, Serializer};
use std::iter::Iterator;
use std::sync::Arc;

crate::impl_wrap_type!(
    BlockNonce,
    U64,
    [u64; 1],
    deserialize_u64,
    serialize_u64_hex
);
// impl_ssz_type!(BlockNonce, ssz::U8);

impl rlp::Encodable for BlockNonce {
    fn rlp_append(&self, s: &mut rlp::RlpStream) {
        let bytes = self.as_u64().to_be_bytes();
        (&bytes[..]).rlp_append(s)
    }
}

impl rlp::Decodable for BlockNonce {
    fn decode(rlp: &rlp::Rlp) -> Result<Self, rlp::DecoderError> {
        let tmp: Vec<u8> = rlp.as_val()?;
        if tmp.len() != 8 {
            return Err(rlp::DecoderError::RlpInvalidLength);
        }
        Ok(Self(U64::from_big_endian(&tmp)))
    }
}

#[derive(
    Default, Clone, Debug, Deserialize, Serialize, RlpEncodable, RlpDecodable, PartialEq, Eq,
)]
#[serde(rename_all = "camelCase")]
pub struct BlockHeader {
    pub parent_hash: SH256,
    pub sha3_uncles: SH256,
    pub miner: SH160,
    pub state_root: SH256,
    pub transactions_root: SH256,
    pub receipts_root: SH256,
    pub logs_bloom: HexBytes,
    pub difficulty: SU256,
    pub number: SU64,
    pub gas_limit: SU64,
    pub gas_used: SU64,
    pub timestamp: SU64,
    pub extra_data: HexBytes,
    pub mix_hash: SH256,
    pub nonce: BlockNonce,
    // BaseFee was added by EIP-1559 and is ignored in legacy headers.
    pub base_fee_per_gas: SU256,
    // WithdrawalsHash was added by EIP-4895 and is ignored in legacy headers.
    pub withdrawals_root: Nilable<SH256>,
}

impl BlockHeader {
    pub fn hash(&self) -> SH256 {
        let data = rlp::encode(self).to_vec();
        let mut hash = SH256::default();
        hash.as_bytes_mut().copy_from_slice(&keccak_hash(&data));
        return hash;
    }
}

pub trait BlockTrait: Clone + DeserializeOwned {}

pub trait BlockHeaderTrait: Clone + DeserializeOwned {
    fn gas_limit(&self) -> SU64;
    fn base_fee(&self) -> Option<SU256>;
    fn number(&self) -> SU64;
    fn miner(&self) -> &SH160;
    fn timestamp(&self) -> SU64;
    fn hash(&self) -> SH256;
}

impl BlockHeaderTrait for BlockHeader {
    fn gas_limit(&self) -> SU64 {
        self.gas_limit
    }
    fn base_fee(&self) -> Option<SU256> {
        Some(self.base_fee_per_gas)
    }
    fn number(&self) -> SU64 {
        self.number
    }
    fn miner(&self) -> &SH160 {
        &self.miner
    }
    fn timestamp(&self) -> SU64 {
        self.timestamp
    }
    fn hash(&self) -> SH256 {
        BlockHeader::hash(&self)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Block {
    #[serde(flatten)]
    pub header: BlockHeader,
    pub transactions: Vec<Transaction>,
    pub withdrawals: Option<Vec<Withdrawal>>, // rlp: optional
}

impl BlockTrait for Block {}

#[derive(Clone, Debug, Deserialize, Serialize, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Withdrawal {
    pub index: SU64,
    pub validator_index: SU64,
    pub address: SH160,
    pub amount: SU64,
}

impl rlp::Encodable for Withdrawal {
    fn rlp_append(&self, s: &mut rlp::RlpStream) {
        s.begin_unbounded_list();
        s.append(&self.index);
        s.append(&self.validator_index);
        s.append(&self.address.as_bytes());
        s.append(&self.amount);
        s.finalize_unbounded_list();
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct BlockSimple<H, W> {
    #[serde(flatten)]
    pub header: H,
    pub transactions: Vec<SH256>,
    pub withdrawals: Option<Vec<W>>, // rlp: optional
}

impl Block {
    pub fn new(
        mut header: BlockHeader,
        txs: Vec<Arc<TransactionInner>>,
        receipts: &[Receipt],
        withdrawals: Option<Vec<Withdrawal>>,
    ) -> Self {
        assert_eq!(txs.len(), receipts.len());
        let empty_root_hash: SH256 =
            "0x56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421".into();
        if txs.len() == 0 {
            header.transactions_root = empty_root_hash.clone();
        } else {
            let txs: Vec<_> = txs.iter().map(|tx| tx.to_bytes()).collect();
            header.transactions_root = triehash::ordered_trie_root::<KeccakHasher, _>(txs).into();
        }
        if receipts.len() == 0 {
            header.receipts_root = empty_root_hash.clone();
        } else {
            header.logs_bloom = create_bloom(receipts.iter()).to_hex();
            let rs: Vec<_> = receipts.iter().map(|r| r.rlp_bytes()).collect();
            header.receipts_root = triehash::ordered_trie_root::<KeccakHasher, _>(rs).into();
        }
        header.sha3_uncles =
            "0x1dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347".into();
        let transactions = txs
            .iter()
            .map(|tx| tx.as_ref().clone().to_transaction(Some(&header)))
            .collect();

        if let Some(withdrawals) = &withdrawals {
            header.withdrawals_root = Some(withdrawal_root(&withdrawals)).into();
        }

        Block {
            header,
            transactions,
            withdrawals,
        }
    }
}

pub fn withdrawal_root(withdrawals: &[Withdrawal]) -> SH256 {
    if withdrawals.len() == 0 {
        "0x56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421".into()
    } else {
        let wd: Vec<Vec<u8>> = withdrawals.iter().map(|r| rlp::encode(r).into()).collect();
        triehash::ordered_trie_root::<KeccakHasher, _>(wd).into()
    }
}

pub fn create_bloom<'a>(receipts: impl Iterator<Item = &'a Receipt>) -> Bloom {
    let mut buf = [0_u8; 6];
    let mut bin = Bloom::new();
    for receipt in receipts {
        for log in &receipt.logs {
            bin.add(&log.address.raw().0[..], &mut buf);
            for b in &log.topics {
                bin.add(&b.raw().0[..], &mut buf);
            }
        }
    }
    return bin;
}

#[derive(Debug)]
pub struct Bloom([u8; 256]);

impl Default for Bloom {
    fn default() -> Self {
        Self(unsafe { std::mem::zeroed() })
    }
}

impl From<HexBytes> for Bloom {
    fn from(val: HexBytes) -> Bloom {
        let mut bin = Bloom::new();
        bin.0.copy_from_slice(&val);
        bin
    }
}

impl From<Bloom> for HexBytes {
    fn from(val: Bloom) -> HexBytes {
        val.to_hex()
    }
}

impl Bloom {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn to_hex(&self) -> HexBytes {
        let mut val = Vec::with_capacity(self.0.len());
        val.extend_from_slice(&self.0);
        val.into()
    }

    pub fn add(&mut self, d: &[u8], buf: &mut [u8; 6]) {
        let (i1, v1, i2, v2, i3, v3) = Self::bloom_values(d, buf);
        self.0[i1] |= v1;
        self.0[i2] |= v2;
        self.0[i3] |= v3;
    }

    fn bloom_values(data: &[u8], hashbuf: &mut [u8; 6]) -> (usize, u8, usize, u8, usize, u8) {
        use tiny_keccak::{Hasher, Keccak};
        const BLOOM_BYTE_LENGTH: usize = 256;
        let mut keccak = Keccak::v256();
        keccak.update(data);
        keccak.finalize(&mut hashbuf[..]);

        // The actual bits to flip
        let v1 = 1 << (hashbuf[1] & 0x7);
        let v2 = 1 << (hashbuf[3] & 0x7);
        let v3 = 1 << (hashbuf[5] & 0x7);
        // The indices for the bytes to OR in
        let mut u16_buf = [0_u8; 2];
        u16_buf.copy_from_slice(&hashbuf[..2]);
        let i1 = BLOOM_BYTE_LENGTH - ((u16::from_be_bytes(u16_buf) & 0x7ff) >> 3) as usize - 1;
        u16_buf.copy_from_slice(&hashbuf[2..4]);
        let i2 = BLOOM_BYTE_LENGTH - ((u16::from_be_bytes(u16_buf) & 0x7ff) >> 3) as usize - 1;
        u16_buf.copy_from_slice(&hashbuf[4..6]);
        let i3 = BLOOM_BYTE_LENGTH - ((u16::from_be_bytes(u16_buf) & 0x7ff) >> 3) as usize - 1;
        return (i1, v1, i2, v2, i3, v3);
    }
}

pub struct KeccakHasher {}
impl hash_db::Hasher for KeccakHasher {
    type Out = [u8; 32];
    type StdHasher = Hash256StdHasher;
    const LENGTH: usize = 32;
    fn hash(x: &[u8]) -> Self::Out {
        keccak_hash(x)
    }
}

#[derive(Clone, Copy, Debug)]
pub enum BlockSelector {
    Number(SU64),
    Hash(SH256),
    Latest,
}

impl From<u64> for BlockSelector {
    fn from(val: u64) -> Self {
        Self::Number(val.into())
    }
}

impl From<SU64> for BlockSelector {
    fn from(val: SU64) -> Self {
        Self::Number(val)
    }
}

impl From<SH256> for BlockSelector {
    fn from(val: SH256) -> Self {
        Self::Hash(val)
    }
}

impl From<String> for BlockSelector {
    fn from(val: String) -> Self {
        if val.as_str() == "latest" {
            return Self::Latest;
        }
        if !val.starts_with("0x") {
            return parse_string_u64(&val).unwrap().as_u64().into();
        }
        let data = HexBytes::from_hex(val.as_bytes()).unwrap();
        if data.len() == 32 {
            let hash = parse_string_h256(&val).unwrap().into();
            return Self::Hash(hash);
        }
        return Self::Number(parse_string_u64(&val).unwrap().as_u64().into());
    }
}

impl Serialize for BlockSelector {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let val = match self {
            Self::Hash(n) => format!("0x{}", hex::encode(&n.0)),
            Self::Number(n) => format!("0x{:x}", n.raw()),
            Self::Latest => "latest".into(),
        };
        serializer.serialize_str(&val)
    }
}

impl Receipt {
    pub fn compare(want: &Receipt, got: &Receipt) -> Result<(), String> {
        let mut reason = <Vec<String>>::new();
        let mut tag = String::from("");
        macro_rules! cmp {
            ($want:tt, $got:tt, $field:tt) => {
                if $want.$field != $got.$field {
                    reason.push(format!(
                        "[{}.{}] not match, want: {:?}, got: {:?}",
                        tag,
                        stringify!($field),
                        $want.$field,
                        $got.$field
                    ));
                }
            };
        }
        cmp!(want, got, r#type);
        cmp!(want, got, root);
        cmp!(want, got, status);
        // cmp!(want, got, cumulative_gas_used);
        cmp!(want, got, logs_bloom);
        // cmp!(want, got, logs.len());
        cmp!(want, got, transaction_hash);
        cmp!(want, got, contract_address);
        cmp!(want, got, gas_used);
        // cmp!(want, got, block_hash);
        // cmp!(want, got, block_number);
        // cmp!(want, got, transaction_index);
        for i in 0..want.logs.len() {
            tag = format!("logs.{}", i);
            let want = &want.logs[i];
            let got = &got.logs[i];
            cmp!(want, got, address);
            cmp!(want, got, topics);
            cmp!(want, got, data);
            // cmp!(want, got, block_number);
            cmp!(want, got, transaction_hash);
            // cmp!(want, got, transaction_index);
            // cmp!(want, got, block_hash);
            // cmp!(want, got, log_index);
            // cmp!(want, got, removed);
        }
        if reason.len() > 0 {
            return Err(reason.join("\n"));
        }
        Ok(())
    }
}

impl Block {
    pub fn compare(want: &Block, got: &Block) -> Result<(), String> {
        let mut reason = <Vec<String>>::new();
        let mut tag: String;
        macro_rules! cmp {
            ($want:tt, $got:tt, $field:tt) => {
                if $want.$field != $got.$field {
                    reason.push(format!(
                        "[{}.{}] not match, want: {:?}, got: {:?}",
                        tag,
                        stringify!($field),
                        $want.$field,
                        $got.$field
                    ));
                }
            };
        }
        {
            tag = "header".into();
            let want = &want.header;
            let got = &got.header;
            cmp!(want, got, parent_hash);
            cmp!(want, got, sha3_uncles);
            cmp!(want, got, miner);
            cmp!(want, got, state_root);
            cmp!(want, got, withdrawals_root);
            cmp!(want, got, transactions_root);
            cmp!(want, got, receipts_root);
            cmp!(want, got, logs_bloom);
            cmp!(want, got, difficulty);
            cmp!(want, got, number);
            cmp!(want, got, gas_limit);
            cmp!(want, got, gas_used);
            cmp!(want, got, timestamp);
            // cmp!(want, got, extra_data);
            cmp!(want, got, mix_hash);
            cmp!(want, got, nonce);
            cmp!(want, got, base_fee_per_gas);
        }

        cmp!(want, got, withdrawals);

        if want.transactions.len() != got.transactions.len() {
            reason.push(format!(
                "[txs.len] not match, want: {:?}, got: {:?}",
                want.transactions.len(),
                got.transactions.len()
            ));
            if want.transactions.len() < got.transactions.len() {
                for (idx, tx) in got.transactions.iter().enumerate() {
                    let search = tx.hash;
                    if !want.transactions.iter().any(|tx| tx.hash == search) {
                        reason.push(format!("[txs.{}] unexpected {:?}", idx, search));
                    }
                }
            }
        } else {
            for i in 0..want.transactions.len() {
                tag = format!("tx.{}", i);
                let want = &want.transactions[i];
                let got = &got.transactions[i];
                // cmp!(want, got, block_hash);
                cmp!(want, got, block_number);
                // cmp!(want, got, from);
                cmp!(want, got, gas);
                cmp!(want, got, gas_price);
                cmp!(want, got, max_fee_per_gas);
                cmp!(want, got, max_priority_fee_per_gas);
                cmp!(want, got, hash);
                cmp!(want, got, input);
                cmp!(want, got, nonce);
                cmp!(want, got, to);
                // cmp!(want, got, transaction_index);
                cmp!(want, got, value);
                cmp!(want, got, r#type);
                cmp!(want, got, access_list);
                // cmp!(want, got, chain_id);
                cmp!(want, got, v);
                cmp!(want, got, r);
                cmp!(want, got, s);
            }
        }

        if reason.len() > 0 {
            return Err(reason.join("\n"));
        }
        Ok(())
    }
}
