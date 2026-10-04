# finance-brief 架构（v2 重写目标 + 现状对齐）

> 适用版本：`finance-brief v0.3.0-dev`（octoscript 平台迁移期）
> 仓库：`git@github.com:ld1287/finance-brief.git`
> 文档目的：记录 **v2 重写目标架构**，并对每节标注 **当前完成状态**
> 当前实现基线：`.todo-c-path3-reality-2026-10-04.md`（gitignored）

本文档 = v2 重写目标架构；**当前实现状态** 详见 `.todo-c-path3-reality-2026-10-04.md`。
"已完成"指代当前 `HEAD = 73637fe` 工作树真实落地的内容（Path 1 card-host 唯一交互 UI）；
"目标"指代 octoscript 平台分层全面接入后（Phase 4-6）的形态。

---

## §0 状态速览

四种状态标记：✅ 已实现 / ⚠️ partial / 🎯 目标 / ❌ 未存在。

| 节 | 状态 |
|----|------|
| §1 背景与目标 | ✅ |
| §2 平台组件分层 | L0/L1/L2 ✅；L3 ❌；L4/L5 🎯 |
| §3 finance-brief 形态 | ⚠️ partial（entry = `bundle/main.splash`） |
| §4 模块边界 | 🎯（当前绕过） |
| §5 capability 清单 | ✅ 17 个已声明 |
| §6 workflow 编排 | ⚠️ partial（`workflow.octoscript` 已写；`main.splash` 直接 `host.call`） |
| §7 adapter 实现 | ✅（`native/src/adapters/`，9 个） |
| §8 渲染管线 | ✅（Path 3 lite 已对齐；Path 1 保留为 script app 平行分支） |
| §9.1 shell 集成 | 🎯（待 OctoScript#56） |
| §9.2 Octos Agent | 🎯 |
| §10 octoscode | 🎯 |
| §11 数据源 | ✅ |
| §12 Phase 计划 | Phase 1 ✅ / 2-3 ⚠️ / 4-6 🎯 |

---

## §1. 背景与目标

`finance-brief` 是 OctoSense 应用（独立 git 仓库），落地在 octoscript 平台分层之上。

**v1 痛点**（2026-10-02 之前）：

- `bundle/main.splash` 单文件承担 UI + 5 个数据源 fetch + 状态机 + cache + CRUD（v1 末期 1689 行 → 当前 572 行）
- 手写 `fetch_get` / `parse_sina_news` / `cache_*.json` —— 违反 octoscript capability-first
- splash 闭包命名 widget path 触发死循环（view_quotes 黑屏 3 次 fix 未根治）
- 没有数据契约（`host.call` 返回什么靠约定）
- 没有 workflow（数据流手写）
- 用户决策（2026-10-02）：切到 octoscript 平台分层

**v2 目标**：

- UI 用 **octoscript L0 卡片**（declarative, data-only）→ 解决黑屏死循环
- 数据用 **octoscript workflow + capability + schema contract** → 解决契约问题
- 渲染用 **octoscript-makepad 管线** → 解决 native widget 渲染
- 通过 **OctoSense shell + Octos Agent + Octoscript-AppCard** 集成 → 解决宿主 / Agent / 应用层支撑
- 项目结构按 octoscript 平台分层，参考 `octoscript-ui-l0/tests/fixtures/trip_planner.octoscript`

---

## §2. 平台组件分层（核心架构）

finance-brief 是**多个独立组件的组合**，每个组件有自己的职责。

| Layer | 名称 | 位置 | 状态 | Phase |
|-------|------|------|------|-------|
| 5 | OctoSense shell（宿主） | `octoOs/OctoSense/` | 🎯 目标 | Phase 4 |
| 4 | Octos Agent 内核 | `octoOs/octos/`（待发现） | 🎯 目标 | Phase 5 |
| 3 | Octoscript-AppCard 应用层 | `OctoSense-App-Hub/skills/card-studio/` | ❌ 未存在 | Phase 0 |
| 2 | Octoscript 核心 runtime | `octoOs/octoscript/` | ✅ 已实现 | Phase 1 |
| 1 | Octoscript-Makepad 渲染管线 | `octoOs/octoscript-makepad/` | ✅ 已实现 | Phase 1 |
| 0 | Makepad 底层 | `octoOs/makepad/` | ✅ 已实现 | Phase 1 |

