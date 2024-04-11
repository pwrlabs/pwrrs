use reqwest::{Client, StatusCode};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use sha3::{Digest, Keccak256};
use url::Url;

use crate::{
    block::{Block, Delegator, NewTransactionData, Validator},
    wallet::Wallet,
};

const DEFAULT_FEE_PER_BYTE: u64 = 100;
const DEFAULT_CHAIN_ID: u8 = 0;

pub struct RPC {
    http_client: Client,
    node_url: Url,
    chain_id: u8,

    fee_per_byte: u64,
}

impl RPC {
    /// Creates a new RPC.
    pub async fn new<S>(node_url: S) -> Result<Self, RpcError>
    where
        S: AsRef<str>,
    {
        let node_url = Url::parse(node_url.as_ref()).map_err(|_| RpcError::InvalidRpcUrl)?;

        let mut s = Self {
            node_url,
            fee_per_byte: DEFAULT_FEE_PER_BYTE,
            http_client: Client::new(),
            chain_id: DEFAULT_CHAIN_ID,
        };

        let chain_id = {
            #[derive(Deserialize)]
            struct Resp {
                #[serde(rename = "chainId")]
                chain_id: u8,
            }

            let response = s
                .http_client
                .get(s.node_url.join("/chainId/").unwrap())
                .send()
                .await
                .map_err(RpcError::Network)?;
            response
                .json::<Resp>()
                .await
                .map_err(RpcError::Deserialization)?
                .chain_id
        };

        s.chain_id = chain_id;

        Ok(s)
    }

    /// Retrieves the current RPC node URL being used.
    pub fn node_url(&self) -> &Url {
        &self.node_url
    }

    /// Fetches the current fee-per-byte rate that's been set locally.
    pub fn fee_per_byte(&self) -> u64 {
        self.fee_per_byte
    }

    /// Queries the RPC node to get the nonce of a specific address.
    ///
    /// The nonce is a count of the number of transactions sent from the sender's address.
    pub async fn nonce_of_address(&self, address: &str) -> Result<u32, RpcError> {
        #[derive(Deserialize)]
        struct Response {
            nonce: u32,
        }

        self.call_rpc_get(&format!("/nonceOfUser/?userAddress={}", address))
            .await
            .map(|r: Response| r.nonce)
    }

    /// Queries the RPC node to obtain the balance of a specific address.
    pub async fn balance_of_address(&self, address: &str) -> Result<u64, RpcError> {
        #[derive(Deserialize)]
        struct Response {
            balance: u64,
        }

        self.call_rpc_get(&format!("/balanceOf/?userAddress={}", address))
            .await
            .map(|r: Response| r.balance)
    }

    /// Retrieves the total count of blocks from the RPC node.
    pub async fn block_count(&self) -> Result<u64, RpcError> {
        #[derive(Deserialize)]
        struct Response {
            #[serde(rename = "blocksCount")]
            blocks_count: u64,
        }

        self.call_rpc_get("/blocksCount/")
            .await
            .map(|r: Response| r.blocks_count)
    }

    /// Retrieves the number of the latest block from the RPC node.
    ///
    /// This method utilizes the `block_count` method to get the total count of blocks
    /// and then subtracts one to get the latest block number
    pub async fn latest_block_count(&self) -> Result<u64, RpcError> {
        self.block_count().await.map(|v| v - 1)
    }

    /// Queries the RPC node to retrieve block details for a specific block number.
    pub async fn block_by_number(&self, block_number: u64) -> Result<Block, RpcError> {
        #[derive(Deserialize)]
        struct Response {
            block: Block,
        }
        self.call_rpc_get(&format!("/block/?blockNumber={}", block_number))
            .await
            .map(|r: Response| r.block)
    }

    pub async fn active_voting_power(&self) -> Result<u64, RpcError> {
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct Response {
            active_voting_power: u64,
        }
        self.call_rpc_get("/activeVotingPower/")
            .await
            .map(|r: Response| r.active_voting_power)
    }

    /// Queries the RPC node to get the total number of validators (standby & active).
    pub async fn total_validator_count(&self) -> Result<u64, RpcError> {
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct Response {
            validators_count: u64,
        }
        self.call_rpc_get("/totalValidatorsCount/")
            .await
            .map(|r: Response| r.validators_count)
    }

