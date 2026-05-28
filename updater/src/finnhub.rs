use anyhow::Result;
use serde::Deserialize;
use sqlx::SqlitePool;
use std::time::Duration;

use kabu_shared::db;

#[derive(Debug, Deserialize)]
struct QuoteResponse {
    c: f64,          // current price
    d: Option<f64>,  // change
    dp: Option<f64>, // percent change
    h: f64,          // high
    l: f64,          // low
    o: f64,          // open
    pc: f64,         // previous close
}

async fn fetch_stock_quote(api_key: &str, symbol: &str) -> Result<QuoteResponse> {
    let client = reqwest::Client::new();
    let url = format!(
        "https://finnhub.io/api/v1/quote?symbol={}&token={}",
        symbol, api_key
    );
    let response = client
        .get(&url)
        .send()
        .await?
        .json::<QuoteResponse>()
        .await?;
    Ok(response)
}

pub async fn update_stock_prices(
    pool: &SqlitePool,
    api_key: &str,
    symbols: &[String],
) -> Result<()> {
    if symbols.is_empty() {
        return Ok(());
    }

    tracing::info!("Updating stock prices for {} symbols", symbols.len());

    for symbol in symbols {
        match fetch_stock_quote(api_key, symbol).await {
            Ok(quote) => {
                if quote.c == 0.0 {
                    tracing::warn!("Got zero price for {}, skipping", symbol);
                    continue;
                }
                db::insert_price(
                    pool,
                    symbol,
                    quote.c,
                    quote.d,
                    quote.dp,
                    Some(quote.h),
                    Some(quote.l),
                    Some(quote.o),
                    Some(quote.pc),
                )
                .await?;
                tracing::info!("{}: ${:.2}", symbol, quote.c);
            }
            Err(e) => {
                tracing::error!("Failed to fetch quote for {}: {}", symbol, e);
            }
        }

        // Finnhub free tier: 60 calls/min
        tokio::time::sleep(Duration::from_millis(1100)).await;
    }

    Ok(())
}
