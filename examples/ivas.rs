use pwr_rs::{
    RPC,
    transaction::types::VMDataTransaction,
    rpc::tx_subscription::IvaTransactionHandler
};
use std::sync::Arc;

#[tokio::main]
async fn main() {
    let rpc = Arc::new(RPC::new("https://pwrrpc.pwrlabs.io/").await.unwrap());

    let vm_id = 1234;
    let starting_block = rpc.get_latest_block_number().await.unwrap();

    struct Handler(Box<dyn Fn(VMDataTransaction) + Send + Sync>);
    impl IvaTransactionHandler for Handler {
        fn process_iva_transactions(&self, tx: VMDataTransaction) {
            (self.0)(tx)
        }
    }

    let handler = Arc::new(Handler(Box::new(|transaction: VMDataTransaction| {
        let data = String::from_utf8(transaction.data).expect("Invalid UTF-8");
        println!("DATA: {data:?}");
    })));

    let shit = rpc.subscribe_to_iva_transactions(vm_id, starting_block, handler, None);
    println!("SHIT: {:?}", shit.get_vm_id());

    loop {
        tokio::time::sleep(std::time::Duration::from_millis(1)).await;
    }
}