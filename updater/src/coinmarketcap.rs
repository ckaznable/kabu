use anyhow::Result;
use serde::Deserialize;
use sqlx::SqlitePool;
use std::collections::HashMap;

use kabu_shared::db;

#[derive(Debug, Deserialize)]
struct CmcResponse {
    data: Option<HashMap<String, CmcCoin>>,
    status: CmcStatus,
}

#[derive(Debug, Deserialize)]
struct CmcStatus {
    error_code: i64,
    error_message: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CmcCoin {
    quote: HashMap<String, CmcQuote>,
}

#[derive(Debug, Deserialize)]
struct CmcQuote {
    price: f64,
    percent_change_24h: Option<f64>,
}

pub async fn update_crypto_prices(
    pool: &SqlitePool,
    api_key: &str,
    symbols: &[String],
) -> Result<()> {
    if symbols.is_empty() {
        return Ok(());
    }

    let client = reqwest::Client::new();
    let joined = symbols.join(",");

    tracing::info!("Fetching crypto prices for: {}", joined);

    let response = client
        .get("https://pro-api.coinmarketcap.com/v1/cryptocurrency/quotes/latest")
        .header("X-CMC_PRO_API_KEY", api_key)
        .query(&[("symbol", joined.as_str()), ("convert", "USD")])
        .send()
        .await?;

    let body = response.text().await?;
    let cmc: CmcResponse = serde_json::from_str(&body).map_err(|e| {
        anyhow::anyhow!(
            "Failed to parse CMC response: {} (body: {})",
            e,
            &body[..body.len().min(300)]
        )
    })?;

    if cmc.status.error_code != 0 {
        anyhow::bail!(
            "CMC API error: {}",
            cmc.status.error_message.unwrap_or_default()
        );
    }

    let data = cmc
        .data
        .ok_or_else(|| anyhow::anyhow!("No data in CMC response"))?;

    for symbol in symbols {
        let upper = symbol.to_uppercase();
        if let Some(coin) = data.get(&upper) {
            if let Some(usd) = coin.quote.get("USD") {
                if usd.price == 0.0 {
                    tracing::warn!("Got zero price for {}, skipping", symbol);
                    continue;
                }
                db::insert_price(
                    pool,
                    symbol,
                    usd.price,
                    None,
                    usd.percent_change_24h,
                    None,
                    None,
                    None,
                    None,
                )
                .await?;
                tracing::info!("{}: ${:.2}", symbol, usd.price);
            }
        } else {
            tracing::warn!("No CMC data for {}", symbol);
        }
    }

    Ok(())
}
