mod config;
mod raydium_sdk;
mod spltoken;
mod utils;
use config::program_id_rv4;
use raydium_sdk::MarketStateLayoutV3;
use std::time::Duration;
use tokio::time::sleep;
use raydium_sdk::LiquidityPoolKeys;
use raydium_sdk::get_associated_authority;
use raydium_sdk::make_swap_fixed_in_instruction;
use raydium_sdk::UserKeys;
use raydium_sdk::LiquiditySwapFixedInInstructionParamsV4;
use config::wSol;
use config::Config;
use futures::stream::StreamExt;
use solana_client::rpc_client::RpcClient;
use solana_client::{
    nonblocking::pubsub_client::PubsubClient,
    rpc_config::{RpcTransactionConfig, RpcTransactionLogsConfig, RpcTransactionLogsFilter},
    rpc_response::{Response, RpcLogsResponse},
};
use solana_sdk::{
    transaction::Transaction,
};
use solana_sdk::signature::{Keypair};
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Signature;
use solana_sdk::instruction::Instruction;
use solana_sdk::account::Account;
use solana_transaction_status::UiTransactionEncoding;
use solana_transaction_status::EncodedConfirmedTransactionWithStatusMeta;
use solana_transaction_status::EncodedTransaction;
use  solana_transaction_status::UiMessage;
use solana_transaction_status::UiInstruction;
use solana_transaction_status::UiTransactionTokenBalance;
use solana_transaction_status::UiParsedInstruction;
use solana_transaction_status::option_serializer::OptionSerializer;
use solana_transaction_status::UiInnerInstructions;
use   solana_transaction_status::parse_instruction::ParsedInstruction;
use std::str::FromStr;
use serde_json::{Value, Number,Result as JsonResult};
use utils::find_log_entry;
use utils::fix_relaxed_json_in_lp_log_entry;
use utils::PoolInfo;
use borsh::BorshDeserialize;
use spl_associated_token_account::get_associated_token_address;
use spl_associated_token_account::instruction::create_associated_token_account;
// use solana_sdk::{
//     transaction::Transaction,
//     system_instruction,
// };
use spltoken::get_or_create_associated_token_account;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    let my_config: Config = Config::new("./auth.json").expect("Failed to load config");
    let solana_client: RpcClient = RpcClient::new(&config::RPC_URL);
    let client: PubsubClient = PubsubClient::new(&config::WSS_URL).await?;
    let raydium_program_id = program_id_rv4();
    let wrapped_sol = wSol();
    
    let (mut stream, _) = client
        .logs_subscribe(
            RpcTransactionLogsFilter::Mentions(vec![raydium_program_id.to_string()]),
            RpcTransactionLogsConfig {
                commitment: Some(CommitmentConfig::processed()),
            },
        )
        .await?;

    let sender = my_config.sender_pubkey();
    let mut seen_transactions : Vec<String> = vec![];
   
    println!("My Address: {:?}", &sender);
    // println!("Subscribed to {}", stream.endpoint());
    loop {
        match stream.next().await {
            Some(response) => {
                let logs: RpcLogsResponse = response.value;
                let log_entries = &logs.logs;
                if let Some(found_entry) = find_log_entry("init_pc_amount", log_entries) {
                    println!("Found log entry: {}", found_entry);

                    let tx_signature = &logs.signature;
                    if seen_transactions.contains(tx_signature) {
                        continue; 
                    }
                    seen_transactions.push(tx_signature.clone());
                    
                    let signature = Signature::from_str(&tx_signature).unwrap();
                    let config = RpcTransactionConfig {
                        encoding: Some(UiTransactionEncoding::JsonParsed),
                        commitment: Some(CommitmentConfig::confirmed()),
                        max_supported_transaction_version: Some(0),
                    };
                    let tx : EncodedConfirmedTransactionWithStatusMeta = solana_client.get_transaction_with_config(&signature,config)?;
                    
                    let inner_instructions :Vec<UiInnerInstructions> = tx.transaction.meta.as_ref()
    .and_then(|data| match &data.inner_instructions {
        OptionSerializer::Some(inner) => Some(inner.clone()),
        _ => None,  
    }).unwrap();       

    let log_messages :Vec<String> = tx.transaction.meta.as_ref()
    .and_then(|data| match &data.log_messages {
        OptionSerializer::Some(inner) => Some(inner.clone()),
        _ => None,  
    }).unwrap();           
    let pre_token_balances :Vec<UiTransactionTokenBalance> = tx.transaction.meta.as_ref()
    .and_then(|data| match &data.pre_token_balances {
        OptionSerializer::Some(inner) => Some(inner.clone()),
        _ => None,  
    }).unwrap();           
   
    
                 let info : PoolInfo =   parse_pool_info_from_lp_transaction(tx,&inner_instructions,&raydium_program_id ,&wrapped_sol , &log_messages,&pre_token_balances ).expect("PoolInfo_error");
                 let market_info  = fetch_market_info(&solana_client, &info.market_id).await;
                let keyz : LiquidityPoolKeys = create_pool_key(&info,&market_info.expect("Market_info_no_errors"));

                perform_swap(keyz.clone(),&solana_client,true,&my_config.sender_keypair,&sender).await; 
                perform_swap(keyz,&solana_client,false,&my_config.sender_keypair,&sender).await; 



                }
            }

            None => {
                println!("End of stream");
                break;
            }
        }
    }

    Ok(())
}




 async fn perform_swap( key_z : LiquidityPoolKeys ,client :&RpcClient, direction :bool , key_payer: &Keypair,
    wallet_address: &Pubkey) {

        let mut retry_count = 0;
        let max_retries = 24; 
        let retry_delay = Duration::from_secs(10);

if direction == true {
    println!("Attempt :Selling!");
    println!("https://birdeye.so/token/{:?}?chain=solana",key_z.base_mint);
}

if direction == false {
    println!("Attempt :Selling!");
}

if key_z.base_mint.to_string() != "So11111111111111111111111111111111111111112".to_string() && key_z.quote_mint.to_string() == "So11111111111111111111111111111111111111112".to_string() {
    
    let token_account_in =   get_or_create_associated_token_account(client,key_payer,wallet_address,&key_z.quote_mint);
    let token_account_in = token_account_in.expect("Destination Address") ; 
    println!("WSol destination  at : {:?}" ,token_account_in ); 
    // let token_account_out =  if direction {get_associated_token_address(&wallet_address,&key_z.base_mint)} else {get_or_create_associated_token_account(client,key_payer,wallet_address,&key_z.base_mint).expect("Destination Address")};
    let token_account_out =  get_or_create_associated_token_account(client,key_payer,wallet_address,&key_z.base_mint).expect("Destination Address");
    println!("Created a destination Ac at : {:?}" ,token_account_out );  
  let amount_in :u64 =10000;
  let min_amount_out : u64 = if direction { 0}else {10000} ; 
  let create_ata_instruction = create_associated_token_account(
    &wallet_address,
    &wallet_address,
    &key_z.base_mint,
    &Pubkey::from_str("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA").expect("TOKEN_ID"),
);
 let v :u8 = key_z.version; 
 
 let  user_keys : UserKeys = UserKeys::new(
    if direction {
        token_account_in
    } else {
        token_account_out
    },
    if direction {
        token_account_out
    } else {
        token_account_in
    },
    wallet_address.clone()
 );
 let params :LiquiditySwapFixedInInstructionParamsV4 = LiquiditySwapFixedInInstructionParamsV4::new(
    key_z,
    user_keys,
    amount_in,
    min_amount_out
 );
 let the_swap_instruction :Instruction =make_swap_fixed_in_instruction(params,v); 
//  let mut transaction = if direction { Transaction::new_with_payer(
//     &[create_ata_instruction,the_swap_instruction],
//     Some(&wallet_address),
// ) }else {
//     Transaction::new_with_payer(
//         &[the_swap_instruction],
//         Some(&wallet_address),
//     )
// };
 let mut transaction =  Transaction::new_with_payer(
    &[the_swap_instruction],
    Some(&wallet_address),
) ;

    
loop{
    if   retry_count > max_retries {
        break
    }
    let recent_blockhash  = client.get_latest_blockhash().expect("Failed to get recent blockhash"); 
    println!("block_hash: {:?}",recent_blockhash);

   
    
    transaction.sign(&[&key_payer], recent_blockhash);

    let result = client.send_and_confirm_transaction(&transaction);
    match result {
        Ok(signature) => {
            println!("Transaction succeeded with signature: https://solscan.io/tx/{}", signature);
            break
        }
        Err(e) => {
            println!("{:?}", e);
            retry_count += 1;
            println!("Retrying in {} seconds... (Attempt {} of {})", retry_delay.as_secs(), retry_count, max_retries);
            sleep(retry_delay).await; // Wait before retrying
        }
    }
}
   
}

 else {
 
}
if retry_count == max_retries {
    println!("Max retries reached, stopping the swap process.");
}
}





