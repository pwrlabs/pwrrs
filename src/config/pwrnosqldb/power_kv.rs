use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use sha3::{Digest, Keccak256};
use crate::config::aes256::AES256;

#[derive(Debug)]
pub enum PowerKvError {
    InvalidInput(String),
    NetworkError(String),
    ServerError(String),
    HexDecodeError(String),
}

impl std::fmt::Display for PowerKvError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PowerKvError::InvalidInput(msg) => write!(f, "Invalid input: {}", msg),
            PowerKvError::NetworkError(msg) => write!(f, "Network error: {}", msg),
            PowerKvError::ServerError(msg) => write!(f, "Server error: {}", msg),
            PowerKvError::HexDecodeError(msg) => write!(f, "Hex decode error: {}", msg),
        }
    }
}

impl std::error::Error for PowerKvError {}

#[derive(Serialize)]
struct StoreDataRequest {
    #[serde(rename = "projectId")]
    project_id: String,
    secret: String,
    key: String,
    value: String,
}

#[derive(Deserialize)]
struct GetValueResponse {
    value: String,
}

#[derive(Deserialize)]
struct ErrorResponse {
    message: Option<String>,
}

pub struct PowerKv {
    pub client: Client,
    pub server_url: String,
    pub project_id: String,
    pub secret: String,
}

impl PowerKv {
    pub fn new(project_id: String, secret: String) -> Result<Self, PowerKvError> {
        if project_id.trim().is_empty() {
            return Err(PowerKvError::InvalidInput(
                "Project ID cannot be null or empty".to_string(),
            ));
        }
        if secret.trim().is_empty() {
            return Err(PowerKvError::InvalidInput(
                "Secret cannot be null or empty".to_string(),
            ));
        }

        let client = Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .map_err(|e| PowerKvError::NetworkError(format!("Failed to create HTTP client: {}", e)))?;

        Ok(PowerKv {
            client,
            server_url: "https://powerkvbe.pwrlabs.io".to_string(),
            project_id,
            secret,
        })
    }

    pub fn get_server_url(&self) -> &str {
        &self.server_url
    }

    pub fn get_project_id(&self) -> &str {
        &self.project_id
    }

    fn to_hex_string(&self, data: &[u8]) -> String {
        hex::encode(data)
    }

    fn from_hex_string(&self, hex_string: &str) -> Result<Vec<u8>, PowerKvError> {
        let hex_str = if hex_string.starts_with("0x") || hex_string.starts_with("0X") {
            &hex_string[2..]
        } else {
            hex_string
        };

        hex::decode(hex_str).map_err(|e| PowerKvError::HexDecodeError(format!("Invalid hex: {}", e)))
    }

    fn to_bytes(&self, data: &dyn std::fmt::Display) -> Vec<u8> {
        data.to_string().into_bytes()
    }

    fn hash256(&self, input: &[u8]) -> Vec<u8> {
        let mut hasher = Keccak256::new();
        hasher.update(input);
        hasher.finalize().to_vec()
    }

    fn pack_data(&self, key: &[u8], data: &[u8]) -> Vec<u8> {
        let mut packed = Vec::new();
        
        // Pack key length (4 bytes, big-endian) + key bytes
        packed.extend_from_slice(&(key.len() as u32).to_be_bytes());
        packed.extend_from_slice(key);
        
        // Pack data length (4 bytes, big-endian) + data bytes
        packed.extend_from_slice(&(data.len() as u32).to_be_bytes());
        packed.extend_from_slice(data);
        
        packed
    }

    fn unpack_data(&self, packed_buffer: &[u8]) -> Result<(Vec<u8>, Vec<u8>), PowerKvError> {
        if packed_buffer.len() < 8 {
            return Err(PowerKvError::InvalidInput("Buffer too small for unpacking".to_string()));
        }

        let mut offset = 0;
        
        // Read key length (4 bytes, big-endian)
        let key_length = u32::from_be_bytes([
            packed_buffer[offset],
            packed_buffer[offset + 1],
            packed_buffer[offset + 2],
            packed_buffer[offset + 3],
        ]) as usize;
        offset += 4;
        
        if offset + key_length > packed_buffer.len() {
            return Err(PowerKvError::InvalidInput("Invalid key length in packed data".to_string()));
        }
        
        // Read key bytes
        let key = packed_buffer[offset..offset + key_length].to_vec();
        offset += key_length;
        
        if offset + 4 > packed_buffer.len() {
            return Err(PowerKvError::InvalidInput("Buffer too small for data length".to_string()));
        }
        
        // Read data length (4 bytes, big-endian)
        let data_length = u32::from_be_bytes([
            packed_buffer[offset],
            packed_buffer[offset + 1],
            packed_buffer[offset + 2],
            packed_buffer[offset + 3],
        ]) as usize;
        offset += 4;
        
        if offset + data_length > packed_buffer.len() {
            return Err(PowerKvError::InvalidInput("Invalid data length in packed data".to_string()));
        }
        
        // Read data bytes
        let data = packed_buffer[offset..offset + data_length].to_vec();
        
        Ok((key, data))
    }

