use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Debug, Serialize)]
struct GeminiRequest {
    contents: Vec<Content>,
    #[serde(rename = "generationConfig")]
    generation_config: GenerationConfig,
}

#[derive(Debug, Serialize)]
struct Content {
    parts: Vec<Part>,
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
enum Part {
    InlineData {
        #[serde(rename = "inlineData")]
        inline_data: InlineData,
    },
    Text {
        text: String,
    },
}

#[derive(Debug, Serialize)]
struct InlineData {
    #[serde(rename = "mimeType")]
    mime_type: String,
    data: String,
}

#[derive(Debug, Serialize)]
struct GenerationConfig {
    #[serde(rename = "responseMimeType")]
    response_mime_type: String,
    #[serde(rename = "responseJsonSchema")]
    response_json_schema: serde_json::Value,
}

#[derive(Debug, Deserialize)]
struct GeminiResponse {
    candidates: Option<Vec<Candidate>>,
    error: Option<GeminiError>,
}

#[derive(Debug, Deserialize)]
struct GeminiError {
    message: String,
}

#[derive(Debug, Deserialize)]
struct Candidate {
    content: ResponseContent,
}

#[derive(Debug, Deserialize)]
struct ResponseContent {
    parts: Vec<ResponsePart>,
}

#[derive(Debug, Deserialize)]
struct ResponsePart {
    text: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ExtractedTransaction {
    pub symbol: String,
    #[serde(rename = "type")]
    pub transaction_type: String,
    pub quantity: f64,
    pub price: f64,
    pub total: f64,
    pub date: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ExtractedResponse {
    transactions: Vec<ExtractedTransaction>,
}

const PROMPT: &str = "Extract all financial/stock transactions from this PDF document. \
    Read every page and inspect every transaction row in the statement. \
    The PDF may contain multiple transactions for the same symbol or date. \
    Return one object for each individual BUY, SELL, or DIVIDEND row. \
    Do not stop after the first transaction. Do not merge rows, summarize, or deduplicate. \
    Ignore holdings summaries, subtotals, balances, and non-transaction rows. \
    Identify each transaction with its stock symbol, quantity, price per share, total amount, and date if available. \
    Return BUY/SELL quantity as an absolute share count, not a signed value. \
    Preserve the statement order in the returned array. \
    Before answering, double-check that the array includes every qualifying transaction row in the PDF.";

fn extract_response_text(candidate: &Candidate) -> Option<String> {
    let text = candidate
        .content
        .parts
        .iter()
        .filter_map(|part| part.text.as_deref())
        .collect::<String>();

    if text.is_empty() { None } else { Some(text) }
}

fn build_response_schema() -> serde_json::Value {
    json!({
        "type": "object",
        "properties": {
            "transactions": {
                "type": "array",
                "items": {
                    "type": "object",
                    "properties": {
                        "symbol": {
                            "type": "string",
                            "description": "Stock ticker symbol in uppercase"
                        },
                        "type": {
                            "type": "string",
                            "enum": ["BUY", "SELL", "DIVIDEND"],
                            "description": "Transaction type"
                        },
                        "quantity": {
                            "type": "number",
                            "description": "Number of shares"
                        },
                        "price": {
                            "type": "number",
                            "description": "Price per share"
                        },
                        "total": {
                            "type": "number",
                            "description": "Total transaction amount"
                        },
                        "date": {
                            "type": "string",
                            "description": "Transaction date in YYYY-MM-DD format"
                        }
                    },
                    "required": ["symbol", "type", "quantity", "price", "total"]
                }
            }
        },
        "required": ["transactions"]
    })
}

pub async fn extract_transactions_from_pdf(
    api_key: &str,
    model: &str,
    pdf_bytes: &[u8],
) -> Result<Vec<ExtractedTransaction>> {
    use base64::Engine;

    let client = reqwest::Client::new();
    let base64_pdf = base64::engine::general_purpose::STANDARD.encode(pdf_bytes);

    let request = GeminiRequest {
        contents: vec![Content {
            parts: vec![
                Part::InlineData {
                    inline_data: InlineData {
                        mime_type: "application/pdf".to_string(),
                        data: base64_pdf,
                    },
                },
                Part::Text {
                    text: PROMPT.to_string(),
                },
            ],
        }],
        generation_config: GenerationConfig {
            response_mime_type: "application/json".to_string(),
            response_json_schema: build_response_schema(),
        },
    };

    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
        model, api_key
    );

    const MAX_RETRIES: u32 = 3;
    const RETRY_DELAY: std::time::Duration = std::time::Duration::from_secs(180);

    let mut last_error = String::new();

    for attempt in 0..=MAX_RETRIES {
        if attempt > 0 {
            tracing::warn!(
                "Gemini high demand, retrying in {}s (attempt {}/{})",
                RETRY_DELAY.as_secs(),
                attempt,
                MAX_RETRIES
            );
            tokio::time::sleep(RETRY_DELAY).await;
        }

        let response = client.post(&url).json(&request).send().await?;
        let gemini_resp: GeminiResponse = response.json().await?;

        if let Some(err) = gemini_resp.error {
            if err.message.contains("high demand") && attempt < MAX_RETRIES {
                last_error = err.message;
                continue;
            }
            anyhow::bail!("Gemini API error: {}", err.message);
        }

        let text = gemini_resp
            .candidates
            .as_ref()
            .and_then(|c| c.first())
            .and_then(extract_response_text)
            .ok_or_else(|| anyhow::anyhow!("No response from Gemini"))?;

        let extracted: ExtractedResponse = serde_json::from_str(&text)?;
        return Ok(extracted.transactions);
    }

    anyhow::bail!(
        "Gemini API failed after {} retries: {}",
        MAX_RETRIES,
        last_error
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_response_text_concatenates_all_text_parts() {
        let candidate = Candidate {
            content: ResponseContent {
                parts: vec![
                    ResponsePart {
                        text: Some("{\"transactions\":[".to_string()),
                    },
                    ResponsePart {
                        text: Some("{\"symbol\":\"AAPL\"}".to_string()),
                    },
                    ResponsePart {
                        text: Some("]}".to_string()),
                    },
                ],
            },
        };

        assert_eq!(
            extract_response_text(&candidate).as_deref(),
            Some("{\"transactions\":[{\"symbol\":\"AAPL\"}]}")
        );
    }

    #[test]
    fn extract_response_text_ignores_empty_parts() {
        let candidate = Candidate {
            content: ResponseContent {
                parts: vec![ResponsePart { text: None }, ResponsePart { text: None }],
            },
        };

        assert_eq!(extract_response_text(&candidate), None);
    }
}
