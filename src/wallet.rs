use std::{fmt::Display, hash::Hash};

use k256::ecdsa::{
    signature::DigestVerifier, signature::Verifier, Error, Signature, SigningKey, VerifyingKey,
};
use serde::{Deserialize, Serialize};
use sha3::{Digest, Keccak256};

#[derive(Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Wallet {
    private_key: SigningKey,
}

impl Wallet {
    #[cfg(feature = "rand")]
    /// Generate a new wallet using random private key.
    pub fn random() -> Self {
        let mut thread_rng = rand::thread_rng();
        let signing_key = SigningKey::random(&mut thread_rng);

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

    pub fn public_key(&self) -> PublicKey {
        PublicKey {
            verifying_key: *self.private_key.verifying_key(),
        }
    }

    pub fn address(&self) -> String {
        let public_key = self.public_key().verifying_key.to_encoded_point(false);
        let digest = Keccak256::new_with_prefix(&public_key.as_bytes()[1..]).finalize();
        format!("0x{}", hex::encode_upper(&digest[12..]))
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

#[derive(Clone, Eq, Copy, PartialEq, Debug, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct PublicKey {
    verifying_key: VerifyingKey,
}

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
impl borsh::BorshSerialize for super::Wallet {
    fn serialize<W: std::io::prelude::Write>(&self, writer: &mut W) -> std::io::Result<()> {
        write!(writer, "{}", self.to_hex())
    }
}

#[cfg(feature = "borsh")]
impl borsh::BorshSerialize for super::PublicKey {
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

#[cfg(feature = "borsh")]
impl borsh::BorshDeserialize for super::PublicKey {
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
        let public_key = wallet.public_key();
        assert_eq!(public_key, PublicKey::from_hex("040cd999a20b0eba1cf86362c738929671902c9b337ab1370d2ba790be68b01227cab9fa9096b87651686bf898acf11857906907ba7fca4f5f5d9513bdd16e0a52").unwrap());
    }

    #[test]
    fn can_get_address_from_public_key() {
        let wallet = Wallet::from_hex(PRIVATE_KEY_HEX).unwrap();
        let address = wallet.address();
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