**Layer 2（Octoscript 核心）子模块**：canonical v0.2 grammar；`octoscript-ui-l0`（check_syntax / realize / 卡片 store）；`octoscript-workflow`（dataflow + 步骤编排 + checkpoint）；`octoscript-capabilities`（capability + audit + lease）；`octoscript-schema`（JSON tool contract）；`octoscript-core / -storage / -protocol / -worker`。

**Layer 1（Octoscript-Makepad）子模块**：`octoscript-render`（VM → UiNode tree）；`octoscript-makepad`（UiNode → makepad dialect）；`octoscript-widgets`（themed native widget kit）；`makepad-d3` / `makepad-plot`（图表 widget）；`components/{material,flutter,...}`（主题组件库）。

**Layer 3 现状**（per ARCH-PLATFORM-REALITY.md §3.2）：无独立目录、无 cargo crate、无 manifest.json；12 个 `.card` 文件仅作 OctoSense shell glance tile 渲染（Layer 5 边界），不是 Agent 输入模板（待 #56 桥接）。

**持续支撑**：
- octoscode（开发者与编码 Agent 协作的终端入口）── 🎯 Phase 6
- OctoSense-App-Hub（应用商店 + 模板 + 卡片设计工具）

**当前 finance-brief 落地位置**：**仅走 Path 1（card-host via OctoSense-App-Hub）**。card-host = OctoSense-App-Hub 提供的 Rust 二进制，读 `bundle/main.splash` + 加载 `native/src/adapters/` 作为 host.call backend。不走 octoscript-makepad 管线；不走 OctoSense shell fullscreen launch（需 #56）。

---

## §3. finance-brief 在新分层下的形态

### §3.1 目标形态（v2 重写）

```
finance-brief/                                          # 独立 git repo
├── bundle/                                             # OctoSense app bundle
│   ├── main.octoscript                                 # 🎯 L0 卡片入口（11 屏 declarative UI）
│   ├── workflow.octoscript                             # ⚠️ 已写（7 fn），未挂到 main
│   ├── capabilities.toml                               # ✅ 实际位于 native/（不在 bundle/）
│   ├── schema/                                         # ✅ news/quote/candle/research/stream/settings/fav
│   ├── *.card × 12                                     # ⚠️ 当前是 glance tile（path3）
│   │                                                   # 🎯 目标 = Octoscript-AppCard Agent 输入模板
│   ├── assets/  listing.json  screenshots/
│   └── manifest.json                                   # ✅ app 元数据 + 5 host 白名单
├── native/src/adapters/                                # ✅ 当前实际位置（per Q-C）
│   ├── mod.rs                                          # 入口 + register_all_adapter
│   ├── news_sina.rs  / quote_tencent.rs  / quote_stooq.rs
│   ├── quote_hyperliquid.rs  / quote_frankfurter.rs
│   ├── synth_candles.rs  / stream_subscribe.rs
│   ├── stream_tick.rs  / datasource_status.rs
├── sources/                                           # ✅ 已抓取的样本（验证用）：sina / tencent / hyperliquid / frankfurter / stooq
├── docs/                                             # MVP-TODO / ARCHITECTURE / DATA-SOURCES / PATH3-REALITY / R-3 / R-4
├── scripts/                                          # run-octosense.sh / run-finance-brief.sh / install-as-system-app.sh / drive-test.sh
├── .todo-*.md                                        # gitignored（本地协调档）
└── README.md
```

### §3.2 关键差异（目标 vs 当前）

