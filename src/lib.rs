//! PWR chain Rust SDK
//!
//! # Create wallet
//! ```
//! # use pwr_rs::wallet::Wallet;
//! let wallet = Wallet::random();
//! ```
//!
//! # Create RPC
//! ```ignore
//! # use pwr_rs::wallet::Wallet;
//! # use pwr_rs::rpc::types::RPC;
//! # let wallet = Wallet::random();
//! let rpc = RPC::new("https://pwrrpc.pwrlabs.io/").unwrap();
//! let balance = rpc.balance_of_address(&wallet.address()).await.unwrap();
//! ```

pub mod block;
pub mod transaction;
pub mod validator;
pub mod delegator;
pub mod rpc;
pub mod wallet;
