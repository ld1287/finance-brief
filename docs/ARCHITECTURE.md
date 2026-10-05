# finance-brief 架构（Octoscript 单一方案）

> 适用版本：`finance-brief v0.4.0-dev`（Octoscript 单一方案迁移后）
> 仓库：`git@github.com:ld1287/finance-brief.git`
> 当前 HEAD（main）：`b19656c`
> 迁移分支：`feat/one-octoscript`

本文档 = finance-brief 唯一架构形态。所有 `.octoscript` 文件直接经
`octoscript-render` 评估为 UiNode 树，再由 `octoscript-makepad::to_makepad_ui`
翻译为 makepad dialect，挂到 `Splash.view` 上渲染为原生控件。

不再存在以下过时形态（per `feat/one-octoscript` 迁移，2026-10-04）：
- ❌ `bundle/main.splash`（572 行 splash DSL）—— 已删除
- ❌ 12 个 `bundle/*.card`（252 行 L0 ledger 静态模板）—— 已删除
- ❌ Path 1 vs Path 3 lite 双路径并存 —— 已收敛
- ❌ `docs/SPLASH-STYLE.md`（splash DSL style guide）—— 已删除
- ❌ `docs/WIDGET-COMPATIBILITY.md`（Layer -1 makepad widget 详细）—— 已删除

---

## §0 状态速览

| 节 | 状态 |
|----|------|
| §1 背景与目标 | ✅ |
| §2 平台组件分层 | L0-L5 ✅ |
| §3 finance-brief 形态 | ✅（已对齐目标 v3） |
| §4 模块边界 | ✅ |
| §5 capability 清单 | ✅ 17 个已声明 |
| §6 workflow 编排 | ✅（`workflow.octoscript` 已挂上 screens） |
| §7 adapter 实现 | ✅（`native/src/adapters/`，9 个） |
| §8 渲染管线 | ✅（octoscript-makepad 完整管线） |
| §9 shell 集成 | 🎯（待 OctoScript#56 桥接） |
| §10 octoscode | 🎯 |
| §11 数据源 | ✅ |
| §12 Phase 计划 | Phase 1-3 ✅ / 4-6 🎯 |

---

## §1. 背景与目标

`finance-brief` 是 OctoSense 应用（独立 git 仓库），落地在 octoscript 平台
分层之上。

**v1 痛点**（2026-10-02 之前）：
- `bundle/main.splash` 单文件承担 UI + 5 个数据源 fetch + 状态机 + cache + CRUD（v1 末期 1689 行 → 当前 572 行）
- 手写 `fetch_get` / `parse_sina_news` / `cache_*.json` —— 违反 octoscript capability-first
- splash 闭包命名 widget path 触发死循环（view_quotes 黑屏 3 次 fix 未根治）
- 没有数据契约（`host.call` 返回什么靠约定）
- 没有 workflow（数据流手写）
- 用户决策（2026-10-02）：切到 octoscript 平台分层

**v2 中间形态**（2026-10-02 至 2026-10-04）：
- splash DSL `let X = ...` + flat-let 单文件
- workflow.octoscript 已写但 main.splash 不调用
- 12 个 `.card` 静态 glance tile（意外副产品）

**v3 当前形态**（2026-10-04 起，`feat/one-octoscript` 分支）：
- 全部 UI 在 `bundle/screens/*.octoscript` × 14
- 入口 `apps/desktop/` Rust app（仿 `Octoscript-Makepad/apps/flutter-samples/`）
- workflow.octoscript 真正被 screens 调用（refresh_workflow 等 7 fn）

---

## §2. 平台组件分层（核心架构）

| Layer | 名称 | 位置 | 状态 | Phase |
|-------|------|------|------|-------|
| 5 | OctoSense shell（宿主） | `OctoSense/` | 🎯 目标 | Phase 4 |
| 4 | Octos Agent 内核 | `OctoSense/octos/` | 🎯 目标 | Phase 5 |
| 3 | Octoscript-AppCard 应用层 | `OctoSense-App-Hub/skills/card-studio/` | ❌ 未存在 | Phase 0 |
| 2 | Octoscript 核心 runtime | `OctoSense/octoscript/` | ✅ 已实现 | Phase 1 |
| 1 | Octoscript-Makepad 渲染管线 | `OctoSense/octoscript-makepad/` | ✅ 已实现 | Phase 1 |
| 0 | Makepad 底层 | `OctoSense/makepad/` | ✅ 已实现 | Phase 1 |

