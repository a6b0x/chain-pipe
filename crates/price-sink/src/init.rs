use clap::Parser;
use config::{Config, File};
use eyre::Result;
use serde::Deserialize;
use std::path::Path;
use tracing::debug;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub nats: NatsConfig,
    pub timescale: TimescaleConfig,
    pub log: Option<LogConfig>,
}

#[derive(Debug, Deserialize)]
pub struct NatsConfig {
    pub server_url: String,
    pub subject_name: String,
    pub stream_name: String,
}

#[derive(Debug, Deserialize)]
pub struct TimescaleConfig {
    pub dsn: String,
}

#[derive(Debug, Deserialize)]
pub struct LogConfig {
    pub level: String,
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(long)]
    server_url: Option<String>,
    #[arg(long)]
    subject_name: Option<String>,
    #[arg(long)]
    stream_name: Option<String>,
    #[arg(long)]
    dsn: Option<String>,
}

impl AppConfig {
    pub fn from_file_or_cli() -> Result<Self> {
        let cli = Cli::parse();
        let config_path1 = Path::new("config/price-sink.toml");
        let config_path2 = Path::new("price-sink.toml");

        let cfg: AppConfig = Config::builder()
            .add_source(File::from(config_path1).required(false))
            .add_source(File::from(config_path2).required(false))
            .set_override_option("nats.server_url", cli.server_url)?
            .set_override_option("nats.subject_name", cli.subject_name)?
            .set_override_option("nats.stream_name", cli.stream_name)?
            .set_override_option("timescale.dsn", cli.dsn)?
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
