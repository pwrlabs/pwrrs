use k256::ecdsa::VerifyingKey;
use serde::Serialize;

#[derive(Clone, PartialEq, Debug, Serialize)]
#[serde(try_from = "String", into = "String")]
pub struct PublicKey {
    pub verifying_key: VerifyingKey,
}

pub struct Wallet {
    pub public_key: Vec<u8>,
    pub private_key: Vec<u8>,
    pub address: Vec<u8>,
    pub rpc_url: String,
}
