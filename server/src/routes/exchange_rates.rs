use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;

use crate::AppState;
use kabu_shared::db;
use kabu_shared::models::ExchangeRate;

pub async fn list(
    State(state): State<AppState>,
) -> Result<Json<Vec<ExchangeRate>>, StatusCode> {
    db::get_latest_rates(&state.db, "USD")
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}
