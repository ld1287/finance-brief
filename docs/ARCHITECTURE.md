# finance-brief 项目架构

> 适用版本：`finance-brief v0.3.0`（重构期，2026-10-02）
> 仓库：`git@github.com:ld1287/finance-brief.git`
> 文档目的：记录目标架构、模块边界、splash ↔ Rust 通讯接口，作为后续重构的事实基准

---

## 1. 背景与目标

`finance-brief` 是 OctoSense 应用（独立 git 仓库），由 splash DSL（`bundle/main.splash`）+ Rust native module（`native/`）两部分构成。

当前状态（2026-10-01）：

- `bundle/main.splash` **1689 行**，混合了 UI + 5 个数据源 fetch + 状态机 + cache + CRUD，单文件承担过多职责
- 参考项目 `OctoSense/apps/news/`（system app）的做法：`main.splash` 440 行纯 UI；数据/网络/存储全部下沉到 Rust（`native/` + `host-service/`）
- 用户决策（2026-10-01 22:25）：splash 只做 UI（widget 树、event handler、调 Rust）；Rust 做数据（fetch、parse、cache、状态、CRUD）；splash ↔ Rust 通过 `host.call` / `host.fetch`；项目要解耦

目标：**splash 从 1689 行瘦身到 ~400 行**；新增 `native/` 承担所有数据/网络/持久化职责。

---

## 2. 目标目录树

> 表中 `[已建]` = 当前文件已存在；`[未建]` = 重构期需要新建。

```
finance-brief/
├── bundle/                                  # splash DSL + 静态资源
│   ├── main.splash                          [已建] 待瘦身 → ~400 行
│   ├── manifest.json                        [已建] capabilities: storage, net
│   ├── listing.json                         [已建]
│   └── assets/
│       └── icon.svg                         [已建]
│
├── fonts/                                   [未建] NotoSansSC 中文字体
│   ├── NotoSansSC-Regular.ttf
│   ├── NotoSansSC-Bold.ttf
│   ├── NotoSansSC-Medium.ttf
│   └── NotoSansSC-Light.ttf
│
├── native/                                  [未建] Rust module（数据/网络/存储）
│   ├── Cargo.toml                           [未建]
│   ├── src/
│   │   ├── lib.rs                           [未建] 模块入口、命令注册
│   │   ├── model.rs                         [未建] NewsRow / QuoteRow / Candle / Research / Favorite
│   │   ├── parse.rs                         [未建] parse_sina_news / parse_tencent_quote / parse_hyperliquid / parse_frankfurter / parse_nasdaq
│   │   ├── synth.rs                         [未建] synth_candles（迁自 splash L183-200）
│   │   ├── store.rs                         [未建] settings.json / cache_*.json（迁自 splash L272-310）
│   │   ├── host.rs                          [未建] host.call / host.fetch 注册点
│   │   └── sources/
│   │       ├── mod.rs                       [未建] sources 模块入口
│   │       ├── sina.rs                      [未建] Sina 财经要闻
│   │       ├── tencent.rs                   [未建] Tencent 行情快照（A/港/美）
│   │       ├── hyperliquid.rs               [未建] Hyperliquid 加密币
│   │       ├── frankfurter.rs               [未建] Frankfurter 外汇
│   │       └── nasdaq.rs                    [未建] NASDAQ 美股（待接入）
│   └── tests/
│       └── smoke.rs                         [未建] 5 个源最小烟测
│
├── docs/
│   ├── ARCHITECTURE.md                      [未建] 本文件
│   ├── TODO.md                              [未建] 重构计划
│   ├── DATA-SOURCES.md                      [未建] 数据源说明
│   ├── R-1-us-stocks.md                     [已建] 美股实时源调研（结论：api.nasdaq.com）
│   └── R-2-kline-feasibility.md             [已建] K 线可行性调研
│
└── scripts/
    ├── run-octosense.sh                     [已建]
    └── install-as-system-app.sh             [已建]
```

---

## 3. 模块边界

清晰划线，**splash 不做数据、native 不画 UI**。

| 关注点 | splash（UI 层） | native（数据层） |
|---|---|---|
| Widget 创建 | ✅ | — |
| View 切换 / 标签页 | ✅ | — |
| `on_tap` 事件处理 | ✅ | — |
| 调 `host.call` / `host.fetch` | ✅ | — |
| HTTP fetch（GET/POST） | — | ✅ |
| JSON / 文本解析 | — | ✅ |
| OHLC / K 线合成 | — | ✅ |
| Cache 读写（`cache_*.json`） | — | ✅ |
| 收藏 / 设置持久化（`settings.json`） | — | ✅ |
| 状态机（loaded / loading / error） | — | ✅ |
| 错误归一化（HTTP 错、解析错、超时） | — | ✅ |
| Theme 切换 | ✅（写本地 state） | — |

