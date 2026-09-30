# R-1: 美股实时数据源调研

## 任务与上下文
- 项目：finance-brief（OctoSense 应用）
- 工作分支：`finance-brief-app`（本地）
- 测试时间窗：2026-09-30 ~18:42 CST（**美东 06:42 EDT，盘前 Pre-Market**）
- 已工作源：Sina / Tencent qt.gtimg.cn（us.* 是昨收）/ Hyperliquid / Frankfurter
- 约束：仅 `net.http_request`（HTTP/HTTPS）；不允许原始 socket；免 key；JSON 优先；不写 HTML scraping 除非无 JSON 可选
- 目标：在候选清单内找到**非"上一交易日收盘价"**的免密美股源

---

## 测试矩阵

| # | 源 | URL | HTTP | 格式 | 编码 | 需 key | 实时性 | 推荐 |
|---|---|---|---|---|---|---|---|---|
| 1 | stooq CSV | `https://stooq.com/q/l/?s=aapl.us&f=sd2t2ohlcv&h&e=csv` | **404** | HTML 错误页 | UTF-8 | 否 | ❌ URL 路径无效 | ✗ |
| 1' | stooq 备用路径 `/q/?s=aapl.us&f=sd2t2ohlcvn` | 同 host | 200 | **HTML（JS challenge）** | UTF-8 | 否 | ❌ 触发 Cloudflare-style JS 验证，`curl` 无法获取数据 | ✗ |
| 1'' | stooq `/q/d/l/?s=aapl.us&i=d` | 同 host | 200 | HTML（JS challenge） | UTF-8 | 否 | ❌ 同上 | ✗ |
| 2 | Yahoo Quote v7 | `https://query1.finance.yahoo.com/v7/finance/quote?symbols=AAPL` | **403** | HTML sad-panda 页 | UTF-8 | 否 | ❌ Yahoo 自 2024 起对云 IP 返回 403；需 cookie + 重定向握手，curl 极不可靠 | ✗ |
| 3 | Yahoo Chart v8 | `https://query1.finance.yahoo.com/v8/finance/chart/AAPL?interval=1m&range=1d` | **403** | HTML sad-panda 页 | UTF-8 | 否 | ❌ 同上 | ✗ |
| 4 | nasdaq.com HTML（zh-CN） | `https://www.nasdaq.com/zh-CN/market-activity/stocks/aapl/real-time` | 200（**先 301 重定向**到 `/zh-CN/market-activity/stocks/aapl`） | HTML | UTF-8 | 否 | — 走 Akamai CDN，HTML scraping 不稳定 | △（fallback only） |
| 5 | **api.nasdaq.com JSON** | `https://api.nasdaq.com/api/quote/AAPL/info?assetclass=stocks` | **200** | **JSON** | **UTF-8** | **否** | ✅ **盘前 tick 分钟级推进，价格同步变化**（见下） | ✅ **首选** |
| 5b | nasdaq JSON，ETF | `?assetclass=etf`，例 SPY | **200** | JSON | UTF-8 | 否 | ✅ 同样实时（SPY `6:44 AM ET` $764.7312） | ✅ 首选（同时覆盖 ETF） |
| 5c | nasdaq JSON，futures | `?assetclass=futures` | **400** Bad or No parameter | JSON | UTF-8 | 否 | 不支持 | ✗ |
| 5d | nasdaq JSON，指数 IXIC | 同 endpoint | **400** Symbol not exists | JSON | UTF-8 | 否 | 指数需另寻端点 | ✗（不覆盖指数） |
| 6 | CNBC JSON api | `https://api.cnbc.com/v1/symbol/AAPL/quote` | **DNS 失败**（`Could not resolve host: api.cnbc.com`） | — | — | 否 | 域名不解析 | ✗ |
| 7 | CNBC HTML | `https://www.cnbc.com/quotes/AAPL` | **200** | HTML（2.4 MB） | UTF-8 | 否 | HTML 中 grep 不到简洁嵌入 JSON；需完整 DOM 解析 | △（fallback only） |
| 8–13 | marketdata / alphavantage / finnhub / tiingo / polygon / twelve-data | — | — | — | — | **是** | — | 跳过（PRD Phase 1 禁付费） |

### 候选源 1–7 的原始 curl 输出佐证

**候选 5（首选）— nasdaq JSON，raw response（截断）：**

```
HTTP/2 200
content-type: application/json; charset=utf-8
server: Kestrel
```

```json
{
  "data": {
    "symbol": "AAPL",
    "companyName": "Apple Inc. Common Stock",
    "exchange": "NASDAQ-GS",
    "marketStatus": "Pre-Market",
    "primaryData": {
      "lastSalePrice": "$329.51",
      "netChange": "+0.11",
      "percentageChange": "+0.03%",
      "deltaIndicator": "up",
      "lastTradeTimestamp": "Sep 30, 2026 6:44 AM ET",
      "isRealTime": true,
      "bidPrice": "$329.50",
      "askPrice": "$329.56",
      "bidSize": "5",
      "askSize": "248",
      "volume": "102,457.094026",
      "currency": null
    },
    "secondaryData": {
      "lastSalePrice": "$329.40",
      "netChange": "-9.00",
      "percentageChange": "-2.66%",
      "lastTradeTimestamp": "Closed at Sep 29, 2026 4:00 PM ET",
      "isRealTime": false,
      ...
    },
    "keyStats": { "fiftyTwoWeekHighLow": { "label": "52 Week Range:", "value": "243.42 - 345.34" } },
    "assetClass": "STOCKS"
  }
}
```

### 实时性验证（决定性证据）

