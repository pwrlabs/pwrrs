pub mod types;
pub mod hex_serde;
pub mod stream;

use self::types::Transaction;
pub use self::types::{NewTransactionData, VidaDataTransaction};
use crate::wallet::types::Wallet;
use crate::config::falcon::Falcon;
use pqcrypto_falcon::falcon512;
use pqcrypto_traits::sign::*;
use sha3::{Digest, Keccak256};

impl NewTransactionData {
    pub fn serialize_for_broadcast(
        &self,
        nonce: u32,
        chain_id: u8,
        fee_per_byte: u64,
        wallet: &Wallet,
    ) -> Result<Vec<u8>, &'static str> {
        let mut bytes = Vec::new();
        bytes.extend(self.identifier().to_be_bytes());
        bytes.extend(chain_id.to_be_bytes());
        bytes.extend(nonce.to_be_bytes());

        bytes.extend(fee_per_byte.to_be_bytes());
        bytes.extend(wallet.address.clone());
        bytes.extend(self.transaction_bytes()?);

        // Hash the transaction
        let mut hasher = Keccak256::new();
        hasher.update(&bytes);
        let txn_hash = hasher.finalize();

        // Sign the hash
        let private_key = falcon512::SecretKey::from_bytes(&wallet.private_key).map_err(|_| "Invalid private key")?;
        let signature = Falcon::sign_512(&txn_hash, &private_key);
        let signature_bytes = signature.as_bytes().to_vec();

        bytes.extend(signature_bytes.clone());
        bytes.extend((signature_bytes.len() as u16).to_be_bytes());
        
