//! Parse layer — turns raw text/JSON from sources into our typed models.
//! Each source has its own parser. Real per-source logic lands here as feeds
//! move from splash into native (G2+: sina news).

use crate::model::{Candle, NewsRow, QuoteRow};

/// Parse a Sina news response body.
///
/// RSS / XML shape we expect per `<item>`:
/// ```xml
/// <item>
///   <title>...</title>
///   <link>...</link>
///   <pubDate>2026-10-01 12:00</pubDate>
///   <media_name>新浪财经</media_name>
///   <intro>...</intro>
/// </item>
/// ```
///
/// We intentionally avoid pulling a real XML parser (no roxmltree) — splash's
/// fallback had the same constraint and string-scanning is enough for the
/// well-formed fragments the feed returns.
pub fn parse_sina_news(body: &str) -> Result<Vec<NewsRow>, String> {
    let mut out = Vec::new();
    let mut rest = body;
    let mut idx = 0usize;
    while let Some(start) = rest.find("<item>") {
        let after = &rest[start + "<item>".len()..];
        let end = after.find("</item>").unwrap_or(after.len());
        let item = &after[..end];
        let title = extract_tag(item, "title").unwrap_or_default();
        let link = extract_tag(item, "link").unwrap_or_default();
        let _pub_date = extract_tag(item, "pubDate").unwrap_or_default();
        let source = extract_tag(item, "media_name").unwrap_or_default();
        let intro = extract_tag(item, "intro").unwrap_or_default();
        if !title.is_empty() {
            out.push(NewsRow {
                id: format!("news:{}", idx),
                title,
                summary: intro,
                source,
                url: link,
                ts: 0,
                body: None,
            });
            idx += 1;
        }
        rest = &after[end..];
    }
    Ok(out)
}

fn extract_tag(item: &str, tag: &str) -> Option<String> {
    let open = format!("<{}>", tag);
    let close = format!("</{}>", tag);
    let s = item.find(&open)? + open.len();
    let e = item[s..].find(&close)? + s;
    let raw = &item[s..e];
    Some(unescape_entities(raw))
}

/// Minimal XML entity unescape — enough for Sina RSS title/intro fields.
fn unescape_entities(raw: &str) -> String {
    raw.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
}

/// Parse a Tencent finance response body.
pub fn parse_tencent_news(_body: &str) -> Result<Vec<NewsRow>, String> {
    // TODO: real Tencent news parser
    Err("not implemented".into())
}

/// Tencent qt.gtimg.cn 行情快照解析（CSV-like, GBK 名称但报价数字是 ASCII）。
/// 输入形如: `v_sh600519="1~贵州茅台~600519~1888.50~1890.00~...";`
/// 实际分隔符是 `~`；字段 1 是名称, 字段 3 是当前价, 字段 32 是涨跌幅。
/// MVP 容忍短样本（>= 3 字段即可拿到名称），价格字段缺失时为 0.0。
pub fn parse_tencent_quote(
    body: &str,
    market: &str,
) -> Result<Vec<crate::model::QuoteRow>, String> {
    let mut out = Vec::new();
    let mut rest = body;
    while let Some(start) = rest.find("=\"") {
        // 跳过 `="` 本身，定位到内容起始处
        let after = &rest[start + 2..];
        let Some(end) = after.find('"') else { break };
        let content = &after[..end];
        // 切分 fields（分隔符 ~）
        let fields: Vec<&str> = content.splitn(60, '~').collect();
        if fields.len() < 3 {
            rest = &after[end..];
            continue;
        }
        // fields[1] = name, fields[3] = price, fields[32] = pct
        let id = format!("{}:{}", market, fields[0].trim_start_matches("v_"));
        let name = fields.get(1).copied().unwrap_or("").to_string();
        let price: f64 = fields
            .get(3)
            .and_then(|s| s.replace(',', "").parse().ok())
            .unwrap_or(0.0);
        let pct: f64 = fields.get(32).and_then(|s| s.parse().ok()).unwrap_or(0.0);
        out.push(crate::model::QuoteRow {
            id,
            code: name.clone(),
            name,
            market: market.to_string(),
            price,
            pct,
            ts: 0,
        });
        rest = &after[end..];
    }
    Ok(out)
}