| 项 | v2 目标 | 当前实际 |
|----|---------|----------|
| UI entry | `bundle/main.octoscript`（L0 declarative） | `bundle/main.splash`（572 行 splash DSL） |
| workflow | `workflow.plan([...])` 编排步骤 | `host.call("cap.X", {...})` 直接调（`main.splash`） |
| capabilities 声明 | `bundle/capabilities.toml`（`[[capability]]` 段） | 实际位于 `native/capabilities.toml`（15 segment / 17 name） |
| `.card` 文件 | Octoscript-AppCard Agent 输入模板（Layer 3） | OctoSense shell glance tile（Layer 5 边界） |
| 渲染管线 | octoscript-makepad（VM → UiNode → makepad dialect） | card-host → makepad（直接 splash DSL → widget） |
| Shell 集成 | fullscreen app launch（Phase 4） | glance tile（无 fullscreen，待 OctoScript#56） |

---

## §4. 模块边界

**清晰划线**：每个模块只做一件事。

| 关注点 | L0 卡片 | workflow | capability | adapter | shell |
|--------|---------|----------|------------|---------|-------|
| UI 渲染（widget 树） | ✅ | — | — | — | — |
| 数据占位 `{{state.x}}` | ✅ | — | — | — | — |
| 数据流步骤编排 | — | ✅ | — | — | — |
| 能力声明（host.call 名） | — | — | ✅ | — | — |
| 能力执行（实际 fetch/parse） | — | — | — | ✅ | — |
| 数据契约（schema 校验） | — | — | ✅ | ✅ | — |
| 卡片渲染挂载 | — | — | — | — | ✅（OctoSense shell） |
| 应用切换 / 状态可检查 | — | — | — | — | ✅（OctoSense shell） |
| 执行失败 → 反映到界面 | — | — | — | — | ✅（Octos Agent） |

> **原则**：
> - L0 卡片是 declarative 数据，不包含计算逻辑
> - workflow 是能力调用序列，不直接 fetch
> - capability 是声明（名 + schema），不实现
> - adapter 是实现（实际 fetch + parse + cache），用 `JsonToolContract` 校验
> - shell 负责挂载 + 切换，不实现业务
>
> **现状**：§4 的模块边界**目标状态** —— 当前 `main.splash` 把 UI + 数据编排 + 能力调用混在一起；§4 是 v2 拆开后的目标边界。

---

## §5. capability 清单

### §5.1 capability 清单（**17 个**）

声明位于 `native/capabilities.toml`（15 个 `[[capability]]` segment，17 个 capability 名：fav/settings 各合并 1 个 segment）；实际调用位于 `bundle/main.splash` 行 80-141 的 15 个 `host_*` 包装。

| # | capability 名 | 类型 | adapter | host / 协议 | 说明 |
|---|--------------|------|---------|-------------|------|
| 1 | `news.refresh` | fetch | `news_sina` | `feed.mix.sina.com.cn` | 拉 Sina 要闻 RSS |
| 2 | `news.read` | fetch | `news_sina` | `feed.mix.sina.com.cn` | 单条新闻详情 |
| 3 | `quote.snapshot` (tab=a/hk) | fetch | `quote_tencent` | `qt.gtimg.cn` | A 股 / 港股快照 |
| 4 | `quote.snapshot` (tab=us) | fetch | `quote_stooq` | `stooq.com` | 美股快照（per Q-R1） |
| 5 | `quote.snapshot` (tab=crypto) | fetch | `quote_hyperliquid` | `api.hyperliquid.xyz` | 加密币 mid |
| 6 | `quote.snapshot` (tab=fx) | fetch | `quote_frankfurter` | `api.frankfurter.dev` | ECB 日级汇率 |
| 7 | `quote.candles` | fetch | `synth_candles` | 合成 OHLC | K 线（demo 数据） |
| 8 | `research.list` | fetch | inline | （card-host 内部） | 研究卡列表 |
| 9 | `research.read` | fetch | inline | （card-host 内部） | 研究卡详情 |
| 10 | `stream.subscribe` | call | `stream_subscribe` | `api.hyperliquid.xyz` | 订阅 symbol 集合（per Q-B） |
| 11 | `stream.unsubscribe` | call | `stream_subscribe` | — | 取消订阅（per Q-B） |
| 12 | `stream.frequency.set` | call | `stream_subscribe` | — | 改 poll 频率（per Q-B） |
| 13 | `stream.tick` | fetch | `stream_tick` | hyperliquid/orderbook + mock | ticker + orderbook 共享（per Q-E） |
| 14 | `datasource.status` | fetch | `datasource_status` | 5 个 cap 聚合 | 数据源健康度矩阵 |
| 15 | `fav.list` | fetch | inline | `state.fav` | 收藏列表 |
| 16 | `fav.toggle` | call | inline | `state.fav` | 收藏 toggle |
| 17 | `settings.load` / `settings.save` | fetch/call | inline | `state.settings` | 读写设置（1 个 [[capability]] segment 两方向） |

