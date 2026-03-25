use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;

use crate::AppState;
use kabu_shared::db;
use kabu_shared::models::{HoldingSummary, PortfolioSummary};

pub async fn summary(
    State(state): State<AppState>,
) -> Result<Json<PortfolioSummary>, StatusCode> {
    let stocks = db::list_stocks(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut holdings = Vec::new();
    let mut total_cost = 0.0;
    let mut total_value = 0.0;

    for stock in stocks {
        let latest_price = db::get_latest_price(&state.db, &stock.symbol)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        let price = latest_price.as_ref().map(|p| p.price);
        let current_value = price.unwrap_or(0.0) * stock.quantity;
        let gain_loss = current_value - stock.cost_basis;
        let gain_loss_percent = if stock.cost_basis > 0.0 {
            (gain_loss / stock.cost_basis) * 100.0
        } else {
            0.0
        };

        total_cost += stock.cost_basis;
        total_value += current_value;

        holdings.push(HoldingSummary {
            stock,
            latest_price: price,
            current_value,
            gain_loss,
            gain_loss_percent,
        });
    }

    let total_gain_loss = total_value - total_cost;
    let total_gain_loss_percent = if total_cost > 0.0 {
        (total_gain_loss / total_cost) * 100.0
    } else {
        0.0
    };

    Ok(Json(PortfolioSummary {
        total_cost,
        total_value,
        total_gain_loss,
        total_gain_loss_percent,
        holdings,
    }))
}
