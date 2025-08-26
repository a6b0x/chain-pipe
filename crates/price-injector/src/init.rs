use clap::Parser;
use config::{Config, File};
use eyre::Result;
use serde::Deserialize;
use std::path::Path;
use tracing::{debug, Level};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub nats: NatsConfig,
    pub log: Option<LogConfig>,
}

#[derive(Debug, Deserialize)]
pub struct NatsConfig {
    pub server_url: String,
    pub subject_input: String,
    pub kv_bucket: String,
    pub subject_output: String,
    pub stream_name: String,
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
    subject_input: Option<String>,
    #[arg(long)]
    subject_output: Option<String>,
    #[arg(long)]
    kv_bucket: Option<String>,
    #[arg(long)]
    stream_name: Option<String>,
}

impl AppConfig {
    pub fn from_file_or_cli() -> Result<Self> {
        let cli = Cli::parse();
        let config_path1 = Path::new("config/price-injector.toml");
        let config_path2 = Path::new("price-injector.toml");

        let cfg: AppConfig = Config::builder()
            .add_source(File::from(config_path1).required(false))
            .add_source(File::from(config_path2).required(false))
            .set_override_option("nats.server_url", cli.server_url)?
            .set_override_option("nats.subject_input", cli.subject_input)?
            .set_override_option("nats.subject_output", cli.subject_output)?
            .set_override_option("nats.kv_bucket", cli.kv_bucket)?
            .set_override_option("nats.stream_name", cli.stream_name)?
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
