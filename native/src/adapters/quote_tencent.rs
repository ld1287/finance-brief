//! Tencent Finance (qt.gtimg.cn) 行情快照 adapter.
//!
//! Backs `quote.snapshot` for `tab=a` (沪深) and `tab=hk` (港股). US real-time
//! is served by `quote_stooq` (per Q-R1: B).
//!
//! Upstream: <https://qt.gtimg.cn/q={symbols}>  e.g. `q=sh600519,sh000001,hk00700`
//! Returns a single line per symbol:
//!
//! ```text
//! v_sh600519="1~贵州茅台~600519~1888.50~1890.00~1875.20~1895.00~1880.30~...";
//! ```
//!
//! Field layout (post-`~`): name, code, last, open, pre_close, high, low,
//! ... then volume/amount at the tail. We only require `last` for the
//! `quote.schema.json` `required` set and surface the rest when available.

use crate::host::CapabilityRuntime;
use serde_json::{json, Value};

pub fn register(rt: &mut CapabilityRuntime) {
    let contract = crate::host::JsonToolContract::new(
        json!({
            "type": "object",
            "properties": {
                "tab":     {"type": "string", "enum": ["a", "hk"]},
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

    rt.register_validated_json_tool("quote_tencent", contract, handle);
}

async fn handle(args: Value) -> Result<Value, String> {
    let tab = args
        .get("tab")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "missing tab".to_string())?;
    let symbols = args
        .get("symbols")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "missing symbols".to_string())?;

    if symbols.is_empty() {
        return Err("symbols must be non-empty".into());
    }

    let joined = symbols
        .iter()
        .filter_map(|v| v.as_str())
        .collect::<Vec<_>>()
        .join(",");
    let url = format!("https://qt.gtimg.cn/q={}", joined);

    let body = reqwest::get(&url)
        .await
        .map_err(|e| format!("tencent GET failed: {e}"))?
        .text()
        .await
        .map_err(|e| format!("tencent body read failed: {e}"))?;

    let rows = parse_tencent(&body, tab)?;
    Ok(json!({ "rows": rows, "fetched_at": now_ts() }))
}

/// Split the single-line response into one `QuoteRow` per `v_<sym>="...";`
/// segment, then unpack the `~`-delimited fields in the documented order.
fn parse_tencent(body: &str, _tab: &str) -> Result<Vec<Value>, String> {
    let mut rows = Vec::new();
    for line in body.split(';') {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let eq_pos = match line.find('=') {
            Some(p) => p,
            None => continue,
        };
        let (head, tail) = line.split_at(eq_pos);
        let symbol = head.trim_start_matches("v_").trim().to_string();
        let payload = tail.trim_start_matches('=').trim().trim_matches('"');
        let parts: Vec<&str> = payload.split('~').collect();

        // Tencent leaves trailing fields blank; always treat index 3 as `last`
        // when present and skip empties.
        let last = parts.get(3).and_then(|s| s.parse::<f64>().ok());
        let Some(last) = last else {
            continue;
        };
        let prev_close = parts
            .get(4)
            .and_then(|s| s.parse::<f64>().ok())
            .unwrap_or(last);
        let _open = parts
            .get(5)
            .and_then(|s| s.parse::<f64>().ok())
            .unwrap_or(last);
        let volume = parts
            .get(6)
            .and_then(|s| s.parse::<i64>().ok())
            .unwrap_or(0);
        let high = parts
            .get(33)
            .and_then(|s| s.parse::<f64>().ok())
            .unwrap_or(last);
        let low = parts
            .get(34)
            .and_then(|s| s.parse::<f64>().ok())
            .unwrap_or(last);

        let change_pct = if prev_close.abs() > f64::EPSILON {
            (last - prev_close) / prev_close * 100.0
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
    fn parse_tencent_extracts_last_and_change() {
        // `prev_close` (idx 4) = 1899.0, `last` (idx 3) = 1888.5 → -0.55%
        let body =
            r#"v_sh600519="1~贵州茅台~600519~1888.50~1899.00~1875.20~12345~...~~~-10.50~-0.55";"#;
        let rows = parse_tencent(body, "a").unwrap();
        assert_eq!(rows.len(), 1);
        let row = &rows[0];
        assert_eq!(row["symbol"], "sh600519");
        assert!((row["last"].as_f64().unwrap() - 1888.50).abs() < 0.01);
        assert!((row["change_pct"].as_f64().unwrap() - (-0.5529)).abs() < 0.01);
    }

    #[test]
    fn parse_tencent_skips_blank_segments() {
        let body = "v_a=\"1~a~1~10~10~10~0\";;v_b=\"\";";
        let rows = parse_tencent(body, "a").unwrap();
        assert_eq!(rows.len(), 1, "blank segments are skipped");
        assert_eq!(rows[0]["symbol"], "a");
    }
}
