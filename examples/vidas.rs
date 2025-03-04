use pwr_rs::{
    RPC,
    transaction::types::VMDataTransaction,
    rpc::tx_subscription::VidaTransactionHandler
};
use std::sync::Arc;
use serde_json::Value;

#[tokio::main]
async fn main() {
    let rpc = Arc::new(RPC::new("https://pwrrpc.pwrlabs.io/").await.unwrap());

    let vm_id = 1234;
    let starting_block = rpc.get_latest_block_number().await.unwrap();

    struct Handler(Box<dyn Fn(VMDataTransaction) + Send + Sync>);
    impl VidaTransactionHandler for Handler {
        fn process_vida_transactions(&self, tx: VMDataTransaction) {
            (self.0)(tx)
        }
    }

    let handler = Arc::new(Handler(Box::new(|txn: VMDataTransaction| {
        let sender = txn.sender;
        let data = txn.data;
        let data_str = String::from_utf8(data).unwrap();
        let object: Value = serde_json::from_str(&data_str).unwrap();
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
    })));

    rpc.subscribe_to_vida_transactions(vm_id, starting_block, handler, None);

    loop {
        tokio::time::sleep(std::time::Duration::from_millis(1)).await;
    }
}