    /// Queries the RPC node to get the total number of standby validators.
    pub async fn standby_validator_count(&self) -> Result<u64, RpcError> {
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct Response {
            validators_count: u64,
        }
        self.call_rpc_get("/standbyValidatorsCount/")
            .await
            .map(|r: Response| r.validators_count)
    }

    /// Queries the RPC node to get the total number of active validators.
    pub async fn active_validator_count(&self) -> Result<u64, RpcError> {
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct Response {
            validators_count: u64,
        }
        self.call_rpc_get("/activeValidatorsCount/")
            .await
            .map(|r: Response| r.validators_count)
    }

    /// Queries the RPC node to get the list of all validators (standby & active).
    pub async fn all_validators(&self) -> Result<Vec<Validator>, RpcError> {
        #[derive(Deserialize)]
        struct Response {
            validators: Vec<Validator>,
        }
        self.call_rpc_get("/allValidators/")
            .await
            .map(|r: Response| r.validators)
    }

    /// Queries the RPC node to get the list of all standby validators.
    pub async fn standby_validators(&self) -> Result<Vec<Validator>, RpcError> {
        #[derive(Deserialize)]
        struct Response {
            validators: Vec<Validator>,
        }
        self.call_rpc_get("/standbyValidators/")
            .await
            .map(|r: Response| r.validators)
    }

    /// Queries the RPC node to get the list of all active validators.
    pub async fn active_validators(&self) -> Result<Vec<Validator>, RpcError> {
        #[derive(Deserialize)]
        struct Response {
            validators: Vec<Validator>,
        }
        self.call_rpc_get("/activeValidators/")
            .await
            .map(|r: Response| r.validators)
    }

    /// Queries the RPC node to get the list of delegators of a validator.
    pub async fn delegators_of_validator(
        &self,
        validator_address: &str,
    ) -> Result<Vec<Delegator>, RpcError> {
        #[derive(Deserialize)]
        struct Response {
            delegators: Vec<Delegator>,
        }
        self.call_rpc_get(&format!(
            "/validator/delegatorsOfValidator/?validatorAddress={}",
            validator_address
        ))
        .await
        .map(|r: Response| r.delegators)
    }

    /// Fetches and updates the current fee per byte from the RPC node.
    pub async fn update_fee_per_byte(&mut self) -> Result<(), RpcError> {
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct Response {
            fee_per_byte: u64,
        }
        let resp = self
            .call_rpc_get("/feePerByte/")
            .await
            .map(|r: Response| r.fee_per_byte)?;

        self.fee_per_byte = resp;

        Ok(())
    }

    /// Broadcasts a transaction to the network via a specified RPC node.
    pub async fn broadcast_transaction(
        &self,
        transaction: &NewTransactionData,
        wallet: &Wallet,
    ) -> Result<String, RpcError> {
        #[derive(Deserialize)]
        struct Responce {
            message: String,
        }

        #[derive(Serialize)]
        struct Request {
            txn: String,
        }

        let nonce = self.nonce_of_address(&wallet.address()).await?;

        let mut hasher = Keccak256::new();
        let txn_bytes = transaction
            .serialize_for_broadcast(nonce, self.chain_id, wallet)
            .map_err(|e| RpcError::FailedToBroadcastTransaction(e.to_string()))?;
        hasher.update(&txn_bytes);
        let txn_hash = hasher.finalize();

        let request = Request {
            txn: hex::encode(txn_bytes),
        };

        let (status, resp) = self
            .call_rpc_post::<Responce, _>("/broadcast/", request)
            .await?;

        if status != 200 {
            Err(RpcError::FailedToBroadcastTransaction(format!(
                "RpcError: {}",
                resp.message
            )))
        } else {
            Ok(format!("0x{}", hex::encode_upper(txn_hash)))
        }
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

    async fn call_rpc_post<Resp, Req>(
        &self,
        path: &str,
        request: Req,
    ) -> Result<(StatusCode, Resp), RpcError>
    where
        Resp: DeserializeOwned,
        Req: Serialize,
    {
        let response = self
            .http_client
            .post(self.node_url.join(path).unwrap())
            .json(&request)
            .send()
            .await
            .map_err(RpcError::Network)?;

        Ok((
            response.status(),
            response.json().await.map_err(RpcError::Deserialization)?,
        ))
    }
}

#[derive(Debug)]
pub enum RpcError {
    FailedToBroadcastTransaction(String),
    InvalidRpcUrl,
    Network(reqwest::Error),
    Deserialization(reqwest::Error),
}
