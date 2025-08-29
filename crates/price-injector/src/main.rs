use alloy::primitives::{Address, FixedBytes, U256};
use eyre::{Ok, Result};
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use std::str::from_utf8;
use tracing::{info, warn};

use chain_model::{Pair, PriceTick, SyncEvent};

mod init;
mod mq;

#[tokio::main]
async fn main() -> Result<()> {
    let app_cfg = init::AppConfig::from_file_or_cli()?;
    app_cfg.init_log()?;
    info!("starting price-injector with config: {app_cfg:#?}");

    let mq_client = mq::MqClient::new(
        &app_cfg.nats.server_url,
        &app_cfg.nats.subject_input,
        &app_cfg.nats.subject_output,
        &app_cfg.nats.stream_name,
    )
    .await?;

    let kv = mq_client.kv_store(&app_cfg.nats.kv_bucket).await?;

    let mut sub = mq_client.jetstream_pull_from(true).await?;
    while let Some(msg_result) = sub.next().await {
        let msg = msg_result?;

        let event: SyncEvent = serde_json::from_slice(&msg.payload)?;

        let entry = kv.entry(event.pair.to_string()).await?;
        if let Some(entry) = entry {
            info!(
                "{} @ {} -> {}",
                entry.key,
                entry.revision,
                from_utf8(&entry.value)?
            );

            let pair: Pair = serde_json::from_slice(&entry.value)?;

            let reserve0 = event.reserve0.to_string().parse::<f64>()?;
            let reserve1 = event.reserve1.to_string().parse::<f64>()?;

            if reserve0 == 0.0 {
                warn!("reserve0 is zero, skip");
                continue;
            }
            let t1_t0 = (reserve1 / reserve0)
                * 10f64.powi(pair.token0.decimals as i32 - pair.token1.decimals as i32);
            let t0_t1 = 1.0 / t1_t0;

            let price_msg = PriceTick {
                pair_address: event.pair.to_string(),
                token0: pair.token0.address.to_string(),
                token1: pair.token1.address.to_string(),
                reserve0: event.reserve0.to_string(),
                reserve1: event.reserve1.to_string(),
                t1_t0,
                t0_t1,
                symbol0: pair.token0.symbol,
                symbol1: pair.token1.symbol,
                transaction_hash: event.transaction_hash.to_string(),
                block_number: event.block_number,
                block_timestamp: event.block_timestamp,
            };

            let payload = serde_json::to_string(&price_msg)?;
            mq_client.produce_record(payload).await?;
            info!("price msg: {price_msg:?}");
        }
        msg.ack()
            .await
            .map_err(|e| eyre::eyre!("ack message failed: {e}"))?;
    }

    Ok(())
}
