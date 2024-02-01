use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::{Keypair, Signer};
use std::fs;
use std::str::FromStr;

pub const WSS_URL: &str = "wss://api.mainnet-beta.solana.com";
pub const RPC_URL: &str = "https://api.mainnet-beta.solana.com";


pub struct Config {
    pub sender_keypair: Keypair,
}

impl Config {
    pub fn new(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let key = fs::read_to_string(path)?;
        let key_bytes: Vec<u8> = serde_json::from_str(&key)?;
        let sender_keypair = Keypair::from_bytes(&key_bytes)?;

        Ok(Config { sender_keypair })
    }

    pub fn sender_pubkey(&self) -> Pubkey {
        self.sender_keypair.pubkey()
    }
}
pub fn receiver() -> Pubkey {
    return Pubkey::from_str("TEST").unwrap();
}
pub fn wSol() -> Pubkey {
    return Pubkey::from_str("So11111111111111111111111111111111111111112").unwrap();
}

pub fn program_id_rv4() -> Pubkey {
    return Pubkey::from_str("675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8").unwrap();
}
