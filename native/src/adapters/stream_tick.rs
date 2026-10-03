//! Stream tick adapter — backs the v2 `stream.tick` capability declared in
//! `finance-brief/native/capabilities.toml` §12.
//!
//! ## Input / output
//!
//! - input:  `{symbols: [string], kind?: "ticker" | "orderbook" | "news" | "all", limit?: i64}`
//! - output: `{rows: [{symbol, ts, last, change_pct, kind}], fetched_at: i64}`
//!
//! MVP behaviour: **mock data** (per `MVP-TODO.md §3.5.3` + Q-E: B). For each
//! requested symbol we synthesise a `last` around `100 ± 10%` and a
//! `change_pct` in `±2.0%`. Real orderbook / news rows are produced by
//! `quote_hyperliquid.rs` + `news_sina.rs`; a follow-up agent will route
//! those through `stream.tick` keyed by `kind` (Q-E: B "共用
//! hyperliquid/orderbook 思路").
//!
//! `output.rows` length equals `input.symbols` length, truncated to `limit`
//! when provided. `kind` defaults to `"ticker"`.

use crate::host::CapabilityRuntime;
use serde_json::{json, Value};

pub fn register(rt: &mut CapabilityRuntime) {
    tracing::info!(capability = "stream_tick", "register adapter");
    let contract = crate::host::JsonToolContract::new(
        json!({
            "type": "object",
            "properties": {
                "symbols": {
                    "type": "array",
                    "items": {"type": "string", "minLength": 1, "maxLength": 32},
                    "minItems": 1,
                    "maxItems": 32
                },
                "kind": {
                    "type": "string",
                    "enum": ["ticker", "orderbook", "news", "all"]
                },
                "limit": {"type": "integer", "minimum": 1, "maximum": 256}
            },
            "required": ["symbols"],
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
                            "ts":         {"type": "integer"},
                            "last":       {"type": "number"},
                            "change_pct": {"type": "number"},
                            "kind":       {"type": "string"}
                        },
                        "required": ["symbol", "ts", "last", "change_pct", "kind"],
                        "additionalProperties": false
                    }
                },
                "fetched_at": {"type": "integer"}
            },
            "required": ["rows", "fetched_at"],
            "additionalProperties": false
        }),
    );

    rt.register_validated_json_tool("stream_tick", contract, handle);
}

async fn handle(args: Value) -> Result<Value, String> {
    let symbols: Vec<String> = args
        .get("symbols")
        .and_then(|v| v.as_array())
        .ok_or_else(|| {
            tracing::warn!("stream.tick missing symbols");
            "missing symbols".to_string()
        })?
        .iter()
        .filter_map(|v| v.as_str().map(str::to_string))
        .collect();

    if symbols.is_empty() {
        tracing::warn!("stream.tick empty symbols");
        return Err("symbols must be non-empty".into());
    }

    let kind = args
        .get("kind")
        .and_then(|v| v.as_str())
        .unwrap_or("ticker")
        .to_string();

    let limit = args
        .get("limit")
        .and_then(|v| v.as_i64())
        .map(|n| n.max(0) as usize);

    let now = now_ts();
    let mut rows: Vec<Value> = symbols
        .iter()
        .map(|sym| build_mock_row(sym, &kind, now))
        .collect();

    if let Some(n) = limit {
        rows.truncate(n);
    }

    tracing::info!(
        symbols = symbols.len(),
        kind = %kind,
        rows = rows.len(),
        "stream.tick mock build",
    );

    Ok(json!({
        "rows": rows,
        "fetched_at": now,
    }))
}

/// Synthesise one mock tick row: `last ∈ [90, 110]`, `change_pct ∈ [-2, 2]`.
/// Symbol contributes to the seed so the same call returns the same row for
/// the same symbol — useful for snapshot diffs in tests.
fn build_mock_row(symbol: &str, kind: &str, ts: i64) -> Value {
    let seed = ts as u64
        ^ symbol
            .chars()
            .fold(0u64, |acc, c| acc.wrapping_mul(131).wrapping_add(c as u64));
    let (last_unit, change_unit) = lcg_pair(seed);
    let last = 100.0 + (last_unit - 0.5) * 20.0; // ±10
    let change_pct = (change_unit - 0.5) * 4.0; // ±2%

    json!({
        "symbol": symbol,
        "ts": ts,
        "last": round2(last),
        "change_pct": round2(change_pct),
        "kind": kind,
    })
}

/// Two pseudo-random values in `[0, 1)` from a 64-bit seed via a small LCG
/// (SplitMix64-ish). No `rand` dep needed for MVP mock data.
fn lcg_pair(mut seed: u64) -> (f64, f64) {
    seed = seed.wrapping_add(0x9E37_79B9_7F4A_7C15);
    seed = (seed ^ (seed >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    seed = (seed ^ (seed >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    let a = ((seed ^ (seed >> 31)) as f64) / (u64::MAX as f64);

    seed = seed.wrapping_add(0x9E37_79B9_7F4A_7C15);
    seed = (seed ^ (seed >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    seed = (seed ^ (seed >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    let b = ((seed ^ (seed >> 31)) as f64) / (u64::MAX as f64);

    (a, b)
}

fn round2(x: f64) -> f64 {
    (x * 100.0).round() / 100.0
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
    fn mock_row_stays_in_band() {
        let row = build_mock_row("sh600519", "ticker", 1_700_000_000);
        assert_eq!(row["symbol"], "sh600519");
        assert_eq!(row["ts"], 1_700_000_000);
        assert_eq!(row["kind"], "ticker");
        let last = row["last"].as_f64().unwrap();
        assert!((90.0..=110.0).contains(&last), "last={last}");
        let cp = row["change_pct"].as_f64().unwrap();
        assert!((-2.0..=2.0).contains(&cp), "change_pct={cp}");
    }

    #[test]
    fn mock_row_is_deterministic_per_symbol_and_ts() {
        let a = build_mock_row("BTC", "ticker", 42);
        let b = build_mock_row("BTC", "ticker", 42);
        assert_eq!(a, b);
    }

    #[test]
    fn lcg_pair_lies_in_unit_interval() {
        let (a, b) = lcg_pair(42);
        assert!((0.0..1.0).contains(&a));
        assert!((0.0..1.0).contains(&b));
    }

    #[test]
    fn round2_truncates_to_two_decimals() {
        assert_eq!(round2(1.23456), 1.23);
        assert_eq!(round2(-1.235), -1.24); // banker's-rounding free
    }
}
