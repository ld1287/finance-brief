//! finance-brief-scaffold: data layer for the Splash UI.
//! Network fetch, JSON/text parse, SQLite-free cache, favourites, settings.
//! Wired up to splash via host.call / host.fetch (see docs/ARCHITECTURE.md §4).

pub mod adapters;
pub mod cache;
pub mod host;
pub mod model;
pub mod parse;
pub mod sources;
pub mod store;
pub mod synth;

/// Initialise the global tracing subscriber for the finance-brief crate.
///
/// Reads `RUST_LOG` (via `EnvFilter::try_from_default_env`) and falls back to
/// the `info` level when the env var is unset or invalid. Safe to call more
/// than once — `try_init` is a no-op on the second invocation rather than
/// panicking, so each test binary can boot its own subscriber.
pub fn init_tracing() {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .with_target(false)
        .try_init();
}

pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

/// v2 entry point: wire every adapter into a single `CapabilityRuntime`.
/// The runtime itself is owned by the splash VM host; this fn only fills it.
/// Call from `host.register` once per process boot, before any script eval.
pub fn register_all(rt: &mut crate::host::CapabilityRuntime) {
    init_tracing();
    tracing::info!("finance-brief register_all begin");
    adapters::register_all_adapter(rt);
}

#[cfg(test)]
mod tests {
    #[test]
    fn scaffold_compiles() {
        assert_eq!(super::version(), "0.1.0");
    }
}
