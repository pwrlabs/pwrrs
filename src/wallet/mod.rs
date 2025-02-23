pub mod types;
pub mod keys;
pub mod aes256;
use hex;

use std::{fmt::Display, hash::Hash};
use k256::ecdsa::{
    signature::DigestVerifier, Error, Signature, SigningKey,
};
use sha3::{Digest, Keccak256};
use aes256::AES256;
use rand::rngs::OsRng;

use crate::wallet::types::{PublicKey, Wallet};
use crate::transaction::types::{NewTransactionData, Transaction};
use crate::rpc::{RPC, BroadcastResponse};

const NODE_URL: &str = "https://pwrrpc.pwrlabs.io/";

impl Wallet {
    #[cfg(feature = "rand")]
    /// Generate a new wallet using random private key.
    pub fn random() -> Self {
        let signing_key = SigningKey::random(&mut OsRng);

        Self {
            private_key: signing_key,
        }
    }

    pub fn from_hex(hex_str: &str) -> Result<Self, Error> {
        let bytes = if hex_str.len() > 2 && (&hex_str[..2] == "0x" || &hex_str[..2] == "0X") {
            hex::decode(&hex_str[2..]).map_err(|_| Error::new())?
        } else {
            hex::decode(hex_str).map_err(|_| Error::new())?
        };
        let private_key = SigningKey::from_slice(&bytes)?;

        Ok(Self { private_key })
    }

    pub fn to_hex(&self) -> String {
        hex::encode_upper(self.private_key.to_bytes())
    }

    pub fn sign(&self, message: &[u8]) -> Result<[u8; 65], Error> {
        let digest = Keccak256::new_with_prefix(message);
        let (sign, rid) = self.private_key.sign_digest_recoverable(digest)?;
        let mut bytes = vec![];
        bytes.extend_from_slice(&sign.to_bytes());

        if rid.to_byte() == 0 || rid.to_byte() == 1 {
            bytes.push(rid.to_byte() + 27);
        }
        Ok(bytes.try_into().unwrap())
    }

    pub fn verify_sign(&self, message: &[u8], signature: &[u8; 65]) -> Result<(), Error> {
        let digest = Keccak256::new_with_prefix(message);
        let sign = Signature::from_slice(&signature[..64])?;
        self.private_key
            .verifying_key()
            .verify_digest(digest, &sign)
    }

    pub fn get_public_key(&self) -> PublicKey {
        PublicKey {
            verifying_key: *self.private_key.verifying_key(),
        }
    }

    pub fn get_private_key(&self) -> String {
        let pk = self.private_key.to_bytes();
        format!("0x{}", hex::encode(pk))
    }

    pub fn store_wallet(&self, path: &str, password: &str) -> Result<(), Box<dyn std::error::Error>> {
        let private_key_bytes = self.private_key.to_bytes().as_slice().to_vec();

        let encrypted_private_key = AES256::encrypt(&private_key_bytes, password)
            .map_err(|e| format!("Encryption error: {:?}", e))?;

        std::fs::write(path, encrypted_private_key)?;

        Ok(())
    }

    pub fn load_wallet(path: &str, password: &str) -> Option<Self> {
        let encrypted_data = std::fs::read(path).ok()?;
        let private_key_bytes = AES256::decrypt(&encrypted_data, password).ok()?;
        let private_key = SigningKey::from_slice(&private_key_bytes).ok()?;
        Some(Self { private_key })
    }

    pub fn get_address(&self) -> String {
        let public_key = self.get_public_key().verifying_key.to_encoded_point(false);
        let digest = Keccak256::new_with_prefix(&public_key.as_bytes()[1..]).finalize();
        format!("0x{}", hex::encode_upper(&digest[12..]))
    }

    pub async fn get_balance(&self) -> u64 {
        let rpc = RPC::new(NODE_URL).await.unwrap();
        let balance = rpc.get_balance_of_address(&self.get_address()).await.unwrap();
        return balance;
    }

    pub async fn get_nonce(&self) -> u32 {
        let rpc = RPC::new(NODE_URL).await.unwrap();
        let nonce = rpc.get_nonce_of_address(&self.get_address()).await.unwrap();
        return nonce;
    }

