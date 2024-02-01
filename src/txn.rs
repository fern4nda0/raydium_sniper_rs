mod client;
mod config;

use client::SolanaClient;
use config::Config;
use solana_sdk::{
    transaction::Transaction,
    system_instruction,
};

fn main() {
    let config: Config = Config::new("./auth.json").expect("Failed to load config");
    let client: SolanaClient = SolanaClient::new(&config);

    let sender = config.sender_pubkey();
    let receiver = config::receiver();
    let lamports_to_transfer = 1000;
    println!("My Address: {:?}", &sender);
    println!("Receiver: {:?}",&receiver);

    let transfer_instruction = system_instruction::transfer(
        &sender,
        &receiver,
        lamports_to_transfer,
    );

    let recent_blockhash  = client.get_latest_blockhash().expect("Failed to get recent blockhash"); 
    
    println!("block_hash: {:?}",recent_blockhash);


    let mut transaction = Transaction::new_with_payer(
        &[transfer_instruction],
        Some(&sender),
    );
    
    transaction.sign(&[&config.sender_keypair], recent_blockhash);

    let result = client.send_and_confirm_transaction(&transaction);

    match result {
        Ok(signature) => println!("Transaction succeeded with signature: https://explorer.solana.com/tx/{}", signature),
        Err(e) => println!("Transaction failed: {}", e),
    }
}

