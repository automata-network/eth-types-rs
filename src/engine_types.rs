use std::prelude::v1::*;

use serde::de::DeserializeOwned;

use crate::{
    Block, BlockHeader, BlockHeaderTrait, BlockTrait, Receipt, Transaction, TransactionInner,
    TxTrait, Withdrawal,
};

pub trait EngineTypes {
    type Transaction: TxTrait;
    type RpcTransaction: DeserializeOwned;
    type BlockHeader: BlockHeaderTrait;
    type Receipt: DeserializeOwned;
    type Withdrawal: DeserializeOwned;
    type Block: BlockTrait;
}

#[derive(Debug, Clone)]
pub struct EthereumEngineTypes;

impl EngineTypes for EthereumEngineTypes {
    type Block = Block;
    type BlockHeader = BlockHeader;
    type Receipt = Receipt;
    type Transaction = TransactionInner;
    type Withdrawal = Withdrawal;
    type RpcTransaction = Transaction;
}
