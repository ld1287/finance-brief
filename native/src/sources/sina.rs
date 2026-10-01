//! Sina (新浪财经) news feed source.
//!
//! As of G2, the splash runtime still owns the HTTP request (`net.http_request`)
//! and passes the raw body to native via a follow-up `host.call("news.parse",
//! {body})` once the bridge is wired. To avoid a double-fetch (native HTTP plus
//! splash HTTP) during the migration, this `fetch` is intentionally a no-op
//! stub that tells callers to route through the splash HTTP path and call
//! `crate::parse::parse_sina_news` directly. Real native-only fetching lands
//! in G3+.

/// Sina RSS feed URL (placeholder — splash currently uses its own URL).
pub const SINA_NEWS_URL: &str = "https://feed.mix.sina.com.cn/api/rolls/list?channel=finance&num=20";

/// Stub fetcher. Splash does the HTTP today; native parses the body.
pub async fn fetch(_args: &serde_json::Value) -> Result<serde_json::Value, String> {
    Err("call parse_sina_news directly from splash".into())
}