/// Parse a Hyperliquid `metaAndAssetCtxs` response body.
///
/// Wire shape: `POST https://api.hyperliquid.xyz/info` with body
/// `{"type":"metaAndAssetCtxs"}` returns a 2-element array:
///   `[0]` = meta object with a `universe` array of `{name, ...}`
///   `[1]` = assetCtxs array of `{markPx, prevDayPx, ...}` (same length as universe)
///
/// `markPx` and `prevDayPx` are usually strings but the API may also send
/// numbers, so we accept both forms. Returns up to 12 non-empty names.
pub fn parse_hyperliquid_quotes(body: &str) -> Result<Vec<QuoteRow>, String> {
    let v: serde_json::Value = match serde_json::from_str(body) {
        Ok(v) => v,
        Err(_) => return Ok(Vec::new()),
    };
    let arr = match v.as_array() {
        Some(a) => a,
        None => return Ok(Vec::new()),
    };
    if arr.len() < 2 {
        return Ok(Vec::new());
    }
    let universe = arr[0].get("universe").and_then(|u| u.as_array());
    let ctxs = arr[1].as_array();
    let (Some(universe), Some(ctxs)) = (universe, ctxs) else {
        return Ok(Vec::new());
    };
    if ctxs.is_empty() {
        return Ok(Vec::new());
    }
    let mut out = Vec::new();
    let n = 12.min(universe.len()).min(ctxs.len());
    for i in 0..n {
        let name = universe[i]
            .get("name")
            .and_then(|n| n.as_str())
            .unwrap_or("")
            .to_string();
        if name.is_empty() {
            continue;
        }
        let mark = parse_number(ctxs[i].get("markPx")).unwrap_or(0.0);
        let prev = parse_number(ctxs[i].get("prevDayPx")).unwrap_or(0.0);
        let pct = if prev > 0.0 {
            (mark - prev) / prev * 100.0
        } else {
            0.0
        };
        out.push(QuoteRow {
            id: format!("crypto:{}", name),
            code: name.clone(),
            name,
            market: "crypto".to_string(),
            price: mark,
            pct,
            ts: 0,
        });
    }
    Ok(out)
}

/// Coerce a JSON value (string or number) into f64. Hyperliquid typically
/// sends numeric fields as strings, but we accept both forms defensively.
fn parse_number(v: Option<&serde_json::Value>) -> Option<f64> {
    match v {
        Some(serde_json::Value::Number(n)) => n.as_f64(),
        Some(serde_json::Value::String(s)) => s.parse::<f64>().ok(),
        _ => None,
    }
}

/// Parse a Frankfurter (FX) response body.
///
/// Wire shape:
/// `GET https://api.frankfurter.dev/v1/latest?base=USD&symbols=CNY,EUR,JPY,GBP,HKD`
/// returns `{"date":"2026-09-30","base":"USD","rates":{"CNY":7.12,"EUR":0.92,...}}`.
///
/// `rates` is a JSON object whose values are usually JSON numbers but the spec
/// is permissive about number/string. Returns one `QuoteRow` per currency
/// (id=`fx:{KEY}` uppercased, code=raw key, name=`USD/{KEY}`, market=`fx`).
/// `pct` is `0.0` (Frankfurter has no historical pct in this endpoint) and
/// `ts` is `0`. `base` is ignored — we always treat USD as the base.
pub fn parse_frankfurter(body: &str) -> Result<Vec<QuoteRow>, String> {
    let v: serde_json::Value = match serde_json::from_str(body) {
        Ok(v) => v,
        Err(_) => return Ok(Vec::new()),
    };
    let rates = match v.get("rates").and_then(|r| r.as_object()) {
        Some(r) => r,
        None => return Ok(Vec::new()),
    };
    let mut out = Vec::new();
    for (key, val) in rates.iter() {
        let Some(rate) = parse_number(Some(val)) else {
            continue;
        };
        out.push(QuoteRow {
            id: format!("fx:{}", key.to_uppercase()),
            code: key.clone(),
            name: format!("USD/{}", key),
            market: "fx".to_string(),
            price: rate,
            pct: 0.0,
            ts: 0,
        });
    }
    Ok(out)
}

