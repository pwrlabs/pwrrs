use pwr_rs::{Wallet};

#[tokio::main]
async fn main() {
    // let wallet = Wallet::new();
    // wallet.store_wallet("falcon_wallet2.dat").unwrap();

    let wallet = Wallet::load_wallet_with_rpc_url("falcon_wallet2.dat", "http://46.101.151.203:8085/").unwrap();
    
    println!("Address: {:?}", wallet.get_address());

    let balance = wallet.get_balance().await;
    println!("Balance: {:?}", balance);

    let nonce = wallet.get_nonce().await;
    println!("Nonce: {:?}", nonce);

    let fee_per_byte = (wallet.get_rpc().await).get_fee_per_byte().await.unwrap();

    // let receiver = "0xd97c25c0842704588dd70a061c09a522699e2b9c";


    let json_object = serde_json::json!({
        "action": "send-message-v1",
        "message": "Fuck!"
    });
    let data: Vec<u8> = serde_json::to_string(&json_object).unwrap().into_bytes();

    let response = wallet.send_vida_data(1, data, fee_per_byte).await;
    if response.success {
        println!("Transaction sent successfully!");
        println!("Transaction hash: {:?}", response.data.unwrap());
    } else {
        println!("Transaction failed: {:?}", response.error);
    }

    // let mut tx = wallet.transfer_pwr(
    //     "0xd97c25c0842704588dd70a061c09a522699e2b9c".to_string(),
    //     1,
    //     fee_per_byte
    // ).await;
    // if tx.success {
    //     println!("TX Hash: {:?}", tx.data.unwrap());
    // } else {
    //     println!("Error: {:?}", tx.error);
    // }

    // let data: Vec<u8> = vec!["Hello World!"].into_iter()
    //     .flat_map(|s| s.as_bytes().to_vec())
    //     .collect();
    // tx = wallet.send_vida_data(
    //     1,
    //     data,
    //     fee_per_byte
    // ).await;
    // if tx.success {
    //     println!("TX Hash: {:?}", tx.data.unwrap());
    // } else {
    //     println!("Error: {:?}", tx.error);
    // }
}