| 时刻（本地 CST） | NY ET | API `lastTradeTimestamp` | API `lastSalePrice` |
|---|---|---|---|
| 18:45:45 | 06:45:45 | `Sep 30, 2026 6:45 AM ET` | `$329.47` |
| 18:46:53（+68s） | 06:46:53 | `Sep 30, 2026 6:46 AM ET` | `$329.5085` |

时间戳与 lastSalePrice 同步推进 → **非昨收、非静态缓存**。

### 重要客户端约束（实施时必须满足）

1. **必须设 `User-Agent: Mozilla/5.0`** —— 无 UA 直接 `000`（连接级失败）。
2. **`Accept: application/json` 可选** —— 不影响结果。
3. **Cookie 持久化强烈推荐** —— 服务器下发 `akaalb_ALB_Default` session cookie。带 cookie jar 6 连发 200；不带 jar 时偶现 404（观察到 1/3 概率），原因疑似 Akamai 灰度 scrub。
4. **`Origin` / `Referer` 不要发** —— 加上 `Origin: https://www.nasdaq.com` 即变 404（实测）。
5. **限速粗估**：每次请求间隔 ≥0.3s 安全；建议 ≥1s 以最稳定。单连接短期可支撑 6+ 次/分钟。
6. **支持 assetclass**：`stocks`、`etf` ✅；`futures`、`index` ❌。
7. **价格字段含 `$` 前缀**（字符串），变化量带 `+`/`-` 前缀；解析时需 `strip('$')`、`strip('+')`。

---

## 推荐首选

- **源**：`api.nasdaq.com`
- **URL 模板**：
  - 股票：`https://api.nasdaq.com/api/quote/{SYMBOL}/info?assetclass=stocks`
  - ETF：`https://api.nasdaq.com/api/quote/{SYMBOL}/info?assetclass=etf`
- **返回示例（truncated）**：见上节 `primaryData` / `secondaryData` 块。
- **为什么**：
  1. 唯一**真正非昨收**的免密 JSON 美股源（其余 1–7 全部失败：stooq JS challenge、Yahoo 403、CNBC JSON DNS 不通、CNBC/nasdaq HTML 需 scraping）。
  2. `primaryData.isRealTime=true`、时间戳与价格同步分钟级推进；`secondaryData` 直接给出昨收 — 一个端点同时给"实时 + 昨收对照"，极适合简报卡片。
  3. 返回 `marketStatus` 字段直接告诉当前是 `Pre-Market` / `Regular` / `Closed`，UI 可据此切换标签。
  4. Kestrel + Akamai（`akaalb`），响应 < 200ms。
- **限制**：
  - **数据延迟**：分钟级时间戳，亚分钟级价格更新（盘前/盘中均验证）。**不是 1Hz WS tick**——但相对 Tencent `us.*` 的"上一交易日 4:00 PM ET 收盘价"是质的提升。
  - **限速**：实测带 jar + 1s 间隔 6/6 稳定 200；不带 jar 偶发 404。建议调用方 ≥1s 间隔。
  - **覆盖**：美股 + 美股 ETF；**不含**指数（IXIC/DJI/SPX）、不含美股期权、不含期货、外汇、加密。
  - **字段编码**：`lastSalePrice`/`bidPrice`/`askPrice` 含 `$` 前缀且为字符串；`netChange` 带 `+`/`-`；`volume` 含千分位逗号且为字符串；`lastTradeTimestamp` 是 `"MMM DD, YYYY h:MM AM/PM ET"` 文本格式，需自定义解析。

---

## manifest.json hosts 片段

当前 hosts：
```json
["feed.mix.sina.com.cn","qt.gtimg.cn","api.hyperliquid.xyz","api.frankfurter.dev"]
```

需追加：
```json
["feed.mix.sina.com.cn","qt.gtimg.cn","api.hyperliquid.xyz","api.frankfurter.dev","api.nasdaq.com"]
```

> 仅加 host（不影响 `bundle.main.splash` 当前 558 行）。后续若启用 HTML fallback（候选 4/7），再加 `www.nasdaq.com`、`www.cnbc.com`。

---

## 备选（按推荐度）

1. **`https://www.nasdaq.com/zh-CN/market-activity/stocks/{symbol}`**（HTML，301 跳掉 `/real-time`）—— 当 JSON API 不可达时降级；需 DOM 解析（如 query selector 抓 `data-symbol-last-sale`），不推荐为首选。
2. **`https://www.cnbc.com/quotes/{SYMBOL}`**（HTML，2.4 MB）—— 含 Next.js `__NEXT_DATA__` JSON 块，可正则提取；同样需 UA 与 cookie。**仅作为最后 fallback**。
3. **继续使用 `qt.gtimg.cn` 美股 `us.*`** —— 现存 fallback，提供"昨收"作参考；新代码里作为"无网络时的退化基线"。

---

## 结论

**首选源：`api.nasdaq.com/api/quote/{SYMBOL}/info?assetclass=stocks`** —— 唯一满足"免 key + JSON + 真正实时（非昨收）"三项硬约束的候选；时间戳与价格分钟级同步推进验证通过；与 splash 中既有 Tencent 旧 `us.*` 接口字段高度互补。限制是分钟级时间戳（非秒级 tick）、覆盖仅美股+ETF、不含指数/期货。Fallback 顺序：① nasdaq.com/zh-CN HTML（DOM 解析）；② cnbc.com HTML（`__NEXT_DATA__` 提取）；③ Tencent `us.*` 昨收作基线展示。建议先在 splash 加 host `api.nasdaq.com`，实现"实时价 + 昨收对照"双字段卡片，再视体验决定是否引入 HTML fallback。