    pub async fn transfer_pwr(&self, recipient: String, amount: u64) -> BroadcastResponse {
        let tx = NewTransactionData::Transfer {
            amount: amount,
            recipient: recipient,
        };
        let response = (self.get_rpc().await).broadcast_transaction(&tx, &self).await;
        return response;
    }

    pub async fn send_vm_data(&self, vm_id: u64, data: Vec<u8>) -> BroadcastResponse {
        let new_tx = NewTransactionData::VmData { vm_id: vm_id, data: data };
        let response = (self.get_rpc().await).broadcast_transaction(&new_tx, &self).await;
        return response;
    }

    pub async fn send_payable_vm_data(&self, vm_id: u64, amount: u64, data: Vec<u8>) -> BroadcastResponse {
        let new_tx = NewTransactionData::PayableVmData { 
            vm_id: vm_id, 
            data: data, 
            amount: amount 
        };
        let response = (self.get_rpc().await).broadcast_transaction(&new_tx, &self).await;
        return response;
    }

    pub async fn claim_vm_id(&self, vm_id: u64) -> BroadcastResponse {
        let new_tx = NewTransactionData::ClaimVmID { vm_id: vm_id };
        let response = (self.get_rpc().await).broadcast_transaction(&new_tx, &self).await;
        return response;
    }

    pub async fn join(&self, ip: String) -> BroadcastResponse {
        let new_tx = NewTransactionData::Join { ip: ip };
        let response = (self.get_rpc().await).broadcast_transaction(&new_tx, &self).await;
        return response;
    }

    pub async fn claim_spot(&self) -> BroadcastResponse {
        let address = self.get_address().to_string().strip_prefix("0x").unwrap_or(&self.get_address()).to_string();
        let new_tx = NewTransactionData::ClaimSpot { validator: address };
        let response = (self.get_rpc().await).broadcast_transaction(&new_tx, &self).await;
        return response;
    }

    pub async fn delegate(&self, validator: String, amount: u64) -> BroadcastResponse {
        let new_tx = NewTransactionData::Delegate {
            amount: amount,
            validator: validator,
        };
        let response = (self.get_rpc().await).broadcast_transaction(&new_tx, &self).await;
        return response;
    }

    pub async fn withdraw(&self, validator: String, shares: u64) -> BroadcastResponse {
        let new_tx = NewTransactionData::Withdraw {
            shares: shares,
            validator: validator,
        };
        let response = (self.get_rpc().await).broadcast_transaction(&new_tx, &self).await;
        return response;
    }

    pub async fn set_guardian(&self, guardian: String, guardian_expiry_date: u64) -> BroadcastResponse {
        let new_tx = NewTransactionData::SetGuardian {
            guardian_expiry_date: guardian_expiry_date,
            guardian: guardian,
        };
        let response = (self.get_rpc().await).broadcast_transaction(&new_tx, &self).await;
        return response;
    }

    pub async fn send_guardian_approval_transaction(&self, transactions: Vec<Transaction>) -> BroadcastResponse {
        let new_tx = NewTransactionData::GuardianApproval { transactions: transactions };
        let response = (self.get_rpc().await).broadcast_transaction(&new_tx, &self).await;
        return response;
    }

    pub async fn remove_guardian(&self) -> BroadcastResponse {
        let new_tx = NewTransactionData::RemoveGuardian;
        let response = (self.get_rpc().await).broadcast_transaction(&new_tx, &self).await;
        return response;
    }

    pub async fn set_conduits(&self, vm_id: u64, conduits: Vec<String>) -> BroadcastResponse {
        let new_tx = NewTransactionData::SetConduits { vm_id: vm_id, conduits: conduits };
        let response = (self.get_rpc().await).broadcast_transaction(&new_tx, &self).await;
        return response;
    }

    pub async fn add_conduits(&self, vm_id: u64, conduits: Vec<u8>) -> BroadcastResponse {
        let new_tx = NewTransactionData::AddConduits { vm_id: vm_id, conduits: conduits };
        let response = (self.get_rpc().await).broadcast_transaction(&new_tx, &self).await;
        return response;
    }

    pub async fn move_stake(
        &self, shares_amount: u64, from_validator: String, to_validator: String
    ) -> BroadcastResponse {
        let new_tx = NewTransactionData::MoveStake {
            shares_amount: shares_amount,
            from_validator: from_validator,
            to_validator: to_validator,
        };
        let response = (self.get_rpc().await).broadcast_transaction(&new_tx, &self).await;
        return response;
    }

