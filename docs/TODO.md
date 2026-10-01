# finance-brief TODO 总览

> 最后更新：2026-10-02 (Phase 3 + T2 完成)
> 项目：finance-brief (OctoSense splash 应用)
> 维护者：zed-agent

---

## 1. 已知 bug（黑屏）

**症状**：launcher 点「行情看盘」tile 后，整张 card 被 WM 拆掉，用户看到 wallpaper（像"黑屏"）

**agent reproduce 现象**：

| 指标 | 观测值 |
|------|--------|
| click 注入点 | frame 436 |
| PNG 解码风暴 | 136×128 × 180 次（launcher tile icon） |
| UI-hang 单调上升 | 359ms → 668ms |
| WM 日志 | `wm: removing client 1` |
| 模块销毁 | `card.1 torn down; isolate SplashVmId(1) freed` |

**当前 root cause 猜测**：

1. `view_quotes()` 在 `on_render` 闭包内 `quote_pane_chart := CandlestickChart{...}` **命名 widget path**，每次 on_render 重建 path → splash 解释器陷入 widget path update 死循环
2. `candle_up_color`/`candle_down_color`/`candle_width_fraction` 不是 Splash DSL 字段名（Rust struct 是 `bullish_color`/`bearish_color`/`candle_width`），widget apply 报错

**已做 fix**（2026-10-01 21:57）：把 L1393-1398 改成 `CandlestickChart{width: Fill height: Fill}`（匿名 + 只留基础字段）；cargo build 成功，binary 时间 21:59

**未验证**：fix 后 reproduce 没跑（用户连续 cancel 了 reproduce agent）

**待办**：

- [ ] 派一个 reproduce agent 启动 shell 抓 widget tree + log（5 分钟内出现象）
- [ ] 如果 widget tree 仍显示 launcher 不显示 `quotes_pane` → splash `view_quotes()` 渲染前就崩（更深层 bug）
- [ ] 如果 widget tree 显示 `quotes_pane` 但里面 widget 全空 → on_render 闭包死循环（建议用 `on_show` 或把 `view_quotes()` 内 widget 创建移到 root scope）

---

## 2. splash 重构计划（1693 → ~400 行）

**目标**：splash 只做 UI（widget 树、view 切换、on_tap handler、调 Rust）

**待迁函数**（从 splash 迁到 `native/`）：

| 旧路径（splash） | 新路径（native） |
|------------------|------------------|
| `fetch_get` / `fetch_post` / `fetch_q` | `native/src/sources/*.rs` |
| `load_news` / `load_tencent` / `load_hyperliquid` / `load_frankfurter` / `load_nasdaq` | 同上 |
| `load_sample` / `load_settings` / `save_settings` | `native/src/store.rs` |
| `synth_candles` | `native/src/synth.rs` |
| 所有 `parse_*` 内部 helper | `native/src/parse.rs` |

**保留函数**（splash 仍保留）：`refresh_pane` / `pick_view` / `pick` / `pick_quote_pane` / `switch_quote_pane_period` / 所有 `view_*` 函数 / `rows_for` / `set_rows` / `err_for` / `set_err` / `sample_for` / `set_theme` / `padn` / `starts` / `fixed` / `pct_str` / `pct_color` / `strip_dollar` / `strip_sign` / `strip_comma` / `hm` / `sub_of` / `replace_all` / `clean_url` / `get` / `num` / `good`

**预计 splash 行数**：1693 → ~400 行（瘦 76%）

**已完成（2026-10-01 Phase 3 + T2）**：

| 阶段 | commit | 内容 |
|------|--------|------|
| 数据源迁 5/5 | 9542f63 | native 实现 parse_sina_news / parse_tencent_quote / parse_hyperliquid_quotes / parse_frankfurter / parse_nasdaq_quote + 单元测试 |
| splash 真接 host.fetch | ef7247e | native host::handle 2/7 routes (QuotesCandles → synth_candles, NewsRefresh → parse_sina_news) + splash synth_candles 改 host.fetch |
| 删死代码 T2 | 68ecf77 | 删 fetch_get/post/q + strip_sign/comma (-44 行) |
| splash 真接 sina | d76c1f5 | load_news on_response 改 mod.host.fetch("news.refresh") + 删 parse_sina_fallback/sina_tag |

