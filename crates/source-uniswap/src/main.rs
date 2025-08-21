use alloy::{primitives::Address, sol_types::SolEvent};
//use chrono::Local;
use eyre::Result;
use futures_util::StreamExt;
use serde_json::json;
use std::str::FromStr;
use tracing::{info, warn};

use uni::UniswapV2Factory;

mod init;
mod mq;
mod uni;

#[tokio::main]
async fn main() -> Result<()> {
    // Load config (file + CLI)
    let app_cfg = init::AppConfig::from_file_or_cli()?;
    app_cfg.init_log()?;
    info!("starting source-uniswap with config: {app_cfg:#?}");

    // Connect to Ethereum
    let factory_addr = Address::from_str(&app_cfg.uniswap_v2.factory_address)?;
    let eth_url = app_cfg.eth_node.ws_url;
    let uniswap_v2 = uni::UniswapV2::new(&eth_url, factory_addr).await?;

    // Connect to NATS
    let mq_client = mq::MqClient::new(&app_cfg.nats.server_url, &app_cfg.nats.subject_name)
        .await
        .expect("Failed to connect to NATS server. Please check the server_url.");

    // Subscribe and forward events
    let mut stream = uniswap_v2.subscribe_pair_created().await?;
    info!("Listening for PairCreated events…");
    while let Some(rpc_log) = stream.next().await {
        let primitives_log = rpc_log.clone().into();
        let decode_log = UniswapV2Factory::PairCreated::decode_log(&primitives_log);
        match decode_log {
            Ok(event) => {
                let payload = json!({"raw": rpc_log,"decoded": event});
                let msg = serde_json::to_string(&payload)?;
                info!("Sending event: {msg}");
                mq_client.produce_record(msg).await?;
            }
            Err(e) => warn!("Decode failed: {e}"),
        }
    }
    Ok(())
}
