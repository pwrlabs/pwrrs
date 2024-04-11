use std::net::IpAddr;

use crate::wallet::Wallet;
use serde::{Deserialize, Serialize};

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
    pub transactions: Vec<Transaction>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Transaction {
    #[serde(default)]
    pub size: u32,

    pub position_in_the_block: u32,

    #[serde(default)]
    pub fee: u64,

    #[serde(default)]
    pub extrafee: u64,

    #[serde(default = "default_hex")]
    pub sender: String,

    #[serde(default = "default_hex")]
    pub receiver: String,

    #[serde(default)]
    pub nonce: u32,

    #[serde(default = "default_hex")]
    pub hash: String,

    #[serde(default)]
    pub block_number: u32,

    #[serde(default)]
    pub timestamp: u64,

    #[serde(default)]
    pub value: u64,

    #[serde(default, with = "hex_serde")]
    pub raw_transaction: Vec<u8>,

    #[serde(default)]
    pub chain_id: u8,

    #[serde(default)]
    pub success: bool,

    #[serde(default)]
    pub error_message: String,

    #[serde(flatten)]
    pub concrete_transaction: ConcreteTransaction,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ConcreteTransaction {
    Transfer,

    #[serde(rename = "VM Data")]
    VmData {
        #[serde(rename = "vmId")]
        vm_id: u64,
        #[serde(with = "hex_serde", default)]
        data: Vec<u8>,
    },

    Delegate {
        #[serde(default = "default_hex")]
        validator: String,
    },

    Withdraw {
        #[serde(default = "default_hex")]
        validator: String,

        #[serde(default)]
        shares: u64,
    },
    Join {
        #[serde(default = "default_hex", rename = "sender")]
        validator: String,

        #[serde(default)]
        ip: String,
    },
    ClainVmId {
        #[serde(rename = "vmId", default)]
        vm_id: u64,
    },
    SetGuardian {
        #[serde(default = "default_hex")]
        guardian: String,

        #[serde(rename = "guardianExpiryDate", default)]
        guardian_expiry_date: u64,
    },

    #[serde(rename = "Payable VM Data")]
    PayableVmData {
        #[serde(rename = "vmId", default)]
        vm_id: u64,
        #[serde(with = "hex_serde", default)]
        data: Vec<u8>,
    },

    #[serde(rename = "Guardian Approval")]
    GuardianApproval {
        #[serde(default)]
        transactions: Vec<Transaction>,
    },

    #[serde(rename = "Conduit Approval")]
    ConduitApproval {
        #[serde(rename = "vmId", default)]
        vm_id: u64,

        #[serde(default)]
        transactions: Vec<Transaction>,
    },

    #[serde(rename = "Remove Guardian")]
    RemoveGuardian,

    #[serde(rename = "Calim Spot")]
    ClaimSpot {
        #[serde(default = "default_hex", rename = "sender")]
        validator: String,
    },

    #[serde(rename = "Set Conduits")]
    SetConduits {
        #[serde(rename = "vmId", default)]
        vm_id: u64,

        #[serde(default)]
        conduits: Vec<String>,
    },

    #[serde(rename = "Add Conduits")]
    AddConduits {
        #[serde(rename = "vmId", default)]
        vm_id: u64,

        #[serde(default)]
        conduits: Vec<String>,
    },

    #[serde(rename = "Move Stake")]
    MoveStake {
        #[serde(default = "default_hex", rename = "fromValidator")]
        from_validator: String,

        #[serde(default = "default_hex", rename = "toValidator")]
        to_validator: String,

        #[serde(default, rename = "sharesAmount")]
        shares_amount: u64,
    },

    #[serde(rename = "Change Early Withdraw Penalty Proposal")]
    ChangeEarlyWithdrawPenaltyProposal {
        #[serde(default)]
        title: String,

        #[serde(default)]
        description: String,

        #[serde(default, rename = "earlyWithdrawPenalty")]
        withdraw_penalty: u64,

        #[serde(default, rename = "earlyWithdrawTime")]
        withdraw_penalty_time: u64,
    },

    #[serde(rename = "Change Fee Per Byte Proposal")]
    ChangeFeePerByteProposal {
        #[serde(default)]
        title: String,

        #[serde(default)]
        description: String,

        #[serde(default = "default_fee_per_byte", rename = "feePerByte")]
        fee_per_byte: u64,
    },

    #[serde(rename = "Change Max Block Size Proposal")]
    ChangeMaxBlockSizeProposal {
        #[serde(default)]
        title: String,

        #[serde(default)]
        description: String,

        #[serde(default = "default_max_block_size", rename = "maxBlockSize")]
        max_block_size: u32,
    },

    #[serde(rename = "Change Max Txn Size Proposal")]
    ChangeMaxTxnSizeProposal {
        #[serde(default)]
        title: String,

        #[serde(default)]
        description: String,

        #[serde(default = "default_max_txn_size", rename = "maxTxnSize")]
        max_txn_size: u32,
    },

    #[serde(rename = "Change Overall Burn Percentage Proposal")]
    ChangeOverallBurnPercentageProposal {
        #[serde(default)]
        title: String,

        #[serde(default)]
        description: String,

        #[serde(
            default = "default_overall_burn_percentage",
            rename = "overallBurnPercentage"
        )]
        burn_percentage: u32,
    },

    #[serde(rename = "Change Reward Per Year Proposal")]
    ChangeRewardPerYearProposal {
        #[serde(default)]
        title: String,

        #[serde(default)]
        description: String,

        #[serde(default = "default_reward_per_year", rename = "rewardPerYear")]
        reward_per_year: u64,
    },

    #[serde(rename = "Change Validator Count Limit Proposal")]
    ChangeValidatorCountLimitProposal {
        #[serde(default)]
        title: String,

        #[serde(default)]
        description: String,

        #[serde(
            default = "default_validator_count_limit",
            rename = "validatorCountLimit"
        )]
        validator_count_limit: u32,
    },

    #[serde(rename = "Change Validator Joining Fee Proposal")]
    ChangeValidatorJoiningFeeProposal {
        #[serde(default)]
        title: String,

        #[serde(default)]
        description: String,

        #[serde(default = "default_joining_fee", rename = "validatorJoiningFee")]
        joining_fee: u64,
    },

    #[serde(rename = "Change Vm Id Claiming Fee Proposal")]
    ChangeVmIdClaimingFeeProposal {
        #[serde(default)]
        title: String,

        #[serde(default)]
        description: String,

        #[serde(default = "default_claiming_fee", rename = "vmIdClaimingFee")]
        claiming_fee: u64,
    },

    #[serde(rename = "Change VM Owner Txn Fee Share Proposal")]
    ChangeVmOwnerTxnFeeShareProposal {
        #[serde(default)]
        title: String,

        #[serde(default)]
        description: String,

        #[serde(default = "default_fee_share", rename = "vmOwnerTxnFeeShare")]
        fee_share: u64,
    },

    #[serde(rename = "Other Proposal")]
    OtherProposalTxn {
        #[serde(default)]
        title: String,

        #[serde(default)]
        description: String,
    },

    #[serde(rename = "Vote On Proposal")]
    VoteOnProposalTxn {
        #[serde(default = "default_hex", rename = "proposalHash")]
        proposal_hash: String,

        #[serde(default = "default_proposal_status", rename = "proposalStatus")]
        proposal_status: String,

        #[serde(default)]
        vote: String,
    },
}

