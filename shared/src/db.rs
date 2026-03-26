use anyhow::Result;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::SqlitePool;
use std::str::FromStr;

use crate::models::{Price, Stock, Transaction};

pub async fn init_pool(database_url: &str) -> Result<SqlitePool> {
    let options = SqliteConnectOptions::from_str(database_url)?.create_if_missing(true);

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(options)
        .await?;

    run_migrations(&pool).await?;
    Ok(pool)
}

async fn run_migrations(pool: &SqlitePool) -> Result<()> {
    sqlx::query(include_str!("../../migrations/001_init.sql"))
        .execute(pool)
        .await?;
    Ok(())
}

// --- Stocks ---

pub async fn list_stocks(pool: &SqlitePool) -> Result<Vec<Stock>> {
    let stocks = sqlx::query_as::<_, Stock>("SELECT * FROM stocks ORDER BY symbol")
        .fetch_all(pool)
        .await?;
    Ok(stocks)
}

pub async fn get_stock(pool: &SqlitePool, id: i64) -> Result<Option<Stock>> {
    let stock = sqlx::query_as::<_, Stock>("SELECT * FROM stocks WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await?;
    Ok(stock)
}

pub async fn get_stock_by_symbol(pool: &SqlitePool, symbol: &str) -> Result<Option<Stock>> {
    let stock = sqlx::query_as::<_, Stock>("SELECT * FROM stocks WHERE symbol = ?")
        .bind(symbol)
        .fetch_optional(pool)
        .await?;
    Ok(stock)
}

pub async fn create_stock(
    pool: &SqlitePool,
    symbol: &str,
    name: Option<&str>,
    quantity: f64,
    cost_basis: f64,
) -> Result<Stock> {
    let result = sqlx::query(
        "INSERT INTO stocks (symbol, name, quantity, cost_basis) VALUES (?, ?, ?, ?)",
    )
    .bind(symbol.to_uppercase())
    .bind(name)
    .bind(quantity)
    .bind(cost_basis)
    .execute(pool)
    .await?;

    let stock = get_stock(pool, result.last_insert_rowid()).await?.unwrap();
    Ok(stock)
}

pub async fn update_stock(
    pool: &SqlitePool,
    id: i64,
    name: Option<&str>,
    quantity: f64,
    cost_basis: f64,
) -> Result<Option<Stock>> {
    sqlx::query(
        "UPDATE stocks SET name = COALESCE(?, name), quantity = ?, cost_basis = ?, updated_at = datetime('now') WHERE id = ?",
    )
    .bind(name)
    .bind(quantity)
    .bind(cost_basis)
    .bind(id)
    .execute(pool)
    .await?;

    get_stock(pool, id).await
}

pub async fn delete_stock(pool: &SqlitePool, id: i64) -> Result<bool> {
    let result = sqlx::query("DELETE FROM stocks WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

// --- Prices ---

pub async fn insert_price(
    pool: &SqlitePool,
    symbol: &str,
    price: f64,
    change: Option<f64>,
    change_percent: Option<f64>,
    high: Option<f64>,
    low: Option<f64>,
    open: Option<f64>,
    previous_close: Option<f64>,
) -> Result<()> {
    sqlx::query(
        "INSERT INTO prices (symbol, price, change, change_percent, high, low, open, previous_close) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(symbol)
    .bind(price)
    .bind(change)
    .bind(change_percent)
    .bind(high)
    .bind(low)
    .bind(open)
    .bind(previous_close)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn get_latest_price(pool: &SqlitePool, symbol: &str) -> Result<Option<Price>> {
    let price = sqlx::query_as::<_, Price>(
        "SELECT * FROM prices WHERE symbol = ? ORDER BY id DESC LIMIT 1",
    )
    .bind(symbol)
    .fetch_optional(pool)
    .await?;
    Ok(price)
}

pub async fn get_all_latest_prices(pool: &SqlitePool) -> Result<Vec<Price>> {
    let prices = sqlx::query_as::<_, Price>(
        "SELECT p.* FROM prices p INNER JOIN (SELECT symbol, MAX(id) as max_id FROM prices GROUP BY symbol) latest ON p.id = latest.max_id",
    )
    .fetch_all(pool)
    .await?;
    Ok(prices)
}

// --- Transactions ---

pub async fn insert_transaction(
    pool: &SqlitePool,
    symbol: &str,
    transaction_type: &str,
    quantity: f64,
    price: f64,
    total_amount: f64,
    transaction_date: Option<&str>,
    source: Option<&str>,
    raw_text: Option<&str>,
) -> Result<Transaction> {
    let result = sqlx::query(
        "INSERT INTO transactions (symbol, transaction_type, quantity, price, total_amount, transaction_date, source, raw_text) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(symbol.to_uppercase())
    .bind(transaction_type)
    .bind(quantity)
    .bind(price)
    .bind(total_amount)
    .bind(transaction_date)
    .bind(source)
    .bind(raw_text)
    .execute(pool)
    .await?;

    let tx = sqlx::query_as::<_, Transaction>("SELECT * FROM transactions WHERE id = ?")
        .bind(result.last_insert_rowid())
        .fetch_one(pool)
        .await?;
    Ok(tx)
}

pub async fn list_transactions(pool: &SqlitePool) -> Result<Vec<Transaction>> {
    let txs = sqlx::query_as::<_, Transaction>(
        "SELECT * FROM transactions ORDER BY created_at DESC",
    )
    .fetch_all(pool)
    .await?;
    Ok(txs)
}

/// Upsert stock from a transaction: create if not exists, update quantity/cost on BUY/SELL.
pub async fn apply_transaction_to_stock(
    pool: &SqlitePool,
    symbol: &str,
    transaction_type: &str,
    quantity: f64,
    total: f64,
) -> Result<()> {
    let existing = get_stock_by_symbol(pool, symbol).await?;

    match transaction_type {
        "BUY" => {
            if let Some(stock) = existing {
                sqlx::query(
                    "UPDATE stocks SET quantity = quantity + ?, cost_basis = cost_basis + ?, updated_at = datetime('now') WHERE id = ?",
                )
                .bind(quantity)
                .bind(total)
                .bind(stock.id)
                .execute(pool)
                .await?;
            } else {
                create_stock(pool, symbol, None, quantity, total).await?;
            }
        }
        "SELL" => {
            if let Some(stock) = existing {
                let new_qty = (stock.quantity - quantity).max(0.0);
                let cost_ratio = if stock.quantity > 0.0 {
                    quantity / stock.quantity
                } else {
                    0.0
                };
                let cost_reduction = stock.cost_basis * cost_ratio;
                sqlx::query(
                    "UPDATE stocks SET quantity = ?, cost_basis = cost_basis - ?, updated_at = datetime('now') WHERE id = ?",
                )
                .bind(new_qty)
                .bind(cost_reduction)
                .bind(stock.id)
                .execute(pool)
                .await?;
            }
        }
        _ => {} // DIVIDEND etc. — no change to holdings
    }

    Ok(())
}

pub async fn list_all_symbols(pool: &SqlitePool) -> Result<Vec<String>> {
    let rows: Vec<(String,)> =
        sqlx::query_as("SELECT DISTINCT symbol FROM stocks ORDER BY symbol")
            .fetch_all(pool)
            .await?;
    Ok(rows.into_iter().map(|r| r.0).collect())
}
