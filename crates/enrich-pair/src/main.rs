use eyre::{Ok, Result};
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

mod erc20;
mod init;
mod mq;

#[derive(Deserialize)]
struct EventMsg {
    decoded: PairCreated,
}
#[derive(Deserialize)]
struct PairCreated {
    pair: String,
    token0: String,
    token1: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let app_cfg = init::AppConfig::from_file_or_cli()?;
    app_cfg.init_log()?;
    info!("starting enrich-pair with config: {app_cfg:#?}");

    let http_provider = erc20::ERC20::new(&app_cfg.eth_node.http_url)
        .await?
        .http_provider;

    let mq_client =
        mq::MqClient::new(&app_cfg.nats.server_url, &app_cfg.nats.subject_input).await?;

    let mut sub = mq_client.subscribe().await?;
    info!("listening on {}", app_cfg.nats.subject_input);

    while let Some(msg) = sub.next().await {
        let text = String::from_utf8_lossy(&msg.payload);
        info!("received raw : {}", text);

        let evt: EventMsg = serde_json::from_str(&text).expect("invalid json");
        info!(
            "pair = {}, token0 = {}, token1 = {}",
            evt.decoded.pair, evt.decoded.token0, evt.decoded.token1
        );

        let (t0, t1) = tokio::join!(
            erc20::Token::new(&evt.decoded.token0, &http_provider),
            erc20::Token::new(&evt.decoded.token1, &http_provider),
        );
        info!(t0 = ?t0, t1 = ?t1);
    }

    Ok(())
}
