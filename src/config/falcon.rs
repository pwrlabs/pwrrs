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

    pub fn sign_512(message: &[u8], secret_key: &falcon512::SecretKey) -> falcon512::DetachedSignature {
        falcon512::detached_sign(message, secret_key)
    }

    pub fn sign_1024(message: &[u8], secret_key: &falcon1024::SecretKey) -> falcon1024::DetachedSignature {
        falcon1024::detached_sign(message, secret_key)
    }

    pub fn verify_512(message: &[u8], signature: &falcon512::DetachedSignature, public_key: &falcon512::PublicKey) -> bool {
        falcon512::verify_detached_signature(&signature, message, &public_key).is_ok()
    }

    pub fn verify_1024(message: &[u8], signature: &falcon1024::DetachedSignature, public_key: &falcon1024::PublicKey) -> bool {
        falcon1024::verify_detached_signature(&signature, message, &public_key).is_ok()
    }
}
