use alloy::primitives::{keccak256, Address, Uint, B256, U256};
use alloy::providers::{DynProvider, Provider, ProviderBuilder};
use alloy::sol;
use eyre::Result;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

sol!(
    #[allow(missing_docs)]
    #[sol(rpc)]
    #[derive(Debug, Serialize)]
    ERC20Token,
    "abi/ERC20.json"
);

pub struct ERC20 {
    pub http_provider: DynProvider,
}

#[derive(Debug)]
pub struct Pair {
    pub address: Address,
    pub token0: Token,
    pub token1: Token,
}

#[derive(Debug)]
pub struct Token {
    pub address: Address,
    pub decimals: u8,
    pub symbol: String,
    pub total_supply: U256,
}

impl ERC20 {
    pub async fn new(http_url: &str) -> Result<Self> {
        let url = http_url.parse()?;
        let provider = ProviderBuilder::new().connect_http(url);
        let http_provider = provider.erased();

        Ok(Self { http_provider })
    }
}

impl Token {
    pub async fn new(token_address: &str, http_provider: &DynProvider) -> Result<Self> {
        let address = Address::from_str(token_address)?;
        let contract = ERC20Token::new(address, http_provider);
        let decimals = contract.decimals().call().await?;
        let symbol = contract.symbol().call().await?;
        let total_supply = contract.totalSupply().call().await?;

        Ok(Token {
            address,
            decimals,
            symbol,
            total_supply,
        })
    }
}
