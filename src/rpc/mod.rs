pub mod types;
pub mod vida;

use reqwest::{Client, StatusCode};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use sha3::{Digest, Keccak256};
use std::collections::HashMap;
use url::Url;

pub use self::types::{RPC, BroadcastResponse, ResponseData, BroadcastRequest};
use self::types::RpcError;
use types::{VidaTransactionSubscription, ProcessVidaTransactions};
use std::sync::Arc;

use crate::{
    block::Block,
    transaction::types::{VidaDataTransaction, Penalty},
    validator::Validator
};

const DEFAULT_CHAIN_ID: u8 = 0;

impl RPC {
    /// Creates a new RPC.
    pub async fn new<S>(node_url: S) -> Result<Self, RpcError>
    where
        S: AsRef<str>,
    {
        let node_url = Url::parse(node_url.as_ref()).map_err(|_| RpcError::InvalidRpcUrl)?;

        let mut s = Self {
            node_url,
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
                .get(s.node_url.join("/chainId").unwrap())
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
    pub fn get_node_url(&self) -> &Url {
        &self.node_url
    }

    /// Fetches the current fee-per-byte rate that's been set locally.
    pub async fn get_fee_per_byte(&self) -> Result<u64, RpcError> {
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct Response {
            fee_per_byte: u64,
        }
        let resp = self
            .call_rpc_get("/feePerByte")
            .await
            .map(|r: Response| r.fee_per_byte)?;

        Ok(resp)
    }

    /// Queries the RPC node to get the nonce of a specific address.
    ///
    /// The nonce is a count of the number of transactions sent from the sender's address.
    pub async fn get_nonce_of_address(&self, address: &str) -> Result<u32, RpcError> {
        #[derive(Deserialize)]
        struct Response {
            nonce: u32,
        }

        self.call_rpc_get(&format!("/nonceOfUser?userAddress={}", address))
            .await
            .map(|r: Response| r.nonce)
    }

    /// Queries the RPC node to obtain the balance of a specific address.
    pub async fn get_balance_of_address(&self, address: &str) -> Result<u64, RpcError> {
        #[derive(Deserialize)]
        struct Response {
            balance: u64,
        }

        self.call_rpc_get(&format!("/balanceOf?userAddress={}", address))
            .await
            .map(|r: Response| r.balance)
    }

    /// Retrieves the total count of blocks from the RPC node.
    pub async fn get_block_count(&self) -> Result<u64, RpcError> {
        #[derive(Deserialize)]
        struct Response {
            #[serde(rename = "blocksCount")]
            blocks_count: u64,
        }

        self.call_rpc_get("/blocksCount")
            .await
            .map(|r: Response| r.blocks_count)
    }

    pub async fn get_blockchain_version(&self) -> Result<u64, RpcError> {
        #[derive(Deserialize)]
        struct Response {
            #[serde(rename = "blockchainVersion")]
            blockchain_version: u64,
        }

        self.call_rpc_get("/blockchainVersion")
            .await
            .map(|r: Response| r.blockchain_version)
    }

    // pub async fn get_fee(&self) -> u64 {
        
    // }

    pub async fn get_ecdsa_verification_fee(&self) -> Result<u64, RpcError> {
        #[derive(Deserialize)]
        struct Response {
            #[serde(rename = "ecdsaVerificationFee")]
            ecdsa_verification_fee: u64,
        }

        self.call_rpc_get("/ecdsaVerificationFee")
            .await
            .map(|r: Response| r.ecdsa_verification_fee)
    }

    pub async fn get_burn_percentage(&self) -> Result<u64, RpcError> {
        #[derive(Deserialize)]
        struct Response {
            #[serde(rename = "burnPercentage")]
            burn_percentage: u64,
        }

        self.call_rpc_get("/burnPercentage")
            .await
            .map(|r: Response| r.burn_percentage)
    }

    pub async fn get_total_voting_power_res(&self) -> Result<u64, RpcError> {
        #[derive(Deserialize)]
        struct Response {
            #[serde(rename = "totalVotingPower")]
            total_voting_power: u64,
        }

        self.call_rpc_get("/totalVotingPower")
            .await
            .map(|r: Response| r.total_voting_power)
    }

    pub async fn get_pwr_rewards_per_year(&self) -> Result<u64, RpcError> {
        #[derive(Deserialize)]
        struct Response {
            #[serde(rename = "pwrRewardsPerYear")]
            pwr_rewards_per_year: u64,
        }

        self.call_rpc_get("/pwrRewardsPerYear")
            .await
            .map(|r: Response| r.pwr_rewards_per_year)
    }

    pub async fn get_withdrawal_lock_time(&self) -> Result<u64, RpcError> {
        #[derive(Deserialize)]
        struct Response {
            #[serde(rename = "withdrawalLockTime")]
            withdrawal_lock_time: u64,
        }

        self.call_rpc_get("/withdrawalLockTime")
            .await
            .map(|r: Response| r.withdrawal_lock_time)
    }

    pub async fn get_all_early_withdraw_penalties(&self) -> Result<HashMap<String, Penalty>, RpcError> {
        #[derive(Deserialize)]
        struct Response {
            #[serde(rename = "earlyWithdrawPenalties")]
            all_early_withdraw_penalties: HashMap<String, Penalty>,
        }

        self.call_rpc_get("/allEarlyWithdrawPenalties")
            .await
            .map(|r: Response| r.all_early_withdraw_penalties)
    }

    pub async fn get_max_block_size(&self) -> Result<u64, RpcError> {
        #[derive(Deserialize)]
        struct Response {
            #[serde(rename = "maxBlockSize")]
            max_block_size: u64,
        }

        self.call_rpc_get("/maxBlockSize")
            .await
            .map(|r: Response| r.max_block_size)
    }

    pub async fn get_max_transaction_size(&self) -> Result<u64, RpcError> {
        #[derive(Deserialize)]
        struct Response {
            #[serde(rename = "maxTransactionSize")]
            max_transaction_size: u64,
        }

        self.call_rpc_get("/maxTransactionSize")
            .await
            .map(|r: Response| r.max_transaction_size)
    }

    pub async fn get_block_number(&self) -> Result<u64, RpcError> {
        #[derive(Deserialize)]
        struct Response {
            #[serde(rename = "blockNumber")]
            block_number: u64,
        }

        self.call_rpc_get("/blockNumber")
            .await
            .map(|r: Response| r.block_number)
    }

    pub async fn get_block_timestamp(&self) -> Result<u64, RpcError> {
        #[derive(Deserialize)]
        struct Response {
            #[serde(rename = "blockTimestamp")]
            block_timestamp: u64,
        }

        self.call_rpc_get("/blockTimestamp")
            .await
            .map(|r: Response| r.block_timestamp)
    }

    pub async fn get_latest_block_number(&self) -> Result<u64, RpcError> {
        self.get_block_number().await
    }

    pub async fn get_proposal_fee(&self) -> Result<u64, RpcError> {
        #[derive(Deserialize)]
        struct Response {
            #[serde(rename = "proposalFee")]
            proposal_fee: u64,
        }

        self.call_rpc_get("/proposalFee")
            .await
            .map(|r: Response| r.proposal_fee)
    }

    pub async fn get_proposal_validity_time(&self) -> Result<u64, RpcError> {
        #[derive(Deserialize)]
        struct Response {
            #[serde(rename = "proposalValidityTime")]
            proposal_validity_time: u64,
        }

        self.call_rpc_get("/proposalValidityTime")
            .await
            .map(|r: Response| r.proposal_validity_time)
    }

    pub async fn get_validator_count_limit(&self) -> Result<u64, RpcError> {
        #[derive(Deserialize)]
        struct Response {
            #[serde(rename = "validatorCountLimit")]
            validator_count_limit: u64,
        }

        self.call_rpc_get("/validatorCountLimit")
            .await
            .map(|r: Response| r.validator_count_limit)
    }

    pub async fn get_validator_slashing_fee(&self) -> Result<u64, RpcError> {
        #[derive(Deserialize)]
        struct Response {
            #[serde(rename = "validatorSlashingFee")]
            validator_slashing_fee: u64,
        }

        self.call_rpc_get("/validatorSlashingFee")
            .await
            .map(|r: Response| r.validator_slashing_fee)
    }

    pub async fn get_validator_operational_fee(&self) -> Result<u64, RpcError> {
        #[derive(Deserialize)]
        struct Response {
            #[serde(rename = "validatorOperationalFee")]
            validator_operational_fee: u64,
        }

        self.call_rpc_get("/validatorOperationalFee")
            .await
            .map(|r: Response| r.validator_operational_fee)
    }

    pub async fn get_validator_joining_fee(&self) -> Result<u64, RpcError> {
        #[derive(Deserialize)]
        struct Response {
            #[serde(rename = "validatorJoiningFee")]
            validator_joining_fee: u64,
        }

        self.call_rpc_get("/validatorJoiningFee")
            .await
            .map(|r: Response| r.validator_joining_fee)
    }

    pub async fn get_minimum_delegating_amount(&self) -> Result<u64, RpcError> {
        #[derive(Deserialize)]
        struct Response {
            #[serde(rename = "minimumDelegatingAmount")]
            minimum_delegating_amount: u64,
        }

        self.call_rpc_get("/minimumDelegatingAmount")
            .await
            .map(|r: Response| r.minimum_delegating_amount)
    }

    pub async fn get_total_delegators_count(&self) -> Result<u64, RpcError> {
        #[derive(Deserialize)]
        struct Response {
            #[serde(rename = "delegatorsCount")]
            total_delegators_count: u64,
        }

        self.call_rpc_get("/totalDelegatorsCount")
            .await
            .map(|r: Response| r.total_delegators_count)
    }

    pub async fn get_validator(
        &self,
        validator_address: &str,
    ) -> Result<Validator, RpcError> {
        #[derive(Deserialize)]
        struct Response {
            validator: Validator,
        }
        self.call_rpc_get(&format!(
            "/validator?validatorAddress={}",
            validator_address
        ))
        .await
        .map(|r: Response| r.validator)
    }

    pub async fn get_delegators_of_pwr(
        &self,
        delegator_address: &str,
        validator_address: &str,
    ) -> Result<u64, RpcError> {
        #[derive(Deserialize)]
        struct Response {
            #[serde(rename = "delegatedPWR")]
            delegators: u64,
        }
        self.call_rpc_get(&format!(
            "/validator/delegator/delegatedPWROfAddress?userAddress={}&validatorAddress={}",
            delegator_address, validator_address
        ))
        .await
        .map(|r: Response| r.delegators)
    }

    pub async fn get_shares_of_delegator(
        &self,
        delegator_address: &str,
        validator_address: &str,
    ) -> Result<u64, RpcError> {
        #[derive(Deserialize)]
        struct Response {
            #[serde(rename = "shares")]
            shares_of_delegator: u64,
        }
        self.call_rpc_get(&format!(
            "/validator/delegator/sharesOfAddress?userAddress={}&validatorAddress={}",
            delegator_address, validator_address
        ))
        .await
        .map(|r: Response| r.shares_of_delegator)
    }

    pub async fn get_share_value(
        &self,
        validator_address: &str,
    ) -> Result<f64, RpcError> {
        #[derive(Deserialize)]
        struct Response {
            #[serde(rename = "shareValue")]
            share_value: f64,
        }
        self.call_rpc_get(&format!(
            "/validator/shareValue?validatorAddress={}",
            validator_address
        ))
        .await
        .map(|r: Response| r.share_value)
    }

    pub async fn get_vida_owner_transaction_fee_share(&self) -> Result<u64, RpcError> {
        #[derive(Deserialize)]
        struct Response {
            #[serde(rename = "vidaOwnerTransactionFeeShare")]
            vida_owner_transaction_fee_share: u64,
        }

        self.call_rpc_get("/vidaOwnerTransactionFeeShare")
            .await
            .map(|r: Response| r.vida_owner_transaction_fee_share)
    }

    pub async fn get_vida_id_claiming_fee(&self) -> Result<u64, RpcError> {
        #[derive(Deserialize)]
        struct Response {
            #[serde(rename = "vidaIdClaimingFee")]
            vida_id_claiming_fee: u64,
        }

        self.call_rpc_get("/vidaIdClaimingFee")
            .await
            .map(|r: Response| r.vida_id_claiming_fee)
    }

    pub async fn get_vida_data_transactions(
        &self,
        starting_block: u64,
        ending_block: u64,
        vida_id: u64,
    ) -> Result<Vec<VidaDataTransaction>, RpcError> {
        let url = format!("/getVidaTransactions?startingBlock={}&endingBlock={}&vidaId={}", 
            starting_block, ending_block, vida_id);
        
        let response_text = self.http_client
            .get(self.node_url.join(&url).unwrap())
            .send()
            .await
            .map_err(RpcError::Network)?
            .text()
            .await
            .map_err(RpcError::Network)?;

        #[derive(Deserialize)]
        struct Response { transactions: Vec<String> }
        
        let response: Response = serde_json::from_str(&response_text)
            .map_err(|e| RpcError::JsonDeserialization(e.to_string()))?;

        response.transactions
            .into_iter()
            .map(|tx_str| serde_json::from_str(&tx_str)
                .map_err(|e| RpcError::JsonDeserialization(e.to_string())))
            .collect()
    }

    pub fn get_vida_id_address(&self, vida_id: i64) -> String {
        let mut hex_address = if vida_id >= 0 { String::from("1") } else { String::from("0") };
    
        let vida_id = if vida_id < 0 { -vida_id } else { vida_id };
        let vida_id_string = vida_id.to_string();
    
        for _ in 0..(39 - vida_id_string.len()) {
            hex_address.push('0');
        }
    
        hex_address.push_str(&vida_id_string);
    
        format!("0x{}", hex_address)
    }

    pub async fn get_owner_of_vida_id(
        &self,
        vida_id: i64,
    ) -> Result<String, RpcError> {
        #[derive(Deserialize)]
        struct Response {
            #[serde(rename = "owner")]
            owner_of_vida_id: String,
        }
        self.call_rpc_get(&format!(
            "/ownerOfVidaId?vidaId={}",
            vida_id
        ))
        .await
        .map(|r: Response| r.owner_of_vida_id)
    }

    pub async fn get_conduits_of_vida(
        &self,
        vida_id: i64,
    ) -> Result<Vec<Validator>, RpcError> {
        #[derive(Deserialize)]
        struct Response {
            conduits: Vec<Validator>,
        }
        self.call_rpc_get(&format!(
            "/conduitsOfVida?vidaId={}",
            vida_id
        ))
        .await
        .map(|r: Response| r.conduits)
    }

    pub async fn get_max_guardian_time(
        &self,
    ) -> Result<u64, RpcError> {
        #[derive(Deserialize)]
        struct Response {
            #[serde(rename = "maxGuardianTime")]
            max_guardian_time: u64,
        }
        self.call_rpc_get("/maxGuardianTime")
            .await
            .map(|r: Response| r.max_guardian_time)
    }
    
    pub async fn get_guardian_of_address(
        &self,
        address: &str,
    ) -> Result<String, RpcError> {
        #[derive(Deserialize)]
        struct Response {
            #[serde(rename = "isGuarded")]
            is_guarded: bool,
            guardian: Option<String>,
        }

        let res: Response = self.call_rpc_get(&format!(
                "/guardianOf?userAddress={}",
                address
            ))
            .await?;
            
        if res.is_guarded {
            if let Some(mut guardian) = res.guardian {
                if !guardian.starts_with("0x") {
                    guardian = format!("0x{}", guardian);
                }
                Ok(guardian)
            } else {
                Ok("None".to_string()) // or use some other default string
            }
        } else {
            Ok("None".to_string())
        }
    }

    /// Retrieves the number of the latest block from the RPC node.
    ///
    /// This method utilizes the `block_count` method to get the total count of blocks
    /// and then subtracts one to get the latest block number
    pub async fn get_latest_block(&self) -> Result<u64, RpcError> {
        self.get_block_count().await.map(|v| v - 1)
    }

    /// Queries the RPC node to retrieve block details for a specific block number.
    pub async fn get_block_by_number(&self, block_number: u64) -> Result<Block, RpcError> {
        #[derive(Deserialize)]
        struct Response {
            block: Block,
        }
        self.call_rpc_get(&format!("/block?blockNumber={}", block_number))
            .await
            .map(|r: Response| r.block)
    }

    pub async fn get_active_voting_power(&self) -> Result<u64, RpcError> {
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct Response {
            active_voting_power: u64,
        }
        self.call_rpc_get("/activeVotingPower")
            .await
            .map(|r: Response| r.active_voting_power)
    }

    /// Queries the RPC node to get the total number of validators (standby & active).
    pub async fn get_validators_count(&self) -> Result<u64, RpcError> {
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct Response {
            validators_count: u64,
        }
        self.call_rpc_get("/totalValidatorsCount")
            .await
            .map(|r: Response| r.validators_count)
    }

    /// Queries the RPC node to get the total number of standby validators.
    pub async fn get_standby_validator_count(&self) -> Result<u64, RpcError> {
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct Response {
            validators_count: u64,
        }
        self.call_rpc_get("/standbyValidatorsCount")
            .await
            .map(|r: Response| r.validators_count)
    }

    /// Queries the RPC node to get the total number of active validators.
    pub async fn get_active_validator_count(&self) -> Result<u64, RpcError> {
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct Response {
            validators_count: u64,
        }
        self.call_rpc_get("/activeValidatorsCount")
            .await
            .map(|r: Response| r.validators_count)
    }

    /// Queries the RPC node to get the list of all validators (standby & active).
    pub async fn get_all_validators(&self) -> Result<Vec<Validator>, RpcError> {
        #[derive(Deserialize)]
        struct Response {
            validators: Vec<Validator>,
        }
        self.call_rpc_get("/allValidators")
            .await
            .map(|r: Response| r.validators)
    }

    /// Queries the RPC node to get the list of all standby validators.
    pub async fn get_standby_validators(&self) -> Result<Vec<Validator>, RpcError> {
        #[derive(Deserialize)]
        struct Response {
            validators: Vec<Validator>,
        }
        self.call_rpc_get("/standbyValidators")
            .await
            .map(|r: Response| r.validators)
    }

    /// Queries the RPC node to get the list of all active validators.
    pub async fn get_active_validators(&self) -> Result<Vec<Validator>, RpcError> {
        #[derive(Deserialize)]
        struct Response {
            validators: Vec<Validator>,
        }
        self.call_rpc_get("/activeValidators")
            .await
            .map(|r: Response| r.validators)
    }

    /// Queries the RPC node to get the list of delegators of a validator.
    pub async fn get_delegators_of_validator(
        &self,
        validator_address: &str,
    ) -> Result<HashMap<String, u64>, RpcError> {
        #[derive(Deserialize)]
        struct Response {
            delegators: HashMap<String, u64>,
        }
        self.call_rpc_get(&format!(
            "/validator/delegatorsOfValidator?validatorAddress={}",
            validator_address
        ))
        .await
        .map(|r: Response| r.delegators)
    }

    /// Broadcasts a transaction to the network via a specified RPC node.
    pub async fn broadcast_transaction(
        &self,
        txn_bytes: Vec<u8>,
    ) -> BroadcastResponse {
        let mut hasher = Keccak256::new();
        hasher.update(&txn_bytes);
        let txn_hash = hasher.finalize();

        let request = BroadcastRequest {
            txn: hex::encode(txn_bytes),
        };

        let (status, resp_data) = match self.call_rpc_post::<ResponseData, _>("/broadcast", request).await {
            Ok(result) => result,
            Err(e) => {
                return BroadcastResponse {
                    success: false,
                    data: None,
                    error: format!("RPC call failed: {e:?}"),
                }
            }
        };

        if status != 200 {
            BroadcastResponse {
                success: false,
                data: None,
                error: format!("Failed to broadcast transaction: {}", resp_data.message),
            }
        } else {
            BroadcastResponse {
                success: true,
                data: Some(format!("0x{}", hex::encode_upper(txn_hash))),
                error: "".to_owned(),
            }
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

    pub fn subscribe_to_vida_transactions_with_poll_interval(
        self: Arc<Self>,
        vida_id: u64, 
        starting_block: u64,
        handler: ProcessVidaTransactions,
        _poll_interval: Option<u64>,
    ) -> VidaTransactionSubscription {
        let mut subscription = VidaTransactionSubscription::new(
            self,
            vida_id,
            starting_block,
            handler,
            _poll_interval.unwrap_or(100),
        );
        subscription.start();
        subscription
    }

    pub fn subscribe_to_vida_transactions(
        self: Arc<Self>,
        vida_id: u64, 
        starting_block: u64,
        handler: ProcessVidaTransactions,
    ) -> VidaTransactionSubscription {
        self.subscribe_to_vida_transactions_with_poll_interval(
            vida_id,
            starting_block,
            handler,
            None
        )
    }
}