> **状态**：✅ **17 个 capability 已声明 + 落地**。
> - 声明位置：`native/capabilities.toml`（15 个 `[[capability]]` segment；fav/settings 各为 1 segment 含 2 names）
> - 实调位置：`bundle/main.splash` 行 80-141 的 15 个 `host_*` 包装
> - adapter 注册位置：`native/src/adapters/mod.rs::register_all_adapter`

### §5.2 数据契约（schema/*.schema.json）

每个 schema 是 JSON Schema 子集（per `octoscript-schema` 实现限制）：

- types: null / boolean / number / integer / string / array / object
- object: properties / required / additionalProperties
- array: items / minItems / maxItems
- scalar: minimum / maximum / minLength / maxLength / enum
- **不支持**：`$ref`、allOf / anyOf / oneOf / not、regex、conditional schemas
- 单 schema 上限 32 KiB，嵌套上限 32 层
- 单 object properties ≤ 128，enum 值 ≤ 128

例：`quote.schema.json`

```json
{
  "type": "object",
  "properties": {
    "symbol": {"type": "string", "minLength": 1, "maxLength": 16},
    "name":   {"type": "string"},
    "last":   {"type": "number"},
    "change_pct": {"type": "number"},
    "ts":     {"type": "integer"},
    "kind":   {"type": "string", "enum": ["a", "us", "hk", "crypto", "fx"]}
  },
  "required": ["symbol", "last", "change_pct", "ts", "kind"],
  "additionalProperties": false
}
```

> **状态**：⚠️ schema 目录存在（`bundle/schema/`），但 **当前 `host.call` 不走 schema 校验**（card-host 简化版仅做 JSON parse + adapter 返回值透传）。
> v2 目标：所有 capability 走 `JsonToolContract` 校验输入输出（per §7）。

### §5.3 L0 卡片 ↔ capability 调用

**v2 目标**：

L0 卡片**不直接调用 capability**，而是：
1. workflow 在 step 里 `use mod.cap.refresh`（declarative 步骤）
2. workflow 的每个 step 输出走 schema 校验
3. 校验后的数据通过 `{{state.x}}` 注入 L0 卡片

L0 卡片示例（参考 `octoscript-ui-l0/tests/fixtures/trip_planner.octoscript`）：

```
let rows = "{{state.news_rows}}"
let busy = "{{state.busy}}"
let err  = "{{state.last_err}}"

View {
    width: Fill
    height: Fill
    flow: Down

    header := Label { text: "新闻简报" }
    status := Label { text: busy ? "刷新中..." : err != "" ? "来源不可用" : "更新于 " + state.updated }

    list := View {
        for r in rows {
            RowCard {
                title: r.title
                meta: r.source
                on_tap: |r| { nav.push("news.detail", {key: r.key}) }
            }
        }
    }
}
```

> **当前**：`main.splash` 不通过 `workflow.plan(...)`，而是每屏在 `loaded_X = false` 的 `View` 块里直接 `let r = host.call("news.refresh", {limit: 30}); loaded_X = true`。这绕过了 §6 的 workflow 编排。

---

## §6. workflow 编排（workflow.octoscript）

**当前文件已存在**（`bundle/workflow.octoscript`，2026-10-03 落地），但 **未被 main.splash 调用**。

