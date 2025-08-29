use alloy::primitives::Address;
use alloy::providers::{DynProvider, Provider, ProviderBuilder};
use alloy::sol;
use eyre::Result;
use serde::Serialize;

use chain_model::{Pair, Token};

sol!(
    #[allow(missing_docs)]
    #[sol(rpc)]
    #[derive(Debug, Serialize)]
    ERC20Token,
    "abi/ERC20.json"
);

sol!(
    #[allow(clippy::too_many_arguments)]
    #[allow(missing_docs)]
    #[sol(rpc)]
    #[derive(Debug, Serialize)]
    UniswapV2Pair,
    "abi/UniswapV2Pair.json"
);

pub struct EthReader {
    pub http_provider: DynProvider,
}

impl EthReader {
    pub async fn new(http_url: &str) -> Result<Self> {
        let url = http_url.parse()?;
        let provider = ProviderBuilder::new().connect_http(url);
        let http_provider = provider.erased();

        Ok(Self { http_provider })
    }

    pub async fn fetch_token(&self, token_address: Address) -> Result<Token> {
        let contract = ERC20Token::new(token_address, &self.http_provider);

        let decimals = contract.decimals().call().await?;
        let symbol = contract.symbol().call().await?;
        let total_supply = contract.totalSupply().call().await?;

        Ok(Token {
            address: token_address,
            decimals,
            symbol,
            total_supply,
        })
    }

    pub async fn fetch_pair_token(
        &self,
        pair_address: Address,
        token0: Address,
        token1: Address,
    ) -> Result<Pair> {
        // let contract = UniswapV2Pair::new(pair_address, &self.http_provider);
        // let token0 = contract.token0().call().await?;
        // let token1 = contract.token1().call().await?;

        let (token0_res, token1_res) =
            tokio::join!(self.fetch_token(token0), self.fetch_token(token1));

        Ok(Pair {
            address: pair_address,
            token0: token0_res?,
            token1: token1_res?,
        })
    }
    pub async fn fetch_pair(&self, pair_address: Address) -> Result<Pair> {
        let contract = UniswapV2Pair::new(pair_address, &self.http_provider);
        let token0 = contract.token0().call().await?;
        let token1 = contract.token1().call().await?;

        let (token0_res, token1_res) =
            tokio::join!(self.fetch_token(token0), self.fetch_token(token1));

        Ok(Pair {
            address: pair_address,
            token0: token0_res?,
            token1: token1_res?,
        })
    }
}
