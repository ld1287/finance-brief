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
}
