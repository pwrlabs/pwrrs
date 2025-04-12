use std::{fmt::Display, hash::Hash};
use k256::ecdsa::{
    signature::Verifier, Error, Signature, VerifyingKey,
};
use sha3::{Digest, Keccak256};

use crate::wallet::types::PublicKey;

pub const NODE_URL: &str = "https://pwrrpc.pwrlabs.io/";

impl PublicKey {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, Error> {
        Ok(Self {
            verifying_key: VerifyingKey::from_sec1_bytes(bytes)?,
        })
    }

    pub fn from_hex(hex_str: &str) -> Result<Self, Error> {
        let bytes = hex::decode(hex_str).map_err(|_| Error::new())?;
        let verifying_key = VerifyingKey::from_sec1_bytes(&bytes)?;

        Ok(Self { verifying_key })
    }

    pub fn to_hex(&self) -> String {
        hex::encode_upper(self.verifying_key.to_sec1_bytes())
    }

    pub fn verify_sign(&self, message: &[u8], signature: &[u8; 65]) -> Result<(), Error> {
        let sign = Signature::from_slice(&signature[..64])?;
        self.verifying_key.verify(message, &sign)
    }

    pub fn as_bytes(&self) -> Box<[u8]> {
        self.verifying_key.to_sec1_bytes()
    }

    pub fn address(&self) -> String {
        let public_key = self.verifying_key.to_encoded_point(false);
        let digest = Keccak256::new_with_prefix(&public_key.as_bytes()[1..]).finalize();
        format!("0x{}", hex::encode_upper(&digest[12..]))
    }
}

impl Hash for PublicKey {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let point = self.verifying_key.to_encoded_point(false);
        state.write(point.as_bytes())
    }
}

impl TryFrom<String> for PublicKey {
    type Error = Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::from_hex(&value)
    }
}

impl From<PublicKey> for String {
    fn from(value: PublicKey) -> Self {
        value.to_hex()
    }
}

impl Display for PublicKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.to_hex())
    }
}

#[cfg(feature = "borsh")]
impl borsh::BorshSerialize for super::PublicKey {
    fn serialize<W: std::io::prelude::Write>(&self, writer: &mut W) -> std::io::Result<()> {
        write!(writer, "{}", self.to_hex())
    }
}

#[cfg(feature = "borsh")]
impl borsh::BorshDeserialize for super::PublicKey {
    fn deserialize_reader<R: std::io::prelude::Read>(reader: &mut R) -> std::io::Result<Self> {
        let s = String::deserialize_reader(reader)?;
        Self::from_hex(&s)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, e.to_string()))
    }
}
