use serde::{Deserialize, Serialize};
use crate::transaction::hex_serde::hex_serde;
use crate::transaction::stream::{
    default_hex,
    default_fee_per_byte,
    default_max_block_size,
    default_max_txn_size,
    default_overall_burn_percentage,
    default_reward_per_year,
    default_validator_count_limit,
    default_joining_fee,
    default_claiming_fee,
    default_fee_share,
    default_proposal_status
};

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
