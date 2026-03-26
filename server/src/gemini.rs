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
struct BatchRequest {
    requests: Vec<GeminiRequest>,
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
struct BatchResponse {
    responses: Option<Vec<GeminiResponse>>,
    error: Option<GeminiError>,
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
    Identify each buy, sell, or dividend transaction with its stock symbol, \
    quantity, price per share, total amount, and date if available.";

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

fn extract_text_from_response(resp: &GeminiResponse) -> Result<&str> {
    if let Some(err) = &resp.error {
        anyhow::bail!("Gemini API error: {}", err.message);
    }
    resp.candidates
        .as_ref()
        .and_then(|c| c.first())
        .and_then(|c| c.content.parts.first())
        .and_then(|p| p.text.as_deref())
        .ok_or_else(|| anyhow::anyhow!("No response from Gemini"))
}

pub async fn extract_transactions_from_pdf(
    api_key: &str,
    model: &str,
    pdf_bytes: &[u8],
) -> Result<Vec<ExtractedTransaction>> {
    use base64::Engine;

    let client = reqwest::Client::new();
    let base64_pdf = base64::engine::general_purpose::STANDARD.encode(pdf_bytes);

    let generation_config = GenerationConfig {
        response_mime_type: "application/json".to_string(),
        response_json_schema: build_response_schema(),
    };

    let batch = BatchRequest {
        requests: vec![GeminiRequest {
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
            generation_config,
        }],
    };

    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/{}:batchGenerateContent?key={}",
        model, api_key
    );

    let response = client.post(&url).json(&batch).send().await?;
    let batch_resp: BatchResponse = response.json().await?;

    if let Some(err) = batch_resp.error {
        anyhow::bail!("Gemini batch API error: {}", err.message);
    }

    let responses = batch_resp
        .responses
        .ok_or_else(|| anyhow::anyhow!("Empty batch response from Gemini"))?;

    let first = responses
        .first()
        .ok_or_else(|| anyhow::anyhow!("No responses in batch result"))?;

    let text = extract_text_from_response(first)?;
    let extracted: ExtractedResponse = serde_json::from_str(text)?;
    Ok(extracted.transactions)
}
