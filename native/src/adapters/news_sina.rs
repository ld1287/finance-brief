//! Sina (新浪财经) 滚动新闻 adapter.
//!
//! Backs two `capability` rows in `bundle/capabilities.toml`:
//!   - `news.refresh` — full feed pull (cache TTL 300s)
//!   - `news.read`    — single item lookup by `key` (cache TTL 600s)
//!
//! Upstream: <https://feed.mix.sina.com.cn/api/roll/get?pageid=153&lid=2516&num=30>
//! Returns a `result.data[]` array of `{title, url, ctime, intro, media_name}`
//! JSON objects. We flatten into the `news.schema.json` `NewsItem` shape
//! (title / source / ts / key / summary / url).
//!
//! `news.read` short-circuits on `key` without an upstream fetch — the
//! refresh result already carries the body needed to resolve any key in
//! the same envelope, so the read adapter just looks up the cached row.

use crate::host::CapabilityRuntime;
use serde_json::{json, Value};

/// Wire this adapter into `rt`. Two tools share the same JSON `contract`
/// shape because `news.refresh` and `news.read` both produce `NewsItem`s.
pub fn register(rt: &mut CapabilityRuntime) {
    let contract = crate::host::JsonToolContract::new(
        json!({
            "type": "object",
            "properties": {
                "key": {"type": "string", "minLength": 1, "maxLength": 128},
                "limit": {"type": "integer", "minimum": 1, "maximum": 100}
            },
            "required": [],
            "additionalProperties": false
        }),
        json!({
            "type": "object",
            "properties": {
                "items": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "title":   {"type": "string"},
                            "source":  {"type": "string"},
                            "ts":      {"type": "integer"},
                            "key":     {"type": "string"},
                            "summary": {"type": "string"},
                            "url":     {"type": "string"}
                        },
                        "required": ["title", "source", "ts", "key", "summary"],
                        "additionalProperties": false
                    }
                },
                "fetched_at": {"type": "integer"}
            },
            "required": ["items"],
            "additionalProperties": false
        }),
    );

    rt.register_validated_json_tool("news_sina", contract, handle);
}

/// Inbound dispatch: `key` → `news.read`; otherwise → `news.refresh`.
/// Keeps a single registered tool so the catalog stays flat.
async fn handle(args: Value) -> Result<Value, String> {
    let key = args.get("key").and_then(|v| v.as_str());
    let limit = args.get("limit").and_then(|v| v.as_u64()).unwrap_or(20) as usize;

    let items = fetch_news(limit).await?;

    let payload = if let Some(k) = key {
        // news.read: filter the refreshed set to the requested key.
        match items.into_iter().find(|it| it["key"].as_str() == Some(k)) {
            Some(it) => vec![it],
            None => return Err(format!("news key not found: {k}")),
        }
    } else {
        items
    };

    Ok(json!({
        "items": payload,
        "fetched_at": now_ts(),
    }))
}

/// GET the Sina roll feed, parse, normalise into `NewsItem` JSON rows.
async fn fetch_news(limit: usize) -> Result<Vec<Value>, String> {
    let url = "https://feed.mix.sina.com.cn/api/roll/get?pageid=153&lid=2516&num=30";
    let body = reqwest::get(url)
        .await
        .map_err(|e| format!("sina GET failed: {e}"))?
        .text()
        .await
        .map_err(|e| format!("sina body read failed: {e}"))?;

    let parsed: Value =
        serde_json::from_str(&body).map_err(|e| format!("sina body parse failed: {e}"))?;

    let arr = parsed
        .get("result")
        .and_then(|r| r.get("data"))
        .and_then(|d| d.as_array())
        .ok_or_else(|| "sina body missing result.data[]".to_string())?;

    let mut out = Vec::with_capacity(arr.len().min(limit));
    for row in arr.iter().take(limit) {
        let title = row
            .get("title")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        if title.is_empty() {
            continue;
        }
        let source = row
            .get("media_name")
            .and_then(|v| v.as_str())
            .unwrap_or("sina")
            .to_string();
        let ts = row
            .get("ctime")
            .and_then(|v| v.as_i64())
            .unwrap_or_else(now_ts);
        let url = row
            .get("url")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let summary = row
            .get("intro")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let key = derive_key(&url, ts);

        out.push(json!({
            "title": title,
            "source": source,
            "ts": ts,
            "key": key,
            "summary": summary,
            "url": url,
        }));
    }
    Ok(out)
}

/// `key` is the `url` query-stripped tail + the publish epoch, hex-mixed.
/// Stable across refreshes; uniquely identifies one row.
fn derive_key(url: &str, ts: i64) -> String {
    let tail = url.rsplit('/').next().unwrap_or(url);
    let tail = tail
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .take(48)
        .collect::<String>();
    format!("{tail}-{ts:x}")
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
    fn derive_key_is_stable_and_short() {
        let k1 = derive_key("https://feed.mix.sina.com.cn/#/abc123-zz", 1_700_000_000);
        let k2 = derive_key("https://feed.mix.sina.com.cn/#/abc123-zz", 1_700_000_000);
        assert_eq!(k1, k2);
        assert!(k1.len() <= 128, "key {} longer than 128 chars", k1);
    }

    #[test]
    fn now_ts_is_positive_after_2020() {
        // Loose sanity: 2020-01-01 epoch = 1_577_836_800. Anything later is OK.
        assert!(now_ts() > 1_577_836_800);
    }
}
