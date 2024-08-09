use serde::{Deserialize, Serialize};

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
