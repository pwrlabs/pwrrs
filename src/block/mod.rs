use serde::{Deserialize, Serialize};
use crate::transaction::types::Transaction;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Block {
    pub transaction_count: u32,
    pub size: u32,
    pub block_number: u32,
    pub block_reward: u64,
    pub timestamp: u64,
    pub block_hash: String,
    pub block_submitter: String,
    pub transactions: Vec<Transaction>,
}
