use crate::config::falcon::Falcon;
use crate::transaction::{NewTransactionData, types::Transaction};
use pqcrypto_falcon::falcon512;
use pqcrypto_traits::sign::*;
use std::fs;
use std::io::Read;
use std::path::Path;
use std::error::Error;
use hex;
use sha3::{Digest, Keccak224};
use crate::wallet::types::Wallet;
use crate::rpc::{RPC, BroadcastResponse};
<<<<<<< HEAD
use crate::wallet::keys::NODE_URL;

impl Wallet {
    pub fn new_with_rpc_url(rpc_url: &str) -> Self {
        let (public_key, secret_key) = Falcon::generate_keypair_512();
        
        // Get the hash of the public key
        let hash = Self::hash224(public_key.as_bytes());
        let address = hash[0..20].to_vec();

        Self {
            public_key: public_key.as_bytes().to_vec(),
            private_key: secret_key.as_bytes().to_vec(),
            address: address,
=======
use crate::config::aes256::AES256;
use crate::wallet::keys::NODE_URL;

impl Wallet {
    #[cfg(feature = "rand")]
    pub fn random_with_rpc_url(rpc_url: &str) -> Self {
        let signing_key = SigningKey::random(&mut OsRng);

        Self {
            private_key: signing_key,
>>>>>>> upstream/main
            rpc_url: rpc_url.to_string(),
        }
    }

<<<<<<< HEAD
    pub fn new() -> Self {
        Self::new_with_rpc_url(NODE_URL)
=======
    #[cfg(feature = "rand")]
    pub fn random() -> Self {
        Self::random_with_rpc_url(NODE_URL)
    }

    pub fn from_hex_with_rpc_url(hex_str: &str, rpc_url: &str) -> Result<Self, Error> {
        let bytes = if hex_str.len() > 2 && (&hex_str[..2] == "0x" || &hex_str[..2] == "0X") {
            hex::decode(&hex_str[2..]).map_err(|_| Error::new())?
        } else {
            hex::decode(hex_str).map_err(|_| Error::new())?
        };
        let private_key = SigningKey::from_slice(&bytes)?;

        Ok(Self { private_key, rpc_url: rpc_url.to_string() })
    }

    pub fn from_hex(hex_str: &str) -> Result<Self, Error> {
        Self::from_hex_with_rpc_url(hex_str, NODE_URL)
>>>>>>> upstream/main
    }

    pub fn from_keys_with_rpc_url(public_key: falcon512::PublicKey, secret_key: falcon512::SecretKey, rpc_url: &str) -> Self {
        // Get the hash of the public key
        let public_key_bytes = public_key.as_bytes();
        let hash = Self::hash224(public_key_bytes);
        let address = hash[0..20].to_vec();

        Self {
            public_key: public_key.as_bytes().to_vec(),
            private_key: secret_key.as_bytes().to_vec(),
            address: address,
            rpc_url: rpc_url.to_string(),
        }
    }

    pub fn from_keys(public_key: falcon512::PublicKey, secret_key: falcon512::SecretKey) -> Self {
        Self::from_keys_with_rpc_url(public_key, secret_key, NODE_URL)
    }

    pub fn sign(&self, message: Vec<u8>) -> Vec<u8> {
        let private_key = falcon512::SecretKey::from_bytes(&self.private_key).unwrap();
        Falcon::sign_512(&message, &private_key).as_bytes().to_vec()
    }

    pub fn verify_sign(&self, message: Vec<u8>, signature: Vec<u8>) -> bool {
        let public_key = falcon512::PublicKey::from_bytes(&self.public_key).unwrap();
        let signature = falcon512::DetachedSignature::from_bytes(&signature).unwrap();
        Falcon::verify_512(&message, &signature, &public_key)
    }

    pub fn store_wallet<P: AsRef<Path>>(&self, file_path: P) -> Result<(), Box<dyn Error>> {    
        let mut buffer = Vec::new();

        buffer.extend_from_slice(&(self.public_key.len() as u32).to_be_bytes());
        buffer.extend_from_slice(&self.public_key);
        
        buffer.extend_from_slice(&(self.private_key.len() as u32).to_be_bytes());
        buffer.extend_from_slice(&self.private_key);
    
        fs::write(file_path, buffer)?;
        
        Ok(())
    }

<<<<<<< HEAD
    pub fn load_wallet_with_rpc_url<P: AsRef<Path>>(file_path: P, rpc_url: &str) -> Result<Self, Box<dyn Error>> {
        let data = fs::read(file_path)?;
        if data.len() < 8 { // At minimum we need two 4-byte length fields
            return Err(format!("File too small: {} bytes", data.len()).into());
        }
    
        let mut cursor = std::io::Cursor::new(&data);
        
        let mut pub_length_bytes = [0u8; 4];
        cursor.read_exact(&mut pub_length_bytes)?;
        let pub_length = u32::from_be_bytes(pub_length_bytes) as usize;
        
        if pub_length == 0 || pub_length > 2048 {
            return Err(format!("Invalid public key length: {}", pub_length).into());
        }
        
        if cursor.position() as usize + pub_length > data.len() {
            return Err(format!("File too small for public key of length {}", pub_length).into());
        }
        
        let mut public_key_bytes = vec![0u8; pub_length];
        cursor.read_exact(&mut public_key_bytes)?;
        
        if cursor.position() as usize + 4 > data.len() {
            return Err("File too small for secret key length".into());
        }
        
        let mut sec_length_bytes = [0u8; 4];
        cursor.read_exact(&mut sec_length_bytes)?;
        let sec_length = u32::from_be_bytes(sec_length_bytes) as usize;
        
        if sec_length == 0 || sec_length > 4096 {
            return Err(format!("Invalid secret key length: {}", sec_length).into());
        }
        
        if cursor.position() as usize + sec_length > data.len() {
            return Err(format!("File too small for secret key of length {}", sec_length).into());
        }
        
        let mut secret_key_bytes = vec![0u8; sec_length];
        cursor.read_exact(&mut secret_key_bytes)?;
        
        let public_key = match falcon512::PublicKey::from_bytes(&public_key_bytes) {
            Ok(key) => key,
            Err(e) => return Err(format!("Failed to parse public key: {}", e).into()),
        };
        
        let secret_key = match falcon512::SecretKey::from_bytes(&secret_key_bytes) {
            Ok(key) => key,
            Err(e) => return Err(format!("Failed to parse secret key: {}", e).into()),
        };
        
        Ok(Self::from_keys_with_rpc_url(public_key, secret_key, rpc_url))
    }

    pub fn load_wallet<P: AsRef<Path>>(file_path: P) -> Result<Self, Box<dyn Error>> {
        Self::load_wallet_with_rpc_url(file_path, NODE_URL)
=======
    pub fn load_wallet_with_rpc_url(path: &str, password: &str, rpc_url: &str) -> Option<Self> {
        let encrypted_data = std::fs::read(path).ok()?;
        let private_key_bytes = AES256::decrypt(&encrypted_data, password).ok()?;
        let private_key = SigningKey::from_slice(&private_key_bytes).ok()?;
        Some(Self { private_key, rpc_url: rpc_url.to_string() })
    }

    pub fn load_wallet(path: &str, password: &str) -> Option<Self> {
        Self::load_wallet_with_rpc_url(path, password, NODE_URL)
>>>>>>> upstream/main
    }

    pub fn get_address(&self) -> String {
        format!("0x{}", hex::encode(&self.address))
    }

    pub fn get_public_key(&self) -> Vec<u8> {
        self.public_key.clone()
    }

    pub fn get_private_key(&self) -> Vec<u8> {
        self.private_key.clone()
    }

    pub async fn get_balance(&self) -> u64 {
        let balance = (self.get_rpc().await).get_balance_of_address(&self.get_address()).await.unwrap();
        return balance;
    }

    pub async fn get_nonce(&self) -> u32 {
        let nonce = (self.get_rpc().await).get_nonce_of_address(&self.get_address()).await.unwrap();
        return nonce;
    }

    async fn get_txn_bytes(&self, tx: NewTransactionData, fee_per_byte: u64) -> Vec<u8> {
        let nonce = (self.get_rpc().await).get_nonce_of_address(
            &self.get_address()
        ).await.unwrap();
        let txn_bytes = tx.falcon512_serialize_for_broadcast(nonce, (self.get_rpc().await).chain_id, fee_per_byte, self).unwrap();
        return txn_bytes;
    }

    pub async fn set_public_key(&self, public_key: String, fee_per_byte: u64) -> BroadcastResponse {
        let tx = NewTransactionData::SetPublicKey {
            public_key: public_key
        };
        let txn_bytes = self.get_txn_bytes(tx, fee_per_byte).await;

        let response = (self.get_rpc().await).broadcast_transaction(txn_bytes).await;
        return response;
    }

    pub async fn join_as_validator(&self, ip: String, fee_per_byte: u64) -> BroadcastResponse {
        let response = self.make_sure_public_key_is_set(fee_per_byte).await;
        if response.as_ref().map_or(false, |r| !r.success) {
            return response.unwrap();
        }

        let tx = NewTransactionData::JoinAsValidator {
            ip: ip
        };
        let txn_bytes = self.get_txn_bytes(tx, fee_per_byte).await;

        let response = (self.get_rpc().await).broadcast_transaction(txn_bytes).await;
        return response;
    }

    pub async fn delegate(&self, validator: String, amount: u64, fee_per_byte: u64) -> BroadcastResponse {
        let response = self.make_sure_public_key_is_set(fee_per_byte).await;
        if response.as_ref().map_or(false, |r| !r.success) {
            return response.unwrap();
        }

        let tx = NewTransactionData::Delegate {
            validator: validator,
            pwr_amount: amount
        };
        let txn_bytes = self.get_txn_bytes(tx, fee_per_byte).await;

        let response = (self.get_rpc().await).broadcast_transaction(txn_bytes).await;
        return response;
    }
    
    pub async fn propose_change_ip(&self, new_ip: String, fee_per_byte: u64) -> BroadcastResponse {
        let response = self.make_sure_public_key_is_set(fee_per_byte).await;
        if response.as_ref().map_or(false, |r| !r.success) {
            return response.unwrap();
        }

        let tx = NewTransactionData::ChangeIp {
            new_ip: new_ip
        };
        let txn_bytes = self.get_txn_bytes(tx, fee_per_byte).await;

        let response = (self.get_rpc().await).broadcast_transaction(txn_bytes).await;
        return response;
    }

    pub async fn claim_active_node_spot(&self, fee_per_byte: u64) -> BroadcastResponse {
        let response = self.make_sure_public_key_is_set(fee_per_byte).await;
        if response.as_ref().map_or(false, |r| !r.success) {
            return response.unwrap();
        }

        let tx = NewTransactionData::ClaimActiveNodeSpot {};
        let txn_bytes = self.get_txn_bytes(tx, fee_per_byte).await;

        let response = (self.get_rpc().await).broadcast_transaction(txn_bytes).await;
        return response;
    }

    pub async fn transfer_pwr(&self, recipient: String, amount: u64, fee_per_byte: u64) -> BroadcastResponse {
        let response = self.make_sure_public_key_is_set(fee_per_byte).await;
        if response.as_ref().map_or(false, |r| !r.success) {
            return response.unwrap();
        }

        let tx = NewTransactionData::Transfer {
            receiver: recipient,
            amount: amount
        };
        let txn_bytes = self.get_txn_bytes(tx, fee_per_byte).await;

        let response = (self.get_rpc().await).broadcast_transaction(txn_bytes).await;
        return response;
    }

    //  Governance Proposal Transactions
    pub async fn propose_change_early_withdraw_penalty(&self, title: String, description: String, withdraw_penalty_time: u64, withdraw_penalty: u32, fee_per_byte: u64) -> BroadcastResponse {
        let response = self.make_sure_public_key_is_set(fee_per_byte).await;
        if response.as_ref().map_or(false, |r| !r.success) {
            return response.unwrap();
        }

        let tx = NewTransactionData::ChangeEarlyWithdrawPenaltyProposal {
            title,
            description,
            withdraw_penalty_time,
            withdraw_penalty
        };
        let txn_bytes = self.get_txn_bytes(tx, fee_per_byte).await;

        let response = (self.get_rpc().await).broadcast_transaction(txn_bytes).await;
        return response;
    }

    pub async fn propose_change_fee_per_byte(&self, title: String, description: String, new_fee_per_byte: u64, fee_per_byte: u64) -> BroadcastResponse {
        let response = self.make_sure_public_key_is_set(fee_per_byte).await;
        if response.as_ref().map_or(false, |r| !r.success) {
            return response.unwrap();
        }

        let tx = NewTransactionData::ChangeFeePerByteProposal {
            title,
            description,
            new_fee_per_byte
        };
        let txn_bytes = self.get_txn_bytes(tx, fee_per_byte).await;

        let response = (self.get_rpc().await).broadcast_transaction(txn_bytes).await;
        return response;
    }

    pub async fn propose_change_max_block_size(&self, title: String, description: String, max_block_size: u32, fee_per_byte: u64) -> BroadcastResponse {
        let response = self.make_sure_public_key_is_set(fee_per_byte).await;
        if response.as_ref().map_or(false, |r| !r.success) {
            return response.unwrap();
        }

        let tx = NewTransactionData::ChangeMaxBlockSizeProposal {
            title,
            description,
            max_block_size
        };
        let txn_bytes = self.get_txn_bytes(tx, fee_per_byte).await;

        let response = (self.get_rpc().await).broadcast_transaction(txn_bytes).await;
        return response;
    }

    pub async fn propose_change_max_txn_size(&self, title: String, description: String, max_txn_size: u32, fee_per_byte: u64) -> BroadcastResponse {
        let response = self.make_sure_public_key_is_set(fee_per_byte).await;
        if response.as_ref().map_or(false, |r| !r.success) {
            return response.unwrap();
        }

        let tx = NewTransactionData::ChangeMaxTxnSizeProposal {
            title,
            description,
            max_txn_size
        };
        let txn_bytes = self.get_txn_bytes(tx, fee_per_byte).await;

        let response = (self.get_rpc().await).broadcast_transaction(txn_bytes).await;
        return response;
    }

    pub async fn propose_change_overall_burn_percentage(&self, title: String, description: String, burn_percentage: u32, fee_per_byte: u64) -> BroadcastResponse {
        let response = self.make_sure_public_key_is_set(fee_per_byte).await;
        if response.as_ref().map_or(false, |r| !r.success) {
            return response.unwrap();
        }

        let tx = NewTransactionData::ChangeOverallBurnPercentageProposal {
            title,
            description,
            burn_percentage
        };
        let txn_bytes = self.get_txn_bytes(tx, fee_per_byte).await;

        let response = (self.get_rpc().await).broadcast_transaction(txn_bytes).await;
        return response;
    }

    pub async fn propose_change_reward_per_year(&self, title: String, description: String, reward_per_year: u64, fee_per_byte: u64) -> BroadcastResponse {
        let response = self.make_sure_public_key_is_set(fee_per_byte).await;
        if response.as_ref().map_or(false, |r| !r.success) {
            return response.unwrap();
        }

        let tx = NewTransactionData::ChangeRewardPerYearProposal {
            title,
            description,
            reward_per_year
        };
        let txn_bytes = self.get_txn_bytes(tx, fee_per_byte).await;

        let response = (self.get_rpc().await).broadcast_transaction(txn_bytes).await;
        return response;
    }

    pub async fn propose_change_validator_count_limit(&self, title: String, description: String, validator_count_limit: u32, fee_per_byte: u64) -> BroadcastResponse {
        let response = self.make_sure_public_key_is_set(fee_per_byte).await;
        if response.as_ref().map_or(false, |r| !r.success) {
            return response.unwrap();
        }

        let tx = NewTransactionData::ChangeValidatorCountLimitProposal {
            title,
            description,
            validator_count_limit
        };
        let txn_bytes = self.get_txn_bytes(tx, fee_per_byte).await;

        let response = (self.get_rpc().await).broadcast_transaction(txn_bytes).await;
        return response;
    }

    pub async fn propose_change_validator_joining_fee(&self, title: String, description: String, joining_fee: u64, fee_per_byte: u64) -> BroadcastResponse {
        let response = self.make_sure_public_key_is_set(fee_per_byte).await;
        if response.as_ref().map_or(false, |r| !r.success) {
            return response.unwrap();
        }

        let tx = NewTransactionData::ChangeValidatorJoiningFeeProposal {
            title,
            description,
            joining_fee
        };
        let txn_bytes = self.get_txn_bytes(tx, fee_per_byte).await;

        let response = (self.get_rpc().await).broadcast_transaction(txn_bytes).await;
        return response;
    }

    pub async fn propose_change_vida_id_claiming_fee(&self, title: String, description: String, claiming_fee: u64, fee_per_byte: u64) -> BroadcastResponse {
        let response = self.make_sure_public_key_is_set(fee_per_byte).await;
        if response.as_ref().map_or(false, |r| !r.success) {
            return response.unwrap();
        }

        let tx = NewTransactionData::ChangeVidaIdClaimingFeeProposal {
            title,
            description,
            claiming_fee
        };
        let txn_bytes = self.get_txn_bytes(tx, fee_per_byte).await;

        let response = (self.get_rpc().await).broadcast_transaction(txn_bytes).await;
        return response;
    }

    pub async fn propose_change_vida_owner_txn_fee_share(&self, title: String, description: String, fee_share: u32, fee_per_byte: u64) -> BroadcastResponse {
        let response = self.make_sure_public_key_is_set(fee_per_byte).await;
        if response.as_ref().map_or(false, |r| !r.success) {
            return response.unwrap();
        }

        let tx = NewTransactionData::ChangeVidaOwnerTxnFeeShareProposal {
            title,
            description,
            fee_share
        };
        let txn_bytes = self.get_txn_bytes(tx, fee_per_byte).await;

        let response = (self.get_rpc().await).broadcast_transaction(txn_bytes).await;
        return response;
    }

    pub async fn propose_other(&self, title: String, description: String, fee_per_byte: u64) -> BroadcastResponse {
        let response = self.make_sure_public_key_is_set(fee_per_byte).await;
        if response.as_ref().map_or(false, |r| !r.success) {
            return response.unwrap();
        }

        let tx = NewTransactionData::OtherProposalTxn {
            title,
            description
        };
        let txn_bytes = self.get_txn_bytes(tx, fee_per_byte).await;

        let response = (self.get_rpc().await).broadcast_transaction(txn_bytes).await;
        return response;
    }

    pub async fn vote_on_proposal(&self, proposal_hash: String, vote: u8, fee_per_byte: u64) -> BroadcastResponse {
        let response = self.make_sure_public_key_is_set(fee_per_byte).await;
        if response.as_ref().map_or(false, |r| !r.success) {
            return response.unwrap();
        }

        let tx = NewTransactionData::VoteOnProposalTxn {
            proposal_hash,
            vote
        };
        let txn_bytes = self.get_txn_bytes(tx, fee_per_byte).await;

        let response = (self.get_rpc().await).broadcast_transaction(txn_bytes).await;
        return response;
    }

<<<<<<< HEAD
    // Falcon Guardian Transactions
    pub async fn guardian_approval(&self, transactions: Vec<Transaction>, fee_per_byte: u64) -> BroadcastResponse {
        let response = self.make_sure_public_key_is_set(fee_per_byte).await;
        if response.as_ref().map_or(false, |r| !r.success) {
            return response.unwrap();
        }

        let tx = NewTransactionData::GuardianApproval {
            transactions
        };
        let txn_bytes = self.get_txn_bytes(tx, fee_per_byte).await;

        let response = (self.get_rpc().await).broadcast_transaction(txn_bytes).await;
        return response;
    }

    pub async fn remove_guardian(&self, fee_per_byte: u64) -> BroadcastResponse {
        let response = self.make_sure_public_key_is_set(fee_per_byte).await;
        if response.as_ref().map_or(false, |r| !r.success) {
            return response.unwrap();
        }

        let tx = NewTransactionData::RemoveGuardian {};
        let txn_bytes = self.get_txn_bytes(tx, fee_per_byte).await;

        let response = (self.get_rpc().await).broadcast_transaction(txn_bytes).await;
        return response;
    }

    pub async fn set_guardian(&self, guardian_expiry_date: u64, guardian: String, fee_per_byte: u64) -> BroadcastResponse {
        let response = self.make_sure_public_key_is_set(fee_per_byte).await;
        if response.as_ref().map_or(false, |r| !r.success) {
            return response.unwrap();
        }

        let tx = NewTransactionData::SetGuardian {
            guardian_expiry_date,
            guardian
        };
        let txn_bytes = self.get_txn_bytes(tx, fee_per_byte).await;

        let response = (self.get_rpc().await).broadcast_transaction(txn_bytes).await;
        return response;
    }

    // Falcon Staking Transactions
    pub async fn move_stake(&self, shares_amount: u64, from_validator: String, to_validator: String, fee_per_byte: u64) -> BroadcastResponse {
        let response = self.make_sure_public_key_is_set(fee_per_byte).await;
        if response.as_ref().map_or(false, |r| !r.success) {
            return response.unwrap();
        }

        let tx = NewTransactionData::MoveStake {
            shares_amount,
            from_validator,
            to_validator
        };
        let txn_bytes = self.get_txn_bytes(tx, fee_per_byte).await;

        let response = (self.get_rpc().await).broadcast_transaction(txn_bytes).await;
        return response;
    }

    pub async fn remove_validator(&self, validator: String, fee_per_byte: u64) -> BroadcastResponse {
        let response = self.make_sure_public_key_is_set(fee_per_byte).await;
        if response.as_ref().map_or(false, |r| !r.success) {
            return response.unwrap();
        }

        let tx = NewTransactionData::RemoveValidator {
            validator
        };
        let txn_bytes = self.get_txn_bytes(tx, fee_per_byte).await;

        let response = (self.get_rpc().await).broadcast_transaction(txn_bytes).await;
        return response;
    }

    pub async fn withdraw(&self, shares: u64, validator: String, fee_per_byte: u64) -> BroadcastResponse {
        let response = self.make_sure_public_key_is_set(fee_per_byte).await;
        if response.as_ref().map_or(false, |r| !r.success) {
            return response.unwrap();
        }

        let tx = NewTransactionData::Withdraw {
            shares,
            validator
        };
        let txn_bytes = self.get_txn_bytes(tx, fee_per_byte).await;

        let response = (self.get_rpc().await).broadcast_transaction(txn_bytes).await;
        return response;
    }

    // Falcon VIDA Transactions
    pub async fn claim_vida_id(&self, vida_id: u64, fee_per_byte: u64) -> BroadcastResponse {
        let response = self.make_sure_public_key_is_set(fee_per_byte).await;
        if response.as_ref().map_or(false, |r| !r.success) {
            return response.unwrap();
        }

        let tx = NewTransactionData::ClaimVidaId {
            vida_id
        };
        let txn_bytes = self.get_txn_bytes(tx, fee_per_byte).await;

        let response = (self.get_rpc().await).broadcast_transaction(txn_bytes).await;
        return response;
    }

    pub async fn conduit_approval(&self, vida_id: u64, transactions: Vec<Transaction>, fee_per_byte: u64) -> BroadcastResponse {
        let response = self.make_sure_public_key_is_set(fee_per_byte).await;
        if response.as_ref().map_or(false, |r| !r.success) {
            return response.unwrap();
        }

        let tx = NewTransactionData::ConduitApproval {
            vida_id,
            transactions
        };
        let txn_bytes = self.get_txn_bytes(tx, fee_per_byte).await;

        let response = (self.get_rpc().await).broadcast_transaction(txn_bytes).await;
        return response;
    }

    pub async fn send_payable_vida_data(&self, vida_id: u64, data: Vec<u8>, amount: u64, fee_per_byte: u64) -> BroadcastResponse {
        let response = self.make_sure_public_key_is_set(fee_per_byte).await;
        if response.as_ref().map_or(false, |r| !r.success) {
            return response.unwrap();
        }

        let tx = NewTransactionData::PayableVidaData {
            vida_id,
            data,
            value: amount
        };
        let txn_bytes = self.get_txn_bytes(tx, fee_per_byte).await;

        let response = (self.get_rpc().await).broadcast_transaction(txn_bytes).await;
        return response;
    }

    pub async fn send_vida_data(&self, vida_id: u64, data: Vec<u8>, fee_per_byte: u64) -> BroadcastResponse {
        return self.send_payable_vida_data(vida_id, data, 0, fee_per_byte).await;
    }

    pub async fn remove_conduits(&self, vida_id: u64, conduits: Vec<String>, fee_per_byte: u64) -> BroadcastResponse {
        let response = self.make_sure_public_key_is_set(fee_per_byte).await;
        if response.as_ref().map_or(false, |r| !r.success) {
            return response.unwrap();
        }

        let tx = NewTransactionData::RemoveConduits {
            vida_id,
            conduits
        };
        let txn_bytes = self.get_txn_bytes(tx, fee_per_byte).await;

        let response = (self.get_rpc().await).broadcast_transaction(txn_bytes).await;
        return response;
    }

    pub async fn set_conduit_mode(&self, vida_id: u64, mode: u8, conduit_threshold: u32, conduits: Vec<String>, conduits_with_voting_power: Vec<(String, u64)>, fee_per_byte: u64) -> BroadcastResponse {
        let response = self.make_sure_public_key_is_set(fee_per_byte).await;
        if response.as_ref().map_or(false, |r| !r.success) {
            return response.unwrap();
        }

        let tx = NewTransactionData::SetConduitMode {
            vida_id,
            mode,
            conduit_threshold,
            conduits,
            conduits_with_voting_power
        };
        let txn_bytes = self.get_txn_bytes(tx, fee_per_byte).await;

        let response = (self.get_rpc().await).broadcast_transaction(txn_bytes).await;
        return response;
    }

    pub async fn set_vida_private_state(&self, vida_id: u64, private_state: bool, fee_per_byte: u64) -> BroadcastResponse {
        let response = self.make_sure_public_key_is_set(fee_per_byte).await;
        if response.as_ref().map_or(false, |r| !r.success) {
            return response.unwrap();
        }

        let tx = NewTransactionData::SetVidaPrivateState {
            vida_id,
            private_state
        };
        let txn_bytes = self.get_txn_bytes(tx, fee_per_byte).await;

        let response = (self.get_rpc().await).broadcast_transaction(txn_bytes).await;
        return response;
    }

    pub async fn set_vida_to_absolute_public(&self, vida_id: u64, fee_per_byte: u64) -> BroadcastResponse {
        let response = self.make_sure_public_key_is_set(fee_per_byte).await;
        if response.as_ref().map_or(false, |r| !r.success) {
            return response.unwrap();
        }

        let tx = NewTransactionData::SetVidaToAbsolutePublic {
            vida_id
        };
        let txn_bytes = self.get_txn_bytes(tx, fee_per_byte).await;

        let response = (self.get_rpc().await).broadcast_transaction(txn_bytes).await;
        return response;
    }

    pub async fn add_vida_sponsored_addresses(&self, vida_id: u64, sponsored_addresses: Vec<String>, fee_per_byte: u64) -> BroadcastResponse {
        let response = self.make_sure_public_key_is_set(fee_per_byte).await;
        if response.as_ref().map_or(false, |r| !r.success) {
            return response.unwrap();
        }

        let tx = NewTransactionData::AddVidaSponsoredAddresses {
            vida_id,
            sponsored_addresses
        };
        let txn_bytes = self.get_txn_bytes(tx, fee_per_byte).await;

        let response = (self.get_rpc().await).broadcast_transaction(txn_bytes).await;
        return response;
    }

    pub async fn add_vida_allowed_senders(&self, vida_id: u64, allowed_senders: Vec<String>, fee_per_byte: u64) -> BroadcastResponse {
        let response = self.make_sure_public_key_is_set(fee_per_byte).await;
        if response.as_ref().map_or(false, |r| !r.success) {
            return response.unwrap();
        }

        let tx = NewTransactionData::AddVidaAllowedSenders {
            vida_id,
            allowed_senders
        };
        let txn_bytes = self.get_txn_bytes(tx, fee_per_byte).await;

        let response = (self.get_rpc().await).broadcast_transaction(txn_bytes).await;
        return response;
    }

    pub async fn remove_vida_allowed_senders(&self, vida_id: u64, allowed_senders: Vec<String>, fee_per_byte: u64) -> BroadcastResponse {
        let response = self.make_sure_public_key_is_set(fee_per_byte).await;
        if response.as_ref().map_or(false, |r| !r.success) {
            return response.unwrap();
        }

        let tx = NewTransactionData::RemoveVidaAllowedSenders {
            vida_id,
            allowed_senders
        };
        let txn_bytes = self.get_txn_bytes(tx, fee_per_byte).await;

        let response = (self.get_rpc().await).broadcast_transaction(txn_bytes).await;
        return response;
    }

    pub async fn remove_sponsored_addresses(&self, vida_id: u64, sponsored_addresses: Vec<String>, fee_per_byte: u64) -> BroadcastResponse {
        let response = self.make_sure_public_key_is_set(fee_per_byte).await;
        if response.as_ref().map_or(false, |r| !r.success) {
            return response.unwrap();
        }

        let tx = NewTransactionData::RemoveSponsoredAddresses {
            vida_id,
            sponsored_addresses
        };
        let txn_bytes = self.get_txn_bytes(tx, fee_per_byte).await;

        let response = (self.get_rpc().await).broadcast_transaction(txn_bytes).await;
        return response;
    }

    pub async fn set_pwr_transfer_rights(&self, vida_id: u64, owner_can_transfer_pwr: bool, fee_per_byte: u64) -> BroadcastResponse {
        let response = self.make_sure_public_key_is_set(fee_per_byte).await;
        if response.as_ref().map_or(false, |r| !r.success) {
            return response.unwrap();
        }

        let tx = NewTransactionData::SetPwrTransferRights {
            vida_id,
            owner_can_transfer_pwr
        };
        let txn_bytes = self.get_txn_bytes(tx, fee_per_byte).await;

        let response = (self.get_rpc().await).broadcast_transaction(txn_bytes).await;
        return response;
    }

    pub async fn transfer_pwr_from_vida(&self, vida_id: u64, receiver: String, amount: u64, fee_per_byte: u64) -> BroadcastResponse {
        let response = self.make_sure_public_key_is_set(fee_per_byte).await;
        if response.as_ref().map_or(false, |r| !r.success) {
            return response.unwrap();
        }

        let tx = NewTransactionData::TransferPwrFromVida {
            vida_id,
            receiver,
            amount
        };
        let txn_bytes = self.get_txn_bytes(tx, fee_per_byte).await;

        let response = (self.get_rpc().await).broadcast_transaction(txn_bytes).await;
        return response;
    }

    async fn make_sure_public_key_is_set(&self, fee_per_byte: u64) -> Option<BroadcastResponse> {
        if self.get_nonce().await == 0 {
            let public_key = hex::encode(self.get_public_key());
            return Some(self.set_public_key(public_key, fee_per_byte).await);
        } else {
            return None;
        }
    }

    pub async fn get_rpc(&self) -> RPC {
        RPC::new(self.rpc_url.as_str()).await.unwrap()
    }

    fn hash224(input: &[u8]) -> Vec<u8> {
        let mut hasher = Keccak224::new();
        hasher.update(input);
        hasher.finalize().to_vec()
=======
    async fn get_rpc(&self) -> RPC {
        RPC::new(self.rpc_url.as_str()).await.unwrap()
    }
}

impl Hash for Wallet {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let bytes = self.private_key.to_bytes();
        state.write(&bytes)
    }
}

impl TryFrom<String> for Wallet {
    type Error = Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::from_hex(&value)
    }
}

impl From<Wallet> for String {
    fn from(value: Wallet) -> Self {
        value.to_hex()
    }
}

impl Display for Wallet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.to_hex())
    }
}

#[cfg(feature = "borsh")]
impl borsh::BorshSerialize for super::Wallet {
    fn serialize<W: std::io::prelude::Write>(&self, writer: &mut W) -> std::io::Result<()> {
        write!(writer, "{}", self.to_hex())
    }
}

#[cfg(feature = "borsh")]
impl borsh::BorshDeserialize for super::Wallet {
    fn deserialize_reader<R: std::io::prelude::Read>(reader: &mut R) -> std::io::Result<Self> {
        let s = String::deserialize_reader(reader)?;
        Self::from_hex(&s)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, e.to_string()))
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    const PRIVATE_KEY_HEX: &str =
        "0x9D4428C6E0638331B4866B70C831F8BA51C11B031F4B55EED4087BBB8EF0151F";

