use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;

use crate::AppState;
use crate::transaction_service::normalize_transaction_input;
use kabu_shared::db;
use kabu_shared::models::{Transaction, UpdateTransaction};

pub async fn list(State(state): State<AppState>) -> Result<Json<Vec<Transaction>>, StatusCode> {
    db::list_transactions(&state.db)
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

pub async fn update(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(input): Json<UpdateTransaction>,
) -> Result<Json<Transaction>, (StatusCode, String)> {
    let existing = db::get_transaction(&state.db, id)
        .await
        .map_err(internal_error)?
        .ok_or((StatusCode::NOT_FOUND, "Transaction not found".to_string()))?;

    let normalized = normalize_transaction_input(
        &input.symbol,
        &input.transaction_type,
        input.quantity,
        input.price,
        input.total_amount,
        input.transaction_date.as_deref(),
    )
    .map_err(|e| (StatusCode::BAD_REQUEST, e))?;

    let updated = db::update_transaction(
        &state.db,
        id,
        &normalized.symbol,
        &normalized.transaction_type,
        normalized.quantity,
        normalized.price,
        normalized.total_amount,
        normalized.transaction_date.as_deref(),
    )
    .await
    .map_err(internal_error)?
    .ok_or((StatusCode::NOT_FOUND, "Transaction not found".to_string()))?;

    db::rebuild_stock_from_transactions(&state.db, &existing.symbol)
        .await
        .map_err(internal_error)?;

    if normalized.symbol != existing.symbol {
        db::rebuild_stock_from_transactions(&state.db, &normalized.symbol)
            .await
            .map_err(internal_error)?;
    }

    Ok(Json(updated))
}

pub async fn delete_one(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<StatusCode, (StatusCode, String)> {
    let existing = db::get_transaction(&state.db, id)
        .await
        .map_err(internal_error)?
        .ok_or((StatusCode::NOT_FOUND, "Transaction not found".to_string()))?;

    let deleted = db::delete_transaction(&state.db, id)
        .await
        .map_err(internal_error)?;

    if !deleted {
        return Ok(StatusCode::NOT_FOUND);
    }

    db::rebuild_stock_from_transactions(&state.db, &existing.symbol)
        .await
        .map_err(internal_error)?;

    Ok(StatusCode::NO_CONTENT)
}

fn internal_error<E: std::fmt::Display>(error: E) -> (StatusCode, String) {
    (StatusCode::INTERNAL_SERVER_ERROR, error.to_string())
}
