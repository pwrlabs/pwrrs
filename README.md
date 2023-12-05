## PWR chain Rust SDK

### Create wallet
```rust
let wallet = Wallet::random();
```

### Create RPC
```rust
let wallet = Wallet::random();
let rpc = RPC::new("https://pwrrpc.pwrlabs.io/").unwrap();
let balance = rpc.balance_of_address(&wallet.address()).await.unwrap();
```
