use borsh::{BorshDeserialize, BorshSerialize};
use once_cell::sync::Lazy;
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
};
use std::str::FromStr;

pub static TOKEN_PROGRAM_ID: Lazy<Pubkey> =
    Lazy::new(|| Pubkey::from_str("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA").unwrap());

pub static MODEL_DATA_PUBKEY: Lazy<Pubkey> =
    Lazy::new(|| Pubkey::from_str("CDSr3ssLcRB6XYPJwAfFt18MZvEZp4LjHcvzBVZ45duo").unwrap());
pub static ASSOCIATED_TOKEN_PROGRAM_ID: Lazy<Pubkey> =
    Lazy::new(|| Pubkey::from_str("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL").unwrap());



#[derive(Debug)]
pub struct LiquiditySwapFixedInInstructionParamsV4 {
    pool_keys: LiquidityPoolKeys,
    user_keys: UserKeys,
    amount_in:  u64,
    min_amount_out:  u64,
}

impl LiquiditySwapFixedInInstructionParamsV4  {
    pub fn new(
        pool_keys: LiquidityPoolKeys,
    user_keys: UserKeys,
    amount_in:  u64,
    min_amount_out:  u64) -> Self {
        LiquiditySwapFixedInInstructionParamsV4{pool_keys,
        user_keys,
        amount_in,
        min_amount_out}
    }

}


#[derive(Debug, Clone)]
pub struct LiquidityPoolKeys {
    pub id: Pubkey,
    pub base_mint: Pubkey,
    pub quote_mint: Pubkey,
    pub lp_mint: Pubkey,
    pub base_decimals: u8,
    pub quote_decimals: u8,
    pub lp_decimals: u8,
    pub version: u8,
    pub program_id: Pubkey,
    pub authority: Pubkey,
    pub open_orders: Pubkey,
    pub    target_orders: Pubkey,
    pub  base_vault: Pubkey,
    pub    quote_vault: Pubkey,
    pub   withdraw_queue: Pubkey,
    pub  lp_vault: Pubkey,
    pub   market_version: u8,
    pub  market_program_id: Pubkey,
    pub  market_id: Pubkey,
    pub market_authority: Pubkey,
    pub market_base_vault: Pubkey,
    pub market_quote_vault: Pubkey,
    pub market_bids: Pubkey,
    pub market_asks: Pubkey,
    pub    market_event_queue: Pubkey,
}

impl LiquidityPoolKeys {
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
        market_authority: Pubkey,
        market_base_vault: Pubkey,
        market_quote_vault: Pubkey,
        market_bids: Pubkey,
        market_asks: Pubkey,
        market_event_queue: Pubkey,
    )-> Self {
        LiquidityPoolKeys {
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
            market_authority,
            market_base_vault,
            market_quote_vault,
            market_bids,
            market_asks,
            market_event_queue,
        }
}}

















#[derive(Debug)]
pub struct UserKeys {
    token_account_in: Pubkey,
    token_account_out: Pubkey,
    owner: Pubkey,
}

impl UserKeys {
  pub fn  new(
        token_account_in: Pubkey,
        token_account_out: Pubkey,
        owner: Pubkey,
    ) -> Self {
        UserKeys{token_account_in,
        token_account_out,
        owner,}
    }
}


#[derive(BorshSerialize, BorshDeserialize, Debug)]
struct SwapInstructionData {
    instruction: u8,
    amount_in: u64,
    min_amount_out: u64,
}

pub fn make_swap_fixed_in_instruction(
    params: LiquiditySwapFixedInInstructionParamsV4,
    version: u8,
) -> Instruction {
    let data = SwapInstructionData {
        instruction: 9, // Instruction variant identifier
        amount_in: params.amount_in,
        min_amount_out: params.min_amount_out,
    }
    .try_to_vec()
    .unwrap(); // Serialize using Borsh

    let mut keys = vec![
        account_meta_readonly(*TOKEN_PROGRAM_ID, false),
        account_meta(params.pool_keys.id, false),
        account_meta_readonly(params.pool_keys.authority, false),
        account_meta(params.pool_keys.open_orders, false),
    ];
    if version == 4 {
        keys.push(account_meta(params.pool_keys.target_orders, false));
    }
    keys.push(account_meta(params.pool_keys.base_vault, false));
    keys.push(account_meta(params.pool_keys.quote_vault, false));
    if version == 5 {
        keys.push(account_meta(*MODEL_DATA_PUBKEY, false));
    }

    // Serum-related accounts
    keys.push(account_meta_readonly(
        params.pool_keys.market_program_id,
        false,
    ));
    keys.push(account_meta(params.pool_keys.market_id, false));
    keys.push(account_meta(params.pool_keys.market_bids, false));
    keys.push(account_meta(params.pool_keys.market_asks, false));
    keys.push(account_meta(params.pool_keys.market_event_queue, false));
    keys.push(account_meta(params.pool_keys.market_base_vault, false));
    keys.push(account_meta(params.pool_keys.market_quote_vault, false));
    keys.push(account_meta_readonly(
        params.pool_keys.market_authority,
        false,
    ));

    // User-related accounts
    keys.push(account_meta(params.user_keys.token_account_in, false));
    keys.push(account_meta(params.user_keys.token_account_out, false));
    keys.push(account_meta_readonly(params.user_keys.owner, true));

    Instruction {
        program_id: params.pool_keys.program_id,
        accounts: keys,
        data,
    }
}
pub fn account_meta(pubkey: Pubkey, is_signer: bool) -> AccountMeta {
    AccountMeta {
        pubkey,
        is_signer,
        is_writable: true, // Set to true
    }
}

pub fn account_meta_readonly(pubkey: Pubkey, is_signer: bool) -> AccountMeta {
    AccountMeta {
        pubkey,
        is_signer,
        is_writable: false, // Set to false for readonly as in radium js SDK idk lmao
    }
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct MarketStateLayoutV3 {
    
  pub  _padding: [u8; 13],

  pub   own_address: Pubkey,
  pub  vault_signer_nonce: u64,

  pub  base_mint: Pubkey,
  pub  quote_mint: Pubkey,

  pub base_vault: Pubkey,
  pub  base_deposits_total: u64,
  pub  base_fees_accrued: u64,

  pub  quote_vault: Pubkey,
  pub quote_deposits_total: u64,
  pub quote_fees_accrued: u64,

  pub quote_dust_threshold: u64,

  pub request_queue: Pubkey,
  pub  event_queue: Pubkey,

  pub bids: Pubkey,
  pub asks: Pubkey,

  pub base_lot_size: u64,
  pub  quote_lot_size: u64,

  pub fee_rate_bps: u64,

  pub   referrer_rebates_accrued: u64,

    _padding_end: [u8; 7],
}

pub fn get_associated_authority(
    program_id: &Pubkey,
    market_id: &Pubkey,
) -> Result< Pubkey , String> {
    let market_id_bytes = market_id.to_bytes(); 
    let seeds = &[&market_id_bytes[..]]; 

    for nonce in 0..100u8 {
        let nonce_bytes = [nonce]; 
        let padding = [0u8; 7]; 

        
        let seeds_with_nonce = [
            seeds[0],     // Market ID bytes
            &nonce_bytes, // Nonce bytes
            &padding,     // Padding bytes
        ];

        match Pubkey::create_program_address(&seeds_with_nonce, program_id) {
            Ok(public_key) => return Ok(public_key),
            Err(_) => continue,
        }
    }

    Err("Unable to find a valid program address".into())
}