finance-brief 当前落地位置：Layer 0-2 全部对齐。apps/desktop 是
flutter-samples 的镜像（Rust 入口 + baked-in Octoscript bundle）。

---

## §3. finance-brief 当前形态

### §3.1 文件树

```
finance-brief/
├── apps/
│   └── desktop/                              # ✅ Rust app entry（仿 flutter-samples）
│       ├── Cargo.toml                        # name = finance_brief_desktop (cdylib+rlib+bin)
│       └── src/
│           ├── main.rs                       # 11 行：fn main() → finance_brief_desktop::app_main()
│           ├── lib.rs                        # ~50 行：pub mod + script_mod! 块 + tests
│           ├── baked.rs                      # ~35 行：kit! + BAKED + DEVICE_PATH
│           ├── app.rs                        # ~140 行：App struct + mount + AppMain
│           ├── nav.rs                        # ~30 行：TAPS + register_nav
│           └── datasources/
│               ├── mod.rs                    # register_capability_handlers + 模块表
│               ├── news.rs                   # news_refresh, news_read
│               ├── quote.rs                  # quote_snapshot_a/us/crypto/fx, quote_candles
│               ├── stream.rs                 # stream_subscribe/unsubscribe/frequency_set/tick
│               ├── datasource_status.rs      # datasource_status
│               ├── research.rs               # research_list, research_read
│               └── local.rs                  # fav_list, fav_toggle, settings_load, settings_save
├── bundle/                                     # ✅ 提交单元
│   ├── screens/                                # ✅ 14 .octoscript 文件
│   │   ├── _kit.octoscript                     # 颜色 token + 复用 fn
│   │   ├── _index.octoscript                   # router
│   │   └── <screen_name>.octoscript × 12       # 12 屏
│   ├── workflow.octoscript                     # ✅ 7 workflow fn（已挂上 screens）
│   ├── capabilities.toml                       # ✅ 17 capability 声明
│   ├── schema/                                 # ✅ 17 JSON schema
│   ├── kit/                                    # 144 palette / axis token 文件
│   ├── assets/  listing.json  screenshots/
│   └── manifest.json                           # ✅ app 元数据 + 5 host 白名单
├── native/
│   ├── src/adapters/                           # ✅ 9 Rust adapter
│   │   ├── mod.rs
│   │   ├── news_sina.rs  / quote_tencent.rs  / quote_stooq.rs
│   │   ├── quote_hyperliquid.rs  / quote_frankfurter.rs
│   │   ├── synth_candles.rs  / stream_subscribe.rs
│   │   ├── stream_tick.rs  / datasource_status.rs
│   └── capabilities.toml                       # 实际位于 native/
├── scripts/                                    # 启动 / 安装脚本
├── docs/                                       # 平台分层 + catalog 文档
└── .todo-*.md                                  # gitignored
```

### §3.2 渲染管线（octoscript-makepad 完整管线）

```
bundle/screens/*.octoscript (14 files, baked-in BAKED in lib.rs)
    ↓ octoscript-render::build (parse + walk, in makepad-script VM)
    ↓ UiNode tree
    ↓ octoscript-makepad::to_makepad_ui (translation)
    ↓ makepad dialect string
    ↓ cx.with_vm (主 VM 评估，与 Splash 同 heap)
    ↓ Splash.view
    ↓ Makepad Widget Tree + Native Widgets (GPU + touch + resize)
```

**关键源码锚点**：
- `apps/desktop/src/lib.rs:74-93` — `BAKED` const（顺序：`_kit`, screens × 12, `_index`）
- `apps/desktop/src/lib.rs:230-280` — `mount`（eval + assign `Splash.view`）
- `Octoscript-Makepad/apps/flutter-samples/src/lib.rs:228-283` — 主版本 `mount`
- `Octoscript-Makepad/crates/octoscript-makepad/src/lib.rs` — `to_makepad_ui`

### §3.3 数据流（host.fetch → Rust adapter）

```
.octoscript screen (`sget(key, default)` + `tapto: "set:..."`)
    ↓ host.fetch("cap.X", args)
    ↓ apps/desktop/src/datasources.rs (mod.fb.<cap> registered)
    ↓ native/src/adapters/<x>.rs (实际 fetch + parse + cache)
    ↓ cache 写入（per docs/SPLASH-STYLE.md §4.4 fire-and-forget）
    ↓ 下一帧 sget 读到 → 屏重 render
```

