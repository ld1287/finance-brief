//! Tencent Finance quote feed source.
//!
//! As of G3, the splash runtime still owns the HTTP request (`net.http_request`)
//! and passes the raw body to native via a follow-up `host.call("quotes.parse_tencent",
//! {market, body})` once the bridge is wired. To avoid a double-fetch (native HTTP
//! plus splash HTTP) during the migration, this `fetch` is intentionally a
//! thin shim that parses an already-fetched body via
//! `crate::parse::parse_tencent_quote`. Real native-only fetching lands in G4+.

/// Tencent qt.gtimg.cn base. Examples:
///   https://qt.gtimg.cn/q=sh600519,sh000001
///   https://qt.gtimg.cn/q=usAAPL,usTSLA
pub const TENCENT_BASE: &str = "https://qt.gtimg.cn/q=";

/// Synchronous parse shim. Args: `{ "market": "sh"|"us", "body": "<raw csv>" }`.
pub fn fetch(args: &serde_json::Value) -> Result<serde_json::Value, String> {
    let market = args.get("market").and_then(|v| v.as_str()).unwrap_or("sh");
    let body = args.get("body").and_then(|v| v.as_str()).unwrap_or("");
    let rows = crate::parse::parse_tencent_quote(body, market)?;
    serde_json::to_value(&rows).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tencent_fetch_market_sh() {
        let args = serde_json::json!({
            "market": "sh",
            "body": r#"v_sh600519="1~贵州茅台~600519~1888.50~1890.00~1875.20~1895.10~1880.30~~~-10.50~-0.55";"#
        });
        let v = fetch(&args).unwrap();
        let rows: Vec<crate::model::QuoteRow> = serde_json::from_value(v).unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].name, "贵州茅台");
        assert!((rows[0].price - 1888.50).abs() < 0.01);
    }

    #[test]
    fn tencent_fetch_default_market() {
        // 缺省 market 时也走解析路径，确保 shim 不 panic
        let args = serde_json::json!({
            "body": r#"v_sh600519="1~茅台~A";"#
        });
        let v = fetch(&args).unwrap();
        let rows: Vec<crate::model::QuoteRow> = serde_json::from_value(v).unwrap();
        assert_eq!(rows.len(), 1);
    }
}
