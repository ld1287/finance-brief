//! Persistence layer — settings + cache.
//! All functions are TODO placeholders in this scaffold; real IO (file-backed
//! JSON, no SQLite) will be added in a follow-up commit.

/// Load persisted user settings.
pub fn load_settings() -> Result<serde_json::Value, String> {
    // TODO: read settings JSON from disk
    Err("not implemented".into())
}

/// Save user settings.
pub fn save_settings(_v: &serde_json::Value) -> Result<(), String> {
    // TODO: write settings JSON to disk
    Err("not implemented".into())
}

/// Load a named cache entry (e.g. `news.refresh`, `quotes.snapshot`).
pub fn load_cache(_name: &str) -> Result<Option<serde_json::Value>, String> {
    // TODO: read named cache blob from disk
    Err("not implemented".into())
}

/// Save a named cache entry.
pub fn save_cache(_name: &str, _v: &serde_json::Value) -> Result<(), String> {
    // TODO: write named cache blob to disk
    Err("not implemented".into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn store_signatures_callable() {
        // Just verify the function signatures are callable with placeholder bodies.
        let _ = load_settings();
        let _ = save_settings(&json!({}));
        let _ = load_cache("news.refresh");
        let _ = save_cache("news.refresh", &json!({}));
    }
}