    pub async fn create_proposal_change_early_withdraw_penalty(
        &self, withdraw_penalty: u32, withdraw_penalty_time: u64, title: String, description: String
    ) -> BroadcastResponse {
        let new_tx = NewTransactionData::ChangeEarlyWithdrawPenaltyProposal {
            title: title,
            description: description,
            withdraw_penalty_time: withdraw_penalty_time,
            withdraw_penalty: withdraw_penalty,
        };
        let response = (self.get_rpc().await).broadcast_transaction(&new_tx, &self).await;
        return response;
    }

    pub async fn create_proposal_change_fee_per_byte(
        &self, fee_per_byte: u64, title: String, description: String
    ) -> BroadcastResponse {
        let new_tx = NewTransactionData::ChangeFeePerByteProposal {
            title: title,
            description: description,
            fee_per_byte: fee_per_byte,
        };
        let response = (self.get_rpc().await).broadcast_transaction(&new_tx, &self).await;
        return response;
    }

    pub async fn create_proposal_change_max_block_size(
        &self, max_block_size: u32, title: String, description: String
    ) -> BroadcastResponse {
        let new_tx = NewTransactionData::ChangeMaxBlockSizeProposal {
            title: title,
            description: description,
            max_block_size: max_block_size,
        };
        let response = (self.get_rpc().await).broadcast_transaction(&new_tx, &self).await;
        return response;
    }

    pub async fn create_proposal_change_max_txn_size(
        &self, max_txn_size: u32, title: String, description: String
    ) -> BroadcastResponse {
        let new_tx = NewTransactionData::ChangeMaxTxnSizeProposal {
            title: title,
            description: description,
            max_txn_size: max_txn_size,
        };
        let response = (self.get_rpc().await).broadcast_transaction(&new_tx, &self).await;
        return response;
    }

    pub async fn create_proposal_change_overall_burn_percentage(
        &self, burn_percentage: u32, title: String, description: String
    ) -> BroadcastResponse {
        let new_tx = NewTransactionData::ChangeOverallBurnPercentageProposal {
            title: title,
            description: description,
            burn_percentage: burn_percentage,
        };
        let response = (self.get_rpc().await).broadcast_transaction(&new_tx, &self).await;
        return response; 
    }

    pub async fn create_proposal_change_reward_per_year(
        &self, reward_per_year: u64, title: String, description: String
    ) -> BroadcastResponse {
        let new_tx = NewTransactionData::ChangeRewardPerYearProposal {
            title: title,
            description: description,
            reward_per_year: reward_per_year,
        };
        let response = (self.get_rpc().await).broadcast_transaction(&new_tx, &self).await;
        return response;
    }

    pub async fn create_proposal_change_validator_count_limit(
        &self, validator_count_limit: u32, title: String, description: String
    ) -> BroadcastResponse {
        let new_tx = NewTransactionData::ChangeValidatorCountLimitProposal {
            title: title,
            description: description,
            validator_count_limit: validator_count_limit,
        };
        let response = (self.get_rpc().await).broadcast_transaction(&new_tx, &self).await;
        return response;
    }

    pub async fn create_proposal_change_validator_joining_fee(
        &self, joining_fee: u64, title: String, description: String
    ) -> BroadcastResponse {
        let new_tx = NewTransactionData::ChangeValidatorJoiningFeeProposal {
            title: title,
            description: description,
            joining_fee: joining_fee,
        };
        let response = (self.get_rpc().await).broadcast_transaction(&new_tx, &self).await;
        return response;
    }

    pub async fn create_proposal_change_vm_id_claiming_fee(
        &self, claiming_fee: u64, title: String, description: String
    ) -> BroadcastResponse {
        let new_tx = NewTransactionData::ChangeVmIdClaimingFeeProposal {
            title: title,
            description: description,
            claiming_fee: claiming_fee,
        };
        let response = (self.get_rpc().await).broadcast_transaction(&new_tx, &self).await;
        return response;
    }

