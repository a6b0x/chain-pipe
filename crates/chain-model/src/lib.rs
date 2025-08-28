use alloy::primitives::{Address, FixedBytes, Uint, U256};
use serde::{Deserialize, Serialize};

/// Decoded `PairCreated` event data.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PairCreatedEvent {
    pub pair: Address,
    pub token0: Address,
    pub token1: Address,
    pub transaction_hash: String,
    pub block_hash: String,
    pub block_number: U256,
    pub block_timestamp: u64,
}

/// Decoded `Sync` event data.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SyncEvent {
    pub pair_address: Address,
    pub pair_reserve0: Uint<112, 2>,
    pub pair_reserve1: Uint<112, 2>,
    pub transaction_hash: FixedBytes<32>,
    pub block_hash: FixedBytes<32>,
    pub block_number: u64,
    pub block_timestamp: u64,
}

/// Represents a token's static information.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Token {
    pub address: String,
    pub decimals: u8,
    pub symbol: String,
    pub total_supply: U256,
}

/// Represents a Uniswap Pair with its two tokens.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Pair {
    pub address: String,
    pub token0: Token,
    pub token1: Token,
}

/// The final price data point to be stored or further processed.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PriceTick {
    pub timestamp: u64,
    pub pair_address: String,
    pub reserve0: String, // Stored as string for full precision
    pub reserve1: String, // Stored as string for full precision
    pub price0: f64,      // For quick, less-precise views
    pub price1: f64,
}