/// Parse a Nasdaq `/api/quote/{tick}/info?assetclass=stocks` response body.
///
/// Wire shape (real API):
/// ```json
/// {"data":{"symbol":"AAPL","primaryData":{"lastSalePrice":"$178.45","percentageChange":"+1.25%","marketStatus":"Open"},"secondaryData":{"lastSalePrice":"$176.20"}}}
/// ```
/// Real responses also contain `\/` (escaped slash) sequences; we strip those
/// before `serde_json` parses the body just to be defensive. `primaryData` is
/// populated when trading is open, so its absence (e.g. weekend) yields an empty
/// vec. `price <= 0` or unparseable price also yields an empty vec — callers
/// should treat that as "no fresh quote".
pub fn parse_nasdaq_quote(body: &str, market: &str) -> Result<Vec<QuoteRow>, String> {
    let body = body.replace("\\/", "/");
    let v: serde_json::Value = match serde_json::from_str(&body) {
        Ok(v) => v,
        Err(e) => return Err(format!("nasdaq quote: invalid json: {e}")),
    };
    let Some(data) = v.get("data") else {
        return Ok(Vec::new());
    };
    let symbol = data
        .get("symbol")
        .and_then(|s| s.as_str())
        .unwrap_or("")
        .to_string();
    let primary = data.get("primaryData");
    let last_sale_raw = primary
        .and_then(|p| p.get("lastSalePrice"))
        .and_then(|p| p.as_str())
        .unwrap_or("");
    let pct_raw = primary
        .and_then(|p| p.get("percentageChange"))
        .and_then(|p| p.as_str())
        .unwrap_or("");
    let _market_status = primary
        .and_then(|p| p.get("marketStatus"))
        .and_then(|p| p.as_str())
        .unwrap_or("");
    let _secondary_last_sale = data
        .get("secondaryData")
        .and_then(|s| s.get("lastSalePrice"))
        .and_then(|s| s.as_str());
    let price: f64 = match strip_dollar(last_sale_raw).parse() {
        Ok(p) if p > 0.0 => p,
        _ => return Ok(Vec::new()),
    };
    let pct: f64 = pct_raw
        .replace('%', "")
        .trim()
        .parse::<f64>()
        .unwrap_or(0.0);
    Ok(vec![QuoteRow {
        id: format!("{}:{}", market, symbol),
        code: symbol.clone(),
        name: symbol,
        market: market.to_string(),
        price,
        pct,
        ts: 0,
    }])
}

fn strip_dollar(s: &str) -> String {
    s.trim().trim_start_matches('$').to_string()
}

