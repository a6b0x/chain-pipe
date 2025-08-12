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
    let app_cfg = init::AppConfig::from_file_or_cli()?;
    app_cfg.init_log()?;
    info!("starting source-uniswap with config: {app_cfg:#?}");

    let factory_addr = Address::from_str(&app_cfg.uniswap_v2.factory_address)?;
    let eth_url = app_cfg.eth_node.ws_url;
    let uniswap_v2 = uni::UniswapV2::new(&eth_url, factory_addr).await?;

    let mq_client = mq::MqClient::new(&app_cfg.fluvio.broker_url, &app_cfg.fluvio.topic_name)
        .await
        .expect("Failed to connect to Fluvio. Please check the broker_url.");
    let topics = mq_client.list_topics().await?;
    let is_exist = topics.iter().any(|t| t == &app_cfg.fluvio.topic_name);
    if !is_exist {
        info!(
            "topic '{}' not found, creating...",
            app_cfg.fluvio.topic_name
        );
        mq_client.create_topic(&app_cfg.fluvio.topic_name).await?;
    } else {
        info!("topic {} already exist", app_cfg.fluvio.topic_name);
    }
    info!("topics: {topics:#?}");

    let mut stream = uniswap_v2.subscribe_pair_created().await?;
    while let Some(rpc_log) = stream.next().await {
        info!("pair created: {rpc_log:#?}");
        let primitives_log = rpc_log.clone().into();
        match UniswapV2Factory::PairCreated::decode_log(&primitives_log) {
            Ok(event) => {
                let payload = json!({
                "raw": rpc_log,
                "decoded": event,
                });
                let msg = serde_json::to_string(&payload)?;
                info!("pair created: {msg}");
                mq_client
                    .produce_record(&app_cfg.fluvio.topic_name, &msg)
                    .await?;
            }
            Err(e) => warn!("decode failed: {e}"),
        }
    }

    Ok(())
}