```
use mod.cap.news
use mod.cap.quote
use mod.cap.research
use mod.cap.fav
use mod.cap.settings
use mod.cap.stream
use mod.cap.datasource

let default_orderbook_symbols = ["AAPL", "MSFT", "TSLA", "BTC", "ETH", "SOL"]
let default_news_limit = 30
let default_ticker_limit = 10

fn refresh_workflow() {
    let plan = workflow.plan([
        settings.load({}),
        fav.list({}),
        workflow.parallel([
            news.refresh({limit: 30}),
            quote.snapshot({tab: "a"}),
            quote.snapshot({tab: "us"}),
            quote.snapshot({tab: "crypto"}),
            quote.snapshot({tab: "fx"}),
            research.list({}),
        ]),
        workflow.log({msg: "refresh ok", kind: "cold_start"}),
    ])
    plan.execute()
}
```

7 个 workflow 函数（per workflow.octoscript）：
- `refresh_workflow` —— 冷启动（settings + fav + 6 路并行）
- `load_settings_workflow` —— #9 settings load
- `toggle_fav_workflow(kind, key)` —— #8 收藏切换
- `load_detail_workflow(key)` —— #3 / #5 detail 屏
- `stream_tick_workflow()` —— #11 后台 tick（per Q-E：hyperliquid/orderbook + mock ticker）
- `datasource_status_workflow()` —— #12 数据源矩阵
- `settings_save_workflow(patch)` —— #9 settings save

> **状态**：⚠️ **partial**。
> - 文件落地 ✅
> - 但 `main.splash` 不调用 —— splash DSL 的执行模型把"步骤"展开为 `host.call(...)` 直接调用 + `loaded_X` flag
> - 真正进入 v2 路径需要 `bundle/main.octoscript` 替换 `main.splash`（Phase 5+ 之后）

---

## §7. Rust adapter 实现（native/src/adapters/）

每个 adapter 实现 `JsonToolContract`：

```rust
// native/src/adapters/quote_tencent.rs
use octoscript_capabilities::{json, JsonToolContract, CapabilityRuntime};

pub fn register(rt: &mut CapabilityRuntime) {
    let contract = JsonToolContract::new(
        json!({
            "type": "object",
            "properties": {
                "tab": {"type": "string", "enum": ["a", "us", "hk"]},
                "symbols": {"type": "array", "items": {"type": "string"}}
            },
            "required": ["tab"],
            "additionalProperties": false
        }),
        json!({
            "type": "object",
            "properties": {
                "rows": {"type": "array", "items": {"$ref": "quote.schema.json"}}
            },
            "required": ["rows"],
            "additionalProperties": false
        }),
    );
    rt.register_validated_json_tool("quote.tencent", contract, handle);
}
```

**当前 9 个 adapter**（per `native/src/adapters/mod.rs`）：

| # | adapter file | 注册 capability | host | 状态 |
|---|--------------|----------------|------|------|
| 1 | `datasource_status.rs` | `datasource.status` | 聚合 | ✅ |
| 2 | `news_sina.rs` | `news.refresh` / `news.read` | `feed.mix.sina.com.cn` | ✅ |
| 3 | `quote_tencent.rs` | `quote.snapshot` (tab=a/hk) | `qt.gtimg.cn` | ✅ |
| 4 | `quote_stooq.rs` | `quote.snapshot` (tab=us) | `stooq.com` | ✅（per Q-R1 替换 nasdaq） |
| 5 | `quote_hyperliquid.rs` | `quote.snapshot` (tab=crypto) | `api.hyperliquid.xyz` | ✅ |
| 6 | `quote_frankfurter.rs` | `quote.snapshot` (tab=fx) | `api.frankfurter.dev` | ✅ |
| 7 | `stream_subscribe.rs` | `stream.subscribe` / `stream.unsubscribe` / `stream.frequency.set` | —（内部状态） | ✅ |
| 8 | `stream_tick.rs` | `stream.tick` | hyperliquid/orderbook + mock ticker | ✅（per Q-E 共用） |
| 9 | `synth_candles.rs` | `quote.candles` | 合成 | ✅ |

