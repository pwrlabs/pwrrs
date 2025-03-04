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
    default_vote,
    // default_proposal_status
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
    JoinAsValidator {
        #[serde(default)]
        ip: String,
    },
    ClaimVmID {
        #[serde(rename = "vmId", default)]
        vm_id: u64,
    },
    SetGuardian {
        #[serde(rename = "guardianExpiryDate", default)]
        guardian_expiry_date: u64,

        #[serde(default = "default_hex")]
        guardian: String,
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
        conduits: Vec<u8>,
    },

    #[serde(rename = "Move Stake")]
    MoveStake {
        #[serde(default, rename = "sharesAmount")]
        shares_amount: u64,

        #[serde(default = "default_hex", rename = "fromValidator")]
        from_validator: String,

        #[serde(default = "default_hex", rename = "toValidator")]
        to_validator: String,
    },

    #[serde(rename = "Change Early Withdraw Penalty Proposal")]
    ChangeEarlyWithdrawPenaltyProposal {
        #[serde(default)]
        title: String,

        #[serde(default)]
        description: String,

        #[serde(default, rename = "earlyWithdrawPenalty")]
        withdraw_penalty: u32,

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
        fee_share: u32,
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

        #[serde(default = "default_vote")]
        vote: u8,
    },
    
    // Falcon transaction types
    #[serde(rename = "Falcon Set Public Key")]
    FalconSetPublicKey {
        #[serde(default)]
        fee_per_byte: u64,
        
        #[serde(with = "hex_serde", default)]
        public_key: Vec<u8>,
    },
    
    #[serde(rename = "Falcon Join As Validator")]
    FalconJoinAsValidator {
        #[serde(default)]
        fee_per_byte: u64,
        
        #[serde(default)]
        ip: String,
    },
    
    #[serde(rename = "Falcon Delegate")]
    FalconDelegate {
        #[serde(default)]
        fee_per_byte: u64,
        
        #[serde(default = "default_hex")]
        validator: String,
        
        #[serde(default, rename = "pwrAmount")]
        pwr_amount: u64,
    },
    
    #[serde(rename = "Falcon Change IP")]
    FalconChangeIp {
        #[serde(default)]
        fee_per_byte: u64,
        
        #[serde(default, rename = "newIp")]
        new_ip: String,
    },
    
    #[serde(rename = "Falcon Claim Active Node Spot")]
    FalconClaimActiveNodeSpot {
        #[serde(default)]
        fee_per_byte: u64,
    },
    
    #[serde(rename = "Falcon Transfer")]
    FalconTransfer {
        #[serde(default)]
        fee_per_byte: u64,
        
        #[serde(default = "default_hex")]
        receiver: String,
        
        #[serde(default)]
        amount: u64,
    },
    
    #[serde(rename = "Falcon VM Data")]
    FalconVmData {
        #[serde(default)]
        fee_per_byte: u64,
        
        #[serde(rename = "vmId", default)]
        vm_id: u64,
        
        #[serde(with = "hex_serde", default)]
        data: Vec<u8>,
    },
}

pub enum NewTransactionData {
    Transfer {
        amount: u64,
        recipient: String,
    },

    VmData {
        vm_id: u64,
        /// 65_000 bytes limit on data
        data: Vec<u8>,
    },

    PayableVmData {
        vm_id: u64,
        data: Vec<u8>,
        amount: u64,
    },

    Delegate {
        amount: u64,
        validator: String,
    },

    Withdraw {
        shares: u64,
        validator: String,
    },

    ClaimVmID {
        vm_id: u64,
    },

    JoinAsValidator {
        ip: String,
    },

    ClaimSpot {
        validator: String,
    },

    SetGuardian {
        guardian_expiry_date: u64,
        guardian: String,
    },

    RemoveGuardian,

    GuardianApproval {
        transactions: Vec<Transaction>,
    },

    MoveStake {
        shares_amount: u64,
        from_validator: String,
        to_validator: String,
    },

    SetConduits {
        vm_id: u64,
        conduits: Vec<String>,
    },

    AddConduits {
        vm_id: u64,
        conduits: Vec<u8>,
    },

    ChangeEarlyWithdrawPenaltyProposal {
        title: String,
        description: String,
        withdraw_penalty: u32,
        withdraw_penalty_time: u64,
    },

    ChangeFeePerByteProposal {
        title: String,
        description: String,
        fee_per_byte: u64,
    },

    ChangeMaxBlockSizeProposal {
        title: String,
        description: String,
        max_block_size: u32,
    },

    ChangeMaxTxnSizeProposal {
        title: String,
        description: String,
        max_txn_size: u32,
    },

    ChangeOverallBurnPercentageProposal {
        title: String,
        description: String,
        burn_percentage: u32,
    },

    ChangeRewardPerYearProposal {
        title: String,
        description: String,
        reward_per_year: u64,
    },

    ChangeValidatorCountLimitProposal {
        title: String,
        description: String,
        validator_count_limit: u32,
    },

    ChangeValidatorJoiningFeeProposal {
        title: String,
        description: String,
        joining_fee: u64,
    },

    ChangeVmIdClaimingFeeProposal {
        title: String,
        description: String,
        claiming_fee: u64,
    },

    ChangeVmOwnerTxnFeeShareProposal {
        title: String,
        description: String,
        fee_share: u32,
    },

    OtherProposalTxn {
        title: String,
        description: String,
    },

    VoteOnProposalTxn {
        proposal_hash: String,
        vote: u8,
    },
    
    // Falcon transaction types
    FalconSetPublicKey {
        fee_per_byte: u64,
        public_key: String,
    },
    
    FalconJoinAsValidator {
        fee_per_byte: u64,
        ip: String,
    },
    
    FalconDelegate {
        fee_per_byte: u64,
        validator: String,
        pwr_amount: u64,
    },
    
    FalconChangeIp {
        fee_per_byte: u64,
        new_ip: String,
    },
    
    FalconClaimActiveNodeSpot {
        fee_per_byte: u64,
    },
    
    FalconTransfer {
        fee_per_byte: u64,
        receiver: String,
        amount: u64,
    },
    
    FalconVmData {
        fee_per_byte: u64,
        vm_id: u64,
        data: Vec<u8>,
    },
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VMDataTransaction {
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

    #[serde(default)]
    pub vm_id: u64,

    #[serde(default, with = "hex_serde")]
    pub data: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Penalty {
    pub withdraw_time: u64,
    pub penalty: String
}