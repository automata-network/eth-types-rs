use std::prelude::v1::*;

use super::{TransactionInner, SH160, SU256};
use crypto::{secp256k1_recover_pubkey, Secp256k1PrivateKey};

#[derive(Clone, Copy)]
pub struct Signer {
    pub chain_id: SU256,
}

impl Signer {
    pub fn new(chain_id: SU256) -> Self {
        Self { chain_id }
    }

    pub fn sender(&self, inner: &TransactionInner) -> SH160 {
        let sig = inner.signature(self.chain_id.as_u64());
        match inner {
            TransactionInner::DynamicFee(tx) => {
                if tx.chain_id != self.chain_id {
                    panic!(
                        "chain id not match, expect: {}, got: {}",
                        self.chain_id, tx.chain_id
                    );
                }
            }
            TransactionInner::AccessList(tx) => {
                if tx.chain_id != self.chain_id {
                    panic!("chain id not match");
                }
            }
            TransactionInner::Legacy(_) => {}
        }

        let msg = self.msg(inner);
        let pubkey = secp256k1_recover_pubkey(&sig, &msg[..]);
        pubkey.eth_accountid().into()
    }

    pub fn sign(&self, tx: &mut TransactionInner, key: &Secp256k1PrivateKey) {
        tx.sign(key, self.chain_id.as_u64())
    }

    pub fn msg(&self, tx: &TransactionInner) -> Vec<u8> {
        tx.sign_msg(&self.chain_id)
    }
}
