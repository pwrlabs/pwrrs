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

    #[serde(default)]
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
    pub identifier: u32,

    #[serde(default = "default_hex")]
    pub transaction_hash: String,

    #[serde(flatten)]
    pub concrete_transaction: ConcreteTransaction,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ConcreteTransaction {
    //  transaction types
    #[serde(rename = " Set Public Key")]
    SetPublicKey {
        #[serde(with = "hex_serde", default)]
        public_key: Vec<u8>,
    },
    
    #[serde(rename = " Join As Validator")]
    JoinAsValidator {
        #[serde(default)]
        ip: String,
    },
    
    #[serde(rename = " Delegate")]
    Delegate {
        #[serde(default = "default_hex")]
        validator: String,
        
        #[serde(default, rename = "pwrAmount")]
        pwr_amount: u64,
    },
    
    #[serde(rename = " Change IP")]
    ChangeIp {
        #[serde(default, rename = "newIp")]
        new_ip: String,
    },
    
    #[serde(rename = " Claim Active Node Spot")]
    ClaimActiveNodeSpot {},
    
    #[serde(rename = " Transfer")]
    Transfer {
        #[serde(default = "default_hex")]
        receiver: String,
        
        #[serde(default)]
        amount: u64,
    },

    //  Governance Proposal Transactions
    #[serde(rename = " Change Early Withdraw Penalty Proposal")]
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

    #[serde(rename = " Change Fee Per Byte Proposal")]
    ChangeFeePerByteProposal {
        #[serde(default)]
        title: String,
        
        #[serde(default)]
        description: String,
        
        #[serde(default = "default_fee_per_byte", rename = "newFeePerByte")]
        new_fee_per_byte: u64,
    },

    #[serde(rename = " Change Max Block Size Proposal")]
    ChangeMaxBlockSizeProposal {
        #[serde(default)]
        title: String,
        
        #[serde(default)]
        description: String,
        
        #[serde(default = "default_max_block_size", rename = "maxBlockSize")]
        max_block_size: u32,
    },

    #[serde(rename = " Change Max Txn Size Proposal")]
    ChangeMaxTxnSizeProposal {
        #[serde(default)]
        title: String,
        
        #[serde(default)]
        description: String,
        
        #[serde(default = "default_max_txn_size", rename = "maxTxnSize")]
        max_txn_size: u32,
    },

    #[serde(rename = " Change Overall Burn Percentage Proposal")]
    ChangeOverallBurnPercentageProposal {
        #[serde(default)]
        title: String,
        
        #[serde(default)]
        description: String,
        
        #[serde(default = "default_overall_burn_percentage", rename = "overallBurnPercentage")]
        burn_percentage: u32,
    },

    #[serde(rename = " Change Reward Per Year Proposal")]
    ChangeRewardPerYearProposal {
        #[serde(default)]
        title: String,
        
        #[serde(default)]
        description: String,
        
        #[serde(default = "default_reward_per_year", rename = "rewardPerYear")]
        reward_per_year: u64,
    },

    #[serde(rename = " Change Validator Count Limit Proposal")]
    ChangeValidatorCountLimitProposal {
        #[serde(default)]
        title: String,
        
        #[serde(default)]
        description: String,
        
        #[serde(default = "default_validator_count_limit", rename = "validatorCountLimit")]
        validator_count_limit: u32,
    },

    #[serde(rename = " Change Validator Joining Fee Proposal")]
    ChangeValidatorJoiningFeeProposal {
        #[serde(default)]
        title: String,
        
        #[serde(default)]
        description: String,
        
        #[serde(default = "default_joining_fee", rename = "validatorJoiningFee")]
        joining_fee: u64,
    },

    #[serde(rename = " Change Vida Id Claiming Fee Proposal")]
    ChangeVidaIdClaimingFeeProposal {
        #[serde(default)]
        title: String,
        
        #[serde(default)]
        description: String,
        
        #[serde(default = "default_claiming_fee", rename = "vidaIdClaimingFee")]
        claiming_fee: u64,
    },

    #[serde(rename = " Change Vida Owner Txn Fee Share Proposal")]
    ChangeVidaOwnerTxnFeeShareProposal {
        #[serde(default)]
        title: String,
        
        #[serde(default)]
        description: String,
        
        #[serde(default = "default_fee_share", rename = "vidaOwnerTxnFeeShare")]
        fee_share: u32,
    },

    #[serde(rename = " Other Proposal")]
    OtherProposalTxn {
        #[serde(default)]
        title: String,
        
        #[serde(default)]
        description: String,
    },

    #[serde(rename = " Vote On Proposal")]
    VoteOnProposalTxn {
        #[serde(default = "default_hex", rename = "proposalHash")]
        proposal_hash: String,
        
        #[serde(default = "default_vote")]
        vote: u8,
    },

    //  Guardian Transactions
    #[serde(rename = " Guardian Approval")]
    GuardianApproval {
        #[serde(default)]
        transactions: Vec<Transaction>,
    },

    #[serde(rename = " Remove Guardian")]
    RemoveGuardian {},

    #[serde(rename = " Set Guardian")]
    SetGuardian {
        #[serde(default, rename = "guardianExpiryDate")]
        guardian_expiry_date: u64,
        
        #[serde(default = "default_hex")]
        guardian: String,
    },

    //  Staking Transactions
    #[serde(rename = " Move Stake")]
    MoveStake {
        #[serde(default, rename = "sharesAmount")]
        shares_amount: u64,
        
        #[serde(default = "default_hex", rename = "fromValidator")]
        from_validator: String,
        
        #[serde(default = "default_hex", rename = "toValidator")]
        to_validator: String,
    },

    #[serde(rename = " Remove Validator")]
    RemoveValidator {
        #[serde(default = "default_hex")]
        validator: String,
    },

    #[serde(rename = " Withdraw")]
    Withdraw {
        #[serde(default)]
        shares: u64,
        
        #[serde(default = "default_hex")]
        validator: String,
    },

    //  VIDA Transactions
    #[serde(rename = " Claim VIDA Id")]
    ClaimVidaId {
        #[serde(default, rename = "vidaId")]
        vida_id: u64,
    },

    #[serde(rename = " Conduit Approval")]
    ConduitApproval {
        #[serde(default, rename = "vidaId")]
        vida_id: u64,
        
        #[serde(default)]
        transactions: Vec<Transaction>,
    },

    #[serde(rename = " Payable Vida Data")]
    PayableVidaData {
        #[serde(default, rename = "vidaId")]
        vida_id: u64,
        
        #[serde(with = "hex_serde", default)]
        data: Vec<u8>,
        
        #[serde(default)]
        value: u64,
    },

    #[serde(rename = " Remove Conduits")]
    RemoveConduits {
        #[serde(default, rename = "vidaId")]
        vida_id: u64,
        
        #[serde(default)]
        conduits: Vec<String>,
    },

    #[serde(rename = " Set Conduit Mode")]
    SetConduitMode {
        #[serde(default, rename = "vidaId")]
        vida_id: u64,
        
        #[serde(default)]
        mode: u8,
        
        #[serde(default, rename = "conduitThreshold")]
        conduit_threshold: u32,
        
        #[serde(default)]
        conduits: Vec<String>,
        
        #[serde(default, rename = "conduitsWithVotingPower")]
        conduits_with_voting_power: Vec<(String, u64)>,
    },

    #[serde(rename = " Set Vida Private State")]
    SetVidaPrivateState {
        #[serde(default, rename = "vidaId")]
        vida_id: u64,
        
        #[serde(default, rename = "privateState")]
        private_state: bool,
    },

    #[serde(rename = " Set Vida To Absolute Public")]
    SetVidaToAbsolutePublic {
        #[serde(default, rename = "vidaId")]
        vida_id: u64,
    },

    #[serde(rename = " Add Vida Sponsored Addresses")]
    AddVidaSponsoredAddresses {
        #[serde(default, rename = "vidaId")]
        vida_id: u64,
        
        #[serde(default, rename = "sponsoredAddresses")]
        sponsored_addresses: Vec<String>,
    },

    #[serde(rename = " Add Vida Allowed Senders")]
    AddVidaAllowedSenders {
        #[serde(default, rename = "vidaId")]
        vida_id: u64,
        
        #[serde(default, rename = "allowedSenders")]
        allowed_senders: Vec<String>,
    },

    #[serde(rename = " Remove Vida Allowed Senders")]
    RemoveVidaAllowedSenders {
        #[serde(default, rename = "vidaId")]
        vida_id: u64,
        
        #[serde(default, rename = "allowedSenders")]
        allowed_senders: Vec<String>,
    },

    #[serde(rename = " Remove Sponsored Addresses")]
    RemoveSponsoredAddresses {
        #[serde(default, rename = "vidaId")]
        vida_id: u64,
        
        #[serde(default, rename = "sponsoredAddresses")]
        sponsored_addresses: Vec<String>,
    },

    #[serde(rename = " Set Pwr Transfer Rights")]
    SetPwrTransferRights {
        #[serde(default, rename = "vidaId")]
        vida_id: u64,
        
        #[serde(default, rename = "ownerCanTransferPwr")]
        owner_can_transfer_pwr: bool,
    },

    #[serde(rename = " Transfer Pwr From Vida")]
    TransferPwrFromVida {
        #[serde(default, rename = "vidaId")]
        vida_id: u64,
        
        #[serde(default = "default_hex")]
        receiver: String,
        
        #[serde(default)]
        amount: u64,
    },
}

