//! PWR chain Rust SDK
//!
//! # Create wallet
//! ```ignore
//! use pwr_rs::Wallet;
//! let wallet = Wallet::new();
//! ```
//!
//! # Create RPC
//! ```ignore
//! use pwr_rs::Wallet;
//! use pwr_rs::RPC;
//! let wallet = Wallet::new();
//! let rpc = RPC::new("https://pwrrpc.pwrlabs.io/").unwrap();
//! let balance = rpc.balance_of_address(&wallet.get_address()).await.unwrap();
//! ```

pub mod block;
pub mod transaction;
pub mod delegator;
pub mod validator;
#[cfg(feature = "rpc")]
pub mod rpc;
pub mod wallet;

mod config;

pub use wallet::types::{PublicKey, Wallet};
pub use rpc::RPC;
