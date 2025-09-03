use chain_model::PriceTick;
use chrono::TimeZone;
use deadpool_postgres::{Manager, Pool};
use eyre::{eyre, Result};
use rust_decimal::Decimal;
use std::str::FromStr;
use tokio_postgres::NoTls;

#[derive(Clone, Debug)]
pub struct TsdbClient {
    pool: Pool,
}

impl TsdbClient {
    pub async fn new(dsn: &str) -> Result<Self> {
        let mgr = Manager::new(dsn.parse()?, NoTls);
        let pool = Pool::builder(mgr).max_size(16).build()?;
        Ok(Self { pool })
    }

    pub async fn write(&self, tick: &PriceTick) -> Result<()> {
        let client = self.pool.get().await?;
        let date_time = chrono::Utc
            .timestamp_opt(tick.block_timestamp as i64, 0)
            .single()
            .ok_or_else(|| eyre!("invalid timestamp"))?;

        let token0_reserve_dec = Decimal::from_str(&tick.token0_reserve.to_string())?;
        let token1_reserve_dec = Decimal::from_str(&tick.token1_reserve.to_string())?;

        client
            .execute(
                "INSERT INTO price_ticks (
                    time, pair_address,
                    token0_address, token0_symbol, token0_reserve,
                    token1_address, token1_symbol, token1_reserve,
                    token0_token1, token1_token0,
                    block_number, transaction_hash
                ) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12)",
                &[
                    &date_time,
                    &tick.pair_address,
                    &tick.token0_address,
                    &tick.token0_symbol,
                    &token0_reserve_dec,
                    &tick.token1_address,
                    &tick.token1_symbol,
                    &token1_reserve_dec,
                    &tick.token0_token1,
                    &tick.token1_token0,
                    &(tick.block_number as i64),
                    &tick.transaction_hash,
                ],
            )
            .await?;
        Ok(())
    }
}
