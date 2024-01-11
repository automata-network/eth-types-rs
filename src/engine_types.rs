use std::prelude::v1::*;

use serde::de::DeserializeOwned;

use crate::{
    Block, BlockHeader, BlockHeaderTrait, BlockTrait, Receipt, ReceiptTrait, Transaction,
    TransactionInner, TxTrait, Withdrawal,
};

pub trait EngineTypes: std::fmt::Debug + Clone + Send + 'static {
    type Transaction: TxTrait + std::fmt::Debug + Clone + Send + 'static;
    type RpcTransaction: DeserializeOwned + std::fmt::Debug + Clone + Send + 'static;
    type BlockHeader: BlockHeaderTrait + std::fmt::Debug + Send + 'static;
    type Receipt: ReceiptTrait + std::fmt::Debug + Send + 'static;
    type Withdrawal: DeserializeOwned + std::fmt::Debug + Send + 'static;
    type Block: BlockTrait + std::fmt::Debug + Send + 'static;
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
