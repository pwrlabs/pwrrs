use k256::ecdsa::{
    SigningKey, VerifyingKey,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Wallet {
    pub private_key: SigningKey,
}

#[derive(Clone, Eq, Copy, PartialEq, Debug, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct PublicKey {
    pub verifying_key: VerifyingKey,
}
