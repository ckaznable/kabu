use kabu_shared::config::Config;

mod finnhub;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let config = Config::load_default()?;
    let pool = kabu_shared::db::init_pool(&config.server.database_url).await?;
    let finnhub_key = config.finnhub.resolve_api_key()?;

    finnhub::update_all_prices(&pool, &finnhub_key).await?;

    tracing::info!("Done");
    Ok(())
}
