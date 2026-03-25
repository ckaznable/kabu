use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;

use crate::AppState;
use kabu_shared::db;
use kabu_shared::models::Transaction;

pub async fn list(State(state): State<AppState>) -> Result<Json<Vec<Transaction>>, StatusCode> {
    db::list_transactions(&state.db)
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}
