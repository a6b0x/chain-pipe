use alloy::primitives::Address;
use eyre::{Ok, Result};
use futures_util::StreamExt;
use serde::Deserialize;
use std::str::FromStr;
use tracing::info;

mod init;
mod mq;
mod pair_erc20;

#[derive(Deserialize)]
struct EventMsg {
    decode_log: PairCreated,
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

    let http_provider = pair_erc20::ERC20::new(&app_cfg.eth_node.http_url)
        .await?
        .http_provider;

    let mq_client = mq::MqClient::new(
        &app_cfg.nats.server_url,
        &app_cfg.nats.subject_input,
        &app_cfg.nats.stream_name,
    )
    .await?;

    let kv = mq_client.kv_store(&app_cfg.nats.kv_bucket).await?;
    if let Some(addrs) = &app_cfg.uniswap_v2.pair_address {
        for addr in addrs {
            let pair_addr = Address::from_str(addr)?;
            let pair = pair_erc20::Pair::new(addr, &http_provider).await?;
            let value = serde_json::to_vec(&pair)?;
            kv.put(addr.clone(), value.into()).await?;
            info!("put pair {} to kv store", addr);
        }
    }

    let mut sub = mq_client.jetstream_pull_from(true).await?;
    info!("listening on {}", app_cfg.nats.subject_input);

    while let Some(msg_result) = sub.next().await {
        let msg = msg_result?;
        let text = String::from_utf8_lossy(&msg.payload);
        info!("received raw : {}", text);

        let evt: EventMsg =
            serde_json::from_str(&text).map_err(|e| eyre::eyre!("invalid json: {e}"))?;
        let (t0, t1) = tokio::join!(
            pair_erc20::Token::new(&evt.decode_log.token0, &http_provider),
            pair_erc20::Token::new(&evt.decode_log.token1, &http_provider),
        );

        let pair_addr = evt.decode_log.pair.clone();
        let pair = pair_erc20::Pair {
            address: Address::from_str(&pair_addr)?,
            token0: t0?,
            token1: t1?,
        };
        let value = serde_json::to_vec(&pair)?;

        kv.put(evt.decode_log.pair, value.into()).await?;
        info!("put pair {} to kv store", pair_addr);

        msg.ack()
            .await
            .map_err(|e| eyre::eyre!("ack message failed: {e}"))?;
    }

    Ok(())
}
