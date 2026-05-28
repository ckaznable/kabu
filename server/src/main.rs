use axum::Router;
use axum::http::StatusCode;
use axum::routing::{any, get, post};
use kabu_shared::config::Config;
use sqlx::SqlitePool;
use tower_http::cors::CorsLayer;
use tower_http::services::{ServeDir, ServeFile};

mod gemini;
mod routes;
mod transaction_service;

#[derive(Clone)]
pub struct AppState {
    pub db: SqlitePool,
    pub gemini_api_key: String,
    pub gemini_model: String,
    pub pdf_password: Option<String>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let config = Config::load_default()?;
    let pool = kabu_shared::db::init_pool(&config.server.database_url).await?;

    let state = AppState {
        db: pool,
        gemini_api_key: config.gemini.resolve_api_key()?,
        gemini_model: config.gemini.model,
        pdf_password: config.pdf.resolve_password(),
    };

    let api = Router::new()
        .route(
            "/api/stocks",
            get(routes::stocks::list).post(routes::stocks::create),
        )
        .route(
            "/api/stocks/{id}",
            get(routes::stocks::get_one)
                .put(routes::stocks::update)
                .delete(routes::stocks::delete_one),
        )
        .route("/api/portfolio/summary", get(routes::portfolio::summary))
        .route(
            "/api/portfolio/snapshots",
            get(routes::portfolio::snapshots),
        )
        .route("/api/pdf/upload", post(routes::pdf::upload))
        .route("/api/transactions", get(routes::transactions::list))
        .route(
            "/api/transactions/{id}",
            axum::routing::put(routes::transactions::update)
                .delete(routes::transactions::delete_one),
        )
        .route("/api/prices/{symbol}", get(routes::prices::history))
        .route("/api/exchange-rates", get(routes::exchange_rates::list))
        .route("/api/{*path}", any(|| async { StatusCode::NOT_FOUND }))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let app = Router::new().merge(api).fallback_service(
        ServeDir::new("frontend/dist")
            .append_index_html_on_directories(true)
            .not_found_service(ServeFile::new("frontend/dist/index.html")),
    );

    let addr = format!("0.0.0.0:{}", config.server.port);
    tracing::info!("Server listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
