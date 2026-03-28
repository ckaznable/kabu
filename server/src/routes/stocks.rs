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
    if input.quantity == 0.0 {
        return Err(StatusCode::BAD_REQUEST);
    }
    let symbol = input.symbol.to_uppercase();
    let (status, stock) = if let Some(existing) = db::get_stock_by_symbol(&state.db, &symbol)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    {
        if existing.asset_type != input.asset_type {
            return Err(StatusCode::BAD_REQUEST);
        }
        let merged_qty = existing.quantity + input.quantity;
        let merged_cost = existing.cost_basis + input.cost_basis;
        let merged = db::update_stock(
            &state.db,
            existing.id,
            input.name.as_deref(),
            merged_qty,
            merged_cost,
        )
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;
        (StatusCode::OK, merged)
    } else {
        let created = db::create_stock(
            &state.db,
            &symbol,
            input.name.as_deref(),
            input.quantity,
            input.cost_basis,
            &input.asset_type,
        )
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        (StatusCode::CREATED, created)
    };

    // Always keep a history entry for manual add action.
    let transaction_type = if input.quantity < 0.0 { "SELL" } else { "BUY" };
    let tx_qty = input.quantity.abs();
    let tx_total = input.cost_basis.abs();
    let unit_price = if tx_qty > 0.0 { tx_total / tx_qty } else { 0.0 };
    db::insert_transaction(
        &state.db,
        &symbol,
        transaction_type,
        tx_qty,
        unit_price,
        tx_total,
        None,
        Some("manual"),
        None,
    )
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Seed initial price from avg cost if no price record exists
    if input.quantity > 0.0 && input.cost_basis > 0.0 {
        let existing = db::get_latest_price(&state.db, &symbol)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        if existing.is_none() {
            let avg = input.cost_basis / input.quantity;
            let _ = db::insert_price(&state.db, &symbol, avg, None, None, None, None, None, None).await;
        }
    }

    Ok((status, Json(stock)))
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