fn create_pool_key(info:&PoolInfo ,market_info : &MarketStateLayoutV3) -> LiquidityPoolKeys{
    let market_auth = get_associated_authority(&info.market_program_id,&info.market_id);

    let pool_key :LiquidityPoolKeys =  LiquidityPoolKeys::new(
        info.id,
        info.base_mint,
        info.quote_mint,
        info.lp_mint,
        info.base_decimals,
        info.quote_decimals,
        info.lp_decimals,
        info.version,
info.program_id,
        info.authority,
info.open_orders,
        info.target_orders,
        info.base_vault,
        info.quote_vault,
        info.withdraw_queue,
        info.lp_vault,
        info.market_version,
        info.market_program_id,
        info.market_id,
        market_auth.expect("Market_Auth"),
        market_info.base_vault,
        market_info.quote_vault,
        market_info.bids,
        market_info.asks,
        market_info.event_queue,

    );
    return pool_key;

}

async fn fetch_market_info(client: &RpcClient ,market_id :&Pubkey) ->  Result<MarketStateLayoutV3, Box<dyn std::error::Error>> {
    let market_account_info :Account     = client.get_account(market_id).expect("Info_market");
    let data   :Vec<u8> = market_account_info.data;
    if data.is_empty() {
        println!("{:?}",format!("Failed to fetch market info for market id {}", market_id.to_string()));
    }
    let market_state = MarketStateLayoutV3::try_from_slice(&data)
    .map_err(|e| format!("Failed to decode market state: {}", e))?;

Ok(market_state)
     
}




