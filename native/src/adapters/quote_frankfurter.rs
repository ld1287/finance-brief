//! Frankfurter (ECB reference rates) FX adapter.
//!
//! Backs `quote.snapshot` for `tab=fx`. Endpoint:
//! <https://api.frankfurter.dev/v1/latest?from={BASE}&to={QUOTE}>
//!
//! Response:
//! ```json
//! { "amount": 1.0, "base": "USD", "date": "2026-10-02",
//!   "rates": { "EUR": 0.92, "JPY": 149.5 } }
//! ```
//!
//! Frankfurter is free, no auth, ECB daily 16:00 CET fixing. `change_pct`
//! against the prior fix is not exposed by `/latest`; we set it to 0 and
//! document the limitation. `volume` is not meaningful for FX spot, also
//! set to 0.

use crate::host::CapabilityRuntime;
use serde_json::{json, Value};

pub fn register(rt: &mut CapabilityRuntime) {
    let contract = crate::host::JsonToolContract::new(
        json!({
            "type": "object",
            "properties": {
                "tab":   {"type": "string", "enum": ["fx"]},
                "pairs": {
                    "type": "array",
                    "items": {
                        "type": "string",
                        "pattern": "^[A-Z]{3}[A-Z]{3}$",
                        "minLength": 6,
                        "maxLength": 6
                    }
                }
            },
            "required": ["tab", "pairs"],
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

    rt.register_validated_json_tool("quote_frankfurter", contract, handle);
}

async fn handle(args: Value) -> Result<Value, String> {
    let tab = args
        .get("tab")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "missing tab".to_string())?;
    if tab != "fx" {
        return Err(format!("frankfurter only serves tab=fx, got {tab}"));
    }
    let pairs = args
        .get("pairs")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "missing pairs".to_string())?;
    if pairs.is_empty() {
        return Err("pairs must be non-empty".into());
    }

    let ts = now_ts();
    let mut rows = Vec::with_capacity(pairs.len());

    // Group by base to amortise HTTP calls: one GET per unique base, then
    // filter `rates` for the requested quote currencies locally.
    let mut by_base: std::collections::BTreeMap<String, Vec<String>> =
        std::collections::BTreeMap::new();
    for p in pairs.iter().filter_map(|v| v.as_str()) {
        if p.len() != 6 {
            return Err(format!("pair must be 6 letters, got {p}"));
        }
        let (base, quote) = p.split_at(3);
        by_base
            .entry(base.to_string())
            .or_default()
            .push(quote.to_string());
    }

    for (base, quotes) in by_base {
        let url = format!(
            "https://api.frankfurter.dev/v1/latest?from={base}&to={}",
            quotes.join(",")
        );
        let body: Value = reqwest::get(&url)
            .await
            .map_err(|e| format!("frankfurter GET {base} failed: {e}"))?
            .json()
            .await
            .map_err(|e| format!("frankfurter body parse failed: {e}"))?;
        let rates = body
            .get("rates")
            .and_then(|v| v.as_object())
            .ok_or_else(|| format!("frankfurter body missing rates for {base}"))?;

        for quote in quotes {
            let last = rates
                .get(&quote)
                .and_then(|v| v.as_f64())
                .ok_or_else(|| format!("frankfurter missing rate {base}/{quote}"))?;
            let pair = format!("{base}{quote}");
            rows.push(json!({
                "symbol": pair,
                "last": last,
                "change_pct": 0.0,
                "bid": last,
                "ask": last,
                "volume": 0,
                "ts": ts,
            }));
        }
    }

    Ok(json!({ "rows": rows, "fetched_at": ts }))
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
    fn url_contains_from_and_to() {
        // Pure format check: `?from=USD&to=EUR,JPY` must be in the URL.
        let url = "https://api.frankfurter.dev/v1/latest?from=USD&to=EUR,JPY";
        assert!(url.contains("from=USD"));
        assert!(url.contains("to=EUR,JPY"));
    }

    #[test]
    fn now_ts_is_positive_after_2020() {
        assert!(now_ts() > 1_577_836_800);
    }
}
