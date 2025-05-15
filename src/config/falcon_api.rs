use reqwest::Client;
use crate::rpc::types::RpcError;
use serde::{Deserialize, de::DeserializeOwned};
use url::Url;

pub struct FalconAPI {
    pub http_client: Client,
    pub node_url: Url,
}

impl FalconAPI {
    pub fn new(node_url: &str) -> Self {
        let http_client = Client::new();
        let node_url = Url::parse(node_url).unwrap();
        Self { http_client, node_url }
    }

    pub async fn generate_keypair(
        &self,
        seed: &str,
    ) -> Result<(String, String), RpcError> {
        #[derive(Deserialize)]
        struct Response {
            data: KeyPairData,
        }

        #[derive(Deserialize)]
        struct KeyPairData {
            #[serde(rename = "publicKey")]
            public_key: String,
            #[serde(rename = "secretKey")]
            secret_key: String,
        }

        self.call_rpc_get(&format!(
            "/generateKeypair?seed={}",
            hex::encode(seed)
        ))
        .await
        .map(|r: Response| (r.data.public_key, r.data.secret_key))
    }

    pub async fn generate_random_keypair(
        &self,
        word_count: u8,
    ) -> Result<(String, String, String), RpcError> {
        #[derive(Deserialize)]
        struct Response {
            data: KeyPairData,
        }

        #[derive(Deserialize)]
        struct KeyPairData {
            #[serde(rename = "publicKey")]
            public_key: String,
            #[serde(rename = "secretKey")]
            secret_key: String,
            #[serde(rename = "seedPhrase")]
            seed_phrase: String,
        }

        self.call_rpc_get(&format!(
            "/generateRandomKeypair?wordCount={}",
            word_count
        ))
        .await
        .map(|r: Response| (r.data.public_key, r.data.secret_key, r.data.seed_phrase))
    }

    async fn call_rpc_get<Resp>(&self, path: &str) -> Result<Resp, RpcError>
    where
        Resp: DeserializeOwned,
    {
        let response = self
            .http_client
            .get(self.node_url.join(path).unwrap())
            .send()
            .await
            .map_err(RpcError::Network)?;
        response.json().await.map_err(RpcError::Deserialization)
    }
}
