use eyre::{eyre, Result};
use futures_util::StreamExt;
use tracing::{info, warn};

use chain_model::PriceTick;

mod init;
mod mq;
mod tsdb;

#[tokio::main]
async fn main() -> Result<()> {
    let app_cfg = init::AppConfig::from_file_or_cli()?;
    app_cfg.init_log()?;
    info!("starting price-sink with config: {app_cfg:#?}");

    let tsdb = tsdb::TsdbClient::new(&app_cfg.timescale.dsn).await?;
    info!("connected to TimescaleDB");

    let mq_client = mq::MqClient::new(
        &app_cfg.nats.server_url,
        &app_cfg.nats.subject_name,
        &app_cfg.nats.stream_name,
    )
    .await?;

    let mut sub = mq_client.jetstream_pull_from(true).await?;
    while let Some(msg_result) = sub.next().await {
        let msg = msg_result?;
        info!("received message: {msg:?}");
        match serde_json::from_slice::<PriceTick>(&msg.payload) {
            Ok(tick) => {
                if let Err(e) = tsdb.write(&tick).await {
                    warn!("failed to write tick: {e}");
                } else {
                    info!("wrote tick {}", tick.transaction_hash);
                    msg.ack()
                        .await
                        .map_err(|e| eyre!("ack message failed: {e}"))?;
                }
            }
            Err(e) => {
                warn!("invalid payload: {e}");
                msg.ack()
                    .await
                    .map_err(|e| eyre!("ack message failed: {e}"))?;
            }
        }
    }

    Ok(())
}