> **测试**：`mod.rs::tests::adapter_module_count_is_nine` 已断言 9 个 module；`register_all_adapter_accepts_mut_runtime` 签名冒烟。
>
> **adapter 路径**：`native/src/adapters/`（per Q-C）；早期假设的 `adapters/` 顶层目录**不存在**。

---

## §8. 渲染管线（octoscript-makepad）

> **v2 对齐状态（2026-10-04）**：✅ Path 3 lite 已对齐目标管线；⚠️ Path 1 保留为 script app 平行分支。
> 当前实现基线 `HEAD = 73637fe`：两条路径**都已 work**。

### §8.1 目标管线 — Path 3 lite（OctoSense shell glance tile，✅ 已对齐）

```
*.card (L0 ledger, 12 静态模板，无 sys.X)
    ↓ octoscript-ui-l0::check_ui_l0 (validate against catalog)
    ↓ octoscript-ui-l0::realize (tree walk + data substitution)
    ↓ octoscript-ui-l0::makepad::lower (UiNode → Makepad DSL 文本)
    ↓ octoscript-makepad::to_makepad_l0_ui (设计层翻译)
    ↓ Splash VM (build_with_capabilities, OctoSense-App-Hub card-host)
    ↓ Makepad Widget Tree + Native Widgets (GPU + touch + resize)
```

**关键源码锚点**：

- `OctoSense-App-Hub/crates/card-host/src/host.rs:115-134` — `card_source()`（区分 script app vs page.card）
- `OctoSense-App-Hub/crates/card-host/src/host.rs:144-154` — `lower()` 调 `octoscript_makepad::l0::prepare` + `to_makepad_l0_ui`
- `octoscript-makepad/crates/octoscript-makepad/src/l0.rs:47-54` — `prepare_with_state`（kit pipeline 入口）

**入口**：`bundle/{launcher, news_list, news_detail, research_list, research_detail, quote_list, kline, favorites, settings, event_stream, datasource_status, disclaimer}.card` × 12

### §8.2 Path 1 — script app 入口（保留，与 octoscript-makepad 平行但共享 widget 层）

**入口**：`bundle/main.splash`（572 行 splash DSL，flat-let 单文件 per `.todo-re-arch-2026-10-04.md`）

**流程**：

1. `octosense_app_policy::script_source(bundle, asset_origin)` —— 命中 script app，返回 `main.splash` 内容
2. 直接进入 Splash VM 求值（**不走** octoscript-ui-l0 检查 + realize + lower）
3. Makepad widget tree → GPU 渲染

**关键源码锚点**：

- `OctoSense-App-Hub/crates/card-host/src/host.rs:117-119` — script_source 返回路径（`card_source()` 第一条分支）
- `bundle/main.splash` — 入口文件（顶部注释：`single-file splash application (B-0 flat-let rewrite)`）
- `scripts/run-finance-brief.sh` — 启动命令

### §8.3 两条路径对比

| 维度 | Path 3 lite | Path 1 |
|------|-------------|--------|
| 入口 | 12 个 `.card` 静态模板 | `main.splash` splash DSL |
| VM 评估 | `octoscript-ui-l0::check_ui_l0 + realize` + `octoscript-makepad::to_makepad_l0_ui` | Splash VM 直接求值 |
| 渲染管线 | octoscript-makepad 完整管线 | 平行分支（绕过 L0 检查） |
| Widget 终点 | Makepad Widget Tree | Makepad Widget Tree |
| 适用场景 | OctoSense shell glance tile | card-host 交互 UI |
| 当前状态 | ✅ 已 work | ✅ 已 work（`HEAD = 73637fe`） |

> **关键 takeaway**：
> 1. **octoscript-makepad 管线已对齐目标**（Path 3 lite via static .card）
> 2. **Path 1 是独立分支**（script app 入口），与 octoscript-makepad **共用 widget 层**但**不共用 VM 评估层**
> 3. **未来统一路径**：当 OctoScript#56 桥接 splash widget tree → L0 ledger 完成后，Path 1 可被 Path 3 lite 完全替代（届时把 `main.splash` 替换为 `main.octoscript` 或 `page.card`）