/// Parse a Nasdaq response body into OHLC candles.
pub fn parse_nasdaq_candles(_body: &str) -> Result<Vec<Candle>, String> {
    // TODO: real Nasdaq candle parser
    Err("not implemented".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_signatures_callable() {
        // Just verify the function signatures are callable with placeholder bodies.
        let _ = parse_sina_news("");
        let _ = parse_tencent_news("");
        let _ = parse_hyperliquid_quotes("");
        let _ = parse_frankfurter("");
        let _ = parse_nasdaq_candles("");
    }

    #[test]
    fn parse_sina_news_sample() {
        let body = r#"<rss><channel><item><title>央行降准 0.5 个百分点</title><link>https://example.com/1</link><pubDate>2026-10-01 12:00</pubDate><media_name>新浪财经</media_name><intro>央行宣布降准 0.5 个百分点</intro></item></channel></rss>"#;
        let rows = parse_sina_news(body).unwrap();
        assert_eq!(rows.len(), 1);
        assert!(rows[0].title.contains("降准"));
        assert_eq!(rows[0].url, "https://example.com/1");
        assert_eq!(rows[0].source, "新浪财经");
        assert_eq!(rows[0].summary, "央行宣布降准 0.5 个百分点");
    }

    #[test]
    fn parse_sina_news_empty() {
        let rows = parse_sina_news("").unwrap();
        assert_eq!(rows.len(), 0);
    }

    #[test]
    fn parse_sina_news_multi() {
        let body = r#"<item><title>A</title></item><item><title>B</title></item>"#;
        let rows = parse_sina_news(body).unwrap();
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].title, "A");
        assert_eq!(rows[1].title, "B");
        assert_eq!(rows[0].id, "news:0");
        assert_eq!(rows[1].id, "news:1");
    }

    #[test]
    fn parse_tencent_quote_sample() {
        let body = r#"v_sh600519="1~贵州茅台~600519~1888.50~1890.00~1875.20~1895.10~1880.30~~~-10.50~-0.55";"#;
        let rows = parse_tencent_quote(body, "sh").unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].name, "贵州茅台");
        assert!((rows[0].price - 1888.50).abs() < 0.01);
    }

    #[test]
    fn parse_tencent_quote_empty() {
        let rows = parse_tencent_quote("", "sh").unwrap();
        assert_eq!(rows.len(), 0);
    }

    #[test]
    fn parse_tencent_quote_multiple() {
        let body = r#"v_sh600519="1~茅台~A";v_sh000001="1~平安银行~B";"#;
        let rows = parse_tencent_quote(body, "sh").unwrap();
        assert_eq!(rows.len(), 2);
    }

    #[test]
    fn parse_hyperliquid_quotes_sample() {
        let body = r#"[
            {"universe":[{"name":"BTC","szDecimals":5},{"name":"ETH","szDecimals":4}]},
            [{"markPx":"67890.1","prevDayPx":"65000"},{"markPx":"3500.5","prevDayPx":"3600"}]
        ]"#;
        let rows = parse_hyperliquid_quotes(body).unwrap();
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].name, "BTC");
        assert_eq!(rows[0].code, "BTC");
        assert_eq!(rows[0].id, "crypto:BTC");
        assert_eq!(rows[0].market, "crypto");
        assert!((rows[0].price - 67890.1).abs() < 0.01);
        let expected_pct = (67890.1_f64 - 65000.0) / 65000.0 * 100.0;
        assert!((rows[0].pct - expected_pct).abs() < 0.001);
        assert_eq!(rows[1].name, "ETH");
        assert!((rows[1].price - 3500.5).abs() < 0.01);
    }

    #[test]
    fn parse_hyperliquid_quotes_empty() {
        // Empty body must not panic; returns Ok(empty).
        let rows = parse_hyperliquid_quotes("").unwrap();
        assert_eq!(rows.len(), 0);
    }

    #[test]
    fn parse_hyperliquid_quotes_short() {
        // len < 2 -> empty vec, no panic.
        let rows = parse_hyperliquid_quotes("[{}, {}]").unwrap();
        assert_eq!(rows.len(), 0);
    }

    #[test]
    fn parse_frankfurter_sample() {
        let body = r#"{"date":"2026-09-30","base":"USD","rates":{"CNY":7.12,"EUR":0.92,"JPY":149.5,"GBP":0.79,"HKD":7.81}}"#;
        let rows = parse_frankfurter(body).unwrap();
        assert_eq!(rows.len(), 5);
        let eur = rows.iter().find(|r| r.code == "EUR").expect("EUR row");
        assert_eq!(eur.name, "USD/EUR");
        assert_eq!(eur.id, "fx:EUR");
        assert_eq!(eur.market, "fx");
        assert!((eur.price - 0.92).abs() < 1e-6);
        assert_eq!(eur.pct, 0.0);
        assert_eq!(eur.ts, 0);
    }

    #[test]
    fn parse_frankfurter_empty() {
        // Empty body must not panic; returns Ok(empty).
        let rows = parse_frankfurter("").unwrap();
        assert_eq!(rows.len(), 0);
    }

    #[test]
    fn parse_frankfurter_no_rates() {
        let body = r#"{"date":"2026-09-30","base":"USD"}"#;
        let rows = parse_frankfurter(body).unwrap();
        assert_eq!(rows.len(), 0);
    }

    #[test]
    fn parse_nasdaq_quote_sample() {
        let body = r#"{"data":{"symbol":"AAPL","primaryData":{"lastSalePrice":"$178.45","percentageChange":"+1.25%","marketStatus":"Open"},"secondaryData":{"lastSalePrice":"$176.20"}}}"#;
        let rows = parse_nasdaq_quote(body, "us").unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].id, "us:AAPL");
        assert!((rows[0].price - 178.45).abs() < 0.01);
        assert!((rows[0].pct - 1.25).abs() < 0.01);
        assert_eq!(rows[0].market, "us");
    }

    #[test]
    fn parse_nasdaq_quote_escaped() {
        // Body contains \/ escapes (e.g. inside a url field) like the real API.
        let body = r#"{"data":{"symbol":"AAPL","primaryData":{"lastSalePrice":"$178.45","percentageChange":"+1.25%","marketStatus":"Open"},"secondaryData":{"lastSalePrice":"$176.20"},"url":"https:\/\/example.com\/quote\/AAPL"}}"#;
        let rows = parse_nasdaq_quote(body, "us").unwrap();
        assert_eq!(rows.len(), 1);
        assert!((rows[0].price - 178.45).abs() < 0.01);
        assert_eq!(rows[0].id, "us:AAPL");
    }

    #[test]
    fn parse_nasdaq_quote_empty() {
        // Empty body must not panic; either Ok(empty) or Err is acceptable.
        let res = parse_nasdaq_quote("", "us");
        match res {
            Ok(rows) => assert_eq!(rows.len(), 0),
            Err(_) => {}
        }
    }
}