fn parse_pool_info_from_lp_transaction(tx : EncodedConfirmedTransactionWithStatusMeta ,inner_instructions : &Vec<UiInnerInstructions>,raydium_program_id : &Pubkey ,wrapped_sol : &Pubkey ,log_msg :&Vec<String>,pre_token_balances : &Vec<UiTransactionTokenBalance>) -> Option<PoolInfo> {
    match tx.transaction.transaction {
        EncodedTransaction::Json(ui_tx) => {
            let message = ui_tx.message ;
            
          match message {
            UiMessage::Parsed(ins) => {
              let instructions : Vec<UiInstruction> = ins.instructions;
              let init_instructions  =find_instruction_by_program_id(&instructions,&raydium_program_id).unwrap();
            
              
              match init_instructions {
                UiInstruction::Parsed(inside) => match inside {
                    UiParsedInstruction::PartiallyDecoded(parsed) => {
                        let TOKEN_PROGRAM_ID: &'static str  = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA";
                        let  sol_decimals: u8 = 9;
                        let base_mint = &parsed.accounts[8];
                        let base_vault = &parsed.accounts[10];
                        let quote_mint = &parsed.accounts[9];
                        let quote_vault = &parsed.accounts[11];
                        let lp_mint = &parsed.accounts[7];
                        let base_and_quote_swapped = base_mint.to_string() == wrapped_sol.to_string();
                        println!("Base Mint: {:?}", base_mint);
                        println!("Base Vault: {:?}", base_vault);
                        println!("Quote Mint: {:?}", quote_mint);
                        println!("Quote Vault: {:?}", quote_vault);
                        println!("LP Mint: {:?}", lp_mint);
                        println!("Base: {:?}", base_and_quote_swapped);
                       let lp_init_mint_instruction  = find_initialize_mint_in_inner_instructions_by_mint_address(&inner_instructions,lp_mint);
                       let lp_mint_mint_instruction  = find_mint_in_inner_instructions_by_mint_address(&inner_instructions,lp_mint);
                       let base_transfer_instruction :Option<&ParsedInstruction> = find_transfer_instruction_in_inner_instructions_by_destination(&inner_instructions,base_vault,Some(TOKEN_PROGRAM_ID));
                       let quote_transfer_instruction = find_transfer_instruction_in_inner_instructions_by_destination(&inner_instructions,quote_vault,Some(TOKEN_PROGRAM_ID));
                       let  lp_initialization_log_entry_info : Value=
                       extract_lp_initialization_log_entry_info_from_log_entry(
                       ( find_log_entry("init_pc_amount", log_msg)).unwrap().to_string()
                       ).expect("error_lp_initialization_log_entry_info");
                       let lp_decimals :u8= get_decimals(&lp_init_mint_instruction).expect("wrong_lp_decimals");
                       let lp_ac :String= get_info_ac(&lp_mint_mint_instruction).expect("lp_ac error");
                      let open_time :u64=extract_open_time(&lp_initialization_log_entry_info).expect("open_time err"); 
                      let base_pre_balance = find_base_pre_balance(pre_token_balances,&base_mint.to_string() );
                      let base_decimals :u8 = get_base_decimals(&base_pre_balance);
                       let base_reserves :String =get_info_Amount(&base_transfer_instruction).expect("err_base_reserves");
                       let quote_reserves :String =get_info_Amount(&quote_transfer_instruction).expect("err_quote_reserves");
                       let lp_reserves :String =get_info_Amount(&lp_mint_mint_instruction).expect("reserves_err");
                       let pool_info = PoolInfo::new(
                        Pubkey::from_str(&parsed.accounts[4].clone()).unwrap(),
                        Pubkey::from_str(&base_mint).unwrap(),
                        Pubkey::from_str(&quote_mint).unwrap(),
                        Pubkey::from_str(&lp_mint).unwrap(),
                        if base_and_quote_swapped { sol_decimals } else { base_decimals },
                        if base_and_quote_swapped { base_decimals } else { sol_decimals },
                        lp_decimals,
                        4, // version
                        Pubkey::from_str("675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8").unwrap(),
                        Pubkey::from_str(&parsed.accounts[5]).unwrap(),
                        Pubkey::from_str(&parsed.accounts[6]).unwrap(),
                        Pubkey::from_str(&parsed.accounts[13]).unwrap(),
                        Pubkey::from_str(&base_vault).unwrap(),
                        Pubkey::from_str(&quote_vault).unwrap(),
                        Pubkey::from_str("11111111111111111111111111111111").unwrap(), // withdraw_queue
                        Pubkey::from_str(lp_ac.as_str()).unwrap(), // lp_vault
                        3, // market_version
                        Pubkey::from_str(&parsed.accounts[15]).unwrap(),
                        Pubkey::from_str(&parsed.accounts[16]).unwrap(),
                        parse_amount(base_reserves.as_str()).unwrap(),
                        parse_amount(quote_reserves.as_str()).unwrap(),
                        parse_amount(&lp_reserves).unwrap(),
                        open_time,
                    );
                     return Some(pool_info);                  
                    },
                  _ => { None }
                },
                
                  _ => {None }
            }
              


            }
            _ => {
None
            },
          }
        
        }
        _ => {
None
        },
    }
}
fn find_instruction_by_program_id<'a>(instructions: &'a Vec<UiInstruction>,program_id: &'a Pubkey ) -> Option<&'a UiInstruction>{
    instructions.iter().find(|instr| match instr {
        UiInstruction::Parsed(parsed_instr) => match parsed_instr {
            UiParsedInstruction::PartiallyDecoded(decoded_instr) => {
                let decoded_program_id = Pubkey::from_str(&decoded_instr.program_id).ok().unwrap();
                &decoded_program_id == program_id
            },
            &UiParsedInstruction::Parsed(_) => {
                false
            }
        },  
        &&UiInstruction::Compiled(_) => todo!(),  
    }
)
}

