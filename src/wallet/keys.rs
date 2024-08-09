use std::{fmt::Display, ops::Deref, str::FromStr};

use secp256k1::{ecdsa::RecoveryId, Error, Message, Secp256k1, SecretKey};
use serde::{Deserialize, Serialize};
use sha3::{Digest, Keccak256};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "String")]
#[serde(into = "String")]
pub struct PrivateKey {
    secret_key: SecretKey,
    public_key: PublicKey,
}

impl PrivateKey {
    pub fn random() -> Self {
        let secp = Secp256k1::new();
        let secret_key = SecretKey::new(&mut rand::thread_rng());
        PrivateKey {
            secret_key,
            public_key: PublicKey(secret_key.public_key(&secp)),
        }
    }

    pub fn public_key(&self) -> PublicKey {
        self.public_key.clone()
    }

    pub fn sign_message(&self, message: &[u8]) -> Result<(RecoveryId, [u8; 65]), Error> {
        let mut hasher = Keccak256::new();
        let secp = Secp256k1::new();

        hasher.update(message);
        let hashed = hasher.finalize();

        let (recovery_id, signature) = secp
            .sign_ecdsa_recoverable(&Message::from_digest_slice(&hashed)?, &self.secret_key)
            .serialize_compact();
        let mut sign_bytes = Vec::from(signature);
        sign_bytes.push(27u8);

        Ok((recovery_id, sign_bytes.try_into().unwrap()))
    }
}

impl TryFrom<&[u8]> for PrivateKey {
    type Error = Error;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let secp = Secp256k1::new();
        let secret_key = SecretKey::from_slice(value)?;
        Ok(Self {
            secret_key,
            public_key: PublicKey(secret_key.public_key(&secp)),
        })
    }
}

impl TryFrom<String> for PrivateKey {
    type Error = Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let s = value.replace("0x", "");

        let secret_key = SecretKey::from_str(&s)?;
        let secp = Secp256k1::new();
        Ok(Self {
            secret_key,
            public_key: PublicKey(secret_key.public_key(&secp)),
        })
    }
}

impl TryFrom<&str> for PrivateKey {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::try_from(value.to_string())
    }
}

impl Display for PrivateKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", hex::encode_upper(self.secret_key.secret_bytes()))
    }
}

impl Into<String> for PrivateKey {
    fn into(self) -> String {
        self.to_string()
    }
}

impl Deref for PrivateKey {
    type Target = PublicKey;

    fn deref(&self) -> &Self::Target {
        &self.public_key
    }
}

impl TryFrom<String> for PublicKey {
    type Error = Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let s = value.replace("0x", "");
        secp256k1::PublicKey::from_str(&s).map(PublicKey)
    }
}

impl TryFrom<&str> for PublicKey {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::try_from(value.to_string())
    }
}

impl Display for PublicKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", hex::encode_upper(self.0.serialize_uncompressed()))
    }
}

impl Into<String> for PublicKey {
    fn into(self) -> String {
        self.to_string()
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "String")]
#[serde(into = "String")]
pub struct PublicKey(secp256k1::PublicKey);

impl PublicKey {
    pub fn address(&self) -> String {
        let public_key = self.0.serialize_uncompressed()[1..].to_vec();
        let mut hasher = Keccak256::new();
        hasher.update(public_key);
        let address = hasher.finalize();
        let mut addr = hex::encode_upper(&address[12..32]);
        addr.insert_str(0, "0x");
        addr
    }
}

impl FromStr for PublicKey {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        secp256k1::PublicKey::from_str(s).map(PublicKey)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const PRIVATE_KEY_HEX: &str =
        "0x9D4428C6E0638331B4866B70C831F8BA51C11B031F4B55EED4087BBB8EF0151F";

    #[test]
    fn private_key_can_be_created_from_hex_string() {
        PrivateKey::try_from(PRIVATE_KEY_HEX).unwrap();
    }

    #[test]
    fn private_key_can_be_encoded_to_hex_string() {
        let private_key = PrivateKey::try_from(PRIVATE_KEY_HEX).unwrap();
        let encoded_private_key = private_key.to_string();
        assert_eq!(format!("0x{}", encoded_private_key), PRIVATE_KEY_HEX);
    }

    #[test]
    fn can_get_public_key_from_private_key() {
        let private_key = PrivateKey::try_from(PRIVATE_KEY_HEX).unwrap();
        let public_key = private_key.public_key();
        assert_eq!(public_key, PublicKey::try_from("040cd999a20b0eba1cf86362c738929671902c9b337ab1370d2ba790be68b01227cab9fa9096b87651686bf898acf11857906907ba7fca4f5f5d9513bdd16e0a52").unwrap());
    }

    #[test]
    fn can_get_address_from_public_key() {
        let private_key = PrivateKey::try_from(PRIVATE_KEY_HEX).unwrap();
        let public_key = private_key.public_key();
        let address = public_key.address();
        assert_eq!(address, "0xA4710E3D79E1ED973AF58E0F269E9B21DD11BC64");
    }

    #[test]
    fn can_sign_message() {
        let private_key = PrivateKey::try_from(PRIVATE_KEY_HEX).unwrap();
        let sign = private_key.sign_message(b"Hello World").unwrap();
        assert_eq!(
            hex::encode_upper(&sign.1),
            "4BFE08E9CDD47B064A812011E8DEC867D35833C072047958729BD5FE950F62B53E47C450BA8FED1D190D6ABB60B20ADC32237C5C072C0E1AA56CDBA023062D621B"
        );
    }
}
