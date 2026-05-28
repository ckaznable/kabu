use axum::extract::{Multipart, State};
use axum::http::StatusCode;
use base64::Engine as _;
use std::collections::HashSet;

use crate::AppState;
use crate::gemini;
use crate::transaction_service::normalize_transaction_input;
use kabu_shared::db;

fn transaction_action_label(transaction_type: &str) -> &'static str {
    match transaction_type {
        "BUY" => "買入",
        "SELL" => "賣出",
        "DIVIDEND" => "股利",
        _ => "交易",
    }
}

fn decrypt_pdf_if_needed(pdf_bytes: &[u8], password: Option<&str>) -> Result<Vec<u8>, String> {
    // Quick check: try loading to see if encrypted
    let doc =
        lopdf::Document::load_mem(pdf_bytes).map_err(|e| format!("Failed to parse PDF: {}", e))?;

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

fn normalize_pdf_payload(bytes: &[u8]) -> Option<Vec<u8>> {
    if bytes.starts_with(b"%PDF") {
        return Some(bytes.to_vec());
    }

    let trimmed = bytes
        .strip_prefix(b"\xef\xbb\xbf")
        .unwrap_or(bytes)
        .trim_ascii();

    if trimmed.starts_with(b"%PDF") {
        return Some(trimmed.to_vec());
    }

    if trimmed.starts_with(b"JVBER") {
        let compact: Vec<u8> = trimmed
            .iter()
            .copied()
            .filter(|byte| !byte.is_ascii_whitespace())
            .collect();

        if let Ok(decoded) = base64::engine::general_purpose::STANDARD.decode(compact) {
            if decoded.starts_with(b"%PDF") {
                return Some(decoded);
            }
        }
    }

    None
}

pub async fn upload(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<StatusCode, (StatusCode, String)> {
    tracing::info!("PDF upload request received");

    let mut pdf_bytes = None;
    let mut file_count = 0;
    let mut skipped_files = 0;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?
    {
        if field.name() == Some("file") {
            file_count += 1;
            let file_name = field.file_name().map(str::to_string);
            let content_type = field.content_type().map(str::to_string);
            let bytes = field
                .bytes()
                .await
                .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;

            if let Some(normalized) = normalize_pdf_payload(&bytes) {
                tracing::info!(
                    file_name = ?file_name,
                    content_type = ?content_type,
                    size = normalized.len(),
                    "PDF attachment selected"
                );
                pdf_bytes = Some(normalized);
                break;
            }

            skipped_files += 1;
            tracing::warn!(
                file_name = ?file_name,
                content_type = ?content_type,
                size = bytes.len(),
                "Skipping non-PDF attachment"
            );
        }
    }

    let pdf_bytes = pdf_bytes.ok_or((
        StatusCode::BAD_REQUEST,
        if file_count == 0 {
            "No file uploaded".to_string()
        } else {
            format!("No PDF attachment found in {file_count} file(s); skipped {skipped_files}")
        },
    ))?;

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

                    tracing::info!(
                        action = transaction_action_label(&normalized.transaction_type),
                        symbol = %normalized.symbol,
                        quantity = normalized.quantity,
                        price = normalized.price,
                        total_amount = normalized.total_amount,
                        date = ?normalized.transaction_date,
                        "PDF transaction extracted"
                    );

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

#[cfg(test)]
mod tests {
    use super::normalize_pdf_payload;

    #[test]
    fn normalize_pdf_payload_accepts_raw_pdf() {
        let bytes = b"%PDF-1.7\nbody";

        assert_eq!(normalize_pdf_payload(bytes), Some(bytes.to_vec()));
    }

    #[test]
    fn normalize_pdf_payload_accepts_base64_pdf() {
        let bytes = b"JVBERi0xLjcKYm9keQ==";

        assert_eq!(
            normalize_pdf_payload(bytes),
            Some(b"%PDF-1.7\nbody".to_vec())
        );
    }

    #[test]
    fn normalize_pdf_payload_accepts_wrapped_base64_pdf() {
        let bytes = b"\nJVBERi0xLjcK\nYm9keQ==\n";

        assert_eq!(
            normalize_pdf_payload(bytes),
            Some(b"%PDF-1.7\nbody".to_vec())
        );
    }

    #[test]
    fn normalize_pdf_payload_rejects_non_pdf() {
        assert_eq!(normalize_pdf_payload(b"not a pdf"), None);
    }
}