pub enum NewTransactionData {
    //  transaction types
    SetPublicKey {
        public_key: String,
    },
    
    JoinAsValidator {
        ip: String,
    },
    
    Delegate {
        validator: String,
        pwr_amount: u64,
    },
    
    ChangeIp {
        new_ip: String,
    },
    
    ClaimActiveNodeSpot {},
    
    Transfer {
        receiver: String,
        amount: u64,
    },

    //  Governance Proposal Transactions
    ChangeEarlyWithdrawPenaltyProposal {
        title: String,
        description: String,
        withdraw_penalty_time: u64,
        withdraw_penalty: u32,
    },

    ChangeFeePerByteProposal {
        title: String,
        description: String,
        new_fee_per_byte: u64,
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

    ChangeVidaIdClaimingFeeProposal {
        title: String,
        description: String,
        claiming_fee: u64,
    },

    ChangeVidaOwnerTxnFeeShareProposal {
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

    //  Guardian Transactions
    GuardianApproval {
        transactions: Vec<Transaction>,
    },

    RemoveGuardian {},

    SetGuardian {
        guardian_expiry_date: u64,
        guardian: String,
    },

    //  Staking Transactions
    MoveStake {
        shares_amount: u64,
        from_validator: String,
        to_validator: String,
    },

    RemoveValidator {
        validator: String,
    },

    Withdraw {
        shares: u64,
        validator: String,
    },

    //  VIDA Transactions
    ClaimVidaId {
        vida_id: u64,
    },

    ConduitApproval {
        vida_id: u64,
        transactions: Vec<Transaction>,
    },

    PayableVidaData {
        vida_id: u64,
        data: Vec<u8>,
        value: u64,
    },

    RemoveConduits {
        vida_id: u64,
        conduits: Vec<String>,
    },

    SetConduitMode {
        vida_id: u64,
        mode: u8,
        conduit_threshold: u32,
        conduits: Vec<String>,
        conduits_with_voting_power: Vec<(String, u64)>,
    },

    SetVidaPrivateState {
        vida_id: u64,
        private_state: bool,
    },

    SetVidaToAbsolutePublic {
        vida_id: u64,
    },

    AddVidaSponsoredAddresses {
        vida_id: u64,
        sponsored_addresses: Vec<String>,
    },

    AddVidaAllowedSenders {
        vida_id: u64,
        allowed_senders: Vec<String>,
    },

    RemoveVidaAllowedSenders {
        vida_id: u64,
        allowed_senders: Vec<String>,
    },

    RemoveSponsoredAddresses {
        vida_id: u64,
        sponsored_addresses: Vec<String>,
    },

    SetPwrTransferRights {
        vida_id: u64,
        owner_can_transfer_pwr: bool,
    },

    TransferPwrFromVida {
        vida_id: u64,
        receiver: String,
        amount: u64,
    },
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VidaDataTransaction {
    #[serde(default)]
    pub size: u32,

    #[serde(default)]
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
    pub vida_id: u64,

    #[serde(default, with = "hex_serde")]
    pub data: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionResponse {
    #[serde(default)]
    pub identifier: u32,

    #[serde(default)]
    pub paid_total_fee: u64,

    #[serde(default)]
    pub amount: u64,

    #[serde(default)]
    pub paid_action_fee: u64,

    #[serde(default)]
    pub nonce: u32,

    #[serde(default = "default_hex")]
    pub transaction_hash: String,

    #[serde(default)]
    pub time_stamp: u64,
    
    #[serde(default)]
    pub fee_per_byte: u64,

    #[serde(default)]
    pub size: u32,

    #[serde(default = "default_hex")]
    pub sender: String,

    #[serde(default)]
    pub success: bool,

    #[serde(default)]
    pub block_number: u32,

    #[serde(default)]
    pub position_in_the_block: u32,

    #[serde(default)]
    pub vida_id: u64,

    #[serde(default = "default_hex")]
    pub receiver: String,

    #[serde(default, with = "hex_serde")]
    pub data: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Penalty {
    pub withdraw_time: u64,
    pub penalty: String
}