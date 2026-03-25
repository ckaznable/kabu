CREATE TABLE IF NOT EXISTS stocks (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    symbol TEXT NOT NULL UNIQUE,
    name TEXT,
    quantity REAL NOT NULL DEFAULT 0,
    cost_basis REAL NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS prices (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    symbol TEXT NOT NULL,
    price REAL NOT NULL,
    change REAL,
    change_percent REAL,
    high REAL,
    low REAL,
    open REAL,
    previous_close REAL,
    timestamp TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS transactions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    symbol TEXT NOT NULL,
    transaction_type TEXT NOT NULL,
    quantity REAL NOT NULL,
    price REAL NOT NULL,
    total_amount REAL NOT NULL,
    transaction_date TEXT,
    source TEXT,
    raw_text TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);