> **原则**：splash 收到的永远是已归一化的结构体（`NewsRow` / `QuoteRow` / `Candle` / `Favorite`），不需要再做字符串清洗。

---

## 4. splash ↔ Rust 通讯接口

splash 通过 OctoScript DSL 的 host API 调 native。两个核心入口：

- **`mod.host.fetch(name, args)`** — 同步请求；`name` 是注册服务名；`args` 是 dict；返回结果或抛错
- **`mod.host.call(name, args)`** — 异步 promise；适合 fire-and-forget（写操作）

> 详细 host API 文档见 `OctoSense` 主仓 `host.md`（system app 协议）。

### 4.1 注册的服务清单

| 服务名 | 类型 | 入参 | 返回 | 说明 |
|---|---|---|---|---|
| `news.refresh` | `fetch` | `{limit: int}` | `{rows: [NewsRow], fetched_at: int}` | 拉 Sina 财经要闻 RSS，parse + cache |
| `quotes.snapshot` | `fetch` | `{tab: "a\|us\|hk\|crypto\|fx", symbols: [string]}` | `{rows: [QuoteRow], fetched_at: int}` | 多源汇总（A/美/港 → Tencent；crypto → Hyperliquid；fx → Frankfurter） |
| `quotes.parse_tencent` | `fetch` | `{market: string, body: string}` | `{rows: [QuoteRow], fetched_at: int}` | **中间态**（G3 注释设计）：splash 仍 net.http_request 拿 body, native 只 parse。等 R-3 真解耦轮次合并到 `quotes.snapshot` |
| `quotes.candles` | `fetch` | `{symbol: string, period: "1m\|5m\|1h\|1d", count: int}` | `{candles: [Candle]}` | K 线（1m/5m/1h 用 synth，1d 用源）；详情见 `R-2-kline-feasibility.md` |
| `favs.toggle` | `call` | `{kind: "news\|quote", key: string}` | `{favored: bool}` | 收藏 toggle，异步写 `settings.json` |
| `favs.list` | `fetch` | `{}` | `{news: [string], quotes: [string]}` | 列出所有收藏 key |
| `settings.load` | `fetch` | `{}` | `Settings` | 读 `settings.json` |
| `settings.save` | `call` | `{patch: dict}` | `{}` | patch 写 `settings.json`（异步） |

### 4.2 数据契约（节选）

```rust
// native/src/model.rs（节选，待实现）
struct NewsRow { title: String, url: String, source: String, ts: i64 }
struct QuoteRow { symbol: String, name: String, last: f64, change_pct: f64, ts: i64, kind: QuoteKind }
struct Candle    { ts: i64, open: f64, high: f64, low: f64, close: f64, vol: f64 }
struct Favorite  { kind: String, key: String, added_at: i64 }
struct Settings  { theme: String, refresh_sec: u32, favs: Vec<Favorite>, news_filter: Vec<String> }
```

---

## 5. splash 瘦身目标（1689 → ~400 行）

### 5.1 迁到 native（数据/网络/持久化层）

| 函数 | 状态 | 迁到 | commit |
|------|------|------|--------|
| `fetch_get` | ✅ 已删 (T2) | `native/src/sources/*` | 68ecf77 |
| `fetch_post` | ✅ 已删 (T2) | 同上 | 68ecf77 |
| `fetch_q` | ✅ 已删 (T2) | 同上 | 68ecf77 |
| `load_news` | 🔄 splash 改 host.fetch, splash 仍 net.http_request (中间态) | `sources/sina.rs` (R-3 合并) | d76c1f5 |
| `load_tencent` | 🔄 splash 仍用 `parse_tencent_fallback` (G9a 待做) | `sources/tencent.rs` | — |
| `load_crypto` | 🔄 splash 仍用 `parse_hyperliquid_fallback` | `sources/hyperliquid.rs` | — |
| `load_fx` | 🔄 splash 仍用 `parse_frankfurter_fallback` | `sources/frankfurter.rs` | — |
| `load_nasdaq` | 🔄 splash 仍用 `parse_nasdaq_fallback` | `sources/nasdaq.rs` | — |
| `load_sample` | ⏳ 移除 | - | - |
| `load_settings` | ⏳ splash 仍直接 fs 读 (T3 待做) | `store.rs` | — |
| `save_settings` | ⏳ splash 仍直接 fs 写 (T3 待做) | `store.rs` | — |
| `toggle_setting` | ⏳ (T3 待做) | splash 调 `settings.save` | — |
| `synth_candles` | ✅ splash 改 host.fetch, native handle QuotesCandles | `synth.rs` | ef7247e |
| `parse_sina_news` | ✅ native 实现 | `parse.rs` | 9542f63 |
| `parse_tencent_quote` | ✅ native 实现 | `parse.rs` | 9542f63 |
| `parse_hyperliquid_quotes` | ✅ native 实现 | `parse.rs` | 9542f63 |
| `parse_frankfurter` | ✅ native 实现 | `parse.rs` | 9542f63 |
| `parse_nasdaq_quote` | ✅ native 实现 | `parse.rs` | 9542f63 |

