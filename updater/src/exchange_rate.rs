use anyhow::Result;
use serde::Deserialize;
use sqlx::SqlitePool;
use std::collections::HashMap;

use kabu_shared::db;

#[derive(Debug, Deserialize)]
struct ExchangeRateApiResponse {
    result: String,
    base_code: Option<String>,
    conversion_rates: Option<HashMap<String, f64>>,
    #[serde(rename = "error-type")]
    error_type: Option<String>,
}

pub async fn update_exchange_rates(
    pool: &SqlitePool,
    api_key: &str,
    base: &str,
    currencies: &[String],
) -> Result<()> {
    if currencies.is_empty() {
        return Ok(());
    }

    let symbols = currencies.join(",");
    tracing::info!(
        "Fetching exchange rates from exchangerate-api: {} -> {}",
        base,
        symbols
    );

    let client = reqwest::Client::new();
    let response = client
        .get(format!(
            "https://v6.exchangerate-api.com/v6/{}/latest/{}",
            api_key, base
        ))
        .send()
        .await?;

    let body = response.text().await?;
    let data: ExchangeRateApiResponse = serde_json::from_str(&body).map_err(|e| {
        anyhow::anyhow!(
            "Failed to parse exchangerate-api response: {} (body: {})",
            e,
            &body[..body.len().min(300)]
        )
    })?;

    if data.result != "success" {
        let error_type = data.error_type.as_deref().unwrap_or("unknown");
        anyhow::bail!(
            "exchangerate-api request failed (result={}, error_type={})",
            data.result,
            error_type
        );
    }

    let base_code = data
        .base_code
        .ok_or_else(|| anyhow::anyhow!("exchangerate-api: missing base_code in response"))?;
    let rates = data
        .conversion_rates
        .ok_or_else(|| anyhow::anyhow!("exchangerate-api: missing conversion_rates in response"))?;

    for currency in currencies {
        let normalized = currency.trim().to_uppercase();
        if normalized.is_empty() {
            continue;
        }
        if let Some(rate) = rates.get(&normalized) {
            db::upsert_exchange_rate(pool, &base_code, &normalized, *rate).await?;
            tracing::info!("{}/{}: {:.4}", base_code, normalized, rate);
        } else {
            tracing::warn!(
                "Currency {} not found in exchangerate-api response for base {}",
                normalized,
                base_code
            );
        }
    }

    Ok(())
}
