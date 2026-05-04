use anyhow::{anyhow, Result};
use serde::Deserialize;
use sqlx::SqlitePool;
use std::time::Duration;

use kabu_shared::db;

#[derive(Debug, Deserialize)]
struct TwseQuoteResponse {
    #[serde(rename = "msgArray")]
    msg_array: Vec<TwseQuoteItem>,
}

#[derive(Debug, Deserialize)]
struct TwseQuoteItem {
    // latest traded price
    z: String,
    // open
    o: String,
    // high
    h: String,
    // low
    l: String,
    // previous close
    y: String,
}

fn parse_price(raw: &str) -> Option<f64> {
    let trimmed = raw.trim();
    if trimmed.is_empty() || trimmed == "-" {
        return None;
    }
    trimmed.parse::<f64>().ok()
}

fn detect_market(symbol: &str) -> Option<&'static str> {
    if symbol.ends_with(".TW") || symbol.chars().all(|c| c.is_ascii_digit()) {
        return Some("tse");
    }
    if symbol.ends_with(".TWO") {
        return Some("otc");
    }
    None
}

fn to_tw_code(symbol: &str) -> String {
    symbol
        .trim()
        .trim_end_matches(".TW")
        .trim_end_matches(".TWO")
        .to_string()
}

async fn fetch_quote(symbol: &str) -> Result<TwseQuoteItem> {
    let market = detect_market(symbol).ok_or_else(|| anyhow!("unsupported TW symbol {}", symbol))?;
    let code = to_tw_code(symbol);
    let ex_ch = format!("{}_{}.tw", market, code);
    let url = format!(
        "https://mis.twse.com.tw/stock/api/getStockInfo.jsp?ex_ch={}",
        ex_ch
    );
    let resp = reqwest::Client::new()
        .get(&url)
        .send()
        .await?
        .json::<TwseQuoteResponse>()
        .await?;

    resp.msg_array
        .into_iter()
        .next()
        .ok_or_else(|| anyhow!("no quote returned for {}", symbol))
}

pub async fn update_tw_stock_prices(pool: &SqlitePool, symbols: &[String]) -> Result<()> {
    if symbols.is_empty() {
        return Ok(());
    }

    tracing::info!("Updating TW stock prices for {} symbols", symbols.len());

    for symbol in symbols {
        match fetch_quote(symbol).await {
            Ok(q) => {
                let price = parse_price(&q.z);
                let open = parse_price(&q.o);
                let high = parse_price(&q.h);
                let low = parse_price(&q.l);
                let prev_close = parse_price(&q.y);

                let Some(current) = price else {
                    tracing::warn!("TW quote has no latest price for {}, skipping", symbol);
                    continue;
                };

                let (change, change_percent) = match prev_close {
                    Some(pc) if pc > 0.0 => {
                        let c = current - pc;
                        (Some(c), Some((c / pc) * 100.0))
                    }
                    _ => (None, None),
                };

                db::insert_price(
                    pool,
                    symbol,
                    current,
                    change,
                    change_percent,
                    high,
                    low,
                    open,
                    prev_close,
                )
                .await?;
                tracing::info!("{}: NT${:.2}", symbol, current);
            }
            Err(e) => tracing::error!("Failed to fetch TW quote for {}: {}", symbol, e),
        }

        tokio::time::sleep(Duration::from_millis(400)).await;
    }

    Ok(())
}
