use regex::Regex;
use solana_sdk::{
    pubkey::Pubkey,
};

pub fn find_log_entry(needle: &str, log_entries: &[String]) -> Option<String> {
    for entry in log_entries {
        if entry.contains(needle) {
            return Some(entry.clone());
        }
    }
    None
}


pub fn fix_relaxed_json_in_lp_log_entry(relaxed_json: &str) -> String {
    let re = Regex::new(r#"([{,])\s*([a-zA-Z_][a-zA-Z0-9_]*)\s*:"#).unwrap();
    re.replace_all(relaxed_json, "$1\"$2\":").into_owned()
}


#[derive(Debug)]
pub struct PoolInfo {
   pub id: Pubkey,
   pub base_mint: Pubkey,
   pub quote_mint: Pubkey,
   pub lp_mint: Pubkey,
   pub  base_decimals: u8,
   pub quote_decimals: u8,
   pub lp_decimals: u8,
   pub version: u8,
   pub program_id: Pubkey,
   pub authority: Pubkey,
   pub open_orders: Pubkey,
   pub target_orders: Pubkey,
   pub  base_vault: Pubkey,
   pub  quote_vault: Pubkey,
   pub  withdraw_queue: Pubkey,
   pub  lp_vault: Pubkey,
   pub  market_version: u8,
   pub  market_program_id: Pubkey,
    pub  market_id: Pubkey,
    pub  base_reserve: u64,
    pub  quote_reserve: u64,
    pub lp_reserve: u64,
    pub  open_time: u64,
}
impl PoolInfo {
   
    pub fn new(
        id: Pubkey,
        base_mint: Pubkey,
        quote_mint: Pubkey,
        lp_mint: Pubkey,
        base_decimals: u8,
        quote_decimals: u8,
        lp_decimals: u8,
        version: u8,
        program_id: Pubkey,
        authority: Pubkey,
        open_orders: Pubkey,
        target_orders: Pubkey,
        base_vault: Pubkey,
        quote_vault: Pubkey,
        withdraw_queue: Pubkey,
        lp_vault: Pubkey,
        market_version: u8,
        market_program_id: Pubkey,
        market_id: Pubkey,
        base_reserve: u64,
        quote_reserve: u64,
        lp_reserve: u64,
        open_time: u64,
    ) -> Self {
        PoolInfo {
            id,
            base_mint,
            quote_mint,
            lp_mint,
            base_decimals,
            quote_decimals,
            lp_decimals,
            version,
            program_id,
            authority,
            open_orders,
            target_orders,
            base_vault,
            quote_vault,
            withdraw_queue,
            lp_vault,
            market_version,
            market_program_id,
            market_id,
            base_reserve,
            quote_reserve,
            lp_reserve,
            open_time,
        }
    }

   
}





// pub fn find_instruction_by_program_id(instructions: &[Instruction], program_id: &str) -> Option<Instruction> {
//     for instruction in instructions {
//         if instruction.program_id == program_id {
//             return Some(instruction.clone());
//         }
//     }
//     None
// }
