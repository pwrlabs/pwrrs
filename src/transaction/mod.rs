pub mod types;

use self::types::NewTransactionData;
use crate::wallet::PrivateKey;

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
