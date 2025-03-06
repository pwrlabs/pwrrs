use pwr_rs::{Falcon512Wallet, RPC};

#[tokio::main]
async fn main() {
    // let wallet = Falcon512Wallet::new();
    // wallet.store_wallet("falcon_wallet.dat").unwrap();

    let wallet = Falcon512Wallet::load_wallet("falcon_wallet.dat").unwrap();
    println!("Address: {:?}", wallet.get_address());

    let balance = wallet.get_balance().await;
    println!("Balance: {:?}", balance);

    let rpc = RPC::new("https://pwrrpc.pwrlabs.io/").await.unwrap();
    let fee_per_byte = rpc.get_fee_per_byte().await.unwrap();

    let tx = wallet.transfer_pwr(
        "0x5b2dee90ceec38add6d2231289be41da90fb4492".to_string(),
        1,
        fee_per_byte
    ).await;
    if tx.success {
        println!("TX Hash: {:?}", tx.data.unwrap());
    } else {
        println!("Error: {:?}", tx.error);
    }
}
