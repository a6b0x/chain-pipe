use eyre::Result;
use tracing::info;

mod init;

fn main() -> Result<()> {
    let app_cfg = init::AppConfig::from_file_or_cli()?;
    app_cfg.init_log()?;
    info!("starting source-uniswap with config: {app_cfg:#?}");
    Ok(())
}