    #[test]
    fn wallet_can_be_created_from_hex_string() {
        Wallet::from_hex(PRIVATE_KEY_HEX).unwrap();
    }

    #[test]
    fn wallet_can_be_encoded_to_hex_string() {
        let wallet = Wallet::from_hex(PRIVATE_KEY_HEX).unwrap();
        let encoded_wallet = wallet.to_hex();
        assert_eq!(format!("0x{}", encoded_wallet), PRIVATE_KEY_HEX);
    }

    #[test]
    fn can_get_public_key_from_wallet() {
        let wallet = Wallet::from_hex(PRIVATE_KEY_HEX).unwrap();
        let public_key = wallet.get_public_key();
        assert_eq!(public_key, PublicKey::from_hex("040cd999a20b0eba1cf86362c738929671902c9b337ab1370d2ba790be68b01227cab9fa9096b87651686bf898acf11857906907ba7fca4f5f5d9513bdd16e0a52").unwrap());
    }

    #[test]
    fn can_get_address_from_public_key() {
        let wallet = Wallet::from_hex(PRIVATE_KEY_HEX).unwrap();
        let address = wallet.get_address();
        assert_eq!(address, "0xA4710E3D79E1ED973AF58E0F269E9B21DD11BC64");
    }

    #[test]
    fn can_sign_message() {
        let wallet = Wallet::from_hex(PRIVATE_KEY_HEX).unwrap();
        let sign = wallet.sign(b"Hello World").unwrap();
        assert_eq!(
            hex::encode_upper(&sign),
            "4BFE08E9CDD47B064A812011E8DEC867D35833C072047958729BD5FE950F62B53E47C450BA8FED1D190D6ABB60B20ADC32237C5C072C0E1AA56CDBA023062D621B"
        );
    }

    #[test]
    fn can_verify_signed_message() {
        let wallet = Wallet::from_hex(PRIVATE_KEY_HEX).unwrap();
        let sign = wallet.sign(b"Hello World").unwrap();
        wallet.verify_sign(b"Hello World", &sign).unwrap();
>>>>>>> upstream/main
    }
}
