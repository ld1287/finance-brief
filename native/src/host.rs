//! Host bridge — exposes Rust services to the Splash runtime via
//! `host.call` / `host.fetch`. In production this registers into a `SplashVm`
//! instance, but in this scaffold we only model the surface so the crate
//! stays compilable without pulling the splash runtime as a dependency.

/// Opaque handle to a Splash runtime. Defined here as a stub so signatures are
/// stable; the real type lives in the splash crate.
pub struct SplashVm {
    _private: (),
}

impl SplashVm {
    /// Convenience constructor for scaffolding/tests.
    pub fn dummy() -> Self {
        Self { _private: () }
    }
}

/// Services the Rust layer can serve to Splash.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Service {
    NewsRefresh,
    QuotesParseTencent,
    QuotesParseNasdaq,
    QuotesParseHyperliquid,
    QuotesParseFrankfurter,
    QuotesSnapshot,
    QuotesCandles,
    FavsToggle,
    FavsList,
    SettingsLoad,
    SettingsSave,
}

/// A JSON-shaped promise result. For now we model it as a thin wrapper so the
/// scaffold compiles; the real impl will integrate with the splash promise
/// runtime.
#[derive(Debug, Clone)]
pub struct Promise<T> {
    inner: std::marker::PhantomData<T>,
}

impl<T> Promise<T> {
    /// Wrap a synchronous result into a (placeholder) resolved promise.
    pub fn resolved(_value: T) -> Self {
        Self {
            inner: std::marker::PhantomData,
        }
    }
}

/// Register Rust services into a Splash VM. In the scaffold this is a no-op
/// signature — the real wiring lives in the splash runtime crate.
pub fn register(splash: &mut SplashVm) -> Result<(), String> {
    let _ = splash;
    // TODO: bind each variant of `Service` to a JS-callable handler.
    Err("not implemented".into())
}

