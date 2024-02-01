use crate::raydium_sdk::TOKEN_PROGRAM_ID;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{pubkey::Pubkey, signature::Keypair, transaction::Transaction};
use spl_associated_token_account::get_associated_token_address;
use spl_associated_token_account::instruction::create_associated_token_account;

pub    fn get_or_create_associated_token_account(
    client: &RpcClient,
    key_payer: &Keypair,
    wallet_address: &Pubkey,
    token_mint_address: &Pubkey,
) -> Result<Pubkey, Box<dyn std::error::Error>> {
    let associated_token_account_address =
        get_associated_token_address(&wallet_address, &token_mint_address);

    let account_exists = client
        .get_account(&associated_token_account_address)
        .is_ok();

    if !account_exists {
        // Create the associated token account

        let create_ata_instruction = create_associated_token_account(
            &wallet_address,
            &wallet_address,
            &token_mint_address,
            &*TOKEN_PROGRAM_ID,
        );

        let recent_blockhash = client
            .get_latest_blockhash()
            .expect("Failed to get recent blockhash");

        let transaction = Transaction::new_signed_with_payer(
            &[create_ata_instruction],
            Some(&wallet_address),
            &[&key_payer],
            recent_blockhash,
        );

        let signature = client.send_and_confirm_transaction(&transaction)?;
        Ok(associated_token_account_address)
    } else {
        Ok(associated_token_account_address)
    }
}
