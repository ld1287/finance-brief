//! Synthesised OHLCV candle series adapter.
//!
//! Backs `quote.candles` for every symbol during local development. Real
//! exchanges will replace this one symbol-at-a-time (per `quote.snapshot`'s
//! source list) but the schema is identical, so the UI does not branch.
//!
//! Determinism: the random walk is seeded by the symbol name so a chart
//! redrawn with the same args shows the same bars. The seed is the
//! `cityhash`-style FNV-1a fold — good enough for stable screenshots, not
//! a cryptographic hash.

use crate::host::CapabilityRuntime;
use serde_json::{json, Value};

pub fn register(rt: &mut CapabilityRuntime) {
    tracing::info!(capability = "synth_candles", "register adapter");
    let contract = crate::host::JsonToolContract::new(
        json!({
            "type": "object",
            "properties": {
                "symbol": {"type": "string", "minLength": 1, "maxLength": 32},
                "period": {"type": "string", "enum": ["1m","5m","15m","1h","4h","1d","1w"]},
                "count":  {"type": "integer", "minimum": 1, "maximum": 4096}
            },
            "required": ["symbol", "period"],
            "additionalProperties": false
        }),
        json!({
            "type": "object",
            "properties": {
                "symbol":      {"type": "string"},
                "period":      {"type": "string"},
                "bars": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "timestamp": {"type": "integer"},
                            "open":      {"type": "number"},
                            "high":      {"type": "number"},
                            "low":       {"type": "number"},
                            "close":     {"type": "number"},
                            "volume":    {"type": "integer"}
                        },
                        "required": ["timestamp", "open", "high", "low", "close", "volume"],
                        "additionalProperties": false
                    }
                },
                "generated_at": {"type": "integer"}
            },
            "required": ["symbol", "period", "bars"],
            "additionalProperties": false
        }),
    );

    rt.register_validated_json_tool("synth_candles", contract, handle);
}

async fn handle(args: Value) -> Result<Value, String> {
    let symbol = args
        .get("symbol")
        .and_then(|v| v.as_str())
        .ok_or_else(|| {
            tracing::warn!("synth_candles missing symbol");
            "missing symbol".to_string()
        })?
        .to_string();
    let period = args
        .get("period")
        .and_then(|v| v.as_str())
        .ok_or_else(|| {
            tracing::warn!("synth_candles missing period");
            "missing period".to_string()
        })?
        .to_string();
    let count = args.get("count").and_then(|v| v.as_u64()).unwrap_or(120) as usize;

    let bars = synth_bars(&symbol, &period, count);
    let period_secs = period_seconds(&period);
    let now = now_ts();

    tracing::info!(
        symbol = %symbol,
        period = %period,
        count = bars.len(),
        "synth_candles build",
    );

    Ok(json!({
        "symbol": symbol,
        "period": period,
        "bars": bars,
        "generated_at": now,
        // Surfaces the chosen period length to the UI for axis labels.
        "period_seconds": period_secs,
    }))
}

/// Deterministic OHLC walk: seeded by symbol + period, so the same args
/// produce the same series across process restarts.
fn synth_bars(symbol: &str, period: &str, count: usize) -> Vec<Value> {
    let mut seed = fnv1a(symbol.as_bytes()) ^ fnv1a(period.as_bytes());
    let step = period_seconds(period);
    let now = now_ts();
    let mut price = 100.0_f64;

    let mut out = Vec::with_capacity(count);
    for i in 0..count {
        // xorshift64 — tiny, fast, deterministic. Good enough for visual
        // mock data; the candle walk dominates any statistical artefact.
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        let r = (seed as f64) / (u64::MAX as f64);
        let drift = (r - 0.5) * 2.0; // -1 .. +1

        let open = price;
        let close = (open + drift).max(0.01);
        let high = open.max(close) + r.abs() * 0.4;
        let low = (open.min(close) - r.abs() * 0.4).max(0.0);
        let volume = 1000 + ((seed % 9000) as i64);

        // Bars are ordered oldest-first; `i=0` is the earliest bar.
        let ts = now - ((count - 1 - i) as i64) * step;
        out.push(json!({
            "timestamp": ts,
            "open": round2(open),
            "high": round2(high),
            "low":  round2(low),
            "close": round2(close),
            "volume": volume,
        }));
        price = close;
    }
    out
}

/// FNV-1a 64-bit fold. Tiny, allocation-free, decent dispersion for ASCII.
fn fnv1a(bytes: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    for &b in bytes {
        h ^= b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}

fn period_seconds(p: &str) -> i64 {
    match p {
        "1m" => 60,
        "5m" => 300,
        "15m" => 900,
        "1h" => 3600,
        "4h" => 14_400,
        "1d" => 86_400,
        "1w" => 604_800,
        _ => 86_400, // sensible default for any future period label
    }
}

fn round2(v: f64) -> f64 {
    (v * 100.0).round() / 100.0
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
    fn synth_bars_is_deterministic() {
        let a = synth_bars("AAPL", "1d", 30);
        let b = synth_bars("AAPL", "1d", 30);
        assert_eq!(a.len(), 30);
        assert_eq!(a, b, "same symbol/period must produce identical series");
    }

    #[test]
    fn synth_bars_respect_ohlc_invariants() {
        let bars = synth_bars("BTC", "1h", 50);
        for b in &bars {
            let h = b["high"].as_f64().unwrap();
            let l = b["low"].as_f64().unwrap();
            let o = b["open"].as_f64().unwrap();
            let c = b["close"].as_f64().unwrap();
            assert!(h >= o && h >= c, "high must dominate o/c");
            assert!(l <= o && l <= c, "low must be below o/c");
        }
    }

    #[test]
    fn period_seconds_known_values() {
        assert_eq!(period_seconds("1m"), 60);
        assert_eq!(period_seconds("1d"), 86_400);
        assert_eq!(period_seconds("1w"), 604_800);
    }
}
