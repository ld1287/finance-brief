//! Hyperliquid (on-chain perps) quote source.
//! TODO: real endpoint URL + auth headers + response mapping.

pub async fn fetch(_args: &serde_json::Value) -> Result<serde_json::Value, String> {
    // TODO: POST https://api.hyperliquid.xyz/info ...
    Err("not implemented".into())
}
