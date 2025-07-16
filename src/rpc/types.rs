use reqwest::{Client};
use serde::{Deserialize, Serialize};
use url::Url;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use crate::transaction::types::VidaDataTransaction;
use std::future::Future;
use std::pin::Pin;

pub struct RPC {
    pub http_client: Client,
    pub node_url: Url,
    pub chain_id: u8,
}

#[derive(Debug)]
pub enum RpcError {
    FailedToBroadcastTransaction(String),
    InvalidRpcUrl,
    Network(reqwest::Error),
    Deserialization(reqwest::Error),
    JsonDeserialization(String),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BroadcastResponse {
    pub success: bool,
    pub data: Option<String>,
    pub error: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ResponseData {
    pub message: String,
}

#[derive(Serialize)]
pub struct BroadcastRequest {
    pub txn: String,
}

/// Trait for block saving functionality that supports both sync and async implementations
pub trait BlockSaver: Send + Sync {
    fn save_block(&self, block_number: u64) -> Pin<Box<dyn Future<Output = ()> + Send + '_>>;
}

/// Implementation for synchronous functions that return nothing
impl<F> BlockSaver for F
where
    F: Fn(u64) + Send + Sync,
{
    fn save_block(&self, block_number: u64) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        self(block_number);
        Box::pin(async move { })
    }
}

/// Wrapper for async functions
pub struct AsyncBlockSaver<F, Fut>
where
    F: Fn(u64) -> Fut + Send + Sync,
    Fut: Future<Output = ()> + Send,
{
    func: F,
}

impl<F, Fut> AsyncBlockSaver<F, Fut>
where
    F: Fn(u64) -> Fut + Send + Sync,
    Fut: Future<Output = ()> + Send,
{
    pub fn new(func: F) -> Self {
        Self { func }
    }
}

impl<F, Fut> BlockSaver for AsyncBlockSaver<F, Fut>
where
    F: Fn(u64) -> Fut + Send + Sync,
    Fut: Future<Output = ()> + Send + 'static,
{
    fn save_block(&self, block_number: u64) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        Box::pin((self.func)(block_number))
    }
}

/// Convenience functions for creating BlockSaver instances
pub mod block_saver {
    use super::*;
    use std::future::Future;

    /// Create a BoxedBlockSaver from a synchronous function
    /// 
    /// # Example
    /// ```rust
    /// use pwr_rs::rpc::block_saver;
    /// 
    /// let sync_saver = block_saver::from_sync(|block_num| {
    ///     println!("Saving block: {}", block_num);
    ///     // Save to file, update database, etc.
    /// });
    /// ```
    pub fn from_sync<F>(func: F) -> Box<dyn BlockSaver>
    where
        F: Fn(u64) + Send + Sync + 'static,
    {
        Box::new(func)
    }

    /// Create a BoxedBlockSaver from an async function
    /// 
    /// # Example
    /// ```rust
    /// use pwr_rs::rpc::block_saver;
    /// 
    /// let async_saver = block_saver::from_async(|block_num| async move {
    ///     // Simulate async database save
    ///     tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    ///     println!("Async saving block: {}", block_num);
    /// });
    /// ```
    pub fn from_async<F, Fut>(func: F) -> Box<dyn BlockSaver>
    where
        F: Fn(u64) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        Box::new(AsyncBlockSaver::new(func))
    }
}

pub struct VidaTransactionSubscription {
    pub pwrrs: Arc<RPC>,
    pub vida_id: u64,
    pub starting_block: u64,
    pub poll_interval: u64,
    pub latest_checked_block: Arc<std::sync::atomic::AtomicU64>,
    pub handler: ProcessVidaTransactions,
    pub block_saver: Option<Box<dyn BlockSaver>>,
    pub wants_to_pause: Arc<AtomicBool>,
    pub paused: Arc<AtomicBool>,
    pub stop: Arc<AtomicBool>,
    pub running: Arc<AtomicBool>,
}

pub type ProcessVidaTransactions = fn(transaction: VidaDataTransaction);
