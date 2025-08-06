use clap::Parser;
use eyre::Result;
use serde::Deserialize;
use std::path::PathBuf;

/// Command-line arguments.
/// These settings override any values from the config file.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    /// WebSocket endpoint
    #[arg(long)]
    ws_url: Option<String>,

    /// Uniswap V2 Router contract address
    #[arg(long)]
    router_address: Option<String>,

    /// Fluvio broker
    #[arg(long)]
    broker_url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub ws_url: String,

    pub router_address: String,

    pub broker_url: String,
}

impl Settings {
    pub fn load() -> Result<Self> {
        let cli = Cli::parse();

        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("config/default.toml");

        let mut settings: Self = config::Config::builder()
            .add_source(config::File::from(path).required(false))
            .build()
            .map_err(eyre::Report::from)?
            .try_deserialize()?;

        if let Some(ws_url) = cli.ws_url {
            settings.ws_url = ws_url;
        }
        if let Some(router_address) = cli.router_address {
            settings.router_address = router_address;
        }
        if let Some(broker_url) = cli.broker_url {
            settings.broker_url = broker_url;
        }

        Ok(settings)
    }
}

fn main() -> Result<()> {
    let settings = Settings::load()?;
    println!("最终配置: {settings:#?}");
    Ok(())
}
