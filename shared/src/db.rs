use anyhow::Result;
use sqlx::SqlitePool;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use std::str::FromStr;

use crate::models::{
    ExchangeRate, HoldingSummary, PortfolioSnapshot, PortfolioSummary, Price, Stock, Transaction,
};

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

    // Add asset_type column to existing databases
    let _ = sqlx::query("ALTER TABLE stocks ADD COLUMN asset_type TEXT NOT NULL DEFAULT 'stock'")
        .execute(pool)
        .await;

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
    asset_type: &str,
) -> Result<Stock> {
    let result = sqlx::query(
        "INSERT INTO stocks (symbol, name, quantity, cost_basis, asset_type) VALUES (?, ?, ?, ?, ?)",
    )
    .bind(symbol.to_uppercase())
    .bind(name)
    .bind(quantity)
    .bind(cost_basis)
    .bind(asset_type)
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

pub async fn get_price_history(pool: &SqlitePool, symbol: &str, limit: i64) -> Result<Vec<Price>> {
    let prices = sqlx::query_as::<_, Price>(
        "SELECT * FROM prices WHERE symbol = ? ORDER BY id DESC LIMIT ?",
    )
    .bind(symbol)
    .bind(limit)
    .fetch_all(pool)
    .await?;
    Ok(prices)
}

pub async fn compute_portfolio_summary(pool: &SqlitePool) -> Result<PortfolioSummary> {
    let stocks = list_stocks(pool).await?;
    let latest_prices = get_all_latest_prices(pool).await?;
    let latest_price_map = latest_prices
        .into_iter()
        .map(|price| (price.symbol.clone(), price))
        .collect::<std::collections::HashMap<_, _>>();

    let mut holdings = Vec::new();
    let mut total_cost = 0.0;
    let mut total_value = 0.0;

    for stock in stocks {
        let latest_price = latest_price_map.get(&stock.symbol);
        let price = latest_price.map(|p| p.price);
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
            latest_change: latest_price.and_then(|p| p.change),
            latest_change_percent: latest_price.and_then(|p| p.change_percent),
            latest_price_timestamp: latest_price.map(|p| p.timestamp.clone()),
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

    Ok(PortfolioSummary {
        total_cost,
        total_value,
        total_gain_loss,
        total_gain_loss_percent,
        holdings,
    })
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

pub async fn get_transaction(pool: &SqlitePool, id: i64) -> Result<Option<Transaction>> {
    let tx = sqlx::query_as::<_, Transaction>("SELECT * FROM transactions WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await?;
    Ok(tx)
}

pub async fn list_transactions(pool: &SqlitePool) -> Result<Vec<Transaction>> {
    let txs = sqlx::query_as::<_, Transaction>(
        "SELECT * FROM transactions ORDER BY COALESCE(transaction_date, created_at) DESC, id DESC",
    )
    .fetch_all(pool)
    .await?;
    Ok(txs)
}

pub async fn update_transaction(
    pool: &SqlitePool,
    id: i64,
    symbol: &str,
    transaction_type: &str,
    quantity: f64,
    price: f64,
    total_amount: f64,
    transaction_date: Option<&str>,
) -> Result<Option<Transaction>> {
    let result = sqlx::query(
        "UPDATE transactions
         SET symbol = ?, transaction_type = ?, quantity = ?, price = ?, total_amount = ?, transaction_date = ?
         WHERE id = ?",
    )
    .bind(symbol.to_uppercase())
    .bind(transaction_type)
    .bind(quantity)
    .bind(price)
    .bind(total_amount)
    .bind(transaction_date)
    .bind(id)
    .execute(pool)
    .await?;

    if result.rows_affected() == 0 {
        return Ok(None);
    }

    get_transaction(pool, id).await
}

pub async fn delete_transaction(pool: &SqlitePool, id: i64) -> Result<bool> {
    let result = sqlx::query("DELETE FROM transactions WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

async fn list_transactions_by_symbol(pool: &SqlitePool, symbol: &str) -> Result<Vec<Transaction>> {
    let symbol = symbol.to_uppercase();
    let txs = sqlx::query_as::<_, Transaction>(
        "SELECT * FROM transactions
         WHERE symbol = ?
         ORDER BY COALESCE(transaction_date, created_at) ASC, id ASC",
    )
    .bind(symbol)
    .fetch_all(pool)
    .await?;
    Ok(txs)
}

pub async fn rebuild_stock_from_transactions(pool: &SqlitePool, symbol: &str) -> Result<()> {
    let symbol = symbol.to_uppercase();
    let existing = get_stock_by_symbol(pool, &symbol).await?;
    let transactions = list_transactions_by_symbol(pool, &symbol).await?;

    let mut quantity = 0.0;
    let mut cost_basis = 0.0;

    for tx in transactions {
        match tx.transaction_type.as_str() {
            "BUY" => {
                quantity += tx.quantity;
                cost_basis += tx.total_amount;
            }
            "SELL" => {
                if quantity <= 0.0 {
                    continue;
                }

                let sell_quantity = tx.quantity.min(quantity);
                let cost_ratio = sell_quantity / quantity;
                let cost_reduction = cost_basis * cost_ratio;
                quantity = (quantity - sell_quantity).max(0.0);
                cost_basis = (cost_basis - cost_reduction).max(0.0);
            }
            _ => {}
        }
    }

    match existing {
        Some(stock) => {
            sqlx::query(
                "UPDATE stocks SET quantity = ?, cost_basis = ?, updated_at = datetime('now') WHERE id = ?",
            )
            .bind(quantity)
            .bind(cost_basis)
            .bind(stock.id)
            .execute(pool)
            .await?;
        }
        None if quantity > 0.0 || cost_basis > 0.0 => {
            create_stock(pool, &symbol, None, quantity, cost_basis, "stock").await?;
        }
        None => {}
    }

    Ok(())
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
                create_stock(pool, symbol, None, quantity, total, "stock").await?;
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

pub async fn list_stocks_by_type(pool: &SqlitePool) -> Result<Vec<(String, String)>> {
    let rows: Vec<(String, String)> =
        sqlx::query_as("SELECT symbol, asset_type FROM stocks ORDER BY symbol")
            .fetch_all(pool)
            .await?;
    Ok(rows)
}

pub async fn insert_portfolio_snapshot(
    pool: &SqlitePool,
    total_cost: f64,
    total_value: f64,
    total_gain_loss: f64,
) -> Result<()> {
    let updated = sqlx::query(
        "UPDATE portfolio_snapshots
         SET total_cost = ?, total_value = ?, total_gain_loss = ?, timestamp = datetime('now')
         WHERE id = (
            SELECT id FROM portfolio_snapshots
            WHERE date(timestamp) = date('now')
            ORDER BY id DESC
            LIMIT 1
         )",
    )
    .bind(total_cost)
    .bind(total_value)
    .bind(total_gain_loss)
    .execute(pool)
    .await?;

    if updated.rows_affected() == 0 {
        sqlx::query(
            "INSERT INTO portfolio_snapshots (total_cost, total_value, total_gain_loss) VALUES (?, ?, ?)",
        )
        .bind(total_cost)
        .bind(total_value)
        .bind(total_gain_loss)
        .execute(pool)
        .await?;
    }

    // Keep only one row per day (the latest one for today).
    sqlx::query(
        "DELETE FROM portfolio_snapshots
         WHERE date(timestamp) = date('now')
           AND id NOT IN (
             SELECT id FROM portfolio_snapshots
             WHERE date(timestamp) = date('now')
             ORDER BY id DESC
             LIMIT 1
           )",
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn list_portfolio_snapshots(pool: &SqlitePool) -> Result<Vec<PortfolioSnapshot>> {
    let rows =
        sqlx::query_as::<_, PortfolioSnapshot>("SELECT * FROM portfolio_snapshots ORDER BY id ASC")
            .fetch_all(pool)
            .await?;
    Ok(rows)
}

// --- Exchange Rates ---

pub async fn upsert_exchange_rate(
    pool: &SqlitePool,
    base: &str,
    currency: &str,
    rate: f64,
) -> Result<()> {
    sqlx::query(
        "INSERT INTO exchange_rates (base, currency, rate) VALUES (?, ?, ?)
         ON CONFLICT DO NOTHING",
    )
    .bind(base)
    .bind(currency)
    .bind(rate)
    .execute(pool)
    .await?;

    // Keep only latest per pair — delete older entries
    sqlx::query(
        "DELETE FROM exchange_rates WHERE base = ? AND currency = ? AND id NOT IN (
            SELECT id FROM exchange_rates WHERE base = ? AND currency = ? ORDER BY id DESC LIMIT 1
        )",
    )
    .bind(base)
    .bind(currency)
    .bind(base)
    .bind(currency)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_latest_rates(pool: &SqlitePool, base: &str) -> Result<Vec<ExchangeRate>> {
    let rates = sqlx::query_as::<_, ExchangeRate>(
        "SELECT e.* FROM exchange_rates e
         INNER JOIN (
             SELECT base, currency, MAX(id) as max_id
             FROM exchange_rates WHERE base = ? GROUP BY base, currency
         ) latest ON e.id = latest.max_id",
    )
    .bind(base)
    .fetch_all(pool)
    .await?;
    Ok(rates)
}
