use pwr_rs::{
    Wallet,
    RPC
};

#[tokio::main]
async fn main() {
    let seed_phrase = "demand april length soap cash concert shuffle result force mention fringe slim";
    let wallet = Wallet::new(seed_phrase).await;
    wallet.store_wallet("example_wallet.dat", "your_password_here").unwrap();

    let wallet = Wallet::load_wallet("example_wallet.dat", "your_password_here").await.unwrap();

    let address = wallet.get_address();
    println!("Address: {address}");

    let nonce = wallet.get_nonce().await;
    println!("Nonce: {nonce}");

    let balance = wallet.get_balance().await;
    println!("Balance: {balance}");

    #[cfg(feature = "rpc")]
    {
        let rpc = RPC::new("https://pwrrpc.pwrlabs.io/").await.unwrap();

        let fee_per_byte = rpc.get_fee_per_byte().await.unwrap();
        println!("FeePerByte: {fee_per_byte}");

        let latest_block = rpc.get_latest_block().await.unwrap();
        println!("LatestBlock: {latest_block}");

        let start_block = 85411;
        let end_block = 85420;
        let vida_id = 123;
        let transactions = rpc.get_vida_data_transactions(start_block, end_block, vida_id).await.unwrap();
        println!("VidaData: {:?}", hex::encode(&transactions[0].data));

        let guardian = rpc.get_guardian_of_address("0xD97C25C0842704588DD70A061C09A522699E2B9C").await.unwrap();
        println!("Guardian: {guardian}");
        
        let block = rpc.get_block_by_number(418).await.unwrap();
        println!("Block: {:?}", block);

        let active_voting_power = rpc.get_active_voting_power().await.unwrap();
        println!("ActiveVotingPower: {active_voting_power}");

        let conduits_vida = rpc.get_conduits_of_vida(69).await.unwrap();
        println!("ConduitsVida: {:?}", conduits_vida);

        let total_validators_count = rpc.get_validators_count().await.unwrap();
        println!("TotalValidatorsCount: {total_validators_count}");

        let standby_validators_count = rpc.get_standby_validator_count().await.unwrap();
        println!("StandbyValidatorsCount: {standby_validators_count}");

        let active_validators_count = rpc.get_active_validator_count().await.unwrap();
        println!("ActiveValidatorsCount: {active_validators_count}");

        // let all_validators = rpc.get_all_validators().await.unwrap();
        // println!("AllValidators: {all_validators:?}");

        let standby_validators = rpc.get_standby_validators().await.unwrap();
        println!("StandbyValidators: {standby_validators:?}");

        // let active_validators = rpc.get_active_validators().await.unwrap();
        // println!("ActiveValidators: {active_validators:?}");

        let tx = wallet.transfer_pwr(
            "0x3B3B69093879E7B6F28366FA3C32762590FF547E".to_string(),
            10,
            fee_per_byte,
        ).await;
        println!("Transfer tx hash: {}", tx.data.unwrap());

        let data: Vec<u8> = vec!["Hello World!"].into_iter()
            .flat_map(|s| s.as_bytes().to_vec())
            .collect();
        let tx = wallet.send_vida_data(1234, data.clone(), fee_per_byte).await;
        println!("Send Vida Data tx hash: {}", tx.data.unwrap());

        let tx = wallet.send_payable_vida_data(1234, data, 1000, fee_per_byte).await;
        println!("Send Payable Vida Data tx hash: {}", tx.data.unwrap());
    }
}
