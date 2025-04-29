use k256::ecdsa::VerifyingKey;
use serde::Serialize;

<<<<<<< HEAD
=======
#[derive(Clone, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Wallet {
    pub private_key: SigningKey,
    pub rpc_url: String,
}

>>>>>>> upstream/main
#[derive(Clone, PartialEq, Debug, Serialize)]
#[serde(try_from = "String", into = "String")]
pub struct PublicKey {
    pub verifying_key: VerifyingKey,
}

<<<<<<< HEAD
pub struct Wallet {
=======
pub struct Falcon512Wallet {
>>>>>>> upstream/main
    pub public_key: Vec<u8>,
    pub private_key: Vec<u8>,
    pub address: Vec<u8>,
    pub rpc_url: String,
}
