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
    pub token0: String,
    pub token1: String,
    pub reserve0: String, // Stored as string for full precision
    pub reserve1: String, // Stored as string for full precision
    pub t1_t0: f64,       // For quick, less-precise views
    pub t0_t1: f64,
    pub symbol0: String,
    pub symbol1: String,
    pub transaction_hash: String,
    pub block_number: u64,
    pub block_timestamp: u64,
}
