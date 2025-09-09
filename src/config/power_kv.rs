use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

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
            server_url: "https://pwrnosqlvida.pwrlabs.io/".to_string(),
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

    pub async fn put(&self, key: &[u8], data: &[u8]) -> Result<bool, PowerKvError> {
        let url = format!("{}/storeData", self.server_url);
        let payload = StoreDataRequest {
            project_id: self.project_id.clone(),
            secret: self.secret.clone(),
            key: self.to_hex_string(key),
            value: self.to_hex_string(data),
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
            let message = if let Ok(error_response) = serde_json::from_str::<ErrorResponse>(&response_text) {
                error_response.message.unwrap_or_else(|| format!("HTTP {}", status.as_u16()))
            } else {
                format!("HTTP {} — {}", status.as_u16(), response_text)
            };
            Err(PowerKvError::ServerError(format!("storeData failed: {}", message)))
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
        let key_hex = self.to_hex_string(key);
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
            
            self.from_hex_string(&response_obj.value)
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
