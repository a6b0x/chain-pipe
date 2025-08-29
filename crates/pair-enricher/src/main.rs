use alloy::primitives::Address;
use eyre::{Ok, Result};
use futures_util::StreamExt;
use serde::Deserialize;
use std::str::FromStr;
use tracing::info;

use chain_model::PairCreatedEvent;

mod init;
mod mq;
mod pair_erc20;

#[tokio::main]
async fn main() -> Result<()> {
    let app_cfg = init::AppConfig::from_file_or_cli()?;
    app_cfg.init_log()?;
    info!("starting enrich-pair with config: {app_cfg:#?}");

    let eth_reader = pair_erc20::EthReader::new(&app_cfg.eth_node.http_url).await?;

    let mq_client = mq::MqClient::new(
        &app_cfg.nats.server_url,
        &app_cfg.nats.subject_input,
        &app_cfg.nats.stream_name,
    )
    .await?;

    let kv = mq_client.kv_store(&app_cfg.nats.kv_bucket).await?;
    if let Some(addrs) = &app_cfg.uniswap_v2.pair_address {
        for addr in addrs {
            let key = Address::from_str(addr)?;
            let pair = eth_reader.fetch_pair(key).await?;
            let value = serde_json::to_vec(&pair)?;
            kv.put(key.to_string(), value.into()).await?;
            info!("put pair {} to kv store", key);
        }
    }

    let mut sub = mq_client.jetstream_pull_from(true).await?;
    info!("listening on {}", app_cfg.nats.subject_input);

    while let Some(msg_result) = sub.next().await {
        let msg = msg_result?;
        let text = String::from_utf8_lossy(&msg.payload);
        info!("received raw : {}", text);

        let event: PairCreatedEvent =
            serde_json::from_slice(&msg.payload).map_err(|e| eyre::eyre!("invalid json: {e}"))?;
        let pair = eth_reader
            .fetch_pair_token(event.pair, event.token0, event.token1)
            .await?;

        let value = serde_json::to_vec(&pair)?;

        kv.put(event.pair.to_string(), value.into()).await?;
        info!("put pair {} to kv store", event.pair);

        msg.ack()
            .await
            .map_err(|e| eyre::eyre!("ack message failed: {e}"))?;
    }

    Ok(())
}
