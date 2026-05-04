use kabu_shared::config::Config;
use kabu_shared::db;

mod coinmarketcap;
mod exchange_rate;
mod finnhub;
mod twse;

fn is_tw_symbol(symbol: &str) -> bool {
    symbol.ends_with(".TW") || symbol.ends_with(".TWO") || symbol.chars().all(|c| c.is_ascii_digit())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let config = Config::load_default()?;
    let pool = db::init_pool(&config.server.database_url).await?;

    let all = db::list_stocks_by_type(&pool).await?;
    let stock_symbols: Vec<String> = all
        .iter()
        .filter(|(_, t)| t != "crypto")
        .map(|(s, _)| s.clone())
        .collect();
    let tw_stock_symbols: Vec<String> = stock_symbols
        .iter()
        .filter(|s| is_tw_symbol(s))
        .cloned()
        .collect();
    let global_stock_symbols: Vec<String> = stock_symbols
        .iter()
        .filter(|s| !is_tw_symbol(s))
        .cloned()
        .collect();
    let crypto_symbols: Vec<String> = all
        .iter()
        .filter(|(_, t)| t == "crypto")
        .map(|(s, _)| s.clone())
        .collect();

    if stock_symbols.is_empty() && crypto_symbols.is_empty() {
        tracing::info!("No assets to update");
    }

    if !global_stock_symbols.is_empty() {
        let finnhub_key = config.finnhub.resolve_api_key()?;
        finnhub::update_stock_prices(&pool, &finnhub_key, &global_stock_symbols).await?;
    }

    if !tw_stock_symbols.is_empty() {
        twse::update_tw_stock_prices(&pool, &tw_stock_symbols).await?;
    }

    if !crypto_symbols.is_empty() {
        let cmc_key = config.coinmarketcap.resolve_api_key()?;
        coinmarketcap::update_crypto_prices(&pool, &cmc_key, &crypto_symbols).await?;
    }

    // Exchange rates
    let exchange_rate_key = config.exchange_rate.resolve_api_key()?;
    exchange_rate::update_exchange_rates(
        &pool,
        &exchange_rate_key,
        &config.exchange_rate.base,
        &config.exchange_rate.currencies,
    )
    .await?;

    let summary = db::compute_portfolio_summary(&pool).await?;
    db::insert_portfolio_snapshot(
        &pool,
        summary.total_cost,
        summary.total_value,
        summary.total_gain_loss,
    )
    .await?;
    tracing::info!(
        "Portfolio snapshot saved: value={:.2}, cost={:.2}",
        summary.total_value,
        summary.total_cost
    );

    tracing::info!("Done");
    Ok(())
}
