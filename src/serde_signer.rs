use std::prelude::v1::*;

use crate::SH256;
use crypto::{keccak_hash, Secp256k1PrivateKey};
use hex::HexBytes;
use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct TimeBasedSigner {
    prvkey: Secp256k1PrivateKey,
}

impl TimeBasedSigner {
    pub fn new(prvkey: Secp256k1PrivateKey) -> Self {
        Self { prvkey }
    }
    pub fn sign<T: Serialize>(&self, ts: u64, t: T) -> TimeBasedSignature<T> {
        let msg = serde_json::to_vec(&(&t, ts)).unwrap();
        let sig = self.prvkey.sign(&msg).to_array();
        TimeBasedSignature(t, ts, HexBytes::from(&sig[..]))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeBasedSignature<T>(T, u64, HexBytes);

impl<T> TimeBasedSignature<T> {
    pub fn time(&self) -> u64 {
        self.1
    }

    pub fn data(&self) -> &T {
        &self.0
    }
}

impl<T: Serialize> TimeBasedSignature<T> {
    pub fn recover(&self) -> Result<SH256, String> {
        if self.2.len() != 65 {
            return Err(format!("invalid signature"));
        }
        let mut sig = [0_u8; 65];
        sig.copy_from_slice(&self.2);

        let msg = keccak_hash(&serde_json::to_vec(&(&self.0, self.1)).unwrap());

        let pubkey = match crypto::secp256k1_ecdsa_recover(&sig, &msg) {
            Some(signer) => signer,
            None => return Err(format!("invalid signature")),
        };
        Ok(keccak_hash(&pubkey).into())
    }
}
