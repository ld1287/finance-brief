//! TTL cache — in-memory + on-disk. Files under CWD as `cache_<key>.json`,
//! payload `{ "value": <T>, "expires_at": <unix_secs> }`.
//!
//! MVP 阶段，无锁冲突（splash runtime 调度），CWD 隔离保证不串 cache。

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{LazyLock, Mutex};

use serde::{de::DeserializeOwned, Serialize};

#[derive(Clone)]
struct CacheEntry {
    raw: String,
    expires_at: u64,
}

static IN_MEM: LazyLock<Mutex<HashMap<String, CacheEntry>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

#[derive(serde::Serialize, serde::Deserialize)]
struct CacheEnvelope {
    value: String,
    expires_at: u64,
}

/// Hit & not expired → `Some(T)`; otherwise `Ok(None)`.
/// Disk corruption / deserialization failure → `Ok(None)` (cache miss must not block upstream).
pub fn load_cached<V: DeserializeOwned>(key: &str, now_secs: u64) -> Result<Option<V>, String> {
    if let Some(entry) = IN_MEM.lock().unwrap().get(key).cloned() {
        if entry.expires_at > now_secs {
            return Ok(serde_json::from_str(&entry.raw).ok());
        }
    }
    let path = path_for(key);
    let body = match std::fs::read_to_string(&path) {
        Ok(s) => s,
        Err(_) => return Ok(None),
    };
    let envelope: CacheEnvelope = match serde_json::from_str(&body) {
        Ok(v) => v,
        Err(_) => return Ok(None),
    };
    IN_MEM.lock().unwrap().insert(
        key.to_string(),
        CacheEntry {
            raw: envelope.value.clone(),
            expires_at: envelope.expires_at,
        },
    );
    if envelope.expires_at > now_secs {
        Ok(serde_json::from_str(&envelope.value).ok())
    } else {
        Ok(None)
    }
}

/// Persist `value` with `ttl_secs` (relative to `now_secs`).
pub fn store_cached<V: Serialize>(
    key: &str,
    value: &V,
    ttl_secs: u64,
    now_secs: u64,
) -> Result<(), String> {
    let raw = serde_json::to_string(value).map_err(|e| e.to_string())?;
    let envelope = CacheEnvelope {
        value: raw.clone(),
        expires_at: now_secs.saturating_add(ttl_secs),
    };
    let body = serde_json::to_string_pretty(&envelope).map_err(|e| e.to_string())?;
    std::fs::write(path_for(key), body).map_err(|e| e.to_string())?;
    IN_MEM.lock().unwrap().insert(
        key.to_string(),
        CacheEntry {
            raw,
            expires_at: envelope.expires_at,
        },
    );
    Ok(())
}

fn path_for(key: &str) -> PathBuf {
    let safe: String = key
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect();
    std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join(format!("cache_{}.json", safe))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn cleanup(key: &str) {
        let _ = std::fs::remove_file(path_for(key));
    }

    #[test]
    fn load_cached_miss_returns_none() {
        let key = "unit-test-miss-2026";
        cleanup(key);
        let v: Option<serde_json::Value> = load_cached(key, 1_700_000_000).unwrap();
        assert!(v.is_none());
    }

    #[test]
    fn store_then_load_within_ttl_hits() {
        let key = "unit-test-hit-2026";
        cleanup(key);
        store_cached(key, &json!({"v": 1}), 60, 1_700_000_000).unwrap();
        let v: serde_json::Value = load_cached(key, 1_700_000_000 + 30).unwrap().unwrap();
        assert_eq!(v["v"], 1);
        cleanup(key);
    }

    #[test]
    fn store_then_load_after_ttl_misses() {
        let key = "unit-test-expire-2026";
        cleanup(key);
        store_cached(key, &json!({"v": 1}), 60, 1_700_000_000).unwrap();
        let v: Option<serde_json::Value> = load_cached(key, 1_700_000_000 + 120).unwrap();
        assert!(v.is_none());
        cleanup(key);
    }
}
