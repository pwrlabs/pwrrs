pub mod types;
pub mod hex_serde;
pub mod stream;

use self::types::Transaction;
pub use self::types::NewTransactionData;
use crate::{Wallet, Falcon512Wallet};
use crate::config::falcon::Falcon;
use pqcrypto_falcon::falcon512;
use pqcrypto_traits::sign::*;

impl NewTransactionData {
    pub fn fee_per_byte(&self) -> Option<u64> {
        match self {
            NewTransactionData::FalconSetPublicKey { fee_per_byte, .. } => Some(*fee_per_byte),
            NewTransactionData::FalconJoinAsValidator { fee_per_byte, .. } => Some(*fee_per_byte),
            NewTransactionData::FalconDelegate { fee_per_byte, .. } => Some(*fee_per_byte),
            NewTransactionData::FalconChangeIp { fee_per_byte, .. } => Some(*fee_per_byte),
            NewTransactionData::FalconClaimActiveNodeSpot { fee_per_byte } => Some(*fee_per_byte),
            NewTransactionData::FalconTransfer { fee_per_byte, .. } => Some(*fee_per_byte),
            NewTransactionData::FalconVmData { fee_per_byte, .. } => Some(*fee_per_byte),
            _ => None,
        }
    }

    pub fn falcon512_serialize_for_broadcast(
        &self,
        nonce: u32,
        chain_id: u8,
        wallet: &Falcon512Wallet,
    ) -> Result<Vec<u8>, &'static str> {
        let mut bytes = Vec::new();
        bytes.extend(self.identifier().to_be_bytes());
        bytes.extend(chain_id.to_be_bytes());
        bytes.extend(nonce.to_be_bytes());
        
        if let Some(fee) = self.fee_per_byte() {
            bytes.extend(fee.to_be_bytes());
        }
        
        bytes.extend(wallet.address.clone());
        bytes.extend(self.transaction_bytes()?);

        let private_key = falcon512::SecretKey::from_bytes(&wallet.private_key).map_err(|_| "Invalid private key")?;
        let signature = Falcon::sign_512(&bytes, &private_key).as_bytes().to_vec();
        bytes.extend((signature.len() as u16).to_be_bytes());
        bytes.extend(signature);

