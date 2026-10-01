//! finance-brief-scaffold: data layer for the Splash UI.
//! Network fetch, JSON/text parse, SQLite-free cache, favourites, settings.
//! Wired up to splash via host.call / host.fetch (see docs/ARCHITECTURE.md §4).

pub mod model;
pub mod parse;
pub mod synth;
pub mod store;
pub mod host;
pub mod sources;

pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[cfg(test)]
mod tests {
    #[test]
    fn scaffold_compiles() {
        assert_eq!(super::version(), "0.1.0");
    }
}