> 注：用户在 2026-10-01 22:25 表述中用的是 `load_hyperliquid` / `load_frankfurter`，本表按 splash 当前真实函数名（`load_crypto` / `load_fx`）列出，行为等价。

### 5.2 留在 splash（UI 层）

以下函数**保留**在 splash，仅作 view 渲染 / 事件路由：

- 入口与视图切换：`refresh_pane` / `pick_view` / `pick` / `pick_quote_pane` / `switch_quote_pane_period`
- 视图构建器（每个 view 一个函数）：`view_launcher` / `view_main_pane` / `view_favs` / `view_settings` / `view_about` / `view_research` / `view_research_detail` / `view_quotes`
- 列表渲染辅助：`rows_for` / `set_rows` / `err_for` / `set_err` / `sample_for` / `set_theme`
- 字符串处理工具：`padn` / `starts` / `fixed` / `pct_str` / `pct_color` / `strip_dollar` / `strip_sign` / `strip_comma` / `hm`（UI 文本格式化，留在 splash）

### 5.3 预估瘦身效果

- 迁出：~17 个函数，约 800 行
- 留下：~30 个 view/工具函数，约 400 行（含 view 树）
- **目标：~400 行**

---

## 6. 数据源表

> 完整数据源说明见 `docs/DATA-SOURCES.md`（待写）。本表为概要。

| # | 源 | URL | 格式 | 限频 | API key | 备注 |
|---|---|---|---|---|---|---|
| 1 | Sina 财经要闻 | `https://feed.mix.sina.com.cn/api/rollout?...` | JSON（伪 RSS，HTML 嵌套） | 无明确限制 | 否 | 主新闻源；需 walk JSON tree 解析 |
| 2 | Tencent 行情快照 | `https://qt.gtimg.cn/q={symbols}` | 文本（GBK，`v_xxx=...`） | 无明确限制 | 否 | A 股 / 港股 / 美股 us.* **昨收**；多 symbol 用 `,` 分隔 |
| 3 | Hyperliquid 加密币 | `https://api.hyperliquid.xyz/info` | JSON | 无明确限制 | 否 | POST `{type:"allMids"}` + `{type:"meta"}`；用于 BTC/ETH/SOL 实时价 + 24h 变化 |
| 4 | Frankfurter 外汇 | `https://api.frankfurter.dev/v1/latest?base=USD&symbols=EUR,JPY,...` | JSON | 无明确限制 | 否 | ECB 日级汇率，工作日更新；用作 fx tab |
| 5 | NASDAQ 美股 | `https://api.nasdaq.com/api/quote/{SYMBOL}/info?assetclass=stocks` | JSON | 无明确限制 | 否 | **首选**美股实时源；Akamai CDN，盘中分钟级 tick；ETF 同 endpoint；详见 `R-1-us-stocks.md` |

> 所有 5 个源已声明在 `bundle/manifest.json#network.hosts`，无需新增白名单。

---

## 7. 下一步（按优先级）

1. **建目录骨架** — 新建 `native/Cargo.toml` + `native/src/{lib,model,parse,synth,store,host}.rs` + `sources/{mod,sina,tencent,hyperliquid,frankfurter,nasdaq}.rs`；先空壳能 `cargo build`
2. **迁 settings + cache** — 优先级最高，因为 splash 启动路径要 `settings.load`；先把 `store.rs` 落地，splash 改为 `host.fetch("settings.load", {})`
3. **迁 Sina 新闻** — 最熟，先跑通 `news.refresh`；在 `parse.rs` 加单测
4. **迁 Tencent / Hyperliquid / Frankfurter** — 三个源结构相似，统一在 `sources/quote.rs`（不引入）由三个文件分担
5. **接入 NASDAQ** — 已在 `R-1-us-stocks.md` 验证；写 `nasdaq.rs` + 在 `quotes.snapshot` 加 `kind=us` 分支
6. **splash 瘦身 PR** — 迁完一轮后整体 grep 一次 `fetch_get`/`fetch_post`/文件 IO 调用，确认 splash 0 处残留
7. **smoke 测试** — `native/tests/smoke.rs` 跑 5 个源（带 `#[ignore]` 默认跳过，需网络时 `--ignored`）
8. **写 `docs/TODO.md` 和 `docs/DATA-SOURCES.md`** — 把本架构决策落到团队文档

---

## 8. 变更日志（仅本架构文档）

| 日期 | 变更 |
|---|---|
| 2026-10-01 | 初版（决策依据：用户 22:25 决策；参考 `OctoSense/apps/news/`） |
