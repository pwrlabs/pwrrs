use super::power_kv::{PowerKv, PowerKvError};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::task::JoinHandle;
use tokio::time::sleep;
use log::{info, warn, error};

#[derive(Clone, PartialEq, Eq, Hash)]
struct ByteArrayWrapper(Vec<u8>);

impl ByteArrayWrapper {
    fn new(data: Vec<u8>) -> Self {
        ByteArrayWrapper(data)
    }

    #[allow(dead_code)]
    fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

pub struct PowerKvCached {
    db: PowerKv,
    cache: Arc<Mutex<HashMap<ByteArrayWrapper, Vec<u8>>>>,
    is_shutdown: Arc<Mutex<bool>>,
    active_writes: Arc<Mutex<Vec<JoinHandle<()>>>>,
}

impl PowerKvCached {
    pub fn new(project_id: String, secret: String) -> Result<Self, PowerKvError> {
        let db = PowerKv::new(project_id, secret)?;
        
        Ok(PowerKvCached {
            db,
            cache: Arc::new(Mutex::new(HashMap::new())),
            is_shutdown: Arc::new(Mutex::new(false)),
            active_writes: Arc::new(Mutex::new(Vec::new())),
        })
    }

    fn to_bytes(&self, data: &dyn std::fmt::Display) -> Vec<u8> {
        data.to_string().into_bytes()
    }

    pub fn put(&self, key: &[u8], value: &[u8]) -> Result<(), PowerKvError> {
        {
            let is_shutdown = self.is_shutdown.lock().unwrap();
            if *is_shutdown {
                return Err(PowerKvError::InvalidInput("PowerKvCached has been shut down".to_string()));
            }
        }

        let key_wrapper = ByteArrayWrapper::new(key.to_vec());
        let value_vec = value.to_vec();

        let old_value = {
            let mut cache = self.cache.lock().unwrap();
            cache.insert(key_wrapper.clone(), value_vec.clone())
        };

        // If oldValue is same as new value, no need to update db
        // If oldValue is None, it means this key is being inserted for the first time, so we need to update db
        if old_value.is_none() || old_value.as_ref() != Some(&value_vec) {
            // Start background write (non-blocking)
            self.start_background_write(key.to_vec(), value_vec, key_wrapper);
        }

        Ok(())
    }

    pub fn put_string(&self, key: &str, value: &str) -> Result<(), PowerKvError> {
        self.put(key.as_bytes(), value.as_bytes())
    }

    pub fn put_number<T: std::fmt::Display>(&self, key: &T, value: &T) -> Result<(), PowerKvError> {
        let key_bytes = self.to_bytes(key);
        let value_bytes = self.to_bytes(value);
        self.put(&key_bytes, &value_bytes)
    }

    fn start_background_write(&self, key_bytes: Vec<u8>, value_bytes: Vec<u8>, key_wrapper: ByteArrayWrapper) {
        let db = self.db.clone();
        let cache = Arc::clone(&self.cache);
        let is_shutdown = Arc::clone(&self.is_shutdown);
        let active_writes = Arc::clone(&self.active_writes);

        let handle = tokio::spawn(async move {
            Self::background_write(db, cache, is_shutdown, key_bytes, value_bytes, key_wrapper).await;
        });

        {
            let mut writes = active_writes.lock().unwrap();
            writes.push(handle);
        }
    }

    async fn background_write(
        db: PowerKv,
        cache: Arc<Mutex<HashMap<ByteArrayWrapper, Vec<u8>>>>,
        is_shutdown: Arc<Mutex<bool>>,
        key_bytes: Vec<u8>,
        value_bytes: Vec<u8>,
        key_wrapper: ByteArrayWrapper,
    ) {
        loop {
            {
                let shutdown = is_shutdown.lock().unwrap();
                if *shutdown {
                    return;
                }
            }

            let current_cached_value = {
                let cache_guard = cache.lock().unwrap();
                cache_guard.get(&key_wrapper).cloned()
            };

            // If cache is updated with different value, stop this background write
            if current_cached_value.is_none() || current_cached_value.as_ref() != Some(&value_bytes) {
                info!("Cache updated for key, stopping background write: {}", 
                      String::from_utf8_lossy(&key_bytes));
                return;
            }

            match db.put(&key_bytes, &value_bytes).await {
                Ok(true) => {
                    info!("Successfully updated key on PWR Chain: {}", 
                          String::from_utf8_lossy(&key_bytes));
                    return;
                }
                Ok(false) => {
                    warn!("Failed to update key on PWR Chain, retrying: {}", 
                          String::from_utf8_lossy(&key_bytes));
                    
                    // Check if another thread has already updated the value
                    match db.get_value(&key_bytes).await {
                        Ok(remote_value) => {
                            if remote_value == value_bytes {
                                info!("Value already updated by another process: {}", 
                                      String::from_utf8_lossy(&key_bytes));
                                return;
                            }
                        }
                        Err(_) => {
                            // Ignore errors when checking remote value
                        }
                    }
                    
                    // Wait 10ms before retry
                    sleep(Duration::from_millis(10)).await;
                }
                Err(e) => {
                    error!("Error updating key on PWR Chain: {} - {}", 
                           String::from_utf8_lossy(&key_bytes), e);
                    
                    // Wait 10ms before retry
                    sleep(Duration::from_millis(10)).await;
                }
            }
        }
    }

