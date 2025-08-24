use clap::{Parser, Subcommand};
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
    pub nats: NatsConfig,
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
pub struct NatsConfig {
    pub server_url: String,
    pub subject_name: String,
}
#[derive(Debug, Deserialize)]
pub struct LogConfig {
    pub level: String,
}

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug, Clone)]
pub enum Commands {
    PairCreatedEvent {
        #[arg(long)]
        ws_url: Option<String>,
        #[arg(long)]
        factory_address: Option<String>,
        #[arg(long)]
        server_url: Option<String>,
        #[arg(long)]
        subject_name: Option<String>,
    },

    SyncEvent {
        #[arg(long)]
        ws_url: Option<String>,
        #[arg(long)]
        server_url: Option<String>,
        #[arg(long)]
        subject_name: Option<String>,
    },
}

impl AppConfig {
    pub fn from_file_or_cli() -> Result<(AppConfig, Commands)> {
        let cli = Cli::parse();
        let cmd = cli.command.clone();
        let config_path1 = Path::new("config/source-uniswap.toml");
        let config_path2 = Path::new("source-uniswap.toml");

        let mut builder = Config::builder()
            .add_source(File::from(config_path1).required(false))
            .add_source(File::from(config_path2).required(false));

        match cli.command {
            Commands::PairCreatedEvent {
                ws_url,
                factory_address,
                server_url,
                subject_name,
            } => {
                builder = builder
                    .set_override_option("eth_node.ws_url", ws_url)?
                    .set_override_option("uniswap_v2.factory_address", factory_address)?
                    .set_override_option("nats.server_url", server_url)?
                    .set_override_option("nats.subject_name", subject_name)?;
            }
            Commands::SyncEvent {
                ws_url,
                server_url,
                subject_name,
            } => {
                builder = builder
                    .set_override_option("eth_node.ws_url", ws_url)?
                    .set_override_option("nats.server_url", server_url)?
                    .set_override_option("nats.subject_name", subject_name)?;
            }
        }

        let cfg: AppConfig = builder
            .build()
            .map_err(eyre::Report::from)?
            .try_deserialize()?;
        Ok((cfg, cmd))
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
