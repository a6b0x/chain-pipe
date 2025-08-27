use alloy::primitives::{Address, FixedBytes, U256};
use eyre::{Ok, Result};
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use std::str::from_utf8;
use tracing::{info, warn};

mod init;
mod mq;

#[derive(Debug, Deserialize)]
struct EventMsg {
    decode_log: SyncEvent,
}
#[derive(Debug, Deserialize, Serialize)]
struct SyncEvent {
    address: Address,
    reserve0: U256,
    reserve1: U256,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Pair {
    pub address: String,
    pub token0: Token,
    pub token1: Token,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Token {
    pub address: String,
    pub decimals: u8,
    pub symbol: String,
    pub total_supply: U256,
}

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

        let event: EventMsg = serde_json::from_slice(&msg.payload)?;

        let entry = kv.entry(event.decode_log.address.to_string()).await?;
        if let Some(entry) = entry {
            info!(
                "{} @ {} -> {}",
                entry.key,
                entry.revision,
                from_utf8(&entry.value)?
            );

            let pair: Pair = serde_json::from_slice(&entry.value)?;

            let reserve0 = event.decode_log.reserve0.to::<u128>() as f64;
            let reserve1 = event.decode_log.reserve1.to::<u128>() as f64;

            if reserve0 == 0.0 {
                warn!("reserve0 is zero, skip");
                continue;
            }
            let price0 = (reserve1 / reserve0)
                * 10f64.powi(pair.token0.decimals as i32 - pair.token1.decimals as i32);
            let price1 = 1.0 / price0;

            let price_msg = serde_json::json!({
                "pair": pair,
                "decode_log": event.decode_log,
                "price0": price0,
                "price1": price1,
            });

            mq_client.produce_record(price_msg.to_string()).await?;
            info!("price msg: {price_msg}");
        }
        msg.ack()
            .await
            .map_err(|e| eyre::eyre!("ack message failed: {e}"))?;
    }

    Ok(())
}