---

## §9. shell 集成（OctoSense + Octos）

### §9.1 OctoSense shell（Phase 4）

**目标**：
- 启动 `OctoSense/desktop/` 或 `OctoSense/phone/` 二进制
- shell 加载 `OctoSense/apps/finance-brief/bundle/`（由 `install-as-system-app.sh` 同步）
- 卡片在 card-host 渲染（用 octoscript-makepad 管线）
- 用户操作 → widget event → VM 重新评估 → 卡片更新
- 应用切换：shell 在 `OctoSense/apps/finance-brief/` ↔ 其它应用

**当前实际**（per `.todo-c-path3-reality-2026-10-04.md` §1）：

- ✅ **Path 1（splash → card-host → makepad）**：当前唯一交互 UI，已工作（`scripts/run-finance-brief.sh`）
- ⚠️ **Path 3 glance tile**：12 个 `.card` 文件被 shell 当 L0 ledger 渲染成静态瓦片（"启动入口"占位）—— **不是设计意图**，是意外副产品（待 #56 桥接才能成为真 AppCard Agent 输入模板）
- ❌ **Path 3 fullscreen launch**：需 OctoScript#56（`https://github.com/OctoSense-org/OctoScript/issues/56`），**不实现**
- ❌ **host-service 模式**：OctoSense shell 不支持动态 host-service 注册（`OctoSense/crates/shell/src/apps.rs:184-220` 全部 `octosense_*_service::register()` 编译期绑定），**不实现**

> **状态**：🎯 **目标**（Phase 4 fullscreen launch）；当前 Path 3 跑不通，详见 `.todo-c-path3-reality-2026-10-04.md`。

### §9.2 Octos Agent（Phase 5）

**目标**：
- finance-brief 启动 agent 任务（如"分析 A 股 K 线趋势"）
- Octos 内核调度 agent（模型、工具、会话）
- 执行进展 → 通过 `fb.action` API 反映到界面（per Octoscript-AppCard）
- 任务失败 → 通过 capability error 反映到界面（状态栏 + toast）

> **状态**：🎯 **目标**（Phase 5）；当前 Octos 内核项目未对 finance-brief 开放。

---

## §10. octoscode 入口（Phase 6）

```
$ octoscode assign "改 finance-brief 新闻屏的 K 线按钮"
    ↓ 主 AI 派 sub-agent
$ sub-agent 修改 bundle/main.octoscript
    ↓ commit（per finance-brief-skill）
$ octoscode verify
    ↓ 自动跑：octoscript check + cargo test + card-host 渲染
$ octoscode show diff
    ↓ 开发者看修改
$ octoscode revert / approve
```

> **状态**：🎯 **目标**（Phase 6）；当前 sub-agent 流程已 work（`.agents/skills/`），但 `octoscode` CLI 集成未实现。

---

## §11. 数据源表

**5 个免费源 + 1 个 socket 协议**（per `bundle/manifest.json#network.hosts` + `native/src/adapters/`）：

| # | 源 | URL / 协议 | 格式 | 限频 | API key | adapter | 备注 |
|---|----|----|------|------|---------|---------|------|
| 1 | Sina 财经要闻 | `https://feed.mix.sina.com.cn/api/rollout?...` | JSON（伪 RSS） | 无 | 否 | `news_sina` | 主新闻源 |
| 2 | Tencent 行情快照 | `https://qt.gtimg.cn/q={symbols}` | 文本（GBK） | 无 | 否 | `quote_tencent` | A 股 / 港股 / 美股昨收 |
| 3 | Hyperliquid 加密币 | `https://api.hyperliquid.xyz/info` | JSON | 无 | 否 | `quote_hyperliquid` | POST `{type:"allMids"}` |
| 4 | Hyperliquid 订单流 | `wss://api.hyperliquid.xyz/ws` | WS / JSON | push | 否 | `stream_tick`（orderbook 部分） | 6 symbol 共享 ticker |
| 5 | Frankfurter 外汇 | `https://api.frankfurter.dev/v1/latest?base=USD&symbols=...` | JSON | 无 | 否 | `quote_frankfurter` | ECB 日级汇率 |
| 6 | **Stooq 美股**（per Q-R1） | `https://stooq.com/q/l/?s={symbol}&f=sd2t2ohlcv&h&e=csv` | CSV | 无 | 否 | `quote_stooq` | 美股实时 |

