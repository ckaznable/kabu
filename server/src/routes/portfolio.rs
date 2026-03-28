use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;

use crate::AppState;
use kabu_shared::db;
use kabu_shared::models::{PortfolioSnapshot, PortfolioSummary};

pub async fn summary(
    State(state): State<AppState>,
) -> Result<Json<PortfolioSummary>, StatusCode> {
    db::compute_portfolio_summary(&state.db)
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

pub async fn snapshots(
    State(state): State<AppState>,
) -> Result<Json<Vec<PortfolioSnapshot>>, StatusCode> {
    db::list_portfolio_snapshots(&state.db)
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}