    pub async fn get_value(&self, key: &[u8]) -> Option<Vec<u8>> {
        let key_wrapper = ByteArrayWrapper::new(key.to_vec());
        
        // Check cache first
        {
            let cache = self.cache.lock().unwrap();
            if let Some(cached_value) = cache.get(&key_wrapper) {
                return Some(cached_value.clone());
            }
        }

        // If not in cache, fetch from remote
        match self.db.get_value(key).await {
            Ok(value) => {
                // Cache the retrieved value
                {
                    let mut cache = self.cache.lock().unwrap();
                    cache.insert(key_wrapper, value.clone());
                }
                Some(value)
            }
            Err(e) => {
                error!("Error retrieving value: {}", e);
                None
            }
        }
    }

    pub async fn get_value_string(&self, key: &str) -> Option<Vec<u8>> {
        self.get_value(key.as_bytes()).await
    }

    pub async fn get_value_number<T: std::fmt::Display>(&self, key: &T) -> Option<Vec<u8>> {
        let key_bytes = self.to_bytes(key);
        self.get_value(&key_bytes).await
    }

    pub async fn get_string_value(&self, key: &[u8]) -> Option<String> {
        let value = self.get_value(key).await?;
        String::from_utf8(value).ok()
    }

    pub async fn get_string_value_from_str(&self, key: &str) -> Option<String> {
        self.get_string_value(key.as_bytes()).await
    }

    pub async fn get_int_value(&self, key: &[u8]) -> Option<i32> {
        let value = self.get_value(key).await?;
        let str_value = String::from_utf8(value).ok()?;
        str_value.parse::<i32>().ok()
    }

    pub async fn get_int_value_from_str(&self, key: &str) -> Option<i32> {
        self.get_int_value(key.as_bytes()).await
    }

    pub async fn get_long_value(&self, key: &[u8]) -> Option<i64> {
        let value = self.get_value(key).await?;
        let str_value = String::from_utf8(value).ok()?;
        str_value.parse::<i64>().ok()
    }

    pub async fn get_long_value_from_str(&self, key: &str) -> Option<i64> {
        self.get_long_value(key.as_bytes()).await
    }

    pub async fn get_double_value(&self, key: &[u8]) -> Option<f64> {
        let value = self.get_value(key).await?;
        let str_value = String::from_utf8(value).ok()?;
        str_value.parse::<f64>().ok()
    }

    pub async fn get_double_value_from_str(&self, key: &str) -> Option<f64> {
        self.get_double_value(key.as_bytes()).await
    }

    pub async fn shutdown(&self) -> Result<(), PowerKvError> {
        info!("Shutting down PowerKvCached...");
        
        {
            let mut is_shutdown = self.is_shutdown.lock().unwrap();
            *is_shutdown = true;
        }

        // Wait for all active writes to complete
        let max_wait_time = Duration::from_secs(60);
        let start_time = std::time::Instant::now();

        loop {
            let active_count = {
                let mut writes = self.active_writes.lock().unwrap();
                // Remove completed tasks
                writes.retain(|handle| !handle.is_finished());
                writes.len()
            };

            if active_count == 0 {
                break;
            }

            if start_time.elapsed() >= max_wait_time {
                warn!("Forced shutdown with {} writes still active", active_count);
                break;
            }

            info!("Waiting for {} background writes to complete...", active_count);
            sleep(Duration::from_millis(100)).await;
        }

        info!("All background writes completed");
        Ok(())
    }
}

// Implement Clone for PowerKv to allow sharing between threads
impl Clone for PowerKv {
    fn clone(&self) -> Self {
        PowerKv {
            client: self.client.clone(),
            server_url: self.server_url.clone(),
            project_id: self.project_id.clone(),
            secret: self.secret.clone(),
        }
    }
}
