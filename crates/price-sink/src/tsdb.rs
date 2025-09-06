use bigdecimal::BigDecimal;
use chain_model::PriceTick;
use chrono::TimeZone;
use eyre::{eyre, Result};
use std::str::FromStr;
use sqlx::{PgPool, postgres::PgPoolOptions};


#[derive(Clone, Debug)]
pub struct TsdbClient {
    pool: PgPool,
}

impl TsdbClient {
    pub async fn new(dsn: &str) -> Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(16)
            .connect(dsn)
            .await?;
        Ok(Self { pool })
    }

    pub async fn write(&self, tick: &PriceTick) -> Result<()> {
        let date_time = chrono::Utc
            .timestamp_opt(tick.block_timestamp as i64, 0)
            .single()
            .ok_or_else(|| eyre!("invalid timestamp"))?;

        let token0_reserve_dec = BigDecimal::from_str(&tick.token0_reserve.to_string())?;
        let token1_reserve_dec = BigDecimal::from_str(&tick.token1_reserve.to_string())?;

        sqlx::query(
            r#"
            INSERT INTO price_ticks (
                time, pair_address,
                token0_address, token0_symbol, token0_reserve,
                token1_address, token1_symbol, token1_reserve,
                token0_token1, token1_token0,
                block_number, transaction_hash
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            "#,
        )
        .bind(date_time)
        .bind(tick.pair_address.to_string())
        .bind(tick.token0_address.to_string())
        .bind(tick.token0_symbol.as_str())
        .bind(token0_reserve_dec)
        .bind(tick.token1_address.to_string())
        .bind(tick.token1_symbol.as_str())
        .bind(token1_reserve_dec)
        .bind(tick.token0_token1)
        .bind(tick.token1_token0)
        .bind(tick.block_number as i64)
        .bind(tick.transaction_hash.to_string())
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}
