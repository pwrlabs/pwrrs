use crate::config::falcon::Falcon;
use crate::transaction::{NewTransactionData, types::Transaction};
use pqcrypto_falcon::{falcon512 as falcon512_pqcrypto};
use pqcrypto_traits::sign::*;
use hex;
use sha3::{Digest, Keccak224};
use crate::wallet::types::Wallet;
use crate::rpc::{RPC, BroadcastResponse};
use crate::wallet::keys::NODE_URL;
use crate::config::aes256::AES256;
use bip39::{Mnemonic, Language};
use rand::{thread_rng, RngCore};
use hmac::Hmac;
use pbkdf2::pbkdf2;
use sha2::Sha512;

impl Wallet {
    pub async fn new_random_with_rpc_url(word_count: u8, rpc_url: &str) -> Self {
        // Validate word count
        if ![12, 15, 18, 21, 24].contains(&word_count) {
            panic!("Word count must be one of 12, 15, 18, 21, or 24");
        }

        // Calculate entropy bytes based on word count
        let entropy_bytes = match word_count {
            12 => 16, // 128 bits
            15 => 20, // 160 bits
            18 => 24, // 192 bits
            21 => 28, // 224 bits
            24 => 32, // 256 bits
            _ => unreachable!(),
        };

        // Generate random entropy
        let mut entropy = vec![0u8; entropy_bytes];
        thread_rng().fill_bytes(&mut entropy);
        
        // Create mnemonic from entropy
        let mnemonic = Mnemonic::from_entropy(&entropy, Language::English)
            .map_err(|_| "Failed to generate mnemonic").unwrap();
        let phrase = mnemonic.phrase();
        
        let seed = Self::generate_seed(phrase);
        let (public_key, secret_key) = Falcon::generate_keypair_512_from_seed(&seed);

        let hash = Self::hash224(&public_key[1..].to_vec());
        let address = hash[0..20].to_vec();

        Self {
            public_key: public_key.to_vec(),
            private_key: secret_key.to_vec(),
            address: address,
            seed_phrase: phrase.as_bytes().to_vec(),
            rpc_url: rpc_url.to_string(),
        }
    }

    pub async fn new_with_rpc_url_and_phrase(seed_phrase: &str, rpc_url: &str) -> Self {
        let seed = Self::generate_seed(seed_phrase);
        let (public_key, secret_key) = Falcon::generate_keypair_512_from_seed(&seed);

        let hash = Self::hash224(&public_key[1..].to_vec());
        let address = hash[0..20].to_vec();

        Self {
            public_key: public_key.to_vec(),
            private_key: secret_key.to_vec(),
            address: address,
            seed_phrase: seed_phrase.as_bytes().to_vec(),
            rpc_url: rpc_url.to_string(),
        }
    }

    pub async fn new_random(word_count: u8) -> Self {
        Self::new_random_with_rpc_url(word_count, NODE_URL).await
    }

    pub async fn new(seed_phrase: &str) -> Self {
        Self::new_with_rpc_url_and_phrase(seed_phrase, NODE_URL).await
    }

    pub fn sign(&self, message: Vec<u8>) -> Vec<u8> {
        let private_key = falcon512_pqcrypto::SecretKey::from_bytes(&self.private_key).unwrap();
        Falcon::sign_512(&message, &private_key).as_bytes().to_vec()
    }

    pub fn verify_sign(&self, message: Vec<u8>, signature: Vec<u8>) -> bool {
        let public_key = falcon512_pqcrypto::PublicKey::from_bytes(&self.public_key).unwrap();
        let signature = falcon512_pqcrypto::DetachedSignature::from_bytes(&signature).unwrap();
        Falcon::verify_512(&message, &signature, &public_key)
    }

    pub fn store_wallet(&self, path: &str, password: &str) -> Result<(), Box<dyn std::error::Error>> {
        let seed_phrase: Vec<u8> = self.seed_phrase.clone();

        let encrypted_private_key = AES256::encrypt(&seed_phrase, password)
            .map_err(|e| format!("Encryption error: {:?}", e))?;

        std::fs::write(path, encrypted_private_key)?;

        Ok(())
    }

    pub async fn load_wallet_with_rpc_url(path: &str, password: &str, rpc_url: &str) -> Option<Self> {
        let encrypted_data = std::fs::read(path).ok()?;
        let seed_phrase = AES256::decrypt(&encrypted_data, password).ok()?;
        let seed_phrase_str = String::from_utf8(seed_phrase).ok()?;

        Some(Self::new_with_rpc_url_and_phrase(seed_phrase_str.as_str(), rpc_url).await)
    }

    pub async fn load_wallet(path: &str, password: &str) -> Option<Self> {
        Self::load_wallet_with_rpc_url(path, password, NODE_URL).await
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

    pub fn get_seed_phrase(&self) -> String {
        String::from_utf8(self.seed_phrase.clone()).ok().unwrap()
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
    }

    fn generate_seed(phrase: &str) -> Vec<u8> {
        let salt = format!("mnemonic");
        let mut seed = vec![0u8; 64];
        pbkdf2::<Hmac<Sha512>>(
            phrase.as_bytes(),
            salt.as_bytes(),
            2048,
            &mut seed
        ).unwrap();
        seed
    }
}
