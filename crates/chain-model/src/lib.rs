use alloy::primitives::{Address, FixedBytes, Uint, U256};
use serde::{Deserialize, Serialize};

/// Decoded `PairCreated` event data.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PairCreatedEvent {
    pub pair: Address,
    pub token0: Address,
    pub token1: Address,
    pub transaction_hash: FixedBytes<32>,
    pub block_number: u64,
    pub block_timestamp: u64,
}

/// Decoded `Sync` event data.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SyncEvent {
    pub pair: Address,
    pub reserve0: Uint<112, 2>,
    pub reserve1: Uint<112, 2>,
    pub transaction_hash: FixedBytes<32>,
    pub block_number: u64,
    pub block_timestamp: u64,
}

/// Represents a token's static information.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Token {
    pub address: Address,
    pub decimals: u8,
    pub symbol: String,
    pub total_supply: U256,
}

/// Represents a Uniswap Pair with its two tokens.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Pair {
    pub address: Address,
    pub token0: Token,
    pub token1: Token,
}

/// The final price data point to be stored or further processed.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PriceTick {
    pub pair_address: String,

    pub token0_address: String,
    pub token0_reserve: Uint<112, 2>,
    pub token0_symbol: String,

    pub token1_address: String,
    pub token1_reserve: Uint<112, 2>,
    pub token1_symbol: String,

    pub token0_token1: f64,
    pub token1_token0: f64,

    pub transaction_hash: String,
    pub block_number: u64,
    pub block_timestamp: u64,
}
