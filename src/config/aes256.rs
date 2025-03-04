use aes::cipher::{block_padding::Pkcs7, BlockDecryptMut, BlockEncryptMut, KeyIvInit};
use pbkdf2::pbkdf2_hmac_array;
use rand::RngCore;
use sha2::Sha256;

type Aes256CbcEnc = cbc::Encryptor<aes::Aes256>;
type Aes256CbcDec = cbc::Decryptor<aes::Aes256>;

const ITERATION_COUNT: u32 = 65536;
const KEY_LENGTH: usize = 32; // 256 bits
const SALT: &[u8] = b"your-salt-value";
const IV_LENGTH: usize = 16;

#[derive(Debug)]
#[allow(dead_code)]
pub enum CryptoError {
    EncryptionError(String),
    DecryptionError(String),
}

pub struct AES256;

impl AES256 {
    pub fn encrypt(data: &[u8], password: &str) -> Result<Vec<u8>, CryptoError> {
        let key = pbkdf2_hmac_array::<Sha256, KEY_LENGTH>(
            password.as_bytes(),
            SALT,
            ITERATION_COUNT,
        );

        let mut iv = [0u8; IV_LENGTH];
        rand::thread_rng().fill_bytes(&mut iv);

        let encryptor = Aes256CbcEnc::new(&key.into(), &iv.into());

        let mut buffer = vec![0u8; data.len() + 16]; // AES block size is 16 bytes
        let ciphertext = encryptor
            .encrypt_padded_b2b_mut::<Pkcs7>(data, &mut buffer)
            .map_err(|e| CryptoError::EncryptionError(e.to_string()))?;

        let mut result = Vec::with_capacity(IV_LENGTH + ciphertext.len());
        result.extend_from_slice(&iv);
        result.extend_from_slice(ciphertext);
        
        Ok(result)
    }

    pub fn decrypt(encrypted_data_with_iv: &[u8], password: &str) -> Result<Vec<u8>, CryptoError> {
        if encrypted_data_with_iv.len() <= IV_LENGTH {
            return Err(CryptoError::DecryptionError("Invalid input length".to_string()));
        }

        let key = pbkdf2_hmac_array::<Sha256, KEY_LENGTH>(
            password.as_bytes(),
            SALT,
            ITERATION_COUNT,
        );

        let (iv, encrypted_data) = encrypted_data_with_iv.split_at(IV_LENGTH);
        let decryptor = Aes256CbcDec::new(&key.into(), iv.into());

        let mut buffer = vec![0u8; encrypted_data.len()];
        let plaintext = decryptor
            .decrypt_padded_b2b_mut::<Pkcs7>(encrypted_data, &mut buffer)
            .map_err(|e| CryptoError::DecryptionError(e.to_string()))?;

        Ok(plaintext.to_vec())
    }
}
