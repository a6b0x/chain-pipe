use alloy::primitives::Address;
use alloy::providers::{DynProvider, Provider, ProviderBuilder, WsConnect};
use alloy::rpc::types::{Filter, Log};
use alloy::sol;
use alloy::sol_types::SolEvent;
use eyre::Result;
use futures_util::StreamExt;

sol!(
    #[allow(missing_docs)]
    #[sol(rpc)]
    #[derive(Debug)]
    UniswapV2Factory,
    "abi/UniswapV2Factory.json"
);

pub struct UniswapV2 {
    pub ws_provider: DynProvider,
    pub factory: UniswapV2Factory::UniswapV2FactoryInstance<DynProvider>,
}

impl UniswapV2 {
    pub async fn new(ws_url: &str, factory_address: Address) -> Result<Self> {
        let ws_connect = WsConnect::new(ws_url);
        let provider = ProviderBuilder::new().connect_ws(ws_connect).await?;

        let ws_provider = provider.erased();

        let factory = UniswapV2Factory::new(factory_address, ws_provider.clone());
        Ok(Self {
            ws_provider,
            factory,
        })
    }

    pub async fn subscribe_pair_created(&self) -> Result<impl StreamExt<Item = Log>> {
        let event_signature = UniswapV2Factory::PairCreated::SIGNATURE_HASH;
        let filter = Filter::new()
            .event_signature(event_signature)
            .address(*self.factory.address());

        let sub = self.ws_provider.subscribe_logs(&filter).await?;
        Ok(sub.into_stream())
    }
}
