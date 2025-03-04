use reqwest::{Client};
use serde::{Deserialize, Serialize};
use url::Url;

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
