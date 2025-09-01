use eyre::{eyre, Ok, Result};
use tracing::{info, warn};

mod init;
#[tokio::main]
async fn main() -> Result<()> {
    let app_cfg = init::AppConfig::from_file_or_cli()?;
    app_cfg.init_log()?;
    info!("starting price-sink with config: {app_cfg:#?}");
    Ok(())
}
