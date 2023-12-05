use std::net::IpAddr;

use serde::{Deserialize, Serialize};

use crate::wallet::PrivateKey;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Block {
    pub transaction_count: u32,
    pub block_size: u32,
    pub block_number: u32,
    pub block_reward: u64,
    pub timestamp: u64,
    pub block_hash: String,
    pub block_submitter: String,
    pub success: bool,
    pub transactions: Vec<Transaction>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Transaction {
    pub size: u32,
    pub position_in_the_block: u32,
    #[serde(rename = "txnFee")]
    pub fee: u64,
    pub from: String,
    pub to: String,
    pub nonce_or_validation_hash: String,
    pub hash: String,

    #[serde(flatten)]
    pub data: TransactionData,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum TransactionData {
    Transfer {
        value: u64,
    },

    #[serde(rename = "VM Data")]
    VmData {
        #[serde(rename = "vmId")]
        vm_id: u64,
        data: String,
    },

    Delegate {
        value: u64,
    },

    Withdraw {
        shares: u64,
    },
    Join,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Delegator {
    pub address: String,
    pub validator_address: String,
    pub shares: u64,
    pub delegated_pwr: u64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Validator {
    pub address: String,
    pub ip: IpAddr,
    pub bad_actor: bool,
    pub voting_power: u64,
    pub total_shares: u64,
    pub delegators_count: u32,
    pub is_active: bool,
}

pub enum NewTransactionData {
    Transfer {
        amount: u64,
        /// 20 bytes address without `0x`
        recipient: String,
    },

    VmData {
        vm_id: u64,
        /// 65_000 bytes limit on data
        data: Vec<u8>,
    },

    Delegate {
        amount: u64,
        /// 20 bytes address without `0x`
        validator: String,
    },

    Whithdaw {
        shares: u64,
        /// 20 bytes address without `0x`
        validator: String,
    },

    ClainVmID {
        vm_id: u64,
    },
}

impl NewTransactionData {
    pub fn serialize_for_broadcast(
        &self,
        nonce: u32,
        private_key: &PrivateKey,
    ) -> Result<Vec<u8>, &'static str> {
        let mut bytes = Vec::new();
        bytes.push(self.identifier());
        bytes.extend(nonce.to_be_bytes());
        bytes.extend(self.transaction_bytes()?);

        let signature = private_key
            .sign_message(&bytes)
            .map_err(|_| "Failed to sign message")?
            .1;
        bytes.extend(signature);
        Ok(bytes)
    }

    fn transaction_bytes(&self) -> Result<Vec<u8>, &'static str> {
        let mut bytes = Vec::new();

        match self {
            NewTransactionData::Transfer { amount, recipient } => {
                bytes.extend(amount.to_be_bytes());
                bytes.extend(hex::decode(recipient).map_err(|_| "Invalid recipient address")?);
            }

            NewTransactionData::VmData { vm_id, data } => {
                bytes.extend(vm_id.to_be_bytes());
                bytes.extend(data);
            }
            NewTransactionData::Delegate { amount, validator } => {
                bytes.extend(amount.to_be_bytes());
                bytes.extend(hex::decode(validator).map_err(|_| "Invalid validator address")?);
            }
            NewTransactionData::Whithdaw { shares, validator } => {
                bytes.extend(shares.to_be_bytes());
                bytes.extend(hex::decode(validator).map_err(|_| "Invalid validator address")?);
            }
            NewTransactionData::ClainVmID { vm_id } => bytes.extend(vm_id.to_be_bytes()),
        }

        Ok(bytes)
    }

    fn identifier(&self) -> u8 {
        match self {
            NewTransactionData::Transfer { .. } => 0,
            NewTransactionData::Delegate { .. } => 3,
            NewTransactionData::Whithdaw { .. } => 4,
            NewTransactionData::VmData { .. } => 5,
            NewTransactionData::ClainVmID { .. } => 6,
        }
    }
}
