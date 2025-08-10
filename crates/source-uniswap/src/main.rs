use chrono::Local;
use eyre::Result;
use tracing::info;

mod init;
mod mq;

#[tokio::main]
async fn main() -> Result<()> {
    let app_cfg = init::AppConfig::from_file_or_cli()?;
    app_cfg.init_log()?;
    info!("starting source-uniswap with config: {app_cfg:#?}");

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

    let msg = format!("Hello World! - Time is {}", Local::now().to_rfc2822());
    mq_client
        .produce_record(&app_cfg.fluvio.topic_name, &msg)
        .await?;

    Ok(())
}
