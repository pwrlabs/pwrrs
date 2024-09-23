use pwr_rs::{
    Wallet, RPC 
};

#[tokio::main]
async fn main() {
    let private_key = "0x04828e90065864c111871769c601d7de2246570b39dd37c19ccac16c14b18f72";
    let wallet = Wallet::from_hex(&private_key).unwrap();

    let address = wallet.get_address();
    println!("Address: {address}");

    let nonce = wallet.get_nonce().await;
    println!("Nonce: {nonce}");

    let balance = wallet.get_balance().await;
    println!("Balance: {balance}");

    #[cfg(feature = "rpc")]
    {
        let rpc = RPC::new("https://pwrrpc.pwrlabs.io/").await.unwrap();

        let blocks_count = rpc.block_count().await.unwrap();
        println!("BlocksCount: {blocks_count}");

        let latest_block_count = rpc.latest_block_count().await.unwrap();
        println!("LatestBlockCount: {latest_block_count}");

        let fuck = rpc.guardian_of_address("0xD97C25C0842704588DD70A061C09A522699E2B9C").await.unwrap();
        println!("Guardian: {fuck}");

        let transactions = rpc.vm_data_transactions(836599, 836600, 69).await.unwrap();
        println!("Transactions: {:?}", transactions);
        
        let block = rpc.block_by_number(836599).await.unwrap();
        println!("{:?}", block);

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

        let trx_hash = wallet.transfer_pwr(
            1000,
            "3B3B69093879E7B6F28366FA3C32762590FF547E".into(),
        ).await;
        println!("Transaction Hash: {trx_hash}");

        let data = vec!["Hello World!"];
        let data_as_bytes: Vec<u8> = data.into_iter()
            .flat_map(|s| s.as_bytes().to_vec())
            .collect();
        let tx_hash = wallet.send_vm_data(1234, data_as_bytes).await;
        println!("Transaction Hash: {tx_hash}");

        let data = vec!["Hello World!"];
        let data_as_bytes: Vec<u8> = data.into_iter()
            .flat_map(|s| s.as_bytes().to_vec())
            .collect();
        let tx_hash = wallet.send_payable_vm_data(1234, data_as_bytes, 1000).await;
        println!("Transaction Hash: {tx_hash}");
    }
}