**验证状态**：
- `cargo test -p finance-brief-scaffold` 28/28 过
- reproduce agent 启动 octosense 加载 finance-brief splash bundle OK (eval 77584 bytes)
- launcher 渲染与 G7 capture 一致 (无 parse error)
- 截屏: `/tmp/g7-repro/T2-verify.png` + `/home/lumina/octoOs/_captures/T2-verify.png`

**待办**：

- [ ] P1-1：迁 fetch_* + load_*（splash 改用 `host.call` / `host.fetch`）
- [ ] P1-2：迁 `synth_candles`（splash 改用 native synth）
- [ ] P1-3：迁 settings store（splash 改用 native store）
- [ ] P1-4：重构 `view_quotes()` 不再用 on_render 闭包创建 widget path 命名（避免再次死循环）
- [ ] P1-5：cargo build + reproduce 验证

---

## 3. native crate 任务清单

### 已完成 (2026-10-01)
- [x] 建 `native/Cargo.toml` + src 骨架
- [x] cargo build + cargo test 通过 (28/28)
- [x] 实现 `parse_sina_news` (RSS, G2)
- [x] 实现 `parse_tencent_quote` (CSV, G3)
- [x] 实现 `parse_hyperliquid_quotes` (JSON POST, G4)
- [x] 实现 `parse_frankfurter` (JSON GET, G5)
- [x] 实现 `parse_nasdaq_quote` (JSON, G6)
- [x] 实现 `synth::synth_candles` (n 根 demo OHLC, G1)
- [x] 实现 `host::handle` 路由 2 个 service (QuotesCandles → synth, NewsRefresh → parse_sina_news)

### 未完成 (留 TODO)
- [ ] 实现 `parse_tencent_news` (RSS, 当前 stub)
- [ ] 实现 `parse_nasdaq_candles` (K 线, 当前 stub)
- [ ] 实现 `store::load_settings` / `save_settings` / `load_cache` / `save_cache` (T3)
- [ ] 实现 `host::handle` 路由剩余 5 个 service:
  - [ ] QuotesSnapshot (T4: native 自己 fetch 多源)
  - [ ] QuotesParseTencent (G9a, 中间态)
  - [ ] FavsToggle / FavsList (T3)
  - [ ] SettingsLoad / SettingsSave (T3)
- [ ] 实现 `host::register` (注册 7 个 service 到 SplashVm, 当前 stub)

---

## 4. 数据源待验证

| 数据源 | URL | 状态 |
|--------|-----|------|
| Sina 财经要闻 | feed.mix.sina.com.cn | 已验证 ✓ |
| Tencent 行情 | qt.gtimg.cn | 已验证 ✓ |
| Hyperliquid | api.hyperliquid.xyz | 已验证 ✓ |
| Frankfurter 外汇 | api.frankfurter.dev | 已验证 ✓ |
| NASDAQ 美股 | api.nasdaq.com | native parse_nasdaq_quote 实现 ✓ (G6)；端到端 splash 真接未做 (留 TODO) |

---

## 5. 字体

- [x] 复制 4 个 NotoSansSC ttf + OFL.txt 到 `fonts/`（2026-10-01）
- [ ] splash `Label` 内显式引用字体（语法待 splash 重构时再加）

---

## 6. 当前阶段验收标准（重构 demo 完成后）

**已通过验证 (2026-10-01)**:
- [x] `cargo test -p finance-brief-scaffold` 28/28 通过
- [x] splash bundle 启动 reproduce 验证 (eval 77584 bytes, launcher 渲染)
- [x] splash 行数 1693 → 1734 (净 -41 行, 含 G2-G8 的 fallback helper 增加)
- [x] native 6 个 parse 实现 + 2 个 host routes
- [x] splash 2 处真接 host.fetch (synth_candles + load_news)

- [ ] splash 1693 → ~400 行（`wc -l bundle/main.splash` 验证）
- [ ] splash 不再 import fetch / load / settings 函数（`grep '^fn ' splash | grep -E "fetch|load|save|store"` 验证为空）
- [ ] native crate 5 个 parse 实现 + 4 个 store 实现 + 7 个 host service 注册
- [ ] `cargo build -p finance-brief-scaffold` EXIT=0
- [ ] `cargo test -p finance-brief-scaffold` 通过
- [ ] 启动 shell 后：
  - launcher 6 tile 渲染
  - 新闻简报：Sina RSS 抓得到（5 条以上）
  - 行情看盘：3 tab + 40 根 demo candles 渲染（不黑屏）
  - 收藏 toggle 工作
  - 重启 shell 后收藏 + settings 保留
