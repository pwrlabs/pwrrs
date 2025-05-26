use libc::{c_int, c_uint, c_void, size_t};
use thiserror::Error;
use pqcrypto_falcon::{falcon512, falcon1024};
use crate::config::deterministic_random::DeterministicSecureRandom;

pub struct Falcon;

#[allow(dead_code)]
impl Falcon {
    pub fn generate_keypair_512() -> (Vec<u8>, Vec<u8>) {
        let keypair = generate_keypair_from_seed(9, &[]).unwrap();
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
        let keypair = generate_keypair_from_seed(10, &[]).unwrap();
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

#[derive(Debug, Error)]
enum FalconError {
    #[error("Random number generation failed")]
    Random,
    #[error("Buffer too small")]
    Size,
    #[error("Invalid format")]
    Format,
    #[error("Invalid signature")]
    BadSig,
    #[error("Invalid argument")]
    BadArg,
    #[error("Internal error")]
    Internal,
    #[error("Unknown error: {0}")]
    Unknown(i32),
}

impl From<i32> for FalconError {
    fn from(code: i32) -> Self {
        match code {
            -1 => FalconError::Random,
            -2 => FalconError::Size,
            -3 => FalconError::Format,
            -4 => FalconError::BadSig,
            -5 => FalconError::BadArg,
            -6 => FalconError::Internal,
            _ => FalconError::Unknown(code),
        }
    }
}

#[derive(Debug)]
struct KeyPair {
    pub public_key: Vec<u8>,
    pub private_key: Vec<u8>,
}

#[repr(C)]
struct PRNGContext {
    state: [u8; 256],
}

#[link(name = "falcon", kind = "dylib")]
extern "C" {
    fn falcon_privkey_size(log_n: c_uint) -> size_t;
    fn falcon_pubkey_size(log_n: c_uint) -> size_t;
    fn falcon_tmpsize_keygen(log_n: c_uint) -> size_t;
    fn falcon_keygen_make(
        rng: *mut PRNGContext,
        log_n: c_uint,
        priv_key: *mut c_void,
        priv_key_len: size_t,
        pub_key: *mut c_void,
        pub_key_len: size_t,
        tmp: *mut c_void,
        tmp_len: size_t,
    ) -> c_int;
    fn prng_init(ctx: *mut PRNGContext);
    fn prng_init_prng_from_system(ctx: *mut PRNGContext) -> c_int;
    fn prng_inject(ctx: *mut PRNGContext, data: *const c_void, data_len: size_t);
    fn prng_flip(ctx: *mut PRNGContext);
}

pub fn private_key_size(log_n: u32) -> usize {
    unsafe { falcon_privkey_size(log_n) }
}

pub fn public_key_size(log_n: u32) -> usize {
    unsafe { falcon_pubkey_size(log_n) }
}

pub fn tmp_size_keygen(log_n: u32) -> usize {
    unsafe { falcon_tmpsize_keygen(log_n) }
}

fn generate_keypair_from_seed(log_n: u32, seed: &[u8]) -> Result<KeyPair, FalconError> {
    if !(1..=10).contains(&log_n) {
        return Err(FalconError::BadArg);
    }

    let priv_key_size = private_key_size(log_n);
    let pub_key_size = public_key_size(log_n);
    let tmp_size = tmp_size_keygen(log_n);

    let mut priv_key = vec![0u8; priv_key_size];
    let mut pub_key = vec![0u8; pub_key_size];
    let mut tmp = vec![0u8; tmp_size];

    let mut rng = PRNGContext { state: [0; 256] };

    if seed.is_empty() {
        let result = unsafe { prng_init_prng_from_system(&mut rng) };
        if result != 0 {
            return Err(FalconError::from(result));
        }
    } else {
        unsafe {
            prng_init(&mut rng);
            prng_inject(&mut rng, seed.as_ptr() as *const c_void, seed.len());
            prng_flip(&mut rng);
        }
    }

    let result = unsafe {
        falcon_keygen_make(
            &mut rng,
            log_n,
            priv_key.as_mut_ptr() as *mut c_void,
            priv_key_size,
            pub_key.as_mut_ptr() as *mut c_void,
            pub_key_size,
            tmp.as_mut_ptr() as *mut c_void,
            tmp_size,
        )
    };

    if result != 0 {
        return Err(FalconError::from(result));
    }

    Ok(KeyPair {
        public_key: pub_key,
        private_key: priv_key,
    })
} 
