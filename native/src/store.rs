//! Persistence layer — settings + cache.
//! File-backed JSON store under the current working directory.

use std::fs;
use std::path::PathBuf;

/// Load persisted user settings. Missing or malformed file → `Ok(json!({}))`.
pub fn load_settings() -> Result<serde_json::Value, String> {
    let path = settings_path();
    let body = match fs::read_to_string(&path) {
        Ok(s) => s,
        Err(_) => return Ok(serde_json::json!({})),
    };
    serde_json::from_str(&body).map_err(|e| e.to_string())
}

/// Save user settings as pretty JSON.
pub fn save_settings(v: &serde_json::Value) -> Result<(), String> {
    let path = settings_path();
    let body = serde_json::to_string_pretty(v).map_err(|e| e.to_string())?;
    fs::write(&path, body).map_err(|e| e.to_string())
}

/// Load a named cache entry. Missing → `Ok(None)`; malformed → `Err`.
pub fn load_cache(name: &str) -> Result<Option<serde_json::Value>, String> {
    let path = cache_path(name);
    let body = match fs::read_to_string(&path) {
        Ok(s) => s,
        Err(_) => return Ok(None),
    };
    serde_json::from_str(&body)
        .map(Some)
        .map_err(|e| e.to_string())
}

/// Save a named cache entry as pretty JSON.
pub fn save_cache(name: &str, v: &serde_json::Value) -> Result<(), String> {
    let path = cache_path(name);
    let body = serde_json::to_string_pretty(v).map_err(|e| e.to_string())?;
    fs::write(&path, body).map_err(|e| e.to_string())
}

fn settings_path() -> PathBuf {
    std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join("settings.json")
}

fn cache_path(name: &str) -> PathBuf {
    std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join(format!("cache_{}.json", name))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn store_round_trip_settings() {
        use std::fs;
        let path = std::env::current_dir().unwrap().join("settings.json");
        let _ = fs::remove_file(&path);
        let v = json!({"theme": "dark", "refresh_sec": 60});
        save_settings(&v).unwrap();
        let loaded = load_settings().unwrap();
        assert_eq!(loaded["theme"], "dark");
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn store_load_settings_missing_returns_empty() {
        use std::fs;
        let path = std::env::current_dir().unwrap().join("settings.json");
        let _ = fs::remove_file(&path);
        let v = load_settings().unwrap();
        assert_eq!(v, json!({}));
    }

    #[test]
    fn store_round_trip_cache() {
        use std::fs;
        let name = "T3a-test";
        let path = std::env::current_dir()
            .unwrap()
            .join(format!("cache_{}.json", name));
        let _ = fs::remove_file(&path);
        let v = json!({"rows": [1, 2, 3]});
        save_cache(name, &v).unwrap();
        let loaded = load_cache(name).unwrap().unwrap();
        assert_eq!(loaded["rows"][2], 3);
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn store_load_cache_missing_returns_none() {
        let loaded = load_cache("T3a-nonexistent-cache").unwrap();
        assert!(loaded.is_none());
    }
}
