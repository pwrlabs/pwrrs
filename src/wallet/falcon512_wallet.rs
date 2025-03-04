use crate::config::falcon::Falcon;
use crate::transaction::NewTransactionData;
use pqcrypto_falcon::falcon512;
use pqcrypto_traits::sign::*;
use std::fs;
use std::io::Read;
use std::path::Path;
use std::error::Error;
use hex;
use sha3::{Digest, Keccak224};
use crate::wallet::types::Falcon512Wallet;
use crate::rpc::{RPC, BroadcastResponse};

const NODE_URL: &str = "https://pwrrpc.pwrlabs.io/";

impl Falcon512Wallet {
    pub fn new() -> Self {
        let (public_key, secret_key) = Falcon::generate_keypair_512();
        
        // Get the hash of the public key
        let hash = Self::hash224(public_key.as_bytes());
        let address = hash[0..20].to_vec();

        Self {
            public_key: public_key.as_bytes().to_vec(),
            private_key: secret_key.as_bytes().to_vec(),
            address: address,
        }
    }

    pub fn from_keys(public_key: falcon512::PublicKey, secret_key: falcon512::SecretKey) -> Self {
        // Get the hash of the public key
        let public_key_bytes = public_key.as_bytes();
        let hash = Self::hash224(public_key_bytes);
        let address = hash[0..20].to_vec();

        Self {
            public_key: public_key.as_bytes().to_vec(),
            private_key: secret_key.as_bytes().to_vec(),
            address: address,
        }
    }

    pub fn sign(&self, message: Vec<u8>) -> Vec<u8> {
        let private_key = falcon512::SecretKey::from_bytes(&self.private_key).unwrap();
        Falcon::sign_512(&message, &private_key).as_bytes().to_vec()
    }

    pub fn verify_sign(&self, message: Vec<u8>, signature: Vec<u8>) -> bool {
        let public_key = falcon512::PublicKey::from_bytes(&self.public_key).unwrap();
        let signature = falcon512::SignedMessage::from_bytes(&signature).unwrap();
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
    
    pub fn load_wallet<P: AsRef<Path>>(file_path: P) -> Result<Self, Box<dyn Error>> {
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
        
        Ok(Self::from_keys(public_key, secret_key))
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

    async fn get_txn_bytes(&self, tx: NewTransactionData) -> Vec<u8> {
        let nonce = (self.get_rpc().await).get_nonce_of_address(
            &self.get_address()
        ).await.unwrap();
        let txn_bytes = tx.falcon512_serialize_for_broadcast(nonce, (self.get_rpc().await).chain_id, self).unwrap();
        return txn_bytes;
    }

    pub async fn set_public_key(&self, public_key: String, fee_per_byte: u64) -> BroadcastResponse {
        let tx = NewTransactionData::FalconSetPublicKey {
            fee_per_byte: fee_per_byte,
            public_key: public_key
        };
        let txn_bytes = self.get_txn_bytes(tx).await;

        let response = (self.get_rpc().await).broadcast_transaction(txn_bytes).await;
        return response;
    }

    pub async fn join_as_validator(&self, ip: String, fee_per_byte: u64) -> BroadcastResponse {
        let response = self.make_sure_public_key_is_set(fee_per_byte).await;
        if response.as_ref().map_or(false, |r| !r.success) {
            return response.unwrap();
        }

        let tx = NewTransactionData::FalconJoinAsValidator {
            fee_per_byte: fee_per_byte,
            ip: ip
        };
        let txn_bytes = self.get_txn_bytes(tx).await;

        let response = (self.get_rpc().await).broadcast_transaction(txn_bytes).await;
        return response;
    }

    pub async fn delegate(&self, validator: String, amount: u64, fee_per_byte: u64) -> BroadcastResponse {
        let response = self.make_sure_public_key_is_set(fee_per_byte).await;
        if response.as_ref().map_or(false, |r| !r.success) {
            return response.unwrap();
        }

        let tx = NewTransactionData::FalconDelegate {
            fee_per_byte: fee_per_byte,
            validator: validator,
            pwr_amount: amount
        };
        let txn_bytes = self.get_txn_bytes(tx).await;

        let response = (self.get_rpc().await).broadcast_transaction(txn_bytes).await;
        return response;
    }
    
    pub async fn change_ip(&self, new_ip: String, fee_per_byte: u64) -> BroadcastResponse {
        let response = self.make_sure_public_key_is_set(fee_per_byte).await;
        if response.as_ref().map_or(false, |r| !r.success) {
            return response.unwrap();
        }

        let tx = NewTransactionData::FalconChangeIp {
            fee_per_byte: fee_per_byte,
            new_ip: new_ip
        };
        let txn_bytes = self.get_txn_bytes(tx).await;

        let response = (self.get_rpc().await).broadcast_transaction(txn_bytes).await;
        return response;
    }

    pub async fn claim_active_node_spot(&self, fee_per_byte: u64) -> BroadcastResponse {
        let response = self.make_sure_public_key_is_set(fee_per_byte).await;
        if response.as_ref().map_or(false, |r| !r.success) {
            return response.unwrap();
        }

        let tx = NewTransactionData::FalconClaimActiveNodeSpot {
            fee_per_byte: fee_per_byte
        };
        let txn_bytes = self.get_txn_bytes(tx).await;

        let response = (self.get_rpc().await).broadcast_transaction(txn_bytes).await;
        return response;
    }

    pub async fn transfer_pwr(&self, recipient: String, amount: u64, fee_per_byte: u64) -> BroadcastResponse {
        let response = self.make_sure_public_key_is_set(fee_per_byte).await;
        if response.as_ref().map_or(false, |r| !r.success) {
            return response.unwrap();
        }

        let tx = NewTransactionData::FalconTransfer {
            fee_per_byte: fee_per_byte,
            receiver: recipient,
            amount: amount
        };
        let txn_bytes = self.get_txn_bytes(tx).await;

        let response = (self.get_rpc().await).broadcast_transaction(txn_bytes).await;
        return response;
    }

    pub async fn send_vm_data(&self, vm_id: u64, data: Vec<u8>, fee_per_byte: u64) -> BroadcastResponse {
        let response = self.make_sure_public_key_is_set(fee_per_byte).await;
        if response.as_ref().map_or(false, |r| !r.success) {
            return response.unwrap();
        }

        let tx = NewTransactionData::FalconVmData {
            fee_per_byte: fee_per_byte,
            vm_id: vm_id,
            data: data
        };
        let txn_bytes = self.get_txn_bytes(tx).await;

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

    async fn get_rpc(&self) -> RPC {
        RPC::new(NODE_URL).await.unwrap()
    }

    fn hash224(input: &[u8]) -> Vec<u8> {
        let mut hasher = Keccak224::new();
        hasher.update(input);
        hasher.finalize().to_vec()
    }
}