详细 adapter 列表见 §7。

---

## §4. 模块边界

| 关注点 | Octoscript 屏 | workflow | capability | adapter | shell | 文件 |
|--------|--------------|----------|------------|---------|-------|------|
| UI 渲染（widget 树） | ✅ | — | — | — | — | — |
| 数据占位 `sget(key, default)` | ✅ | — | — | — | — | — |
| 数据流步骤编排 | — | ✅ | — | — | — | — |
| 能力声明（host.call 名） | — | — | ✅ | — | — | — |
| 能力执行（实际 fetch/parse） | — | — | — | ✅ | — | — |
| 数据契约（schema 校验） | — | — | ✅ | ✅ | — | — |
| 卡片渲染挂载 | — | — | — | — | ✅（OctoSense shell） | — |
| 应用切换 / 状态可检查 | — | — | — | — | ✅（OctoSense shell） | — |
| 渲染管线 baked 拼装 | — | — | — | — | — | `apps/desktop/src/baked.rs` |
| App struct + AppMain hookups | — | — | — | — | — | `apps/desktop/src/app.rs` |
| 路由（NAV polling） | — | — | — | — | — | `apps/desktop/src/nav.rs` |
| `host.fetch` 桥到 native adapter | — | — | — | — | — | `apps/desktop/src/datasources/` |

**原则**：
- Octoscript 屏是 declarative 数据，不包含计算逻辑（除了 L1 arithmetic，per `R-4-l0-cards.md §5`）
- workflow 是能力调用序列，不直接 fetch
- capability 是声明（名 + schema），不实现
- adapter 是实现（实际 fetch + parse + cache），用 `JsonToolContract` 校验
- shell 负责挂载 + 切换，不实现业务

---

## §5. capability 清单

17 个 capability，对应 `apps/desktop/src/datasources.rs` 中 18 个
`mod.fb.<cap>` 注册（`quote.snapshot` 有 4 个 tab 变体）。

详见 `docs/R-3-octoscript-platform.md §5.0.1` 与
`docs/WIDGET-COMPAT.md`（widget 兼容性 map）。Schema 在 `bundle/schema/*.schema.json`。

---

## §6. workflow 编排（workflow.octoscript）

**当前文件**（`bundle/workflow.octoscript`，162 行）：7 fn 已挂到 screens：

- `refresh_workflow()` —— 冷启动，screens 在 mount 时通过 `host.fetch` 触发
- `load_settings_workflow()` —— 屏 9 settings load
- `toggle_fav_workflow(kind, key)` —— 屏 8 收藏切换
- `load_detail_workflow(key)` —— 屏 3 / 5 详情
- `stream_tick_workflow()` —— 屏 11 后台 tick
- `datasource_status_workflow()` —— 屏 12 数据源矩阵
- `settings_save_workflow(patch)` —— 屏 9 设置保存

> ✅ **已对齐目标**：v2 中间形态下 `workflow.octoscript` 已写但 main.splash 不调用；v3 单一方案下 screens 通过 `mod.fb.<cap>` 真正触发 workflow 函数。

---

## §7. Rust adapter 实现（native/src/adapters/）

每个 adapter 实现 `JsonToolContract`（v2 形态保留，未改）：

| # | adapter file | 注册 capability | host | 状态 |
|---|--------------|----------------|------|------|
| 1 | `datasource_status.rs` | `datasource.status` | 聚合 | ✅ |
| 2 | `news_sina.rs` | `news.refresh` / `news.read` | `feed.mix.sina.com.cn` | ✅ |
| 3 | `quote_tencent.rs` | `quote.snapshot` (tab=a/hk) | `qt.gtimg.cn` | ✅ |
| 4 | `quote_stooq.rs` | `quote.snapshot` (tab=us) | `stooq.com` | ✅ |
| 5 | `quote_hyperliquid.rs` | `quote.snapshot` (tab=crypto) | `api.hyperliquid.xyz` | ✅ |
| 6 | `quote_frankfurter.rs` | `quote.snapshot` (tab=fx) | `api.frankfurter.dev` | ✅ |
| 7 | `stream_subscribe.rs` | `stream.subscribe` / `stream.unsubscribe` / `stream.frequency.set` | —（内部状态） | ✅ |
| 8 | `stream_tick.rs` | `stream.tick` | hyperliquid/orderbook + mock ticker | ✅ |
| 9 | `synth_candles.rs` | `quote.candles` | 合成 | ✅ |

