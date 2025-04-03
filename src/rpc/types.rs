use reqwest::{Client};
use serde::{Deserialize, Serialize};
use url::Url;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use crate::transaction::types::VMDataTransaction;

pub struct RPC {
    pub http_client: Client,
    pub node_url: Url,
    pub chain_id: u8,
}

#[derive(Debug)]
pub enum RpcError {
    FailedToBroadcastTransaction(String),
    InvalidRpcUrl,
    Network(reqwest::Error),
    Deserialization(reqwest::Error),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BroadcastResponse {
    pub success: bool,
    pub data: Option<String>,
    pub error: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ResponseData {
    pub message: String,
}

#[derive(Serialize)]
pub struct BroadcastRequest {
    pub txn: String,
}

pub struct VidaTransactionSubscription {
    pub pwrrs: Arc<RPC>,
    pub vida_id: u64,
    pub starting_block: u64,
    pub latest_checked_block: Arc<std::sync::atomic::AtomicU64>,
    pub handler: ProcessVidaTransactions,
    pub pause: Arc<AtomicBool>,
    pub stop: Arc<AtomicBool>,
    pub running: Arc<AtomicBool>,
}

pub type ProcessVidaTransactions = fn(transaction: VMDataTransaction);
