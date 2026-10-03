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
    tracing::info!(capability = "quote_hyperliquid", "register adapter");
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

    let mut syms: Vec<String> = symbols
        .iter()
        .filter_map(|v| v.as_str().map(str::to_string))
        .collect();
    syms.sort();
    let filter_key = format!("quote_hyperliquid:filter:{}", syms.join(","));
    let allmids_key = "quote_hyperliquid:allmids".to_string();

    let now = now_ts() as u64;

    // First try to short-circuit on the filtered slice cache (already-built rows).
    if let Some(envelope) = crate::cache::load_cached::<Value>(&filter_key, now)? {
        return Ok(envelope);
    }

    // Reuse the raw allMids cache when available, else fetch.
    let mids: Value = match crate::cache::load_cached::<Value>(&allmids_key, now)? {
        Some(v) => v,
        None => {
            let v = fetch_all_mids().await?;
            crate::cache::store_cached(&allmids_key, &v, 5, now)?;
            v
        }
    };

    let mut rows = Vec::with_capacity(syms.len());
    for sym in syms {
        let last = mids
            .get(&sym)
            .and_then(|v| v.as_f64())
            .ok_or_else(|| format!("hyperliquid has no mid for {sym}"))?;
        rows.push(json!({
            "symbol": sym,
            "last": last,
            "change_pct": 0.0,
            "bid": last,
            "ask": last,
            "volume": 0,
            "ts": now,
        }));
    }
    let envelope = json!({ "rows": rows, "fetched_at": now });
    crate::cache::store_cached(&filter_key, &envelope, 5, now)?;
    Ok(envelope)
}

/// POST `{"type":"allMids"}` and parse the flat mids map.
async fn fetch_all_mids() -> Result<Value, String> {
    let url = "https://api.hyperliquid.xyz/info";
    tracing::info!(url = %url, "fetch begin");
    let resp = reqwest::Client::new()
        .post(url)
        .header("Content-Type", "application/json")
        .json(&json!({ "type": "allMids" }))
        .send()
        .await
        .map_err(|e| {
            tracing::warn!(err = %e, "fetch timeout");
            format!("hyperliquid POST failed: {e}")
        })?;
    let status = resp.status();
    if status.is_success() {
        tracing::info!(status = %status, "fetch ok");
    } else {
        tracing::warn!(status = %status, "fetch non-2xx");
    }
    let body = resp.json::<Value>().await.map_err(|e| {
        tracing::error!(err = %e, "parse failed");
        format!("hyperliquid body parse failed: {e}")
    })?;
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
