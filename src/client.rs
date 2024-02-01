use solana_client::{
    rpc_client::RpcClient};
use solana_client::client_error::ClientError;
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::account::Account;
use super::config::RPC_URL;

use solana_sdk::{
    transaction::Transaction,
    signature::Signature,
    hash::Hash,
};


pub struct SolanaClient {
    client: RpcClient,
}



impl SolanaClient {
    pub fn new() -> Self {
        let client = RpcClient::new_with_commitment(
            RPC_URL,
            CommitmentConfig::confirmed(),
        );
        SolanaClient { client }
    }
    pub fn get_latest_blockhash(&self) -> Result<Hash,  ClientError>  {
        self.client.get_latest_blockhash()
    }
    pub fn send_and_confirm_transaction(&self, transaction: &Transaction) -> Result<Signature, ClientError> {
        self.client.send_and_confirm_transaction(transaction)
    }
    pub fn get_account(&self, pubkey: &Pubkey) -> Result<Account, ClientError> {
     self.client.get_account(pubkey)
    }
}
