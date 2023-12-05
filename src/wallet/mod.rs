use std::{ops::Deref, str::FromStr};

use secp256k1::Error;

pub use self::keys::{PrivateKey, PublicKey};

pub mod keys;

pub struct Wallet {
    private_key: PrivateKey,
}

impl Wallet {
    /// Generate a new private_key using random private key.
    pub fn random() -> Self {
        Self {
            private_key: PrivateKey::random(),
        }
    }

    pub fn public_key(&self) -> PublicKey {
        self.private_key.public_key()
    }

    pub fn private_key(&self) -> PrivateKey {
        self.private_key.clone()
    }

    pub fn address(&self) -> String {
        self.private_key().address()
    }

    //pub fn balance(&self) -> u64 {}

    //pub fn nonce(&self) -> i32 {}

    //pub fn transfer(&self) {}
    //pub fn transfer_with_nonce(&self) {}
    //pub fn send_vm_data_transaction(&self) {}
    //pub fn send_vm_data_transaction_with_nonce(&self) {}
    //pub fn delegate(&self) {}
    //pub fn delegate_with_nonce(&self) {}
    //pub fn withdarw(&self) {}
    //pub fn withdarw_with_nonce(&self) {}
    //pub fn claim_vm_id(&self) {}
    //pub fn claim_vm_id_with_nonce(&self) {}
}

impl From<PrivateKey> for Wallet {
    fn from(private_key: PrivateKey) -> Self {
        Self { private_key }
    }
}

impl FromStr for Wallet {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        PrivateKey::try_from(s).map(|private_key| Self { private_key })
    }
}

impl Deref for Wallet {
    type Target = PrivateKey;

    fn deref(&self) -> &Self::Target {
        &self.private_key
    }
}
