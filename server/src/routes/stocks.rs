use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;

use crate::AppState;
use kabu_shared::db;
use kabu_shared::models::{CreateStock, Stock, UpdateStock};

pub async fn list(State(state): State<AppState>) -> Result<Json<Vec<Stock>>, StatusCode> {
    db::list_stocks(&state.db)
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

pub async fn get_one(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<Stock>, StatusCode> {
    db::get_stock(&state.db, id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .map(Json)
        .ok_or(StatusCode::NOT_FOUND)
}

pub async fn create(
    State(state): State<AppState>,
    Json(input): Json<CreateStock>,
) -> Result<(StatusCode, Json<Stock>), StatusCode> {
    db::create_stock(
        &state.db,
        &input.symbol,
        input.name.as_deref(),
        input.quantity,
        input.cost_basis,
    )
    .await
    .map(|s| (StatusCode::CREATED, Json(s)))
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

pub async fn update(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(input): Json<UpdateStock>,
) -> Result<Json<Stock>, StatusCode> {
    db::update_stock(
        &state.db,
        id,
        input.name.as_deref(),
        input.quantity,
        input.cost_basis,
    )
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .map(Json)
    .ok_or(StatusCode::NOT_FOUND)
}

pub async fn delete_one(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<StatusCode, StatusCode> {
    db::delete_stock(&state.db, id)
        .await
        .map(|deleted| {
            if deleted {
                StatusCode::NO_CONTENT
            } else {
                StatusCode::NOT_FOUND
            }
        })
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}
