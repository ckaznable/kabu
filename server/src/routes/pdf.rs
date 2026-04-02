use axum::extract::{Multipart, State};
use axum::http::StatusCode;
use std::collections::HashSet;

use crate::gemini;
use crate::transaction_service::normalize_transaction_input;
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
    tracing::info!("PDF upload request received");

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
                let mut affected_symbols = HashSet::new();

                for tx in &extracted {
                    let normalized = match normalize_transaction_input(
                        &tx.symbol,
                        &tx.transaction_type,
                        tx.quantity,
                        tx.price,
                        tx.total,
                        tx.date.as_deref(),
                    ) {
                        Ok(tx) => tx,
                        Err(e) => {
                            tracing::error!("Failed to normalize extracted transaction: {}", e);
                            continue;
                        }
                    };

                    if let Err(e) = db::insert_transaction(
                        &state.db,
                        &normalized.symbol,
                        &normalized.transaction_type,
                        normalized.quantity,
                        normalized.price,
                        normalized.total_amount,
                        normalized.transaction_date.as_deref(),
                        Some("pdf"),
                        None,
                    )
                    .await
                    {
                        tracing::error!("Failed to insert transaction: {}", e);
                        continue;
                    }
                    affected_symbols.insert(normalized.symbol);
                }

                for symbol in affected_symbols {
                    if let Err(e) = db::rebuild_stock_from_transactions(&state.db, &symbol).await {
                        tracing::error!("Failed to rebuild stock for {}: {}", symbol, e);
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
