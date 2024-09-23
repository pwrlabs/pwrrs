pub mod types;

use reqwest::{Client, StatusCode};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use sha3::{Digest, Keccak256};
use url::Url;

pub use self::types::RPC;
use self::types::RpcError;

use crate::{
    block::Block,
    transaction::types::{NewTransactionData, Transaction},
    delegator::Delegator,
    validator::Validator,
    wallet::types::Wallet
};

const DEFAULT_FEE_PER_BYTE: u64 = 100;
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

    pub async fn blockchain_version(&self) -> Result<u64, RpcError> {
        #[derive(Deserialize)]
        struct Response {
            #[serde(rename = "blockchainVersion")]
            blockchain_version: u64,
        }

        self.call_rpc_get("/blockchainVersion/")
            .await
            .map(|r: Response| r.blockchain_version)
    }

    pub async fn fee(&self) -> Result<u64, RpcError> {
        #[derive(Deserialize)]
        struct Response {
            #[serde(rename = "blockchainVersion")]
            blockchain_version: u64,
        }

        self.call_rpc_get("/blockchainVersion/")
            .await
            .map(|r: Response| r.blockchain_version)
    }

    pub async fn ecdsa_verification_fee(&self) -> Result<u64, RpcError> {
        #[derive(Deserialize)]
        struct Response {
            #[serde(rename = "ecdsaVerificationFee")]
            ecdsa_verification_fee: u64,
        }

        self.call_rpc_get("/ecdsaVerificationFee/")
            .await
            .map(|r: Response| r.ecdsa_verification_fee)
    }

    pub async fn burn_percentage(&self) -> Result<u64, RpcError> {
        #[derive(Deserialize)]
        struct Response {
            #[serde(rename = "burnPercentage")]
            burn_percentage: u64,
        }

        self.call_rpc_get("/burnPercentage/")
            .await
            .map(|r: Response| r.burn_percentage)
    }

    pub async fn total_voting_power_res(&self) -> Result<u64, RpcError> {
        #[derive(Deserialize)]
        struct Response {
            #[serde(rename = "totalVotingPower")]
            total_voting_power: u64,
        }

        self.call_rpc_get("/totalVotingPower/")
            .await
            .map(|r: Response| r.total_voting_power)
    }

    pub async fn pwr_rewards_per_year(&self) -> Result<u64, RpcError> {
        #[derive(Deserialize)]
        struct Response {
            #[serde(rename = "pwrRewardsPerYear")]
            pwr_rewards_per_year: u64,
        }

        self.call_rpc_get("/pwrRewardsPerYear/")
            .await
            .map(|r: Response| r.pwr_rewards_per_year)
    }

    pub async fn withdrawal_lock_time(&self) -> Result<u64, RpcError> {
        #[derive(Deserialize)]
        struct Response {
            #[serde(rename = "withdrawalLockTime")]
            withdrawal_lock_time: u64,
        }

        self.call_rpc_get("/withdrawalLockTime/")
            .await
            .map(|r: Response| r.withdrawal_lock_time)
    }

    pub async fn all_early_withdraw_penalties(&self) -> Result<u64, RpcError> {
        #[derive(Deserialize)]
        struct Response {
            #[serde(rename = "allEarlyWithdrawPenalties")]
            all_early_withdraw_penalties: u64,
        }

        self.call_rpc_get("/allEarlyWithdrawPenalties/")
            .await
            .map(|r: Response| r.all_early_withdraw_penalties)
    }

    pub async fn max_block_size(&self) -> Result<u64, RpcError> {
        #[derive(Deserialize)]
        struct Response {
            #[serde(rename = "maxBlockSize")]
            max_block_size: u64,
        }

        self.call_rpc_get("/maxBlockSize/")
            .await
            .map(|r: Response| r.max_block_size)
    }

    pub async fn max_transaction_size(&self) -> Result<u64, RpcError> {
        #[derive(Deserialize)]
        struct Response {
            #[serde(rename = "maxTransactionSize")]
            max_transaction_size: u64,
        }

        self.call_rpc_get("/maxTransactionSize/")
            .await
            .map(|r: Response| r.max_transaction_size)
    }

    pub async fn block_number(&self) -> Result<u64, RpcError> {
        #[derive(Deserialize)]
        struct Response {
            #[serde(rename = "blockNumber")]
            block_number: u64,
        }

        self.call_rpc_get("/blockNumber/")
            .await
            .map(|r: Response| r.block_number)
    }

    pub async fn block_timestamp(&self) -> Result<u64, RpcError> {
        #[derive(Deserialize)]
        struct Response {
            #[serde(rename = "blockTimestamp")]
            block_timestamp: u64,
        }

        self.call_rpc_get("/blockTimestamp/")
            .await
            .map(|r: Response| r.block_timestamp)
    }

    pub async fn lates_block_number(&self) -> Result<u64, RpcError> {
        self.block_number().await
    }

    pub async fn proposal_fee(&self) -> Result<u64, RpcError> {
        #[derive(Deserialize)]
        struct Response {
            #[serde(rename = "proposalFee")]
            proposal_fee: u64,
        }

        self.call_rpc_get("/proposalFee/")
            .await
            .map(|r: Response| r.proposal_fee)
    }

    pub async fn proposal_validity_time(&self) -> Result<u64, RpcError> {
        #[derive(Deserialize)]
        struct Response {
            #[serde(rename = "proposalValidityTime")]
            proposal_validity_time: u64,
        }

        self.call_rpc_get("/proposalValidityTime/")
            .await
            .map(|r: Response| r.proposal_validity_time)
    }

    pub async fn validator_count_limit(&self) -> Result<u64, RpcError> {
        #[derive(Deserialize)]
        struct Response {
            #[serde(rename = "validatorCountLimit")]
            validator_count_limit: u64,
        }

        self.call_rpc_get("/validatorCountLimit/")
            .await
            .map(|r: Response| r.validator_count_limit)
    }

    pub async fn validator_slashing_fee(&self) -> Result<u64, RpcError> {
        #[derive(Deserialize)]
        struct Response {
            #[serde(rename = "validatorSlashingFee")]
            validator_slashing_fee: u64,
        }

        self.call_rpc_get("/validatorSlashingFee/")
            .await
            .map(|r: Response| r.validator_slashing_fee)
    }

    pub async fn validator_operational_fee(&self) -> Result<u64, RpcError> {
        #[derive(Deserialize)]
        struct Response {
            #[serde(rename = "validatorOperationalFee")]
            validator_operational_fee: u64,
        }

        self.call_rpc_get("/validatorOperationalFee/")
            .await
            .map(|r: Response| r.validator_operational_fee)
    }

    pub async fn validator_joining_fee(&self) -> Result<u64, RpcError> {
        #[derive(Deserialize)]
        struct Response {
            #[serde(rename = "validatorJoiningFee")]
            validator_joining_fee: u64,
        }

        self.call_rpc_get("/validatorJoiningFee/")
            .await
            .map(|r: Response| r.validator_joining_fee)
    }

    pub async fn minimum_delegating_amount(&self) -> Result<u64, RpcError> {
        #[derive(Deserialize)]
        struct Response {
            #[serde(rename = "minimumDelegatingAmount")]
            minimum_delegating_amount: u64,
        }

        self.call_rpc_get("/minimumDelegatingAmount/")
            .await
            .map(|r: Response| r.minimum_delegating_amount)
    }

    pub async fn total_delegators_count(&self) -> Result<u64, RpcError> {
        #[derive(Deserialize)]
        struct Response {
            #[serde(rename = "totalDelegatorsCount")]
            total_delegators_count: u64,
        }

        self.call_rpc_get("/totalDelegatorsCount/")
            .await
            .map(|r: Response| r.total_delegators_count)
    }

    pub async fn validator(
        &self,
        validator_address: &str,
    ) -> Result<Vec<Validator>, RpcError> {
        #[derive(Deserialize)]
        struct Response {
            validators: Vec<Validator>,
        }
        self.call_rpc_get(&format!(
            "/validator/?validatorAddress={}",
            validator_address
        ))
        .await
        .map(|r: Response| r.validators)
    }

    pub async fn delegators_of_pwr(
        &self,
        delegator_address: &str,
        validator_address: &str,
    ) -> Result<Vec<Delegator>, RpcError> {
        #[derive(Deserialize)]
        struct Response {
            delegators: Vec<Delegator>,
        }
        self.call_rpc_get(&format!(
            "/validator/delegator/delegatedPWROfAddress/?userAddress={}&validatorAddress={}",
            delegator_address, validator_address
        ))
        .await
        .map(|r: Response| r.delegators)
    }

    pub async fn shares_of_delegator(
        &self,
        delegator_address: &str,
        validator_address: &str,
    ) -> Result<u64, RpcError> {
        #[derive(Deserialize)]
        struct Response {
            #[serde(rename = "sharesOfAddress")]
            shares_of_delegator: u64,
        }
        self.call_rpc_get(&format!(
            "/validator/delegator/sharesOfAddress/?userAddress={}&validatorAddress={}",
            delegator_address, validator_address
        ))
        .await
        .map(|r: Response| r.shares_of_delegator)
    }

    pub async fn share_value(
        &self,
        validator_address: &str,
    ) -> Result<u64, RpcError> {
        #[derive(Deserialize)]
        struct Response {
            #[serde(rename = "shareValue")]
            share_value: u64,
        }
        self.call_rpc_get(&format!(
            "/validator/shareValue/?validatorAddress={}",
            validator_address
        ))
        .await
        .map(|r: Response| r.share_value)
    }

    // 
    pub async fn vm_owner_transaction_fee_share(&self) -> Result<u64, RpcError> {
        #[derive(Deserialize)]
        struct Response {
            #[serde(rename = "vmOwnerTransactionFeeShare")]
            vm_owner_transaction_fee_share: u64,
        }

        self.call_rpc_get("/vmOwnerTransactionFeeShare/")
            .await
            .map(|r: Response| r.vm_owner_transaction_fee_share)
    }

    pub async fn vm_id_claiming_fee(&self) -> Result<u64, RpcError> {
        #[derive(Deserialize)]
        struct Response {
            #[serde(rename = "vmIdClaimingFee")]
            vm_id_claiming_fee: u64,
        }

        self.call_rpc_get("/vmIdClaimingFee/")
            .await
            .map(|r: Response| r.vm_id_claiming_fee)
    }

    pub async fn vm_data_transactions(
        &self,
        starting_block: u64,
        ending_block: u64,
        vm_id: u64,
    ) -> Result<Vec<Transaction>, RpcError> {
        #[derive(Deserialize)]
        struct Response {
            transactions: Vec<Transaction>,
        }
        self.call_rpc_get(&format!(
            "/getVmTransactions/?startingBlock={}&endingBlock={}&vmId={}",
            starting_block, ending_block, vm_id
        ))
        .await
        .map(|r: Response| r.transactions)
    }

    pub async fn vm_id_address(&self, vm_id: i64) -> String {
        let mut hex_address = if vm_id >= 0 { String::from("1") } else { String::from("0") };
    
        let vm_id = if vm_id < 0 { -vm_id } else { vm_id };
        let vm_id_string = vm_id.to_string();
    
        for _ in 0..(38 - vm_id_string.len()) { // Adjust padding to 38 since we already add '1' or '0'
            hex_address.push('0');
        }
    
        hex_address.push_str(&vm_id_string);
    
        format!("0x{}", hex_address)
    }

    pub async fn owner_of_vm_id(
        &self,
        vm_id: i64,
    ) -> Result<String, RpcError> {
        #[derive(Deserialize)]
        struct Response {
            #[serde(rename = "owner")]
            owner_of_vm_id: String,
        }
        self.call_rpc_get(&format!(
            "/ownerOfVmId/?vmId={}",
            vm_id
        ))
        .await
        .map(|r: Response| r.owner_of_vm_id)
    }

    pub async fn conduits_of_vm(
        &self,
        vm_id: i64,
    ) -> Result<Vec<Validator>, RpcError> {
        #[derive(Deserialize)]
        struct Response {
            conduits: Vec<Validator>,
        }
        self.call_rpc_get(&format!(
            "/conduitsOfVm/?vmId={}",
            vm_id
        ))
        .await
        .map(|r: Response| r.conduits)
    }

    pub async fn max_guardian_time(
        &self,
    ) -> Result<u64, RpcError> {
        #[derive(Deserialize)]
        struct Response {
            #[serde(rename = "maxGuardianTime")]
            max_guardian_time: u64,
        }
        self.call_rpc_get(&format!(
            "/maxGuardianTime/"
        ))
        .await
        .map(|r: Response| r.max_guardian_time)
    }
    
    pub async fn guardian_of_address(
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
                "/guardianOf/?userAddress={}",
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

        let nonce = self.nonce_of_address(&wallet.get_address()).await?;

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
