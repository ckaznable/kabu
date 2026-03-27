use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow, Type)]
pub struct Stock {
    pub id: i64,
    pub symbol: String,
    pub name: Option<String>,
    pub quantity: f64,
    pub cost_basis: f64,
    pub asset_type: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct CreateStock {
    pub symbol: String,
    pub name: Option<String>,
    pub quantity: f64,
    pub cost_basis: f64,
    #[serde(default = "default_asset_type")]
    pub asset_type: String,
}

fn default_asset_type() -> String {
    "stock".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct UpdateStock {
    pub name: Option<String>,
    pub quantity: f64,
    pub cost_basis: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow, Type)]
pub struct Price {
    pub id: i64,
    pub symbol: String,
    pub price: f64,
    pub change: Option<f64>,
    pub change_percent: Option<f64>,
    pub high: Option<f64>,
    pub low: Option<f64>,
    pub open: Option<f64>,
    pub previous_close: Option<f64>,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow, Type)]
pub struct Transaction {
    pub id: i64,
    pub symbol: String,
    pub transaction_type: String,
    pub quantity: f64,
    pub price: f64,
    pub total_amount: f64,
    pub transaction_date: Option<String>,
    pub source: Option<String>,
    pub raw_text: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct PortfolioSummary {
    pub total_cost: f64,
    pub total_value: f64,
    pub total_gain_loss: f64,
    pub total_gain_loss_percent: f64,
    pub holdings: Vec<HoldingSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct HoldingSummary {
    pub stock: Stock,
    pub latest_price: Option<f64>,
    pub current_value: f64,
    pub gain_loss: f64,
    pub gain_loss_percent: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow, Type)]
pub struct ExchangeRate {
    pub id: i64,
    pub base: String,
    pub currency: String,
    pub rate: f64,
    pub timestamp: String,
}
