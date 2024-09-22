pub mod types;
pub mod hex_serde;
pub mod stream;

use self::types::Transaction;
pub use self::types::NewTransactionData;
use crate::Wallet;

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
            NewTransactionData::Join { ip } => {
                bytes.extend(ip.clone().into_bytes());
            }
            NewTransactionData::ClaimSpot { validator } => {
                bytes.extend(hex::decode(validator).map_err(|_| "Invalid validator address")?);
            }
            NewTransactionData::Delegate { amount, validator } => {
                bytes.extend(amount.to_be_bytes());
                bytes.extend(hex::decode(validator).map_err(|_| "Invalid validator address")?);
            }
            NewTransactionData::Withdraw { shares, validator } => {
                bytes.extend(shares.to_be_bytes());
                bytes.extend(hex::decode(validator).map_err(|_| "Invalid validator address")?);
            }
            NewTransactionData::VmData { vm_id, data } => {
                bytes.extend(vm_id.to_be_bytes());
                bytes.extend(data);
            }
            NewTransactionData::ClaimVmID { vm_id } => bytes.extend(vm_id.to_be_bytes()),
            NewTransactionData::SetGuardian { guardian_expiry_date, guardian } => {
                bytes.extend(guardian_expiry_date.to_be_bytes());
                bytes.extend(hex::decode(guardian).map_err(|_| "Invalid guardian address")?);
            }
            NewTransactionData::GuardianApproval { transactions } => {
                for transaction in transactions {
                    bytes.extend(self.to_bytes(&transaction).map_err(|_| "Invalid transaction data")?);
                }
            }
            NewTransactionData::PayableVmData { vm_id, data, amount } => {
                bytes.extend(vm_id.to_be_bytes());
                bytes.extend(data);
                bytes.extend(amount.to_be_bytes());
            }
            NewTransactionData::RemoveGuardian => {},
            NewTransactionData::SetConduits { vm_id, conduits } => {
                bytes.extend(vm_id.to_be_bytes());
                for conduit in conduits {
                    bytes.extend(hex::decode(conduit).map_err(|_| "Invalid conduit hex string")?);
                }
            }
            NewTransactionData::AddConduits { vm_id, conduits } => {
                bytes.extend(vm_id.to_be_bytes());
                for conduit in conduits {
                    bytes.extend(hex::decode(conduit).map_err(|_| "Invalid conduit hex string")?);
                }
            }
            NewTransactionData::MoveStake { shares_amount, from_validator, to_validator } => {
                bytes.extend(shares_amount.to_be_bytes());
                bytes.extend(hex::decode(from_validator).map_err(|_| "Invalid address")?);
                bytes.extend(hex::decode(to_validator).map_err(|_| "Invalid address")?);
            }
            // NewTransactionData::ChangeEarlyWithdrawPenaltyProposal { title, withdraw_penalty_time, withdraw_penalty, description } => {
            //     bytes.extend(title.clone().into_bytes());
            //     bytes.extend(withdraw_penalty_time.to_be_bytes());
            //     bytes.extend(withdraw_penalty.to_be_bytes());
            //     bytes.extend(description.clone().into_bytes());
            // }
            // NewTransactionData::ChangeFeePerByteProposal { title, description, fee_per_byte } => {
            //     bytes.extend(title.clone().into_bytes());
            //     bytes.extend(description.clone().into_bytes());
            //     bytes.extend(fee_per_byte.to_be_bytes());
            // }
            // NewTransactionData::ChangeMaxBlockSizeProposal { title, description, max_block_size } => {
            //     bytes.extend(title.clone().into_bytes());
            //     bytes.extend(description.clone().into_bytes());
            //     bytes.extend(max_block_size.to_be_bytes());
            // }
            // NewTransactionData::ChangeMaxTxnSizeProposal { title, description, max_txn_size } => {
            //     bytes.extend(title.clone().into_bytes());
            //     bytes.extend(description.clone().into_bytes());
            //     bytes.extend(max_txn_size.to_be_bytes());
            // }
            // NewTransactionData::ChangeOverallBurnPercentageProposal { title, description, burn_percentage } => {
            //     bytes.extend(title.clone().into_bytes());
            //     bytes.extend(description.clone().into_bytes());
            //     bytes.extend(burn_percentage.to_be_bytes());
            // }
            // NewTransactionData::ChangeRewardPerYearProposal { title, description, reward_per_year } => {
            //     bytes.extend(title.clone().into_bytes());
            //     bytes.extend(description.clone().into_bytes());
            //     bytes.extend(reward_per_year.to_be_bytes());
            // }
            // NewTransactionData::ChangeValidatorCountLimitProposal { title, description, validator_count_limit } => {
            //     bytes.extend(title.clone().into_bytes());
            //     bytes.extend(description.clone().into_bytes());
            //     bytes.extend(validator_count_limit.to_be_bytes());
            // }
            // NewTransactionData::ChangeValidatorJoiningFeeProposal { title, description, joining_fee } => {
            //     bytes.extend(title.clone().into_bytes());
            //     bytes.extend(description.clone().into_bytes());
            //     bytes.extend(joining_fee.to_be_bytes());
            // }
            // NewTransactionData::ChangeVmIdClaimingFeeProposal { title, description, claiming_fee } => {
            //     bytes.extend(title.clone().into_bytes());
            //     bytes.extend(description.clone().into_bytes());
            //     bytes.extend(claiming_fee.to_be_bytes());
            // }
            // NewTransactionData::ChangeVmOwnerTxnFeeShareProposal { title, description, fee_share } => {
            //     bytes.extend(title.clone().into_bytes());
            //     bytes.extend(description.clone().into_bytes());
            //     bytes.extend(fee_share.to_be_bytes());
            // }
            // NewTransactionData::OtherProposalTxn { title, description } => {
            //     bytes.extend(title.clone().into_bytes());
            //     bytes.extend(description.clone().into_bytes());
            // }
            // NewTransactionData::VoteOnProposalTxn { proposal_hash, vote } => {
            //     bytes.extend(proposal_hash.clone().into_bytes());
            //     bytes.extend(vote.to_be_bytes());
            // }
        }

        Ok(bytes)
    }

    fn identifier(&self) -> u8 {
        match self {
            NewTransactionData::Transfer         { .. } => 0,
            NewTransactionData::Join             { .. } => 1,
            NewTransactionData::ClaimSpot        { .. } => 2,
            NewTransactionData::Delegate         { .. } => 3,
            NewTransactionData::Withdraw         { .. } => 4,
            NewTransactionData::VmData           { .. } => 5,
            NewTransactionData::ClaimVmID        { .. } => 6,
            NewTransactionData::SetGuardian      { .. } => 8,
            NewTransactionData::RemoveGuardian          => 9,
            NewTransactionData::GuardianApproval { .. } => 10,
            NewTransactionData::PayableVmData    { .. } => 11,
            NewTransactionData::SetConduits      { .. } => 13,
            NewTransactionData::AddConduits      { .. } => 14,
            NewTransactionData::MoveStake        { .. } => 16,
            //
            // NewTransactionData::ChangeEarlyWithdrawPenaltyProposal  { .. } => 17,
            // NewTransactionData::ChangeFeePerByteProposal            { .. } => 18,
            // NewTransactionData::ChangeMaxBlockSizeProposal          { .. } => 19,
            // NewTransactionData::ChangeMaxTxnSizeProposal            { .. } => 20,
            // NewTransactionData::ChangeOverallBurnPercentageProposal { .. } => 21,
            // NewTransactionData::ChangeRewardPerYearProposal         { .. } => 22,
            // NewTransactionData::ChangeValidatorCountLimitProposal   { .. } => 23,
            // NewTransactionData::ChangeValidatorJoiningFeeProposal   { .. } => 24,
            // NewTransactionData::ChangeVmIdClaimingFeeProposal       { .. } => 25,
            // NewTransactionData::ChangeVmOwnerTxnFeeShareProposal    { .. } => 26,
            // NewTransactionData::OtherProposalTxn                    { .. } => 27,
            // NewTransactionData::VoteOnProposalTxn                   { .. } => 28,
        }
    }

    fn to_bytes(&self, transactions: &Transaction) -> Result<Vec<u8>, &'static str> {
        let mut bytes = Vec::new();

        bytes.extend(transactions.size.to_be_bytes());
        bytes.extend(transactions.position_in_the_block.to_be_bytes());
        bytes.extend(transactions.fee.to_be_bytes());
        bytes.extend(transactions.extrafee.to_be_bytes());
        bytes.extend(transactions.nonce.to_be_bytes());
        bytes.extend(transactions.block_number.to_be_bytes());
        bytes.extend(transactions.timestamp.to_be_bytes());
        bytes.extend(transactions.value.to_be_bytes());
        bytes.extend(transactions.chain_id.to_be_bytes());
        bytes.extend(hex::decode(&transactions.sender).map_err(|_| "Invalid sender address")?);
        bytes.extend(hex::decode(&transactions.receiver).map_err(|_| "Invalid receiver address")?);
        bytes.extend(hex::decode(&transactions.hash).map_err(|_| "Invalid transaction hash")?);
        bytes.extend(&transactions.raw_transaction);
        bytes.push(transactions.success as u8);

        if !transactions.error_message.is_empty() {
            bytes.extend(transactions.error_message.as_bytes());
        }
        Ok(bytes)
    }
}
