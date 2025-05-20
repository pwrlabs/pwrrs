use pqcrypto_falcon::{
    falcon512 as falcon512_pqcrypto,
    falcon1024 as falcon1024_pqcrypto,
};
use crate::config::deterministic_random::DeterministicSecureRandom;
use crate::config::falcon_wrapper::{generate_keypair_from_seed, generate_keypair};

pub struct Falcon;

#[allow(dead_code)]
impl Falcon {
    pub fn generate_keypair_512() -> (Vec<u8>, Vec<u8>) {
        let keypair = generate_keypair(9).unwrap();
        (keypair.public_key, keypair.private_key)
    }

    pub fn generate_keypair_512_from_seed(seed: &[u8]) -> (Vec<u8>, Vec<u8>) {
        let mut deterministic_random = DeterministicSecureRandom::new(seed);

        let mut random_bytes = [0u8; 48];
        deterministic_random.next_bytes(&mut random_bytes);

        let mut deterministic_random = DeterministicSecureRandom::new(seed);
        deterministic_random.next_bytes(&mut random_bytes);

        let keypair = generate_keypair_from_seed(9, &random_bytes).unwrap();
        (keypair.public_key, keypair.private_key)
    }

    pub fn generate_keypair_1024() -> (Vec<u8>, Vec<u8>) {
        let keypair = generate_keypair(10).unwrap();
        (keypair.public_key, keypair.private_key)
    }

    pub fn generate_keypair_1024_from_seed(seed: &[u8]) -> (Vec<u8>, Vec<u8>) {
        let mut deterministic_random = DeterministicSecureRandom::new(seed);
        
        let mut random_bytes = [0u8; 48];
        deterministic_random.next_bytes(&mut random_bytes);
        
        let mut deterministic_random = DeterministicSecureRandom::new(seed);
        deterministic_random.next_bytes(&mut random_bytes);
        
        let keypair = generate_keypair_from_seed(10, &random_bytes).unwrap();
        (keypair.public_key, keypair.private_key)
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