    pub async fn put(&self, key: &[u8], data: &[u8]) -> Result<bool, PowerKvError> {
        // Hash the key with Keccak256
        let key_hash = self.hash256(key);
        
        // Pack the original key and data
        let packed_data = self.pack_data(key, data);
        
        // Encrypt the packed data
        let encrypted_data = AES256::encrypt(&packed_data, &self.secret)
            .map_err(|e| PowerKvError::ServerError(format!("Encryption failed: {:?}", e)))?;

        let url = format!("{}/storeData", self.server_url);
        let payload = StoreDataRequest {
            project_id: self.project_id.clone(),
            secret: self.secret.clone(),
            key: self.to_hex_string(&key_hash),
            value: self.to_hex_string(&encrypted_data),
        };

        let response = self
            .client
            .post(&url)
            .json(&payload)
            .send()
            .await
            .map_err(|e| PowerKvError::NetworkError(format!("Request failed: {}", e)))?;

        let status = response.status();
        let response_text = response
            .text()
            .await
            .map_err(|e| PowerKvError::NetworkError(format!("Failed to read response: {}", e)))?;

        if status.is_success() {
            Ok(true)
        } else {
            Err(PowerKvError::ServerError(format!("storeData failed: {} - {}", status.as_u16(), response_text)))
        }
    }

    pub async fn put_string(&self, key: &str, data: &str) -> Result<bool, PowerKvError> {
        self.put(key.as_bytes(), data.as_bytes()).await
    }

    pub async fn put_number<T: std::fmt::Display>(&self, key: &T, data: &T) -> Result<bool, PowerKvError> {
        let key_bytes = self.to_bytes(key);
        let data_bytes = self.to_bytes(data);
        self.put(&key_bytes, &data_bytes).await
    }

    pub async fn get_value(&self, key: &[u8]) -> Result<Vec<u8>, PowerKvError> {
        // Hash the key with Keccak256
        let key_hash = self.hash256(key);
        let key_hex = self.to_hex_string(&key_hash);
        let url = format!("{}/getValue", self.server_url);

        let mut params = HashMap::new();
        params.insert("projectId", &self.project_id);
        params.insert("key", &key_hex);

        let response = self
            .client
            .get(&url)
            .query(&params)
            .send()
            .await
            .map_err(|e| PowerKvError::NetworkError(format!("Request failed: {}", e)))?;

        let status = response.status();
        let response_text = response
            .text()
            .await
            .map_err(|e| PowerKvError::NetworkError(format!("Failed to read response: {}", e)))?;

        if status.is_success() {
            let response_obj: GetValueResponse = serde_json::from_str(&response_text)
                .map_err(|_| PowerKvError::ServerError(format!("Unexpected response shape from /getValue: {}", response_text)))?;
            
            // Handle both with/without 0x prefix
            let clean_hex = if response_obj.value.starts_with("0x") || response_obj.value.starts_with("0X") {
                &response_obj.value[2..]
            } else {
                &response_obj.value
            };
            
            let encrypted_value = self.from_hex_string(clean_hex)?;
            
            // Decrypt the data
            let decrypted_data = AES256::decrypt(&encrypted_value, &self.secret)
                .map_err(|e| PowerKvError::ServerError(format!("Decryption failed: {:?}", e)))?;
            
            // Unpack the data to get original key and data
            let (_original_key, actual_data) = self.unpack_data(&decrypted_data)?;
            
            Ok(actual_data)
        } else {
            let message = if let Ok(error_response) = serde_json::from_str::<ErrorResponse>(&response_text) {
                error_response.message.unwrap_or_else(|| format!("HTTP {}", status.as_u16()))
            } else {
                format!("HTTP {} — {}", status.as_u16(), response_text)
            };
            Err(PowerKvError::ServerError(format!("getValue failed: {}", message)))
        }
    }

    pub async fn get_value_string(&self, key: &str) -> Result<Vec<u8>, PowerKvError> {
        self.get_value(key.as_bytes()).await
    }

    pub async fn get_value_number<T: std::fmt::Display>(&self, key: &T) -> Result<Vec<u8>, PowerKvError> {
        let key_bytes = self.to_bytes(key);
        self.get_value(&key_bytes).await
    }

    pub async fn get_string_value(&self, key: &[u8]) -> Result<String, PowerKvError> {
        let data = self.get_value(key).await?;
        String::from_utf8(data).map_err(|e| PowerKvError::ServerError(format!("Invalid UTF-8: {}", e)))
    }

    pub async fn get_string_value_from_str(&self, key: &str) -> Result<String, PowerKvError> {
        self.get_string_value(key.as_bytes()).await
    }

    pub async fn get_int_value(&self, key: &[u8]) -> Result<i32, PowerKvError> {
        let data = self.get_value(key).await?;
        let str_value = String::from_utf8(data).map_err(|e| PowerKvError::ServerError(format!("Invalid UTF-8: {}", e)))?;
        str_value.parse::<i32>().map_err(|e| PowerKvError::ServerError(format!("Invalid integer: {}", e)))
    }

    pub async fn get_int_value_from_str(&self, key: &str) -> Result<i32, PowerKvError> {
        self.get_int_value(key.as_bytes()).await
    }

    pub async fn get_long_value(&self, key: &[u8]) -> Result<i64, PowerKvError> {
        let data = self.get_value(key).await?;
        let str_value = String::from_utf8(data).map_err(|e| PowerKvError::ServerError(format!("Invalid UTF-8: {}", e)))?;
        str_value.parse::<i64>().map_err(|e| PowerKvError::ServerError(format!("Invalid long: {}", e)))
    }

    pub async fn get_long_value_from_str(&self, key: &str) -> Result<i64, PowerKvError> {
        self.get_long_value(key.as_bytes()).await
    }

    pub async fn get_double_value(&self, key: &[u8]) -> Result<f64, PowerKvError> {
        let data = self.get_value(key).await?;
        let str_value = String::from_utf8(data).map_err(|e| PowerKvError::ServerError(format!("Invalid UTF-8: {}", e)))?;
        str_value.parse::<f64>().map_err(|e| PowerKvError::ServerError(format!("Invalid double: {}", e)))
    }

    pub async fn get_double_value_from_str(&self, key: &str) -> Result<f64, PowerKvError> {
        self.get_double_value(key.as_bytes()).await
    }
}
