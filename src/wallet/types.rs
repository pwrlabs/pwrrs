use k256::ecdsa::{
    SigningKey, VerifyingKey,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Wallet {
    pub private_key: SigningKey,
    pub rpc_url: String,
}

#[derive(Clone, PartialEq, Debug, Serialize)]
#[serde(try_from = "String", into = "String")]
pub struct PublicKey {
    pub verifying_key: VerifyingKey,
}

pub struct Falcon512Wallet {
    pub public_key: Vec<u8>,
    pub private_key: Vec<u8>,
    pub address: Vec<u8>,
    pub rpc_url: String,
}
