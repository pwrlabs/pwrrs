use pwr_rs::{
    RPC,
    transaction::types::VMDataTransaction,
};
use std::sync::Arc;

fn handler(txn: VMDataTransaction) {
    let sender = txn.sender;
    let data = txn.data;
    let data_str = String::from_utf8(data).unwrap();
    let object: serde_json::Value = serde_json::from_str(&data_str).unwrap();
    let obj_map = object.as_object().unwrap();

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
    let rpc = Arc::new(RPC::new("https://pwrrpc.pwrlabs.io/").await.unwrap());
    let vm_id = 1;
    let starting_block = rpc.get_latest_block_number().await.unwrap();

    let subscription = rpc.subscribe_to_vida_transactions(vm_id, starting_block, handler, None);

    subscription.pause();
    subscription.resume();
    // subscription.stop();

    println!("Latest checked block: {}", subscription.get_latest_checked_block());

    if subscription.is_running() {
        println!("Press Enter to exit...");
        let mut buffer = String::new();
        std::io::stdin().read_line(&mut buffer).unwrap();
    }
}
