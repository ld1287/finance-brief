# finance-brief TODO 总览

> 最后更新：2026-10-01
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

**待办**：

- [ ] P1-1：迁 fetch_* + load_*（splash 改用 `host.call` / `host.fetch`）
- [ ] P1-2：迁 `synth_candles`（splash 改用 native synth）
- [ ] P1-3：迁 settings store（splash 改用 native store）
- [ ] P1-4：重构 `view_quotes()` 不再用 on_render 闭包创建 widget path 命名（避免再次死循环）
- [ ] P1-5：cargo build + reproduce 验证

---

## 3. native crate 任务清单（已建骨架）

- [x] 建 `native/Cargo.toml` + `src/{lib,model,parse,synth,store,host,sources/{mod,sina,tencent,hyperliquid,frankfurter,nasdaq}}.rs` 占位骨架
- [x] cargo build + cargo test 通过
- [ ] 实现 `parse_sina_news`（RSS 解析）
- [ ] 实现 `parse_tencent_quote`（CSV 解析）
- [ ] 实现 `parse_hyperliquid`（JSON）
- [ ] 实现 `parse_frankfurter`（JSON）
- [ ] 实现 `parse_nasdaq`（JSON，待调研）
- [ ] 实现 `store::load_settings` / `save_settings` / `load_cache` / `save_cache`
- [ ] 实现 `host::register`（注册 7 个 service 到 SplashVm）
- [ ] 实现 `host::handle` 路由

---

## 4. 数据源待验证

| 数据源 | URL | 状态 |
|--------|-----|------|
| Sina 财经要闻 | feed.mix.sina.com.cn | 已验证 ✓ |
| Tencent 行情 | qt.gtimg.cn | 已验证 ✓ |
| Hyperliquid | api.hyperliquid.xyz | 已验证 ✓ |
| Frankfurter 外汇 | api.frankfurter.dev | 已验证 ✓ |
| NASDAQ 美股 | api.nasdaq.com | **未验证**，待接入 |

---

## 5. 字体

- [x] 复制 4 个 NotoSansSC ttf + OFL.txt 到 `fonts/`（2026-10-01）
- [ ] splash `Label` 内显式引用字体（语法待 splash 重构时再加）

---

## 6. 当前阶段验收标准（重构 demo 完成后）

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
