use axum::extract::{Multipart, State};
use axum::http::StatusCode;
use axum::Json;

use crate::gemini;
use crate::AppState;
use kabu_shared::db;
use kabu_shared::models::Transaction;

fn decrypt_pdf_if_needed(
    pdf_bytes: &[u8],
    password: Option<&str>,
) -> Result<Vec<u8>, String> {
    let mut doc = lopdf::Document::load_mem(pdf_bytes)
        .map_err(|e| format!("Failed to parse PDF: {}", e))?;

    if doc.is_encrypted() {
        let password = password.ok_or("PDF is encrypted but no password configured")?;
        doc.decrypt(password)
            .map_err(|e| format!("Failed to decrypt PDF: {}", e))?;
        let mut output = Vec::new();
        doc.save_to(&mut output)
            .map_err(|e| format!("Failed to export decrypted PDF: {}", e))?;
        Ok(output)
    } else {
        Ok(pdf_bytes.to_vec())
    }
}

pub async fn upload(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<Json<Vec<Transaction>>, (StatusCode, String)> {
    let mut pdf_bytes = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?
    {
        if field.name() == Some("file") {
            pdf_bytes = Some(
                field
                    .bytes()
                    .await
                    .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?,
            );
        }
    }

    let pdf_bytes =
        pdf_bytes.ok_or((StatusCode::BAD_REQUEST, "No file uploaded".to_string()))?;

    let decrypted = decrypt_pdf_if_needed(&pdf_bytes, state.pdf_password.as_deref())
        .map_err(|e| (StatusCode::BAD_REQUEST, e))?;

    let extracted =
        gemini::extract_transactions_from_pdf(&state.gemini_api_key, &state.gemini_model, &decrypted)
            .await
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Failed to process PDF: {}", e),
                )
            })?;

    let mut transactions = Vec::new();
    for tx in &extracted {
        let transaction = db::insert_transaction(
            &state.db,
            &tx.symbol,
            &tx.transaction_type,
            tx.quantity,
            tx.price,
            tx.total,
            tx.date.as_deref(),
            Some("pdf"),
            None,
        )
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        transactions.push(transaction);
    }

    Ok(Json(transactions))
}
