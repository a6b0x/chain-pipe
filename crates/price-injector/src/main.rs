use alloy::primitives::U256;
use eyre::{eyre, Ok, Result};
use futures_util::StreamExt;
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

            let reserve0 = U256::from(event.reserve0);
            let reserve1 = U256::from(event.reserve1);

            if reserve0.is_zero() {
                warn!("reserve0 is zero, skip");
                msg.ack()
                    .await
                    .map_err(|e| eyre!("ack message failed: {e}"))?;
                continue;
            }

            // Calculate price with adjusted decimals using U256 to avoid precision loss.
            let scaling_factor = U256::from(10_u64).pow(U256::from(18_u64));
            let decimal_diff = pair.token0.decimals as i32 - pair.token1.decimals as i32;
            // 10^|decimal_diff|
            let adjustment = U256::from(10u64).pow(U256::from(decimal_diff.unsigned_abs() as u32));

            let token1_token0_precise = if decimal_diff >= 0 {
                (reserve1 * adjustment * scaling_factor) / reserve0
            } else {
                (reserve1 * scaling_factor) / (reserve0 * adjustment)
            };

            // For quick, less-precise views, convert to f64 at the end.
            let token0_token1 = token1_token0_precise.to_string().parse::<f64>()? / 1e18;

            let token1_token0 = if token0_token1 == 0.0 {
                0.0
            } else {
                1.0 / token0_token1
            };

            let price_msg = PriceTick {
                pair_address: event.pair.to_string(),

                token0_address: pair.token0.address.to_string(),
                token0_reserve: event.reserve0.to_string(),
                token0_symbol: pair.token0.symbol,

                token1_address: pair.token1.address.to_string(),
                token1_reserve: event.reserve1.to_string(),
                token1_symbol: pair.token1.symbol,

                token0_token1,
                token1_token0,

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
            .map_err(|e| eyre!("ack message failed: {e}"))?;
    }

    Ok(())
}
