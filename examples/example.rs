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

        let blocks_count = rpc.get_block_count().await.unwrap();
        println!("BlocksCount: {blocks_count}");

        let latest_block_count = rpc.get_latest_block_count().await.unwrap();
        println!("LatestBlockCount: {latest_block_count}");

        let start_block = 65208;
        let end_block = 65210;
        let vm_id = 1234;
        let transactions = rpc.get_vm_data_transactions(start_block, end_block, vm_id).await.unwrap();
        println!("VMData: {:?}", transactions);

        let guardian = rpc.get_guardian_of_address("0xD97C25C0842704588DD70A061C09A522699E2B9C").await.unwrap();
        println!("Guardian: {guardian}");
        
        let block = rpc.get_block_by_number(65220).await.unwrap();
        println!("{:?}", block);

        let active_voting_power = rpc.get_active_voting_power().await.unwrap();
        println!("ActiveVotingPower: {active_voting_power}");

        let conduits_vm = rpc.get_conduits_of_vm(69).await.unwrap();
        println!("ConduitsVM: {:?}", conduits_vm);

        let total_validators_count = rpc.get_validators_count().await.unwrap();
        println!("TotalValidatorsCount: {total_validators_count}");

        let standby_validators_count = rpc.get_standby_validator_count().await.unwrap();
        println!("StandbyValidatorsCount: {standby_validators_count}");

        let active_validators_count = rpc.get_active_validator_count().await.unwrap();
        println!("ActiveValidatorsCount: {active_validators_count}");

        let all_validators = rpc.get_all_validators().await.unwrap();
        println!("AllValidators: {all_validators:?}");

        let standby_validators = rpc.get_standby_validators().await.unwrap();
        println!("StandbyValidators: {standby_validators:?}");

        let active_validators = rpc.get_active_validators().await.unwrap();
        println!("ActiveValidators: {active_validators:?}");

        let trx_hash = wallet.transfer_pwr(
            "0x3B3B69093879E7B6F28366FA3C32762590FF547E".into(),
            1000,
        ).await;
        println!("Transfer tx hash: {trx_hash}");

        let data = vec!["Hello World!"];
        let data_as_bytes: Vec<u8> = data.into_iter()
            .flat_map(|s| s.as_bytes().to_vec())
            .collect();
        let tx_hash = wallet.send_vm_data(1234, data_as_bytes).await;
        println!("Send VM Data tx hash: {tx_hash}");

        let data = vec!["Hello World!"];
        let data_as_bytes: Vec<u8> = data.into_iter()
            .flat_map(|s| s.as_bytes().to_vec())
            .collect();
        let tx_hash = wallet.send_payable_vm_data(1234, 1000, data_as_bytes).await;
        println!("Send Payable VM Data tx hash: {tx_hash}");
    }
}
