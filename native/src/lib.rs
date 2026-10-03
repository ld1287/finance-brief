//! finance-brief-scaffold: data layer for the Splash UI.
//! Network fetch, JSON/text parse, SQLite-free cache, favourites, settings.
//! Wired up to splash via host.call / host.fetch (see docs/ARCHITECTURE.md §4).

pub mod adapters;
pub mod host;
pub mod model;
pub mod parse;
pub mod sources;
pub mod store;
pub mod synth;

pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

/// v2 entry point: wire every adapter into a single `CapabilityRuntime`.
/// The runtime itself is owned by the splash VM host; this fn only fills it.
/// Call from `host.register` once per process boot, before any script eval.
pub fn register_all(rt: &mut crate::host::CapabilityRuntime) {
    adapters::register_all_adapter(rt);
}

#[cfg(test)]
mod tests {
    #[test]
    fn scaffold_compiles() {
        assert_eq!(super::version(), "0.1.0");
    }
}
