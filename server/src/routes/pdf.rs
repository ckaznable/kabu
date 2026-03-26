use axum::extract::{Multipart, State};
use axum::http::StatusCode;

use crate::gemini;
use crate::AppState;
use kabu_shared::db;

fn decrypt_pdf_if_needed(
    pdf_bytes: &[u8],
    password: Option<&str>,
) -> Result<Vec<u8>, String> {
    // Quick check: try loading to see if encrypted
    let doc = lopdf::Document::load_mem(pdf_bytes)
        .map_err(|e| format!("Failed to parse PDF: {}", e))?;

    if !doc.is_encrypted() {
        return Ok(pdf_bytes.to_vec());
    }

    let password = password.ok_or("PDF is encrypted but no password configured")?;
    let mut doc = lopdf::Document::load_mem_with_password(pdf_bytes, password)
        .map_err(|e| format!("Failed to decrypt PDF: {}", e))?;

    let mut output = Vec::new();
    doc.save_modern(&mut output)
        .map_err(|e| format!("Failed to export decrypted PDF: {}", e))?;
    Ok(output)
}

pub async fn upload(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<StatusCode, (StatusCode, String)> {
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

    tokio::spawn(async move {
        match gemini::extract_transactions_from_pdf(
            &state.gemini_api_key,
            &state.gemini_model,
            &decrypted,
        )
        .await
        {
            Ok(extracted) => {
                for tx in &extracted {
                    if let Err(e) = db::insert_transaction(
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
                    {
                        tracing::error!("Failed to insert transaction: {}", e);
                        continue;
                    }
                    if let Err(e) = db::apply_transaction_to_stock(
                        &state.db,
                        &tx.symbol,
                        &tx.transaction_type,
                        tx.quantity,
                        tx.total,
                    )
                    .await
                    {
                        tracing::error!("Failed to update stock for {}: {}", tx.symbol, e);
                    }
                }
                tracing::info!("PDF processed: {} transactions extracted", extracted.len());
            }
            Err(e) => {
                tracing::error!("Failed to process PDF: {}", e);
            }
        }
    });

    Ok(StatusCode::ACCEPTED)
}
