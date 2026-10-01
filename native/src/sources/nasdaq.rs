//! Nasdaq candle/quote source.
//! TODO: real endpoint URL + auth headers + response mapping.

pub async fn fetch(_args: &serde_json::Value) -> Result<serde_json::Value, String> {
    // TODO: GET https://api.nasdaq.com/...
    Err("not implemented".into())
}