fn default_hex() -> String {
    "0x".to_string()
}

fn default_fee_per_byte() -> u64 {
    8
}

fn default_max_block_size() -> u32 {
    4
}

fn default_max_txn_size() -> u32 {
    8
}

fn default_overall_burn_percentage() -> u32 {
    4
}

fn default_reward_per_year() -> u64 {
    8
}

fn default_validator_count_limit() -> u32 {
    4
}

fn default_joining_fee() -> u64 {
    8
}

fn default_claiming_fee() -> u64 {
    8
}

fn default_fee_share() -> u64 {
    8
}

fn default_proposal_status() -> String {
    "ongoing".to_string()
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
    #[serde(default)]
    pub bad_actor: bool,
    pub voting_power: u64,
    pub total_shares: u64,
    pub delegators_count: u32,
    #[serde(default)]
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
        chain_id: u8,
        wallet: &Wallet,
    ) -> Result<Vec<u8>, &'static str> {
        let mut bytes = Vec::new();
        bytes.push(self.identifier());
        bytes.extend(chain_id.to_be_bytes());
        bytes.extend(nonce.to_be_bytes());
        bytes.extend(self.transaction_bytes()?);

        let signature = wallet.sign(&bytes).map_err(|_| "Failed to sign message")?;
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

mod hex_serde {
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(data: &Vec<u8>, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let st = hex::encode(data);
        s.serialize_str(&st)
    }

    pub fn deserialize<'de, D>(d: D) -> Result<Vec<u8>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let hex_str = String::deserialize(d)?;
        if hex_str.len() > 2 && (&hex_str[..2] == "0x" || &hex_str[..2] == "0X") {
            hex::decode(&hex_str[2..])
                .map_err(|e| serde::de::Error::custom(format!("Expected hex string: {e}")))
        } else {
            hex::decode(hex_str)
                .map_err(|e| serde::de::Error::custom(format!("Expected hex string: {e}")))
        }
    }
}
