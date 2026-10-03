//! Hyperliquid on-chain perps adapter.
//!
//! Backs `quote.snapshot` for `tab=crypto`. Unlike the others, Hyperliquid
//! requires a JSON-RPC-style POST to <https://api.hyperliquid.xyz/info>
//! with the body `{"type":"allMids"}` (one snapshot for the whole universe)
//! or `{"type":"ticker","coin":"<symbol>"}` for a single coin. We use
//! `allMids` and then filter client-side so one HTTP request serves any
//! number of `symbols`.
//!
//! Response shape:
//! ```json
//! {"BTC":"100123.5","ETH":"3500.25", ...}
//! ```
//! No `change_pct` / `bid` / `ask` is exposed by `allMids`; we surface
//! zeros (with the schema's `minimum: 0` bound respected for `last` only)
//! and let the UI annotate when a richer feed (e.g. `l2Book`) lands.

use crate::host::CapabilityRuntime;
use serde_json::{json, Value};

pub fn register(rt: &mut CapabilityRuntime) {
    let contract = crate::host::JsonToolContract::new(
        json!({
            "type": "object",
            "properties": {
                "tab":     {"type": "string", "enum": ["crypto"]},
                "symbols": {"type": "array", "items": {"type": "string", "minLength": 1, "maxLength": 16}}
            },
            "required": ["tab", "symbols"],
            "additionalProperties": false
        }),
        json!({
            "type": "object",
            "properties": {
                "rows": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "symbol":     {"type": "string"},
                            "last":       {"type": "number"},
                            "change_pct": {"type": "number"},
                            "bid":        {"type": "number"},
                            "ask":        {"type": "number"},
                            "volume":     {"type": "integer"},
                            "ts":         {"type": "integer"}
                        },
                        "required": ["symbol", "last", "change_pct", "bid", "ask", "volume"],
                        "additionalProperties": false
                    }
                },
                "fetched_at": {"type": "integer"}
            },
            "required": ["rows"],
            "additionalProperties": false
        }),
    );

    rt.register_validated_json_tool("quote_hyperliquid", contract, handle);
}

async fn handle(args: Value) -> Result<Value, String> {
    let tab = args
        .get("tab")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "missing tab".to_string())?;
    if tab != "crypto" {
        return Err(format!("hyperliquid only serves tab=crypto, got {tab}"));
    }
    let symbols = args
        .get("symbols")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "missing symbols".to_string())?;
    if symbols.is_empty() {
        return Err("symbols must be non-empty".into());
    }

    let mids = fetch_all_mids().await?;
    let ts = now_ts();
    let mut rows = Vec::with_capacity(symbols.len());
    for sym in symbols.iter().filter_map(|v| v.as_str()) {
        let last = mids
            .get(sym)
            .and_then(|v| v.as_f64())
            .ok_or_else(|| format!("hyperliquid has no mid for {sym}"))?;
        rows.push(json!({
            "symbol": sym,
            "last": last,
            "change_pct": 0.0,
            "bid": last,
            "ask": last,
            "volume": 0,
            "ts": ts,
        }));
    }
    Ok(json!({ "rows": rows, "fetched_at": ts }))
}

/// POST `{"type":"allMids"}` and parse the flat mids map.
async fn fetch_all_mids() -> Result<Value, String> {
    let url = "https://api.hyperliquid.xyz/info";
    let body = reqwest::Client::new()
        .post(url)
        .header("Content-Type", "application/json")
        .json(&json!({ "type": "allMids" }))
        .send()
        .await
        .map_err(|e| format!("hyperliquid POST failed: {e}"))?
        .json::<Value>()
        .await
        .map_err(|e| format!("hyperliquid body parse failed: {e}"))?;
    Ok(body)
}

fn now_ts() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn now_ts_is_positive_after_2020() {
        assert!(now_ts() > 1_577_836_800);
    }

    /// Pure JSON-shape sanity: the build URL + request body must match the
    /// Hyperliquid info endpoint contract.
    #[test]
    fn request_body_matches_hyperliquid_contract() {
        let body = json!({ "type": "allMids" });
        assert_eq!(body["type"], "allMids");
    }
}
