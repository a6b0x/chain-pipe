use alloy::{primitives::Address, sol_types::SolEvent};
//use chrono::Local;
use eyre::Result;
use futures_util::StreamExt;
use serde_json::json;
use std::str::FromStr;
use tracing::{info, warn};

use chain_model::SyncEvent;
use init::AppConfig;
use init::Commands;
use uni::{UniswapV2Factory, UniswapV2Pair};
mod init;
mod mq;
mod uni;

#[tokio::main]
async fn main() -> Result<()> {
    // Load config (file + CLI)
    let (app_cfg, app_cmd) = init::AppConfig::from_file_or_cli()?;
    app_cfg.init_log()?;
    info!("starting source-uniswap with config: {app_cfg:#?}");

    match app_cmd {
        Commands::PairCreatedEvent { .. } => run_pair_created(app_cfg).await?,
        Commands::SyncEvent { .. } => run_sync_event(app_cfg).await?,
    }

    Ok(())
}

async fn run_pair_created(app_cfg: AppConfig) -> Result<()> {
    // Connect to Ethereum
    let factory_addr = Address::from_str(&app_cfg.uniswap_v2.factory_address)?;
    let eth_url = app_cfg.eth_node.ws_url;
    let uniswap_v2 = uni::UniswapV2::new(&eth_url, factory_addr).await?;

    // Connect to NATS
    let mq_client = mq::MqClient::new(&app_cfg.nats.server_url, &app_cfg.nats.subject_name)
        .await
        .expect("Failed to connect to NATS server. Please check the server_url.");

    // Subscribe and forward events
    let mut stream_event = uniswap_v2.subscribe_pair_created_event().await?;
    info!("Listening for PairCreated events…");
    while let Some(rpc_log) = stream_event.next().await {
        let primitives_log = rpc_log.clone().into();
        let decode_log = UniswapV2Factory::PairCreated::decode_log(&primitives_log);
        match decode_log {
            Ok(event) => {
                let payload = json!({"rpc_log": rpc_log,"decode_log": event});
                let msg = serde_json::to_string(&payload)?;
                info!("Sending event: {msg}");
                mq_client.produce_record(msg).await?;
            }
            Err(e) => warn!("Decode failed: {e}"),
        }
    }
    Ok(())
}

async fn run_sync_event(app_cfg: AppConfig) -> Result<()> {
    let uniswap = uni::UniswapV2::new(&app_cfg.eth_node.ws_url, Address::ZERO).await?;
    let mq_client = mq::MqClient::new(&app_cfg.nats.server_url, &app_cfg.nats.subject_name).await?;

    let mut stream_event = uniswap.subscribe_sync_event().await?;
    info!("Listening for Sync events…");

    while let Some(rpc_log) = stream_event.next().await {
        let primitives_log = rpc_log.clone().into();
        let decode_log = UniswapV2Pair::Sync::decode_log(&primitives_log);
        match decode_log {
            Ok(event) => {
                // let payload = json!({"rpc_log": rpc_log,"decode_log": event});
                let payload = SyncEvent {
                    pair_address: event.address,
                    pair_reserve0: event.reserve0,
                    pair_reserve1: event.reserve1,
                    transaction_hash: rpc_log.transaction_hash.unwrap_or_default(),
                    block_hash: rpc_log.block_hash.unwrap_or_default(),
                    block_number: rpc_log.block_number.unwrap_or_default(),
                    block_timestamp: rpc_log.block_timestamp.unwrap_or_default(),
                };
                let msg = serde_json::to_string(&payload)?;
                info!("Sending event: {msg}");
                mq_client.produce_record(msg).await?;
            }
            Err(e) => warn!("Decode failed: {e}"),
        }
    }
    Ok(())
}