        Ok(bytes)
    }

    pub fn transaction_bytes(&self) -> Result<Vec<u8>, &'static str> {
        let mut bytes = Vec::new();

        match self {
            NewTransactionData::SetPublicKey { public_key, .. } => {
                let public_key = hex::decode(&public_key).map_err(|_| "Invalid public key")?;

                bytes.extend((public_key.len() as u16).to_be_bytes());
                bytes.extend(public_key);
            },
            NewTransactionData::JoinAsValidator { ip, .. } => {
                let ip_bytes = ip.as_bytes();
                bytes.extend((ip_bytes.len() as u16).to_be_bytes());
                bytes.extend(ip_bytes);
            },
            NewTransactionData::Delegate { validator, pwr_amount, .. } => {
                bytes.extend(hex::decode(&validator[2..]).map_err(|_| "Invalid validator address")?);
                bytes.extend(pwr_amount.to_be_bytes());
            },
            NewTransactionData::ChangeIp { new_ip, .. } => {
                let ip_bytes = new_ip.as_bytes();
                bytes.extend((ip_bytes.len() as u16).to_be_bytes());
                bytes.extend(ip_bytes);
            },
            NewTransactionData::ClaimActiveNodeSpot { .. } => { },
            NewTransactionData::Transfer { receiver, amount, .. } => {
                bytes.extend(hex::decode(&receiver[2..]).map_err(|_| "Invalid receiver address")?);
                bytes.extend(amount.to_be_bytes());
            },
            //  Governance Proposal Transactions
            NewTransactionData::ChangeEarlyWithdrawPenaltyProposal { title, description, withdraw_penalty_time, withdraw_penalty, .. } => {
                let title_bytes = title.as_bytes();
                bytes.extend((title_bytes.len() as u32).to_be_bytes());
                bytes.extend(title_bytes);
                bytes.extend(withdraw_penalty_time.to_be_bytes());
                bytes.extend(withdraw_penalty.to_be_bytes());
                bytes.extend(description.as_bytes());
            },
            NewTransactionData::ChangeFeePerByteProposal { title, description, new_fee_per_byte, .. } => {
                let title_bytes = title.as_bytes();
                bytes.extend((title_bytes.len() as u32).to_be_bytes());
                bytes.extend(title_bytes);
                bytes.extend(new_fee_per_byte.to_be_bytes());
                bytes.extend(description.as_bytes());
            },
            NewTransactionData::ChangeMaxBlockSizeProposal { title, description, max_block_size, .. } => {
                let title_bytes = title.as_bytes();
                bytes.extend((title_bytes.len() as u32).to_be_bytes());
                bytes.extend(title_bytes);
                bytes.extend(max_block_size.to_be_bytes());
                bytes.extend(description.as_bytes());
            },
            NewTransactionData::ChangeMaxTxnSizeProposal { title, description, max_txn_size, .. } => {
                let title_bytes = title.as_bytes();
                bytes.extend((title_bytes.len() as u32).to_be_bytes());
                bytes.extend(title_bytes);
                bytes.extend(max_txn_size.to_be_bytes());
                bytes.extend(description.as_bytes());
            },
            NewTransactionData::ChangeOverallBurnPercentageProposal { title, description, burn_percentage, .. } => {
                let title_bytes = title.as_bytes();
                bytes.extend((title_bytes.len() as u32).to_be_bytes());
                bytes.extend(title_bytes);
                bytes.extend(burn_percentage.to_be_bytes());
                bytes.extend(description.as_bytes());
            },
            NewTransactionData::ChangeRewardPerYearProposal { title, description, reward_per_year, .. } => {
                let title_bytes = title.as_bytes();
                bytes.extend((title_bytes.len() as u32).to_be_bytes());
                bytes.extend(title_bytes);
                bytes.extend(reward_per_year.to_be_bytes());
                bytes.extend(description.as_bytes());
            },
            NewTransactionData::ChangeValidatorCountLimitProposal { title, description, validator_count_limit, .. } => {
                let title_bytes = title.as_bytes();
                bytes.extend((title_bytes.len() as u32).to_be_bytes());
                bytes.extend(title_bytes);
                bytes.extend(validator_count_limit.to_be_bytes());
                bytes.extend(description.as_bytes());
            },
            NewTransactionData::ChangeValidatorJoiningFeeProposal { title, description, joining_fee, .. } => {
                let title_bytes = title.as_bytes();
                bytes.extend((title_bytes.len() as u32).to_be_bytes());
                bytes.extend(title_bytes);
                bytes.extend(joining_fee.to_be_bytes());
                bytes.extend(description.as_bytes());
            },
            NewTransactionData::ChangeVidaIdClaimingFeeProposal { title, description, claiming_fee, .. } => {
                let title_bytes = title.as_bytes();
                bytes.extend((title_bytes.len() as u32).to_be_bytes());
                bytes.extend(title_bytes);
                bytes.extend(claiming_fee.to_be_bytes());
                bytes.extend(description.as_bytes());
            },
            NewTransactionData::ChangeVidaOwnerTxnFeeShareProposal { title, description, fee_share, .. } => {
                let title_bytes = title.as_bytes();
                bytes.extend((title_bytes.len() as u32).to_be_bytes());
                bytes.extend(title_bytes);
                bytes.extend(fee_share.to_be_bytes());
                bytes.extend(description.as_bytes());
            },
            NewTransactionData::OtherProposalTxn { title, description, .. } => {
                let title_length = title.as_bytes().len() as u32;
                bytes.extend(title_length.to_be_bytes());

                bytes.extend(title.as_bytes());
                bytes.extend(description.as_bytes());
            },
            NewTransactionData::VoteOnProposalTxn { proposal_hash, vote, .. } => {
                bytes.extend(hex::decode(&proposal_hash[2..]).map_err(|_| "Invalid proposal hash")?);
                bytes.extend(vote.to_be_bytes());
            },
            //  Guardian Transactions
            NewTransactionData::GuardianApproval { transactions, .. } => {
                let decoded_transactions: Vec<Vec<u8>> = transactions
                    .iter()
                    .map(|transaction| self.to_bytes(&transaction).map_err(|_| "Invalid transaction data"))
                    .collect::<Result<_, _>>()?;

                let lengths_bytes: Vec<u8> = decoded_transactions
                    .iter().map(|tx| (tx.len() as u32).to_be_bytes()).flatten().collect();

                let transactions_bytes: Vec<u8> = decoded_transactions.iter().flatten().cloned().collect();

                bytes.extend(lengths_bytes);
                bytes.extend(transactions_bytes);
            },
            NewTransactionData::RemoveGuardian { .. } => { },
            NewTransactionData::SetGuardian { guardian_expiry_date, guardian, .. } => {
                bytes.extend(guardian_expiry_date.to_be_bytes());
                bytes.extend(hex::decode(&guardian[2..]).map_err(|_| "Invalid guardian address")?);
            },
            //  Staking Transactions
            NewTransactionData::MoveStake { shares_amount, from_validator, to_validator, .. } => {
                bytes.extend(shares_amount.to_be_bytes());
                bytes.extend(hex::decode(&from_validator[2..]).map_err(|_| "Invalid from validator address")?);
                bytes.extend(hex::decode(&to_validator[2..]).map_err(|_| "Invalid to validator address")?);
            },
            NewTransactionData::RemoveValidator { validator, .. } => {
                bytes.extend(hex::decode(&validator[2..]).map_err(|_| "Invalid validator address")?);
            },
            NewTransactionData::Withdraw { shares, validator, .. } => {
                bytes.extend(shares.to_be_bytes());
                bytes.extend(hex::decode(&validator[2..]).map_err(|_| "Invalid validator address")?);
            },
            //  VIDA Transactions
            NewTransactionData::ClaimVidaId { vida_id, .. } => {
                bytes.extend(vida_id.to_be_bytes());
            },
            NewTransactionData::ConduitApproval { vida_id, transactions, .. } => {
                bytes.extend(vida_id.to_be_bytes());
                let decoded_transactions: Vec<Vec<u8>> = transactions
                    .iter()
                    .map(|transaction| self.to_bytes(&transaction).map_err(|_| "Invalid transaction data"))
                    .collect::<Result<_, _>>()?;

                let lengths_bytes: Vec<u8> = decoded_transactions
                    .iter().map(|tx| (tx.len() as u32).to_be_bytes()).flatten().collect();

                let transactions_bytes: Vec<u8> = decoded_transactions.iter().flatten().cloned().collect();

                bytes.extend(lengths_bytes);
                bytes.extend(transactions_bytes);
            },
            NewTransactionData::PayableVidaData { vida_id, data, value, .. } => {
                bytes.extend(vida_id.to_be_bytes());
                bytes.extend((data.len() as u32).to_be_bytes());
                bytes.extend(data);
                bytes.extend(value.to_be_bytes());
            },
            NewTransactionData::RemoveConduits { vida_id, conduits, .. } => {
                bytes.extend(vida_id.to_be_bytes());
                for conduit in conduits {
                    bytes.extend(hex::decode(&conduit[2..]).map_err(|_| "Invalid conduit address")?);
                }
            },
            NewTransactionData::SetConduitMode { vida_id, mode, conduit_threshold, conduits, conduits_with_voting_power, .. } => {
                bytes.extend(vida_id.to_be_bytes());
                bytes.extend(mode.to_be_bytes());
                bytes.extend(conduit_threshold.to_be_bytes());

                if !conduits.is_empty() {
                    bytes.extend((conduits.len() as u32).to_be_bytes());
                    for conduit in conduits {
                        bytes.extend(hex::decode(&conduit[2..]).map_err(|_| "Invalid conduit address")?);
                    }
                } else if !conduits_with_voting_power.is_empty() {
                    bytes.extend((conduits_with_voting_power.len() as u32).to_be_bytes());
                    for (conduit, voting_power) in conduits_with_voting_power {
                        bytes.extend(hex::decode(&conduit[2..]).map_err(|_| "Invalid conduit address")?);
                        bytes.extend(voting_power.to_be_bytes());
                    }
                } else {
                    bytes.extend(0u32.to_be_bytes());
                }
            },
            NewTransactionData::SetVidaPrivateState { vida_id, private_state, .. } => {
                bytes.extend(vida_id.to_be_bytes());
                bytes.extend((*private_state as u8).to_be_bytes());
            },
            NewTransactionData::SetVidaToAbsolutePublic { vida_id, .. } => {
                bytes.extend(vida_id.to_be_bytes());
            },
            NewTransactionData::AddVidaSponsoredAddresses { vida_id, sponsored_addresses, .. } => {
                bytes.extend(vida_id.to_be_bytes());
                for address in sponsored_addresses {
                    bytes.extend(hex::decode(&address[2..]).map_err(|_| "Invalid sponsored address")?);
                }
            },
            NewTransactionData::AddVidaAllowedSenders { vida_id, allowed_senders, .. } => {
                bytes.extend(vida_id.to_be_bytes());
                for sender in allowed_senders {
                    bytes.extend(hex::decode(&sender[2..]).map_err(|_| "Invalid allowed sender address")?);
                }
            },
            NewTransactionData::RemoveVidaAllowedSenders { vida_id, allowed_senders, .. } => {
                bytes.extend(vida_id.to_be_bytes());
                for sender in allowed_senders {
                    bytes.extend(hex::decode(&sender[2..]).map_err(|_| "Invalid allowed sender address")?);
                }
            },
            NewTransactionData::RemoveSponsoredAddresses { vida_id, sponsored_addresses, .. } => {
                bytes.extend(vida_id.to_be_bytes());
                for address in sponsored_addresses {
                    bytes.extend(hex::decode(&address[2..]).map_err(|_| "Invalid sponsored address")?);
                }
            },
            NewTransactionData::SetPwrTransferRights { vida_id, owner_can_transfer_pwr, .. } => {
                bytes.extend(vida_id.to_be_bytes());
                bytes.extend((*owner_can_transfer_pwr as u8).to_be_bytes());
            },
            NewTransactionData::TransferPwrFromVida { vida_id, receiver, amount, .. } => {
                bytes.extend(vida_id.to_be_bytes());
                bytes.extend(hex::decode(&receiver[2..]).map_err(|_| "Invalid receiver address")?);
                bytes.extend(amount.to_be_bytes());
            },
        }

        Ok(bytes)
    }

    fn identifier(&self) -> u32 {
        match self {
            //  transaction types
            NewTransactionData::SetPublicKey        { .. } => 1001,
            NewTransactionData::JoinAsValidator     { .. } => 1002,
            NewTransactionData::Delegate            { .. } => 1003,
            NewTransactionData::ChangeIp            { .. } => 1004,
            NewTransactionData::ClaimActiveNodeSpot { .. } => 1005,
            NewTransactionData::Transfer            { .. } => 1006,
            //  Governance Proposal Transactions
            NewTransactionData::ChangeEarlyWithdrawPenaltyProposal  { .. } => 1009,
            NewTransactionData::ChangeFeePerByteProposal            { .. } => 1010,
            NewTransactionData::ChangeMaxBlockSizeProposal          { .. } => 1011,
            NewTransactionData::ChangeMaxTxnSizeProposal            { .. } => 1012,
            NewTransactionData::ChangeOverallBurnPercentageProposal { .. } => 1013,
            NewTransactionData::ChangeRewardPerYearProposal         { .. } => 1014,
            NewTransactionData::ChangeValidatorCountLimitProposal   { .. } => 1015,
            NewTransactionData::ChangeValidatorJoiningFeeProposal   { .. } => 1016,
            NewTransactionData::ChangeVidaIdClaimingFeeProposal       { .. } => 1017,
            NewTransactionData::ChangeVidaOwnerTxnFeeShareProposal    { .. } => 1018,
            NewTransactionData::OtherProposalTxn                    { .. } => 1019,
            NewTransactionData::VoteOnProposalTxn                   { .. } => 1020,
            //  Guardian Transactions
            NewTransactionData::GuardianApproval    { .. } => 1021,
            NewTransactionData::RemoveGuardian      { .. } => 1022,
            NewTransactionData::SetGuardian         { .. } => 1023,
            //  Staking Transactions
            NewTransactionData::MoveStake           { .. } => 1024,
            NewTransactionData::RemoveValidator     { .. } => 1025,
            NewTransactionData::Withdraw            { .. } => 1026,
            //  VIDA Transactions
            NewTransactionData::ClaimVidaId           { .. } => 1028,
            NewTransactionData::ConduitApproval     { .. } => 1029,
            NewTransactionData::PayableVidaData       { .. } => 1030,
            NewTransactionData::RemoveConduits      { .. } => 1031,
            NewTransactionData::SetConduitMode      { .. } => 1033,
            NewTransactionData::SetVidaPrivateState   { .. } => 1034,
            NewTransactionData::SetVidaToAbsolutePublic { .. } => 1035,
            NewTransactionData::AddVidaSponsoredAddresses { .. } => 1036,
            NewTransactionData::AddVidaAllowedSenders { .. } => 1037,
            NewTransactionData::RemoveVidaAllowedSenders { .. } => 1038,
            NewTransactionData::RemoveSponsoredAddresses { .. } => 1039,
            NewTransactionData::SetPwrTransferRights { .. } => 1040,
            NewTransactionData::TransferPwrFromVida   { .. } => 1041,
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