        Ok(bytes)
    }

    pub fn serialize_for_broadcast(
        &self,
        nonce: u32,
        chain_id: u8,
        wallet: &Wallet,
    ) -> Result<Vec<u8>, &'static str> {
        let mut bytes = Vec::new();
        bytes.extend(self.identifier().to_be_bytes());
        bytes.extend(chain_id.to_be_bytes());
        bytes.extend(nonce.to_be_bytes());
        bytes.extend(self.transaction_bytes()?);

        let signature = wallet.sign(&bytes).map_err(|_| "Failed to sign message")?;
        bytes.extend(signature);
        Ok(bytes)
    }

    pub fn transaction_bytes(&self) -> Result<Vec<u8>, &'static str> {
        let mut bytes = Vec::new();

        match self {
            NewTransactionData::Transfer { amount, recipient } => {
                bytes.extend(amount.to_be_bytes());
                bytes.extend(hex::decode(recipient[2..].to_string()).map_err(|_| "Invalid recipient address")?);
            }
            NewTransactionData::JoinAsValidator { ip } => {
                bytes.extend(ip.clone().into_bytes());
            }
            NewTransactionData::ClaimSpot { validator } => {
                bytes.extend(hex::decode(validator[2..].to_string()).map_err(|_| "Invalid validator address")?);
            }
            NewTransactionData::Delegate { amount, validator } => {
                bytes.extend(amount.to_be_bytes());
                bytes.extend(hex::decode(validator[2..].to_string()).map_err(|_| "Invalid validator address")?);
            }
            NewTransactionData::Withdraw { shares, validator } => {
                bytes.extend(shares.to_be_bytes());
                bytes.extend(hex::decode(validator[2..].to_string()).map_err(|_| "Invalid validator address")?);
            }
            NewTransactionData::VmData { vm_id, data } => {
                bytes.extend(vm_id.to_be_bytes());
                bytes.extend((data.len() as u32).to_be_bytes());
                bytes.extend(data);
            }
            NewTransactionData::ClaimVmID { vm_id } => bytes.extend(vm_id.to_be_bytes()),
            NewTransactionData::SetGuardian { guardian_expiry_date, guardian } => {
                bytes.extend(guardian_expiry_date.to_be_bytes());
                bytes.extend(hex::decode(guardian[2..].to_string()).map_err(|_| "Invalid guardian address")?);
            }
            NewTransactionData::GuardianApproval { transactions } => {
                let decoded_transactions: Vec<Vec<u8>> = transactions
                    .iter()
                    .map(|transaction| self.to_bytes(&transaction).map_err(|_| "Invalid transaction data"))
                    .collect::<Result<_, _>>()?;

                let lengths_bytes: Vec<u8> = decoded_transactions
                    .iter().map(|tx| (tx.len() as u32).to_be_bytes()).flatten().collect();

                let transactions_bytes: Vec<u8> = decoded_transactions.iter().flatten().cloned().collect();

                bytes.extend(lengths_bytes);
                bytes.extend(transactions_bytes);
            }
            NewTransactionData::PayableVmData { vm_id, data, amount } => {
                bytes.extend(vm_id.to_be_bytes());
                bytes.extend((data.len() as u32).to_be_bytes());
                bytes.extend(data);
                bytes.extend(amount.to_be_bytes());
            }
            NewTransactionData::RemoveGuardian => {},
            NewTransactionData::SetConduits { vm_id, conduits } => {
                bytes.extend(vm_id.to_be_bytes());

                let decoded_conduits: Vec<Vec<u8>> = conduits
                    .iter()
                    .map(|c| hex::decode(c[2..].to_string()).map_err(|_| "Invalid conduit address"))
                    .collect::<Result<_, _>>()?;

                let lengths_bytes: Vec<u8> = decoded_conduits
                    .iter().map(|c| (c.len() as u32).to_be_bytes()).flatten().collect();

                let conduits_bytes: Vec<u8> = decoded_conduits.iter().flatten().cloned().collect();
                
                bytes.extend(lengths_bytes);
                bytes.extend(conduits_bytes);
            }
            NewTransactionData::AddConduits { vm_id, conduits } => {
                bytes.extend(vm_id.to_be_bytes());
                bytes.extend(conduits);
            }
            NewTransactionData::MoveStake { shares_amount, from_validator, to_validator } => {
                bytes.extend(shares_amount.to_be_bytes());
                bytes.extend(hex::decode(from_validator[2..].to_string()).map_err(|_| "Invalid address")?);
                bytes.extend(hex::decode(to_validator[2..].to_string()).map_err(|_| "Invalid address")?);
            }
            NewTransactionData::ChangeEarlyWithdrawPenaltyProposal { title, withdraw_penalty_time, withdraw_penalty, description } => {
                let title_length = title.as_bytes().len() as u32;
                bytes.extend(title_length.to_be_bytes());

                bytes.extend(title.as_bytes());
                bytes.extend(withdraw_penalty_time.to_be_bytes());
                bytes.extend(withdraw_penalty.to_be_bytes());
                bytes.extend(description.as_bytes());
            }
            NewTransactionData::ChangeFeePerByteProposal { title, description, fee_per_byte } => {
                let title_length = title.as_bytes().len() as u32;
                bytes.extend(title_length.to_be_bytes());
                
                bytes.extend(title.as_bytes());
                bytes.extend(fee_per_byte.to_be_bytes());
                bytes.extend(description.as_bytes());
            }
            NewTransactionData::ChangeMaxBlockSizeProposal { title, description, max_block_size } => {
                let title_length = title.as_bytes().len() as u32;
                bytes.extend(title_length.to_be_bytes());

                bytes.extend(title.as_bytes());
                bytes.extend(max_block_size.to_be_bytes());
                bytes.extend(description.as_bytes());
            }
            NewTransactionData::ChangeMaxTxnSizeProposal { title, description, max_txn_size } => {
                let title_length = title.as_bytes().len() as u32;
                bytes.extend(title_length.to_be_bytes());

                bytes.extend(title.as_bytes());
                bytes.extend(max_txn_size.to_be_bytes());
                bytes.extend(description.as_bytes());
            }
            NewTransactionData::ChangeOverallBurnPercentageProposal { title, description, burn_percentage } => {
                let title_length = title.as_bytes().len() as u32;
                bytes.extend(title_length.to_be_bytes());
                
                bytes.extend(title.as_bytes());
                bytes.extend(burn_percentage.to_be_bytes());
                bytes.extend(description.as_bytes());
            }
            NewTransactionData::ChangeRewardPerYearProposal { title, description, reward_per_year } => {
                let title_length = title.as_bytes().len() as u32;
                bytes.extend(title_length.to_be_bytes());

                bytes.extend(title.as_bytes());
                bytes.extend(reward_per_year.to_be_bytes());
                bytes.extend(description.as_bytes());
            }
            NewTransactionData::ChangeValidatorCountLimitProposal { title, description, validator_count_limit } => {
                let title_length = title.as_bytes().len() as u32;
                bytes.extend(title_length.to_be_bytes());
                
                bytes.extend(title.as_bytes());
                bytes.extend(validator_count_limit.to_be_bytes());
                bytes.extend(description.as_bytes());
            }
            NewTransactionData::ChangeValidatorJoiningFeeProposal { title, description, joining_fee } => {
                let title_length = title.as_bytes().len() as u32;
                bytes.extend(title_length.to_be_bytes());
                
                bytes.extend(title.as_bytes());
                bytes.extend(joining_fee.to_be_bytes());
                bytes.extend(description.as_bytes());
            }
            NewTransactionData::ChangeVmIdClaimingFeeProposal { title, description, claiming_fee } => {
                let title_length = title.as_bytes().len() as u32;
                bytes.extend(title_length.to_be_bytes());
                
                bytes.extend(title.as_bytes());
                bytes.extend(claiming_fee.to_be_bytes());
                bytes.extend(description.as_bytes());
            }
            NewTransactionData::ChangeVmOwnerTxnFeeShareProposal { title, description, fee_share } => {
                let title_length = title.as_bytes().len() as u32;
                bytes.extend(title_length.to_be_bytes());
                
                bytes.extend(title.as_bytes());
                bytes.extend(fee_share.to_be_bytes());
                bytes.extend(description.as_bytes());
            }
            NewTransactionData::OtherProposalTxn { title, description } => {
                let title_length = title.as_bytes().len() as u32;
                bytes.extend(title_length.to_be_bytes());

                bytes.extend(title.as_bytes());
                bytes.extend(description.as_bytes());
            }
            NewTransactionData::VoteOnProposalTxn { proposal_hash, vote } => {
                bytes.extend(hex::decode(proposal_hash[2..].to_string()).map_err(|_| "Invalid proposal hash")?);
                bytes.extend(vote.to_be_bytes());
            },

            // Falcon transaction types
            NewTransactionData::FalconSetPublicKey { public_key, .. } => {
                let public_key = hex::decode(&public_key).map_err(|_| "Invalid public key")?;

                bytes.extend((public_key.len() as u16).to_be_bytes());
                bytes.extend(public_key);
            },
            NewTransactionData::FalconJoinAsValidator { ip, .. } => {
                let ip_bytes = ip.as_bytes();
                bytes.extend((ip_bytes.len() as u16).to_be_bytes());
                bytes.extend(ip_bytes);
            },
            NewTransactionData::FalconDelegate { validator, pwr_amount, .. } => {
                bytes.extend(hex::decode(&validator[2..]).map_err(|_| "Invalid validator address")?);
                bytes.extend(pwr_amount.to_be_bytes());
            },
            NewTransactionData::FalconChangeIp { new_ip, .. } => {
                let ip_bytes = new_ip.as_bytes();
                bytes.extend((ip_bytes.len() as u16).to_be_bytes());
                bytes.extend(ip_bytes);
            },
            NewTransactionData::FalconClaimActiveNodeSpot { .. } => { },
            NewTransactionData::FalconTransfer { receiver, amount, .. } => {
                bytes.extend(hex::decode(&receiver[2..]).map_err(|_| "Invalid receiver address")?);
                bytes.extend(amount.to_be_bytes());
            },
            NewTransactionData::FalconVmData { vm_id, data, .. } => {
                bytes.extend(vm_id.to_be_bytes());
                bytes.extend((data.len() as u32).to_be_bytes());
                bytes.extend(data);
            },
        }

        Ok(bytes)
    }

    fn identifier(&self) -> u32 {
        match self {
            NewTransactionData::Transfer         { .. } => 0,
            NewTransactionData::JoinAsValidator  { .. } => 1,
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
            // validator remove transactin => 7 - deleted
            // conduit approval transaction => 12 - deleted
            NewTransactionData::ChangeEarlyWithdrawPenaltyProposal  { .. } => 17,
            NewTransactionData::ChangeFeePerByteProposal            { .. } => 18,
            NewTransactionData::ChangeMaxBlockSizeProposal          { .. } => 19,
            NewTransactionData::ChangeMaxTxnSizeProposal            { .. } => 20,
            NewTransactionData::ChangeOverallBurnPercentageProposal { .. } => 21,
            NewTransactionData::ChangeRewardPerYearProposal         { .. } => 22,
            NewTransactionData::ChangeValidatorCountLimitProposal   { .. } => 23,
            NewTransactionData::ChangeValidatorJoiningFeeProposal   { .. } => 24,
            NewTransactionData::ChangeVmIdClaimingFeeProposal       { .. } => 25,
            NewTransactionData::ChangeVmOwnerTxnFeeShareProposal    { .. } => 26,
            NewTransactionData::OtherProposalTxn                    { .. } => 27,
            NewTransactionData::VoteOnProposalTxn                   { .. } => 28,

            // Falcon transaction types
            NewTransactionData::FalconSetPublicKey        { .. } => 1001,
            NewTransactionData::FalconJoinAsValidator     { .. } => 1002,
            NewTransactionData::FalconDelegate            { .. } => 1003,
            NewTransactionData::FalconChangeIp            { .. } => 1004,
            NewTransactionData::FalconClaimActiveNodeSpot { .. } => 1005,
            NewTransactionData::FalconTransfer            { .. } => 1006,
            NewTransactionData::FalconVmData              { .. } => 1007,
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
