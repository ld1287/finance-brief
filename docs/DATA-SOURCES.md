# Data Sources

finance-brief 当前依赖 5 个免密 HTTP 数据源，全部在 `bundle/manifest.json` 的 `network.hosts` 中声明。所有 `net.http_request` 必须命中该白名单（host policy）。

## 1. 数据源总表

| # | 数据源 | URL | 用途 | 免密 | 限频 | 当前状态 | 备注 |
|---|---|---|---|---|---|---|---|
| 1 | Sina 财经要闻 | feed.mix.sina.com.cn | 新闻 RSS | ✓ | 未明确（建议 60 req/min） | ✓ 已验证 | RSS XML 格式 |
| 2 | Tencent qt.gtimg | qt.gtimg.cn | 行情快照（A股/美股/港股） | ✓ | 未明确 | ✓ 已验证 | CSV 格式，UTF-8 |
| 3 | Hyperliquid | api.hyperliquid.xyz | 加密币 last + 24h | ✓ | 未公开 | ✓ 已验证 | JSON POST |
| 4 | Frankfurter | api.frankfurter.dev | 外汇汇率 | ✓ | 公开 API | ✓ 已验证 | JSON GET |
| 5 | NASDAQ | api.nasdaq.com | 美股 | ✓ | 部分端点免密 | ⏳ 待验证 | 需调研合规性 |

## 2. 各数据源详情

### 2.1 Sina 财经要闻
- **URL 模板**：`https://feed.mix.sina.com.cn/api/roll/get?lid=1686&num=30&page=1`
- **请求方法**：GET
- **响应格式**：JSON（外层包裹 `result.data` 数组）
- **关键字段**：`result.data[*].title`、`url`、`ctime`、`intro`
- **错误处理**：超时 10s 重试 1 次；4xx 跳过整批；5xx 重试 1 次后降级为空列表
- **样例响应**：`{"result":{"data":[{"title":"...","url":"https://...","ctime":"2026-01-01 09:00"}]}}`
- **splash → Rust 调用方式**：现 `bundle/main.splash` `load_news()` → 迁后 `native::sources::sina::fetch` + `parse_sina_news`（本任务不实际改动）
- **已知问题**：lid 列表偶发变更，需在 splash 侧做 lid→栏目映射
### 2.2 Tencent qt.gtimg
- **URL 模板**：`https://qt.gtimg.cn/q=sh000001,usAAPL,hs00700`
- **请求方法**：GET
- **响应格式**：CSV-like 字符串（UTF-8，含中文）
- **关键字段**：按 `~` 分隔，第 3 列为名称、第 6 列为当前价、第 32 列为涨跌幅、第 38 列为成交额
- **错误处理**：超时 8s 重试 1 次；非 200 直接跳过该 symbol
- **样例响应**：`v_sh000001="1~上证指数~000001~3500.0~...~-0.12~..."`
- **splash → Rust 调用方式**：现 `load_tencent(tid, syms, listed)` → 迁后 `native::sources::tencent::fetch` + `parse_tencent_quote`（本任务不实际改动）
- **已知问题**：字段下标偶尔漂移；按 key 解析比按 index 解析更稳
### 2.3 Hyperliquid
- **URL 模板**：`https://api.hyperliquid.xyz/info`，body `{"type":"allMids"}`
- **请求方法**：POST（application/json）
- **响应格式**：JSON 对象 `{ "<COIN>": "<price_string>" }`
- **关键字段**：`BTC`、`ETH`、`SOL` 等 mid 价格；24h 走 `metaAndAssetCtxs`
- **错误处理**：超时 8s 重试 2 次；429 退避 2s；5xx 跳过
- **样例响应**：`{"BTC":"67000.5","ETH":"3500.0"}`
- **splash → Rust 调用方式**：现 `load_hyperliquid()` → 迁后 `native::sources::hyperliquid::fetch` + `parse_hyperliquid`（本任务不实际改动）
- **已知问题**：偶发返回空对象；fallback 显示上一帧缓存
### 2.4 Frankfurter
- **URL 模板**：`https://api.frankfurter.dev/latest?from=USD&to=EUR,JPY,CNY`
- **请求方法**：GET
- **响应格式**：JSON
- **关键字段**：`amount`（基准单位）、`rates`（目标币种 map）、`base`、`date`
- **错误处理**：超时 8s 重试 1 次；周末汇率仍返回最近工作日
- **样例响应**：`{"amount":1.0,"base":"USD","date":"2026-01-01","rates":{"EUR":0.92,"JPY":150.0}}`
- **splash → Rust 调用方式**：现 `load_frankfurter()` → 迁后 `native::sources::frankfurter::fetch` + `parse_frankfurter`（本任务不实际改动）
- **已知问题**：节假日返回上一交易日数据；UI 需标注 `date`
### 2.5 NASDAQ
- **URL 模板**（待定）：`https://api.nasdaq.com/api/quote/AAPL/info?assetclass=stocks`
- **请求方法**：GET（需带 `User-Agent` header）
- **响应格式**：JSON
- **关键字段**：`data.primaryData.lastSalePrice`、`data.primaryData.percentChange`
- **错误处理**：超时 8s；4xx/5xx 跳过；合规未确认前不写入白名单生产路径
- **样例响应**：`{"data":{"primaryData":{"lastSalePrice":"190.5","percentChange":"+0.5%"}}}`
- **splash → Rust 调用方式**：未实现（仅在 R-1 调研阶段，本任务不实际改动）→ 迁后 `native::sources::nasdaq::fetch` + `parse_nasdaq`
- **已知问题**：免密端点受限，需评估合规与 ToS 后再启用

## 3. 网络白名单（manifest.json）

```json
{
  "network": {
    "hosts": [
      "feed.mix.sina.com.cn",
      "qt.gtimg.cn",
      "api.hyperliquid.xyz",
      "api.frankfurter.dev",
      "api.nasdaq.com"
    ]
  }
}
```

以上 5 个 host 已在 `bundle/manifest.json` 的 `network.hosts` 声明（manifest/listing.json 本任务不修改）。每个 splash 的 `net.http_request` 必须命中该清单（host policy），否则 splash runtime 会拒绝请求。

## 4. 数据源迁移计划（splash → native）

| 源 | splash 函数 | 迁到 native |
|---|---|---|
| Sina | `load_news()` | `native::sources::sina::fetch` + `parse_sina_news` |
| Tencent | `load_tencent(tid, syms, listed)` | `native::sources::tencent::fetch` + `parse_tencent_quote` |
| Hyperliquid | `load_hyperliquid()` | `native::sources::hyperliquid::fetch` + `parse_hyperliquid` |
| Frankfurter | `load_frankfurter()` | `native::sources::frankfurter::fetch` + `parse_frankfurter` |
| NASDAQ | （未实现） | `native::sources::nasdaq::fetch` + `parse_nasdaq` |

## 5. 付费数据源调查

- 金十数据 jin10.com：**付费 API**，MVP 不接入
- Wind / Bloomberg / EastMoney Choice：**全部排除**（授权与商业限制）

## 6. 后续扩展

- WebSocket 接入：Hyperliquid 已提供 ws endpoint，可加实时 K 线推送
- 更多美股接口：参考 `docs/R-1-us-stocks.md` 的调研结论
