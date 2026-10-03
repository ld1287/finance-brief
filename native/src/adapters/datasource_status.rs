//! Datasource health rollup — backs the v2 `datasource.status` capability
//! declared in `finance-brief/native/capabilities.toml` §13.
//!
//! ## Input / output
//!
//! - input:  `{capabilities?: [string]}` — when omitted, returns the rollup
//!   for the 5 default datasources: `news_sina`, `quote_tencent`,
//!   `quote_stooq`, `quote_hyperliquid`, `quote_frankfurter`.
//! - output: `{rows: [{name, kind, status, last_ok, last_err, latency_ms,
//!                    success_count, err_count}], fetched_at: i64}`
//!
//! MVP behaviour: **stub**. Every datasource returns a fixed healthy record
//! (`status: "ok"`, `latency_ms: 120`, one successful call, no errors). Real
//! audit log aggregation (per-request success / error counters, rolling p50
//! and p95 latency) is left to a follow-up agent (R-3 / supersede), which
//! will replay a request log shipped by the splash VM.

use crate::host::CapabilityRuntime;
use serde_json::{json, Value};

/// Default datasources covered by the rollup. `kind` is the human-readable
/// data family — surfaced so the splash health panel can group rows.
const DEFAULT_DATASOURCES: &[(&str, &str)] = &[
    ("news_sina", "news"),
    ("quote_tencent", "quote"),
    ("quote_stooq", "quote"),
    ("quote_hyperliquid", "quote"),
    ("quote_frankfurter", "fx"),
];

pub fn register(rt: &mut CapabilityRuntime) {
    let contract = crate::host::JsonToolContract::new(
        json!({
            "type": "object",
            "properties": {
                "capabilities": {
                    "type": "array",
                    "items": {"type": "string", "minLength": 1, "maxLength": 32},
                    "maxItems": 32
                }
            },
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
                            "name":          {"type": "string"},
                            "kind":          {"type": "string"},
                            "status":        {"type": "string", "enum": ["ok", "degraded", "down"]},
                            "last_ok":       {"type": "integer"},
                            "last_err":      {"type": ["string", "null"]},
                            "latency_ms":    {"type": ["number", "null"]},
                            "success_count": {"type": "number"},
                            "err_count":     {"type": "number"}
                        },
                        "required": [
                            "name", "kind", "status", "last_ok", "last_err",
                            "latency_ms", "success_count", "err_count"
                        ],
                        "additionalProperties": false
                    }
                },
                "fetched_at": {"type": "integer"}
            },
            "required": ["rows", "fetched_at"],
            "additionalProperties": false
        }),
    );

    rt.register_validated_json_tool("datasource_status", contract, handle);
}

async fn handle(args: Value) -> Result<Value, String> {
    let requested: Option<Vec<String>> =
        args.get("capabilities")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            });

    let entries: Vec<(&'static str, &'static str)> = match requested {
        None => DEFAULT_DATASOURCES.to_vec(),
        Some(reqs) => reqs
            .iter()
            .filter_map(|n| {
                DEFAULT_DATASOURCES
                    .iter()
                    .find(|(name, _)| *name == n.as_str())
                    .copied()
            })
            .collect(),
    };

    let now = now_ts();
    let rows: Vec<Value> = entries
        .iter()
        .map(|(name, kind)| build_stub_row(name, kind, now))
        .collect();

    Ok(json!({
        "rows": rows,
        "fetched_at": now,
    }))
}

fn build_stub_row(name: &str, kind: &str, now: i64) -> Value {
    json!({
        "name": name,
        "kind": kind,
        "status": "ok",
        "last_ok": now,
        "last_err": null,
        "latency_ms": 120.0,
        "success_count": 1,
        "err_count": 0,
    })
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
    fn stub_row_matches_contract() {
        let row = build_stub_row("quote_tencent", "quote", 1_700_000_000);
        assert_eq!(row["name"], "quote_tencent");
        assert_eq!(row["kind"], "quote");
        assert_eq!(row["status"], "ok");
        assert_eq!(row["last_ok"], 1_700_000_000);
        assert_eq!(row["last_err"], Value::Null);
        assert_eq!(row["latency_ms"], 120.0);
        assert_eq!(row["success_count"], 1);
        assert_eq!(row["err_count"], 0);
    }

    #[test]
    fn defaults_have_five_entries() {
        assert_eq!(DEFAULT_DATASOURCES.len(), 5);
    }

    #[test]
    fn defaults_match_capabilities_toml_section_13() {
        // Sanity: the five names match what capabilities.toml §13 advertises.
        let names: Vec<&str> = DEFAULT_DATASOURCES.iter().map(|(n, _)| *n).collect();
        assert!(names.contains(&"news_sina"));
        assert!(names.contains(&"quote_tencent"));
        assert!(names.contains(&"quote_stooq"));
        assert!(names.contains(&"quote_hyperliquid"));
        assert!(names.contains(&"quote_frankfurter"));
    }
}
