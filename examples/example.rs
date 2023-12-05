use pwr_rs::{block::NewTransactionData, rpc::RPC, wallet::PrivateKey};

#[tokio::main]
async fn main() {
    let private_key = PrivateKey::random();

    let address = private_key.address();
    println!("Address: {address}");
    let rpc = RPC::new("https://pwrrpc.pwrlabs.io/").unwrap();

    let nonce = rpc.nonce_of_address(&address).await.unwrap();
    println!("Nonce: {nonce}");

    let balance = rpc.balance_of_address(&address).await.unwrap();
    println!("Balance: {balance}");

    let blocks_count = rpc.block_count().await.unwrap();
    println!("BlocksCount: {blocks_count}");

    let latest_block_count = rpc.latest_block_count().await.unwrap();
    println!("LatestBlockCount: {latest_block_count}");

    let block = rpc.block_by_number(1337).await.unwrap();
    println!("Block: {block:?}");

    let active_voting_power = rpc.active_voting_power().await.unwrap();
    println!("ActiveVotingPower: {active_voting_power}");

    let total_validators_count = rpc.total_validator_count().await.unwrap();
    println!("TotalValidatorsCount: {total_validators_count}");

    let standby_validators_count = rpc.standby_validator_count().await.unwrap();
    println!("StandbyValidatorsCount: {standby_validators_count}");

    let active_validators_count = rpc.active_validator_count().await.unwrap();
    println!("ActiveValidatorsCount: {active_validators_count}");

    let all_validators = rpc.all_validators().await.unwrap();
    println!("AllValidators: {all_validators:?}");

    let standby_validators = rpc.standby_validators().await.unwrap();
    println!("StandbyValidators: {standby_validators:?}");

    let active_validators = rpc.active_validators().await.unwrap();
    println!("ActiveValidators: {active_validators:?}");

    let owner_of_vm = rpc.owner_of_vm(1337).await.unwrap();
    println!("OwnerOfVM: {owner_of_vm}");

    let new_trx = NewTransactionData::Transfer {
        amount: 10,
        recipient: "61bd8fc1e30526aaf1c4706ada595d6d236d9883".into(),
    };
    let trx_hash = rpc
        .broadcast_transaction(&new_trx, &private_key)
        .await
        .unwrap();
    println!("Transaction Hash: {trx_hash}");
}