> 所有 host 已声明在 `bundle/manifest.json#network.hosts`（5 个 https 域 + wss），无需新增白名单。
>
> **历史变更**：美股源 per Q-R1 决策落地为 Stooq CSV。

---

## §12. 下一步（按 Phase）

| Phase | 内容 | 当前状态 |
|-------|------|---------|
| Phase 0（持续） | Octoscript-AppCard 文档 / 模板管理 | ⚠️ partial（12 个 `.card` 当前仅作 glance tile，非 Agent 模板） |
| Phase 1（前期） | R-3 / R-4 调研 → M-1/M-2/O-4/O-5/O-6/O-7 验证 | ✅ 已实现（octoscript + octoscript-makepad 平台层稳定） |
| Phase 2（前期→中） | F-CAP/F-L0-{1..11}/F-WF/F-ADP 设计 + 实现 | ⚠️ partial（capability 17 个 + 9 adapter ✅；L0 卡片未上线；workflow.octoscript 已写但未挂） |
| Phase 3（中） | C-TEST/C-RENDER 验证（card-host） | ✅ 已实现（Path 1 card-host + drive-test.sh） |
| Phase 4（中→后期） | S-SHELL 集成（OctoSense shell） | 🎯 目标（Path 3 fullscreen launch 待 OctoScript#56） |
| Phase 5（后期） | A-AGENT 集成（Octos Agent） | 🎯 目标 |
| Phase 6（持续） | D-CODE 集成（octoscode） | 🎯 目标 |

每个 Phase 完成后派 review agent 验收。

> **本节不做新规划**：详细 TODO 见 `.todo-c-path3-reality-2026-10-04.md`（gitignored）。

---

## §13. 变更日志

| 日期 | 变更 |
|------|------|
| 2026-10-01 | v1 初版（splash DSL 直写，1689 行） |
| 2026-10-02 | v2 重写（octoscript 平台分层；L0 卡片 + workflow + capability + adapter） |
| 2026-10-03 | Phase B-1/B-2 落地（adapters → `native/src/adapters/`；美股切 Stooq；17 capability；9 adapter） |
| 2026-10-04 | **现状对齐版**：双层文档（v2 目标 + 当前完成度），反映 Path 1 唯一交互 UI；标注 §3 / §5 / §6 / §8 / §9 / §12 的当前状态 |

---

## 附录 A：v3 决策附录（2026-10-02 用户确认）

| 决策 | 内容 | 影响 |
|---|---|---|
| Q-A | 9 L0 + 3 L1（K线 / 事件流 / 数据源状态页 state 部分） | UI 表达层；3 屏需 L1 expression |
| Q-B | + `stream.subscribe` / `stream.unsubscribe` / `stream.frequency.set` | capability catalog 从 14 → **17** |
| Q-C | adapters 集成 `native/src/adapters/` | 不新建独立 crate |
| Q-D | 1s/快讯 + 500ms/symbol + settings 可改 | #11 默认频率 + #9 加频率切换 UI |
| Q-E | #11 共用 hyperliquid/orderbook + mock 快讯 | 不再独立 mock；adapter `stream_tick.rs` |
| Q-R1 | 美股用 **Stooq CSV** | `quote_stooq.rs` 替换早期 `quote_nasdaq.rs` |
| Q-R2 | makepad CandlestickChart + L1 expression | K 线渲染路径 |
| Q-F | #12 = 5×8 字段（默认） | 数据源状态页表格 |

完整决策理由见 `.todo-octoscript-rewrite-2026-10-02.md` §2 + §2.1（gitignored）。