> **测试**：`mod.rs::tests::adapter_module_count_is_nine` 已断言 9 个 module；
> `register_all_adapter_accepts_mut_runtime` 签名冒烟。

---

## §8. 渲染管线

> **v3 对齐状态（2026-10-04）**：✅ 单一 Octoscript 方案已 work。
> 当前实现基线：`feat/one-octoscript` 分支 HEAD。

### §8.1 完整管线（octoscript-makepad）

详见 §3.2。**单一路径**：所有屏走同一条线，无平行分支。

### §8.2 入口与汇编

**入口**：`apps/desktop/src/lib.rs::BAKED` const 固定拼接顺序：
`_kit` → 12 screens（launcher, news_list, ..., disclaimer） → `_index`。

**汇编时机**：编译时通过 `include_str!` 嵌入二进制；运行时无磁盘读
（hot-reload override 是可选项，per `apps/desktop/src/lib.rs::DEVICE_PATH`）。

### §8.3 K 线图（屏 6）

使用 `mod.plot.CandlestickChart` widget（per `docs/KLINE-WIDGET.md`）。
`apps/desktop/src/lib.rs::script_mod` 必须调用
`makepad_plot::script_mod(vm)` 来注册 widget —— v3 当前未做，记入 §12 next。

---

## §9. shell 集成（OctoSense + Octos）

### §9.1 OctoSense shell（Phase 4）

🎯 **目标**：finance-brief 作为 fullscreen app 从 OctoSense shell 启动。

**阻塞项**：OctoScript#56（splash widget tree → L0 ledger 桥接）——
https://github.com/OctoSense-org/OctoScript/issues/56

> **状态**：🎯 目标。当前未尝试 shell 集成。

### §9.2 Octos Agent（Phase 5）

🎯 目标，不在 v3 scope。

---

## §10. octoscode 入口（Phase 6）

🎯 目标，不在 v3 scope。

---

## §11. 数据源表

同 README §2。

---

## §12. 下一步

| 步骤 | 状态 |
|------|------|
| `apps/desktop/src/datasources.rs` 9 fn 接 9 adapter | 🎯 TODO |
| `apps/desktop/src/lib.rs::script_mod` 注册 `makepad_plot` widget | 🎯 TODO |
| `bundle/screens/*.octoscript` 在真机 / 桌面验证 12 屏全部 render | 🎯 TODO |
| `octoscode` CLI 集成 | 🎯 Phase 6 |
| OctoSense shell fullscreen launch | 🎯 Phase 4（依赖 #56） |

---

## §13. 变更日志

| 日期 | 改动 | commit |
|------|------|--------|
| 2026-10-04 | apps/desktop 重设计：拆 3 文件为 9 文件，crate 改名 finance-brief → finance_brief_desktop | (commit pending) |
| 2026-10-04 | v3: 单一 Octoscript 方案迁移；删 main.splash + 12 .card + 6 splash docs；新建 apps/desktop/ + bundle/screens/ | `feat/one-octoscript` 分支 |
| 2026-10-03 | v2: splash DSL flat-let 单文件；workflow.octoscript 写但未挂 | `b19656c` |
| 2026-10-02 | v1: 1689 行 main.splash；用户决策切到 octoscript 平台 | — |

---

## 附录 A：v3 决策附录

| # | 决策 |
|---|------|
| Q1 | UI 表达层 = Octoscript L0/L1 卡片（declarative） |
| Q2 | 数据层 = Octoscript workflow + capability + schema contract |
| Q3 | 渲染管线 = Octoscript-Makepad 完整管线 |
| Q4 | 平台形态 = flutter-samples 镜像（Rust 入口 + baked-in Octoscript bundle） |
| Q5 | 范围 = 12 屏 + 1 备用槽 = 13 屏（实际 v3 把 #10 免责声明并入 launcher tile，13 屏） |
| Q6 | 提交策略 = `feat/one-octoscript` 分支，单 Octoscript commit；通过 review 后合 main |
| Q7 | 谁写代码 = 主 AI（splash DSL → Octoscript 是机械翻译，可由主 AI 一次走完） |
| Q8 | 改哪个仓库 = octoOs/finance-brief（octoOs = OctoSenseorg） |