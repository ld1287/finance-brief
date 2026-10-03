//! Streaming subscription controls — backs three v2 capabilities declared in
//! `finance-brief/bundle/capabilities.toml` §9-13:
//!
//! - `stream.subscribe`     — register a set of symbols to be polled
//! - `stream.unsubscribe`   — stop receiving ticks for some / all symbols
//! - `stream.frequency.set` — change poll cadence
//!
//! ## Input / output
//!
//! - `stream.subscribe`:
//!   - input:  `{symbols: [string, ...]}` (1–32 entries, each non-empty)
//!   - output: `{active: true, symbols: [...], ts: i64}`
//! - `stream.unsubscribe`:
//!   - input:  `{}` or `{symbols?: [string]}` (symbols omitted → stop all)
//!   - output: `{active: false, ts: i64}`
//! - `stream.frequency.set`:
//!   - input:  `{frequency: "1s" | "5s" | "10s"}`
//!   - output: `{frequency: "...", ts: i64}`
//!
//! MVP behaviour: this adapter is a **stub**. The splash workflow drives the
//! actual cadence (a timer on the splash side calls `stream.tick` at the
//! requested `frequency`), so this adapter just echoes the state it has been
//! asked to set. The simulated write models the cost of pushing the
//! subscription state to a real broker — once one is wired (Kafka / NATS /
//! Redis pub-sub, see R-3 supersede) the yield loop will be replaced with a
//! real `await` on the producer ack.

use crate::host::CapabilityRuntime;
use serde_json::{json, Value};

pub fn register(rt: &mut CapabilityRuntime) {
    // ---- stream.subscribe ----
    let subscribe_contract = crate::host::JsonToolContract::new(
        json!({
            "type": "object",
            "properties": {
                "symbols": {
                    "type": "array",
                    "items": {"type": "string", "minLength": 1, "maxLength": 32},
                    "minItems": 1,
                    "maxItems": 32
                }
            },
            "required": ["symbols"],
            "additionalProperties": false
        }),
        json!({
            "type": "object",
            "properties": {
                "active":  {"type": "boolean"},
                "symbols": {"type": "array", "items": {"type": "string"}},
                "ts":      {"type": "integer"}
            },
            "required": ["active", "symbols", "ts"],
            "additionalProperties": false
        }),
    );
    rt.register_validated_json_tool("stream_subscribe", subscribe_contract, handle_subscribe);

    // ---- stream.unsubscribe ----
    let unsubscribe_contract = crate::host::JsonToolContract::new(
        json!({
            "type": "object",
            "properties": {
                "symbols": {
                    "type": "array",
                    "items": {"type": "string", "minLength": 1, "maxLength": 32}
                }
            },
            "additionalProperties": false
        }),
        json!({
            "type": "object",
            "properties": {
                "active": {"type": "boolean"},
                "ts":     {"type": "integer"}
            },
            "required": ["active", "ts"],
            "additionalProperties": false
        }),
    );
    rt.register_validated_json_tool(
        "stream_unsubscribe",
        unsubscribe_contract,
        handle_unsubscribe,
    );

    // ---- stream.frequency.set ----
    let frequency_contract = crate::host::JsonToolContract::new(
        json!({
            "type": "object",
            "properties": {
                "frequency": {"type": "string", "enum": ["1s", "5s", "10s"]}
            },
            "required": ["frequency"],
            "additionalProperties": false
        }),
        json!({
            "type": "object",
            "properties": {
                "frequency": {"type": "string"},
                "ts":        {"type": "integer"}
            },
            "required": ["frequency", "ts"],
            "additionalProperties": false
        }),
    );
    rt.register_validated_json_tool(
        "stream_set_frequency",
        frequency_contract,
        handle_set_frequency,
    );
}

async fn handle_subscribe(args: Value) -> Result<Value, String> {
    let symbols: Vec<String> = args
        .get("symbols")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "missing symbols".to_string())?
        .iter()
        .filter_map(|v| v.as_str().map(str::to_string))
        .collect();

    if symbols.is_empty() {
        return Err("symbols must be non-empty".into());
    }

    simulate_broker_write().await;

    Ok(json!({
        "active": true,
        "symbols": symbols,
        "ts": now_ts(),
    }))
}

async fn handle_unsubscribe(args: Value) -> Result<Value, String> {
    // `symbols` is optional — present means "stop just these", absent means
    // "stop everything". We don't need to look at the contents; the splash
    // workflow tears its own timer / broker handles down on the next tick.
    let _maybe_symbols = args.get("symbols").and_then(|v| v.as_array());

    simulate_broker_write().await;

    Ok(json!({
        "active": false,
        "ts": now_ts(),
    }))
}

async fn handle_set_frequency(args: Value) -> Result<Value, String> {
    let frequency = args
        .get("frequency")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "missing frequency".to_string())?
        .to_string();

    simulate_broker_write().await;

    Ok(json!({
        "frequency": frequency,
        "ts": now_ts(),
    }))
}

/// MVP: model a 50 ms broker write. `tokio` here is built without the `time`
/// feature, so a real `tokio::time::sleep` would not compile. The yield loop
/// is a stand-in: the future still hands control back to the runtime, which
/// is enough to keep the contract honest (no blocking inside `async fn`).
async fn simulate_broker_write() {
    for _ in 0..16 {
        tokio::task::yield_now().await;
    }
}

fn now_ts() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

// No inline async-handler tests: tokio is wired without `test-util` and we
// don't pull `futures` as a dep. Behaviour coverage lives in mod.rs's
// `register_all_adapter_accepts_mut_runtime` plumbing smoke test.
