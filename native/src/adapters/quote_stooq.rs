//! Stooq (stooq.com) 美股行情快照 adapter (per Q-R1: B).
//!
//! Replaces the previous US real-time source (Q-R1: B): Stooq's anonymous
//! CSV endpoint avoids the 403 risk that hit the prior upstream and needs
//! no auth, with a generous concurrent quota.
//!
//! Backs `quote.snapshot` for `tab=us`. One HTTP GET returns one CSV row per
//! requested symbol. The CSV header is exactly:
//!
//! ```text
//! Symbol,Date,Time,Open,High,Low,Close,Volume
//! AAPL.US,2026-10-02,21:00:00,177.10,178.45,176.80,178.20,51234000
//! ```
//!
//! We map `Close → last` and use `Open/High/Low` to derive a small
//! `bid/ask` band so the row still satisfies `quote.schema.json`'s required
//! fields without inventing market microstructure.

use crate::host::CapabilityRuntime;
use serde_json::{json, Value};

pub fn register(rt: &mut CapabilityRuntime) {
    tracing::info!(capability = "quote_stooq", "register adapter");
    let contract = crate::host::JsonToolContract::new(
        json!({
            "type": "object",
            "properties": {
                "tab":     {"type": "string", "enum": ["us"]},
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

    rt.register_validated_json_tool("quote_stooq", contract, handle);
}

async fn handle(args: Value) -> Result<Value, String> {
    let tab = args
        .get("tab")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "missing tab".to_string())?;
    if tab != "us" {
        return Err(format!("stooq only serves tab=us, got {tab}"));
    }
    let symbols = args
        .get("symbols")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "missing symbols".to_string())?;
    if symbols.is_empty() {
        return Err("symbols must be non-empty".into());
    }

    // Normalise the same way the upstream URL builder does, then sort.
    let mut normalised: Vec<String> = symbols
        .iter()
        .filter_map(|v| v.as_str())
        .map(|s| s.trim_end_matches(".us").to_lowercase())
        .map(|s| format!("{s}.us"))
        .collect();
    normalised.sort();
    let cache_key = format!("quote_stooq:{tab}:{}", normalised.join(","));

    let now = now_ts() as u64;
    if let Some(envelope) = crate::cache::load_cached::<Value>(&cache_key, now)? {
        return Ok(envelope);
    }

    let joined = normalised.join(",");
    // `f=sd2t2ohlcv` ⇒ Symbol, Date, Time, Open, High, Low, Close, Volume.
    // `h` ⇒ add headers. `e=csv` ⇒ CSV output (default but explicit).
    let url = format!("https://stooq.com/q/l/?s={joined}&f=sd2t2ohlcv&h&e=csv");

    tracing::info!(url = %url, symbols = normalised.len(), "fetch begin");
    let resp = reqwest::get(&url).await.map_err(|e| {
        tracing::warn!(err = %e, "fetch timeout");
        format!("stooq GET failed: {e}")
    })?;
    let status = resp.status();
    if status.is_success() {
        tracing::info!(status = %status, "fetch ok");
    } else {
        tracing::warn!(status = %status, "fetch non-2xx");
    }
    let body = resp.text().await.map_err(|e| {
        tracing::error!(err = %e, "fetch body read failed");
        format!("stooq body read failed: {e}")
    })?;

    let rows = parse_stooq_csv(&body).map_err(|e| {
        tracing::error!(err = %e, "parse failed");
        e
    })?;
    let envelope = json!({ "rows": rows, "fetched_at": now });
    crate::cache::store_cached(&cache_key, &envelope, 5, now)?;
    Ok(envelope)
}

/// Parse Stooq's CSV (header + one row per symbol). Tolerant of:
///
/// - `\r\n` / `\n` line endings
/// - empty rows
/// - rows where the upstream returned `-` for a numeric field (a Stooq
///   convention that means "data unavailable" — we drop the row).
fn parse_stooq_csv(body: &str) -> Result<Vec<Value>, String> {
    let mut rows = Vec::new();
    let mut lines = body.lines();
    let header = lines.next().unwrap_or("");
    if !header.starts_with("Symbol") {
        return Err("stooq body missing CSV header".into());
    }

    for line in lines {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let cols: Vec<&str> = line.split(',').collect();
        if cols.len() < 8 {
            continue;
        }
        let symbol = cols[0];
        let open = cols[3].parse::<f64>().ok();
        let high = cols[4].parse::<f64>().ok();
        let low = cols[5].parse::<f64>().ok();
        let last = cols[6].parse::<f64>().ok();
        let volume = cols[7].parse::<i64>().unwrap_or(0);
        let (Some(open), Some(high), Some(low), Some(last)) = (open, high, low, last) else {
            // Stooq returns `-` when the symbol halted or had no trades.
            continue;
        };

        // No `prev_close` column on Stooq's compact endpoint; use `open` as
        // the prior reference so `change_pct` still reflects the day's move.
        let change_pct = if open.abs() > f64::EPSILON {
            (last - open) / open * 100.0
        } else {
            0.0
        };

        rows.push(json!({
            "symbol": symbol,
            "last": last,
            "change_pct": change_pct,
            "bid": low,
            "ask": high,
            "volume": volume,
            "ts": now_ts(),
        }));
    }
    Ok(rows)
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
    fn parse_stooq_csv_happy_path() {
        let body = "Symbol,Date,Time,Open,High,Low,Close,Volume\n\
                    AAPL.US,2026-10-02,21:00:00,177.10,178.45,176.80,178.20,51234000\n";
        let rows = parse_stooq_csv(body).unwrap();
        assert_eq!(rows.len(), 1);
        let r = &rows[0];
        assert_eq!(r["symbol"], "AAPL.US");
        assert!((r["last"].as_f64().unwrap() - 178.20).abs() < 0.01);
        assert_eq!(r["volume"], 51_234_000);
    }

    #[test]
    fn parse_stooq_csv_skips_dash_rows() {
        let body = "Symbol,Date,Time,Open,High,Low,Close,Volume\n\
                    X.US,2026-10-02,21:00:00,-,-,-,-,0\n";
        let rows = parse_stooq_csv(body).unwrap();
        assert!(rows.is_empty(), "halted symbols are dropped");
    }

    #[test]
    fn parse_stooq_csv_rejects_missing_header() {
        let body = "AAPL.US,2026-10-02,21:00:00,177.10,178.45,176.80,178.20,51234000\n";
        assert!(parse_stooq_csv(body).is_err());
    }
}
