use pqcrypto_falcon::{
    falcon512 as falcon512_pqcrypto,
    falcon1024 as falcon1024_pqcrypto,
};
use falcon_rust::{
    falcon512 as falcon512_rust,
    falcon1024 as falcon1024_rust,
};
use crate::config::deterministic_random::DeterministicSecureRandom;
use rand::thread_rng;
use rand::Rng;

pub struct Falcon;

#[allow(dead_code)]
impl Falcon {
    pub fn generate_keypair_512() -> (falcon512_rust::PublicKey, falcon512_rust::SecretKey) {
        let (sk, pk) = falcon512_rust::keygen(thread_rng().gen());
        (pk, sk)
    }

    pub fn generate_keypair_512_from_seed(seed: &[u8]) -> (falcon512_rust::PublicKey, falcon512_rust::SecretKey) {
        let mut deterministic_random = DeterministicSecureRandom::new(seed);

        let mut random_bytes = [0u8; 48];
        deterministic_random.next_bytes(&mut random_bytes);

        let mut deterministic_random = DeterministicSecureRandom::new(seed);
        deterministic_random.next_bytes(&mut random_bytes);

        let (sk, pk) = falcon512_rust::keygen(random_bytes[..32].try_into().unwrap());
        (pk, sk)
    }

    pub fn generate_keypair_1024() -> (falcon1024_rust::PublicKey, falcon1024_rust::SecretKey) {
        let (sk, pk) = falcon1024_rust::keygen(thread_rng().gen());
        (pk, sk)
    }

    pub fn generate_keypair_1024_from_seed(seed: &[u8]) -> (falcon1024_rust::PublicKey, falcon1024_rust::SecretKey) {
        let mut deterministic_random = DeterministicSecureRandom::new(seed);
        
        let mut random_bytes = [0u8; 48];
        deterministic_random.next_bytes(&mut random_bytes);
        
        let mut deterministic_random = DeterministicSecureRandom::new(seed);
        deterministic_random.next_bytes(&mut random_bytes);
        
        let (sk, pk) = falcon1024_rust::keygen(random_bytes[..32].try_into().unwrap());
        (pk, sk)
    }

    pub fn sign_512(message: &[u8], secret_key: &falcon512_pqcrypto::SecretKey) -> falcon512_pqcrypto::DetachedSignature {
        falcon512_pqcrypto::detached_sign(message, secret_key)
    }

    pub fn sign_1024(message: &[u8], secret_key: &falcon1024_pqcrypto::SecretKey) -> falcon1024_pqcrypto::DetachedSignature {
        falcon1024_pqcrypto::detached_sign(message, secret_key)
    }

    pub fn verify_512(message: &[u8], signature: &falcon512_pqcrypto::DetachedSignature, public_key: &falcon512_pqcrypto::PublicKey) -> bool {
        falcon512_pqcrypto::verify_detached_signature(&signature, message, &public_key).is_ok()
    }

    pub fn verify_1024(message: &[u8], signature: &falcon1024_pqcrypto::DetachedSignature, public_key: &falcon1024_pqcrypto::PublicKey) -> bool {
        falcon1024_pqcrypto::verify_detached_signature(&signature, message, &public_key).is_ok()
    }
}