    pub async fn create_proposal_change_vm_owner_txn_fee_share(
        &self, fee_share: u32, title: String, description: String
    ) -> BroadcastResponse {
        let new_tx = NewTransactionData::ChangeVmOwnerTxnFeeShareProposal {
            title: title,
            description: description,
            fee_share: fee_share,
        };
        let response = (self.get_rpc().await).broadcast_transaction(&new_tx, &self).await;
        return response;
    }

    pub async fn create_proposal_other_proposal(&self, title: String, description: String) -> BroadcastResponse {
        let new_tx = NewTransactionData::OtherProposalTxn { title: title, description: description };
        let response = (self.get_rpc().await).broadcast_transaction(&new_tx, &self).await;
        return response;
    }

    pub async fn vote_on_proposal(&self, proposal_hash: String, vote: u8) -> BroadcastResponse {
        let new_tx = NewTransactionData::VoteOnProposalTxn { proposal_hash: proposal_hash, vote: vote };
        let response = (self.get_rpc().await).broadcast_transaction(&new_tx, &self).await;
        return response;
    }

    async fn get_rpc(&self) -> RPC {
        RPC::new(NODE_URL).await.unwrap()
    }
}

impl Hash for Wallet {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let bytes = self.private_key.to_bytes();
        state.write(&bytes)
    }
}

impl TryFrom<String> for Wallet {
    type Error = Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::from_hex(&value)
    }
}

impl From<Wallet> for String {
    fn from(value: Wallet) -> Self {
        value.to_hex()
    }
}

impl Display for Wallet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.to_hex())
    }
}

#[cfg(feature = "borsh")]
impl borsh::BorshSerialize for super::Wallet {
    fn serialize<W: std::io::prelude::Write>(&self, writer: &mut W) -> std::io::Result<()> {
        write!(writer, "{}", self.to_hex())
    }
}

#[cfg(feature = "borsh")]
impl borsh::BorshDeserialize for super::Wallet {
    fn deserialize_reader<R: std::io::prelude::Read>(reader: &mut R) -> std::io::Result<Self> {
        let s = String::deserialize_reader(reader)?;
        Self::from_hex(&s)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, e.to_string()))
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    const PRIVATE_KEY_HEX: &str =
        "0x9D4428C6E0638331B4866B70C831F8BA51C11B031F4B55EED4087BBB8EF0151F";

    #[test]
    fn wallet_can_be_created_from_hex_string() {
        Wallet::from_hex(PRIVATE_KEY_HEX).unwrap();
    }

    #[test]
    fn wallet_can_be_encoded_to_hex_string() {
        let wallet = Wallet::from_hex(PRIVATE_KEY_HEX).unwrap();
        let encoded_wallet = wallet.to_hex();
        assert_eq!(format!("0x{}", encoded_wallet), PRIVATE_KEY_HEX);
    }

    #[test]
    fn can_get_public_key_from_wallet() {
        let wallet = Wallet::from_hex(PRIVATE_KEY_HEX).unwrap();
        let public_key = wallet.get_public_key();
        assert_eq!(public_key, PublicKey::from_hex("040cd999a20b0eba1cf86362c738929671902c9b337ab1370d2ba790be68b01227cab9fa9096b87651686bf898acf11857906907ba7fca4f5f5d9513bdd16e0a52").unwrap());
    }

    #[test]
    fn can_get_address_from_public_key() {
        let wallet = Wallet::from_hex(PRIVATE_KEY_HEX).unwrap();
        let address = wallet.get_address();
        assert_eq!(address, "0xA4710E3D79E1ED973AF58E0F269E9B21DD11BC64");
    }

    #[test]
    fn can_sign_message() {
        let wallet = Wallet::from_hex(PRIVATE_KEY_HEX).unwrap();
        let sign = wallet.sign(b"Hello World").unwrap();
        assert_eq!(
            hex::encode_upper(&sign),
            "4BFE08E9CDD47B064A812011E8DEC867D35833C072047958729BD5FE950F62B53E47C450BA8FED1D190D6ABB60B20ADC32237C5C072C0E1AA56CDBA023062D621B"
        );
    }

    #[test]
    fn can_verify_signed_message() {
        let wallet = Wallet::from_hex(PRIVATE_KEY_HEX).unwrap();
        let sign = wallet.sign(b"Hello World").unwrap();
        wallet.verify_sign(b"Hello World", &sign).unwrap();
    }
}
