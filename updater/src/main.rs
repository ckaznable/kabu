use kabu_shared::config::Config;
use kabu_shared::db;

mod coinmarketcap;
mod exchange_rate;
mod finnhub;

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
    let crypto_symbols: Vec<String> = all
        .iter()
        .filter(|(_, t)| t == "crypto")
        .map(|(s, _)| s.clone())
        .collect();

    if stock_symbols.is_empty() && crypto_symbols.is_empty() {
        tracing::info!("No assets to update");
    }

    if !stock_symbols.is_empty() {
        let finnhub_key = config.finnhub.resolve_api_key()?;
        finnhub::update_stock_prices(&pool, &finnhub_key, &stock_symbols).await?;
    }

    if !crypto_symbols.is_empty() {
        let cmc_key = config.coinmarketcap.resolve_api_key()?;
        coinmarketcap::update_crypto_prices(&pool, &cmc_key, &crypto_symbols).await?;
    }

    // Exchange rates
    exchange_rate::update_exchange_rates(
        &pool,
        &config.exchange_rate.base,
        &config.exchange_rate.currencies,
    )
    .await?;

    tracing::info!("Done");
    Ok(())
}
