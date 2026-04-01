pub struct NormalizedTransactionInput {
    pub symbol: String,
    pub transaction_type: String,
    pub quantity: f64,
    pub price: f64,
    pub total_amount: f64,
    pub transaction_date: Option<String>,
}

pub fn normalize_transaction_input(
    symbol: &str,
    transaction_type: &str,
    quantity: f64,
    price: f64,
    total_amount: f64,
    transaction_date: Option<&str>,
) -> Result<NormalizedTransactionInput, String> {
    let symbol = symbol.trim().to_uppercase();
    if symbol.is_empty() {
        return Err("Symbol is required".to_string());
    }

    let transaction_type = transaction_type.trim().to_uppercase();
    if !matches!(transaction_type.as_str(), "BUY" | "SELL" | "DIVIDEND") {
        return Err("Unsupported transaction type".to_string());
    }

    if !quantity.is_finite() || !price.is_finite() || !total_amount.is_finite() {
        return Err("Transaction values must be finite numbers".to_string());
    }

    let quantity = match transaction_type.as_str() {
        // PDF broker statements should use whole-share counts. Normalize sign noise from LLMs.
        "BUY" | "SELL" => quantity.abs().round(),
        _ => quantity.abs(),
    };

    if matches!(transaction_type.as_str(), "BUY" | "SELL") && quantity < 1.0 {
        return Err("BUY/SELL quantity must be at least 1 share".to_string());
    }

    let transaction_date = transaction_date.and_then(|value| {
        let value = value.trim();
        if value.is_empty() {
            None
        } else {
            Some(value.to_string())
        }
    });

    Ok(NormalizedTransactionInput {
        symbol,
        transaction_type,
        quantity,
        price: price.abs(),
        total_amount: total_amount.abs(),
        transaction_date,
    })
}
