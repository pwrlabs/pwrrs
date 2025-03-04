use pqcrypto_falcon::{falcon512, falcon1024};

pub struct Falcon;

#[allow(dead_code)]
impl Falcon {
    pub fn generate_keypair_512() -> (falcon512::PublicKey, falcon512::SecretKey) {
        falcon512::keypair()
    }

    pub fn generate_keypair_1024() -> (falcon1024::PublicKey, falcon1024::SecretKey) {
        falcon1024::keypair()
    }

    pub fn sign_512(message: &[u8], secret_key: &falcon512::SecretKey) -> falcon512::SignedMessage {
        falcon512::sign(message, secret_key)
    }

    pub fn sign_1024(message: &[u8], secret_key: &falcon1024::SecretKey) -> falcon1024::SignedMessage {
        falcon1024::sign(message, secret_key)
    }

    pub fn verify_512(message: &[u8], signature: &falcon512::SignedMessage, public_key: &falcon512::PublicKey) -> bool {
        let verifiedmsg = falcon512::open(&signature, &public_key).unwrap();
        if verifiedmsg == message {
            true
        } else {
            false
        }
    }

    pub fn verify_1024(message: &[u8], signature: &falcon1024::SignedMessage, public_key: &falcon1024::PublicKey) -> bool {
        let verifiedmsg = falcon1024::open(&signature, &public_key).unwrap();
        if verifiedmsg == message {
            true
        } else {
            false
        }
    }
}
