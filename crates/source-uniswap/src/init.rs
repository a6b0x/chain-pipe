use clap::Parser;
use config::{Config, File};
use eyre::Result;
use serde::Deserialize;
use std::path::Path;
use tracing::{debug, Level};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub eth_node: EthNodeConfig,
    pub uniswap_v2: UniswapV2Config,
    pub fluvio: FluvioConfig,
    pub log: Option<LogConfig>,
}
#[derive(Debug, Deserialize)]
pub struct EthNodeConfig {
    pub ws_url: String,
}
#[derive(Debug, Deserialize)]
pub struct UniswapV2Config {
    pub factory_address: String,
}

#[derive(Debug, Deserialize)]
pub struct FluvioConfig {
    pub broker_url: String,
    pub topic_name: String,
}
#[derive(Debug, Deserialize)]
pub struct LogConfig {
    pub level: String,
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(long)]
    ws_url: Option<String>,

    #[arg(long)]
    factory_address: Option<String>,

    #[arg(long)]
    broker_url: Option<String>,

    #[arg(long)]
    topic_name: Option<String>,
}

impl AppConfig {
    pub fn from_file_or_cli() -> Result<AppConfig> {
        let cli = Cli::parse();
        let config_path1 = Path::new("config/source-uniswap.toml");
        let config_path2 = Path::new("source-uniswap.toml");

        let cfg: AppConfig = Config::builder()
            .add_source(File::from(config_path1).required(false))
            .add_source(File::from(config_path2).required(false))
            .set_override_option("eth_node.ws_url", cli.ws_url)?
            .set_override_option("uniswap_v2.factory_address", cli.factory_address)?
            .set_override_option("fluvio.broker_url", cli.broker_url)?
            .set_override_option("fluvio.topic_name", cli.topic_name)?
            .build()
            .map_err(eyre::Report::from)?
            .try_deserialize()?;
        Ok(cfg)
    }

    pub fn init_log(&self) -> Result<()> {
        let level_str = self
            .log
            .as_ref()
            .map(|l| l.level.as_str())
            .unwrap_or("info");

        let filter = EnvFilter::try_from_default_env()
            .or_else(|_| EnvFilter::try_new(level_str))
            .map_err(eyre::Report::from)?;

        tracing_subscriber::registry()
            .with(fmt::layer())
            .with(filter)
            .init();

        debug!("log level configured to: '{}'", level_str);
        Ok(())
    }
}
