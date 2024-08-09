use reqwest::Client;
use url::Url;

pub struct RPC {
    pub http_client: Client,
    pub node_url: Url,
    pub fee_per_byte: u64,
}

#[derive(Debug)]
pub enum RpcError {
    FailedToBroadcastTransaction(String),
    Network(reqwest::Error),
    Deserialization(reqwest::Error),
}
