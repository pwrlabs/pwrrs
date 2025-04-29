
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BlockTransaction {
    pub identifier: u32,
    pub transaction_hash: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Block {
    #[serde(default)]
    pub processed_without_critical_errors: bool,
    pub block_hash: String,
    pub previous_block_hash: String,
    pub proposer: String,
    #[serde(default)]
    pub blockchain_version: u64,
    #[serde(default)]
    pub burned_fees: u64,
    pub block_reward: u64,
    pub transactions: Vec<BlockTransaction>,
    #[serde(default)]
    pub timestamp: u64,
    pub size: u32,
    pub block_number: u32,
    pub root_hash: String,
    #[serde(default)]
    pub new_shares_per_spark: u64,
}
