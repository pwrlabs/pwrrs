use pwr_rs::{
    RPC,
    transaction::types::VidaDataTransaction,
};
use std::sync::Arc;

fn handler(txn: VidaDataTransaction) {
    // Get the address of the transaction sender
    let sender = txn.sender;
    // Get the data sent in the transaction (In Hex Format)
    let data = txn.data;
    // Convert data string to bytes
    let data_str = String::from_utf8(data).unwrap();
    let object: serde_json::Value = serde_json::from_str(&data_str).unwrap();
    let obj_map = object.as_object().unwrap();

    // Check the action and execute the necessary code
    if obj_map.get("action").and_then(|val| val.as_str()) == Some("send-message-v1")
    {
        if let Some(message_str) = obj_map
            .get("message")
            .and_then(|val| val.as_str())
        {
            println!("Message from {}: {}", sender, message_str);
        }
    }
}

#[tokio::main]
async fn main() {
    let rpc = RPC::new("http://46.101.151.203:8085/").await.unwrap();
    let rpc = Arc::new(rpc);
    let vida_id = 1; // Replace with your VIDA's ID

    // Since our VIDA is global chat room and we don't care about historical messages,
    // we will start reading transactions startng from the latest PWR Chain block
    let starting_block = rpc.get_latest_block_number().await.unwrap();

    let subscription = rpc.subscribe_to_vida_transactions(vida_id, starting_block, handler);

    // To pause, resume, and stop the subscription
    subscription.pause();
    subscription.resume();
    // subscription.stop();

    // To get the latest checked block
    println!("Latest checked block: {}", subscription.get_latest_checked_block());

    // To exit the program
    if subscription.is_running() {
        println!("Press Enter to exit...");
        let mut buffer = String::new();
        std::io::stdin().read_line(&mut buffer).unwrap();
    }
}
