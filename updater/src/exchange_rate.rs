use anyhow::Result;
use serde::Deserialize;
use sqlx::SqlitePool;
use std::collections::HashMap;

use kabu_shared::db;

#[derive(Debug, Deserialize)]
struct FrankfurterResponse {
    base: String,
    rates: HashMap<String, f64>,
}

pub async fn update_exchange_rates(
    pool: &SqlitePool,
    base: &str,
    currencies: &[String],
) -> Result<()> {
    if currencies.is_empty() {
        return Ok(());
    }

    let symbols = currencies.join(",");
    tracing::info!("Fetching exchange rates: {} -> {}", base, symbols);

    let client = reqwest::Client::new();
    let response = client
        .get("https://api.frankfurter.dev/v1/latest")
        .query(&[("base", base), ("symbols", &symbols)])
        .send()
        .await?;

    let body = response.text().await?;
    let data: FrankfurterResponse = serde_json::from_str(&body).map_err(|e| {
        anyhow::anyhow!(
            "Failed to parse Frankfurter response: {} (body: {})",
            e,
            &body[..body.len().min(300)]
        )
    })?;

    for (currency, rate) in &data.rates {
        db::upsert_exchange_rate(pool, &data.base, currency, *rate).await?;
        tracing::info!("{}/{}: {:.4}", data.base, currency, rate);
    }

    Ok(())
}