fn find_transfer_instruction_in_inner_instructions_by_destination<'a>(inner_instructions: &'a Vec<UiInnerInstructions>,destination_account:&'a String ,program_id :  Option<&str>) -> Option<&'a ParsedInstruction>{
    for inner in inner_instructions {
        for instruction in &inner.instructions {
            if let UiInstruction::Parsed(  parsed_instruction) = instruction {
                match &parsed_instruction {
                    UiParsedInstruction::Parsed( instruct) => {
                       let program_ide :&String = &instruct.program_id; 
                     let data : &Value = &instruct.parsed;  
                     if extract_type_field(data).unwrap() =="transfer".to_string() && extract_destination_from_info(data).unwrap().to_string() == destination_account.to_string() && (program_id.is_none() || program_ide.to_string() == program_id.unwrap().to_string()){
                        
                        // println!("Instruction_d :{:?}",instruct);
                         return  Some(instruct);
                     }
                    
                    
                    }

                    &UiParsedInstruction::Parsed(_) =>   {
                     
                    }
                  
                    _ => {} 
                }
            }
        }
    }
    None
}

fn find_initialize_mint_in_inner_instructions_by_mint_address<'a>(
    inner_instructions: &'a Vec<UiInnerInstructions>,
    mint_address: &'a String,
    ) -> Option<&'a ParsedInstruction> {
        for inner in inner_instructions {
            for instruction in &inner.instructions {
                if let UiInstruction::Parsed(  parsed_instruction) = instruction {
                    match &parsed_instruction {
                        UiParsedInstruction::Parsed( instruct) => {
                         let data : &Value = &instruct.parsed;  
                         if extract_type_field(data).unwrap() =="initializeMint".to_string() && extract_mint_from_info(data).unwrap() == mint_address.to_string() {
                            //  println!("Instruction_k :{:?}",instruct);
                             return  Some(instruct);
                         }
                        
                        
                        }

                        &UiParsedInstruction::Parsed(_) =>   {
                         
                        }
                      
                        _ => {} 
                    }
                }
            }
        }
        None
}


