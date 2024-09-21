## PWR Chain Rust SDK

PWR Rust is a Rust library for interacting with the PWR network.
It provides an easy interface for wallet management and sending transactions on PWR.

<div align="center">
<!-- markdownlint-restore -->

[![Pull Requests welcome](https://img.shields.io/badge/PRs-welcome-ff69b4.svg?style=flat-square)](https://github.com/pwrlabs/pwrrs/issues?q=is%3Aissue+is%3Aopen+label%3A%22help+wanted%22)
[![crates-badge](https://img.shields.io/crates/v/pwr-rs.svg)](https://crates.io/crates/pwr-rs)
<a href="https://github.com/pwrlabs/pwrrs/blob/main/LICENSE/">
  <img src="https://img.shields.io/badge/license-MIT-black">
</a>
<!-- <a href="https://github.com/pwrlabs/pwrrs/stargazers">
  <img src='https://img.shields.io/github/stars/pwrlabs/pwrrs?color=yellow' />
</a> -->
<a href="https://pwrlabs.io/">
  <img src="https://img.shields.io/badge/powered_by-PWR Chain-navy">
</a>
<a href="https://www.youtube.com/@pwrlabs">
  <img src="https://img.shields.io/badge/Community%20calls-Youtube-red?logo=youtube"/>
</a>
<a href="https://twitter.com/pwrlabs">
  <img src="https://img.shields.io/twitter/follow/pwrlabs?style=social"/>
</a>

</div>

## Installation

```bash
# latest official release (main branch)
cargo add pwr-rs
```

## 🌐 Documentation

How to [Guides](https://pwrlabs.io) 🔜 & [API](https://pwrlabs.io) 💻

Play with [Code Examples](https://github.com/keep-pwr-strong/pwr-components/) 🎮

## 💫 Getting Started

**Import the library:**

```rust
use pwr_rs::{
    Wallet, 
    RPC, 
};
```

**Set your RPC node:**

```rust
let rpc = RPC::new("https://pwrrpc.pwrlabs.io/").await.unwrap();
```

**Generate a new wallet:**

```rust
let private_key = "0xac0974bec...f80";
let wallet = Wallet::from_hex(&private_key).unwrap();
```

**Get wallet address:**

```rust
let address = wallet.address();
```

**Get wallet balance:**

```rust
let balance = wallet.get_balance().await;
```

**Transfer PWR tokens:**

```rust
/// recipientAddress: 20 bytes address without `0x`
wallet.transfer_pwr(1000, "recipientAddress".into()).await;
```

Sending a transcation to the PWR Chain returns a Response object, which specified if the transaction was a success, and returns relevant data.
If the transaction was a success, you can retrieive the transaction hash, if it failed, you can fetch the error.

```rust
use pwr_rs::Wallet;
async fn main() {
    let private_key = "0xac0974bec...f80";
    let wallet = Wallet::from_hex(&private_key).unwrap();

    /// recipientAddress: 20 bytes address without `0x`
    let trx_hash = wallet.transfer_pwr(1000, "recipientAddress".into()).await;
    println!("Transaction Hash: {trx_hash}");
}
```

**Send data to a VM:**

```rust
use pwr_rs::Wallet;
async fn main() {
    let private_key = "0xac0974bec...f80";
    let wallet = Wallet::from_hex(&private_key).unwrap();

    let data = vec!["Hello World!"];
    let data_as_bytes: Vec<u8> = data.into_iter().flat_map(|s| s.as_bytes().to_vec()).collect();

    let tx_hash = wallet.send_vm_data(123, data_as_bytes).await;
    println!("Transaction Hash: {tx_hash}");
}
```

### Other Static Calls

**Get RPC Node Url:**

Returns currently set RPC node URL.

```rust
let url = rpc.node_url();
```

**Get Fee Per Byte: **

Gets the latest fee-per-byte rate.

```rust
let fee = rpc.fee_per_byte();
```

**Get Balance Of Address:**

Gets the balance of a specific address.

```rust
let balance = rpc.balance_of_address("0x...").await.unwrap();
```

**Get Nonce Of Address:**

Gets the nonce/transaction count of a specific address.

```rust
let nonce = rpc.nonce_of_address("0x...").await.unwrap();
```

## ✏️ Contributing

If you consider to contribute to this project please read [CONTRIBUTING.md](https://github.com/pwrlabs/pwrrs/blob/main/CONTRIBUTING.md) first.

You can also join our dedicated channel for [pwr-rs](https://discord.com/channels/1141787507189624992/1180224756033790014) on the [PWR Chain Discord](https://discord.com/invite/YASmBk9EME)

## 📜 License

Copyright (c) 2024 PWR Labs

Licensed under the [MIT license](https://github.com/pwrlabs/pwrrs/blob/main/LICENSE).
