use kabu_shared::config::Config;
use std::time::Duration;

mod finnhub;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let config = Config::load_default()?;
    let pool = kabu_shared::db::init_pool(&config.server.database_url).await?;
    let finnhub_key = config.finnhub.resolve_api_key()?;

    tracing::info!(
        "Updater started, interval: {}s",
        config.updater.interval_secs
    );

    loop {
        if let Err(e) = finnhub::update_all_prices(&pool, &finnhub_key).await {
            tracing::error!("Failed to update prices: {}", e);
        }
        tokio::time::sleep(Duration::from_secs(config.updater.interval_secs)).await;
    }
}