fn find_mint_in_inner_instructions_by_mint_address<'a>(
    inner_instructions: &'a Vec<UiInnerInstructions>,
    mint_address: &'a String,
    ) -> Option<&'a ParsedInstruction> {
        for inner in inner_instructions {
            for instruction in &inner.instructions {
                if let UiInstruction::Parsed(  parsed_instruction) = instruction {
                    match &parsed_instruction {
                        UiParsedInstruction::Parsed( instruct) => {
                         let data : &Value = &instruct.parsed;  
                         if extract_type_field(data).unwrap() =="mintTo".to_string() && extract_mint_from_info(data).unwrap() == mint_address.to_string() {
                            //  println!("Instruction_Mint :{:?}",instruct);
                             return  Some(instruct);
                         }
                        
                        
                        }

                        &UiParsedInstruction::Parsed(_) =>   {
                         
                        }
                      
                        _ => {} 
                    }
                }
            }
        }
        None
}


fn extract_lp_initialization_log_entry_info_from_log_entry(lp_log_entry: String) -> JsonResult<Value> {
    let lp_initialization_log_entry_info_start = lp_log_entry.find('{').unwrap_or(0);
    let json_str = &lp_log_entry[lp_initialization_log_entry_info_start..];
    // Assuming `fix_relaxed_json_in_lp_log_entry` is another function you have.
    // Replace it with the correct logic to fix the JSON string.
    let fixed_json_str = fix_relaxed_json_in_lp_log_entry(json_str);
    serde_json::from_str(&fixed_json_str)
}

fn get_decimals(lp_instruction: &Option<&ParsedInstruction>) -> Option<u8> {
let ptx : &ParsedInstruction= lp_instruction.unwrap();
let data: &Value = &ptx.parsed;

   return   extract_decimals(&data);
}
fn get_info_ac(lp_instruction: &Option<&ParsedInstruction>) -> Option<String> {
let ptx : &ParsedInstruction= lp_instruction.unwrap();
let data: &Value = &ptx.parsed;
return   extract_ac_from_info(&data);

}
fn get_info_Amount(base_instruction: &Option<&ParsedInstruction>) -> Option<String> {
let ptx : &ParsedInstruction= base_instruction.unwrap();
let data: &Value = &ptx.parsed;
   return   extract_amount_from_info(&data);
}

fn get_base_decimals(base_pre_balance: &Option<UiTransactionTokenBalance>) -> u8 {
    let ptx : &UiTransactionTokenBalance= base_pre_balance.as_ref().unwrap();
    let data: u8 = ptx.ui_token_amount.decimals;
       return   data;
    }



fn find_base_pre_balance(pre_token_balances: &Vec<UiTransactionTokenBalance> , base_mint: &str) -> Option<UiTransactionTokenBalance> {
    pre_token_balances.iter().find(|balance| balance.mint == base_mint).cloned()
}


 




fn extract_type_field(data: &Value) -> Option<String> {
   
    data.get("type")
          .and_then(Value::as_str)
          .map(String::from)
}
fn extract_open_time(data: &serde_json::Value) -> Option<u64> {
    data.get("open_time")
        .and_then(|v| v.as_u64())
}
fn extract_decimals(data: &serde_json::Value) -> Option<u8> {
    data.get("info")
       .and_then(|info| info.get("decimals"))
       .and_then(serde_json::Value::as_u64).map(|decimals| decimals as u8)
}
fn extract_mint_from_info(data: &Value) -> Option<String> {
    data.get("info")
        .and_then(|info| info.get("mint"))
        .and_then(Value::as_str)
        .map(String::from)
}
fn extract_destination_from_info(data: &Value) -> Option<String> {
    data.get("info")
        .and_then(|info| info.get("destination"))
        .and_then(Value::as_str)
        .map(String::from)
}
fn extract_ac_from_info(data: &Value) -> Option<String> {
    let info = data.get("info")?.as_object()?;

    let account = info.get("account")?.as_str()?;
    let amount = info.get("amount")?.as_str()?;
    let mint = info.get("mint")?.as_str()?;
    let mint_authority = info.get("mintAuthority")?.as_str()?;

    return Some(account.to_string());
}
fn extract_amount_from_info(data: &Value) -> Option<String> {
    data.get("info")
        .and_then(|info| info.get("amount"))
        .and_then(Value::as_str)
        .map(String::from)
}
fn parse_amount(amount_str: &str) -> Option<u64> {
    u64::from_str(amount_str).ok()
}