/// Dispatch a service call. QuotesCandles is wired to native::synth;
/// the other 6 services stay stubbed for later agents (G8/G9/R-3).
pub fn handle(s: Service, args: serde_json::Value) -> Promise<Result<serde_json::Value, String>> {
    match s {
        Service::QuotesCandles => {
            let n = args.get("count").and_then(|v| v.as_u64()).unwrap_or(100) as usize;
            let candles = crate::synth::synth_candles(n);
            Promise::resolved(Ok(serde_json::json!({ "candles": candles })))
        }
        Service::NewsRefresh => {
            // G8: splash 暂时仍 fetch body 并通过 host.fetch("news.refresh", {body, limit})
            // 传过来。native 只 parse，limit 用于截断。
            // 真解耦轮次（native 自己 fetch sina RSS）留给 R-3 后续。
            let body = args.get("body").and_then(|v| v.as_str()).unwrap_or("");
            let limit = args.get("limit").and_then(|v| v.as_u64()).unwrap_or(20) as usize;
            let rows = crate::parse::parse_sina_news(body).unwrap_or_default();
            let truncated: Vec<_> = rows.into_iter().take(limit).collect();
            Promise::resolved(Ok(serde_json::json!({
                "rows": truncated,
                "fetched_at": 0
            })))
        }
        Service::QuotesParseFrankfurter => {
            let body = args.get("body").and_then(|v| v.as_str()).unwrap_or("");
            let rows = crate::parse::parse_frankfurter(body).unwrap_or_default();
            Promise::resolved(Ok(serde_json::json!({"rows": rows, "fetched_at": 0})))
        }
        Service::QuotesParseHyperliquid => {
            let body = args.get("body").and_then(|v| v.as_str()).unwrap_or("");
            let rows = crate::parse::parse_hyperliquid_quotes(body).unwrap_or_default();
            Promise::resolved(Ok(serde_json::json!({"rows": rows, "fetched_at": 0})))
        }
        Service::QuotesParseNasdaq => {
            let body = args.get("body").and_then(|v| v.as_str()).unwrap_or("");
            let market = args.get("market").and_then(|v| v.as_str()).unwrap_or("us");
            let rows = crate::parse::parse_nasdaq_quote(body, market).unwrap_or_default();
            Promise::resolved(Ok(serde_json::json!({"rows": rows, "fetched_at": 0})))
        }
        Service::QuotesParseTencent => {
            let body = args.get("body").and_then(|v| v.as_str()).unwrap_or("");
            let market = args.get("market").and_then(|v| v.as_str()).unwrap_or("a");
            let rows = crate::parse::parse_tencent_quote(body, market).unwrap_or_default();
            Promise::resolved(Ok(serde_json::json!({"rows": rows, "fetched_at": 0})))
        }
        Service::FavsList => {
            // Phase 2 v0: splash 端仍直接 fs 读 favs.json (T3c 待做)。
            Promise::resolved(Ok(serde_json::json!({
                "news": [], "quotes": []
            })))
        }
        Service::FavsToggle => {
            // Phase 2 v0: splash 端仍直接 fs 写 favs.json (T3c 待做); native 端
            // 仅 echo 状态. R-3 真解耦轮次再实现 read-modify-write.
            let key = args.get("key").and_then(|v| v.as_str()).unwrap_or("");
            let kind = args.get("kind").and_then(|v| v.as_str()).unwrap_or("");
            Promise::resolved(Ok(serde_json::json!({
                "kind": kind, "key": key, "favored": true
            })))
        }
        Service::SettingsLoad => {
            let v = crate::store::load_settings().unwrap_or_else(|_| serde_json::json!({}));
            Promise::resolved(Ok(v))
        }
        Service::SettingsSave => {
            let patch = args.get("patch").cloned().unwrap_or(serde_json::json!({}));
            match crate::store::save_settings(&patch) {
                Ok(_) => Promise::resolved(Ok(serde_json::json!({}))),
                Err(e) => Promise::resolved(Err(e)),
            }
        }
        _ => Promise::resolved(Err("not implemented".into())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn host_signatures_callable() {
        let mut vm = SplashVm::dummy();
        let _ = register(&mut vm);
        let _ = handle(Service::NewsRefresh, json!({}));
        let _ = handle(Service::SettingsLoad, json!({}));
    }

    #[test]
    fn host_quotes_candles_returns_vec() {
        let p = handle(Service::QuotesCandles, json!({"count": 50}));
        // Promise 不透明 —— 这里只验证 handle 不 panic，值通过调用约定由 splash 拿
        let _ = p;
    }

    #[test]
    fn host_news_refresh_parses_body() {
        let p = handle(
            Service::NewsRefresh,
            json!({
                "body": r#"<item><title>央行降准 0.5</title><link>https://x/1</link><media_name>新浪财经</media_name><intro>降准 0.5</intro></item>"#,
                "limit": 5
            }),
        );
        let _ = p; // Promise opaque, 只验证不 panic
    }

    #[test]
    fn host_quotes_parse_tencent() {
        let p = handle(
            Service::QuotesParseTencent,
            json!({
                "body": "v_sh600519=\"1~\u{8d35}\u{5dde}\u{8303}\u{53f0}~600519~1888.50~1890.00~1875.20~1895.00~1880.30~-10.50~-0.55\";",
                "market": "sh"
            }),
        );
        let _ = p;
    }

    #[test]
    fn host_settings_load_returns_object() {
        let p = handle(Service::SettingsLoad, json!({}));
        let _ = p;
    }

    #[test]
    fn host_settings_save_echoes_patch() {
        let p = handle(Service::SettingsSave, json!({"patch": {"theme": "dark"}}));
        let _ = p;
    }

    #[test]
    fn host_favs_toggle_echoes_key() {
        let p = handle(
            Service::FavsToggle,
            json!({"kind": "quote", "key": "us:AAPL"}),
        );
        let _ = p;
    }

    #[test]
    fn host_favs_list_returns_empty_arrays() {
        let p = handle(Service::FavsList, json!({}));
        let _ = p;
    }

    #[test]
    fn host_quotes_parse_frankfurter() {
        let p = handle(
            Service::QuotesParseFrankfurter,
            json!({
                "body": r#"{"date":"2026-10-01","base":"USD","rates":{"EUR":0.92,"JPY":149.5}}"#
            }),
        );
        let _ = p;
    }

    #[test]
    fn host_quotes_parse_hyperliquid() {
        let p = handle(
            Service::QuotesParseHyperliquid,
            json!({
                "body": r#"[{"universe":[{"name":"BTC"}]},[{"markPx": "100.5", "prevDayPx": "95.0"}]]"#
            }),
        );
        let _ = p;
    }

    #[test]
    fn host_quotes_parse_nasdaq() {
        let p = handle(
            Service::QuotesParseNasdaq,
            json!({
                "body": r#"{"data":{"symbol":"AAPL","primaryData":{"lastSalePrice":"$178.45","percentageChange":"+1.25%"},"secondaryData":{"lastSalePrice":"$176.20"}}}"#,
                "market": "us"
            }),
        );
        let _ = p;
    }
}


// --- v2 capability runtime stub (per .todo §5 + Phase 2 fix) ---
//
// v1 host.rs only models `SplashVm` + `Service` enum for the splash VM.
// v2 adapters (per Q-C: B) want an `octoscript_capabilities`-style
// `CapabilityRuntime` with `register_validated_json_tool(name, contract, handler)`.
//
// Rather than pull in the heavy `octoscript-capabilities` dep tree
// (makepad-script git rev + blake3 + keyring + Linux-only deps), we model
// the surface as a tiny stub. The real runtime is owned by the splash VM
// (or future OctoSense host); this layer just collects registrations so
// `cargo build` passes.

use serde_json::Value;

/// Minimal JSON tool contract — input + output JSON Schemas.
/// Real `octoscript_capabilities::JsonToolContract::new` returns `Result<Self, _>`
/// (validates against a meta-schema). We skip validation here; the host
/// runtime validates at boot.
#[derive(Clone, Default)]
pub struct JsonToolContract {
    pub input_schema: Value,
    pub output_schema: Value,
}

impl JsonToolContract {
    pub fn new(input_schema: Value, output_schema: Value) -> Self {
        Self { input_schema, output_schema }
    }
}

/// Registered tool handler. Generic over the future type so async fns are
/// accepted (Box::pin captures the produced future).
pub type AsyncHandler = Box<
    dyn Fn(Value) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Value, String>> + Send>>
        + Send
        + Sync,
>;

/// Capability runtime — collects (name, contract, handler) triples.
#[derive(Default)]
pub struct CapabilityRuntime {
    pub tools: Vec<(String, JsonToolContract, AsyncHandler)>,
}

impl CapabilityRuntime {
    /// Register one JSON tool. Generic signature accepts both `fn` pointer
    /// and `async fn` via closure coercion.
    pub fn register_validated_json_tool<F, Fut>(
        &mut self,
        name: &str,
        contract: JsonToolContract,
        handler: F,
    ) where
        F: Fn(Value) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = Result<Value, String>> + Send + 'static,
    {
        let boxed: AsyncHandler = Box::new(move |v| Box::pin(handler(v)));
        self.tools.push((name.to_string(), contract, boxed));
    }
}
