# finance-brief 架构（v2：Octoscript 平台分层）

> 适用版本：`finance-brief v0.4.0-dev`（octoscript 平台迁移期，2026-10-02 重写）
> 仓库：`git@github.com:ld1287/finance-brief.git`
> 文档目的：记录目标架构、平台分层、模块边界、L0 卡片 → adapters → shell 链路
> 重写背景：v1 用 splash DSL 直写 UI（1437 行混合 UI + 数据 + 业务）→ v2 切到 octoscript 平台分层

---

## 1. 背景与目标

`finance-brief` 是 OctoSense 应用（独立 git 仓库），落地在 octoscript 平台分层之上。

**v1 痛点**（2026-10-02 之前）：

- `bundle/main.splash` 1437 行，单文件承担 UI + 5 个数据源 fetch + 状态机 + cache + CRUD
- 手写 `fetch_get` / `parse_sina_news` / `cache_*.json` —— 违反 octoscript capability-first
- splash 闭包命名 widget path 触发死循环（view_quotes 黑屏 3 次 fix 未根治）
- 没有数据契约（host.call 返回什么靠约定）
- 没有 workflow（数据流手写）
- 用户决策（2026-10-02）：切到 octoscript 平台分层

**v2 目标**：

- UI 用 **octoscript L0 卡片**（declarative, data-only）→ 解决黑屏死循环
- 数据用 **octoscript workflow + capability + schema contract** → 解决契约问题
- 渲染用 **octoscript-makepad 管线** → 解决 native widget 渲染
- 通过 **OctoSense shell + Octos Agent + Octoscript-AppCard** 集成 → 解决宿主/Agent/应用层支撑
- 项目结构按 octoscript 平台分层，参考 `octoscript-ui-l0/tests/fixtures/trip_planner.octoscript`

---

## 2. 平台组件分层（核心架构）

finance-brief 是**多个独立组件的组合**，每个组件有自己的职责：

```
┌────────────────────────────────────────────────────────────┐
│ Layer 5  OctoSense shell（项目外，宿主）                    │
│   octoOs/OctoSense/                                        │
│   角色：用户打开 / 操作 / 切换应用                          │
│   卡片在同一应用环境中出现                                  │
│   状态与结果可以检查                                        │
│   时间分层：后期（Phase 4）                                │
├────────────────────────────────────────────────────────────┤
│ Layer 4  Octos Agent 内核（项目外，Agent）                  │
│   octoOs/octos/（待发现）                                  │
│   角色：执行模型、工具、会话、执行进展                      │
│   任务失败如何反映到界面                                    │
│   时间分层：后期（Phase 5）                                │
├────────────────────────────────────────────────────────────┤
│ Layer 3  Octoscript-AppCard 应用层（项目外，持续开发）      │
│   octoOs/OctoSense-App-Hub/skills/card-studio/             │
│   角色：把意图路由给应用 Agent                              │
│   管理生成卡片的应用层：请求、生成、校验、数据绑定、交互更新│
│   时间分层：持续（Phase 0）                                │
├────────────────────────────────────────────────────────────┤
│ Layer 2  Octoscript 核心 runtime（项目外，前期）            │
│   octoOs/octoscript/                                       │
│   角色：受约束的应用表达与能力机制                          │
│   UI L0 卡片路径、结构、数据来源、状态与事件如何被宿主接住│
│   - canonical v0.2 grammar                                 │
│   - octoscript-ui-l0（check_syntax / realize / 卡片 store）│
│   - octoscript-workflow（dataflow + 步骤编排 + checkpoint） │
│   - octoscript-capabilities（capability + audit + lease）  │
│   - octoscript-schema（JSON tool contract）                │
│   - octoscript-core / -storage / -protocol / -worker       │
│   时间分层：前期（Phase 1）                                │
├────────────────────────────────────────────────────────────┤
│ Layer 1  Octoscript-Makepad 渲染管线（项目外，前期）        │
│   octoOs/octoscript-makepad/                               │
│   角色：把界面变成原生控件和真实交互                        │
│   点击是否改变状态、窗口变化后能否继续用                    │
│   - crates/octoscript-render（VM → UiNode tree）           │
│   - crates/octoscript-makepad（UiNode → makepad dialect） │
│   - crates/octoscript-widgets（themed native widget kit）  │
│   - crates/makepad-d3 / makepad-plot（图表 widget）        │
│   - components/{material,flutter,...}（主题组件库）        │
│   时间分层：前期（Phase 1）                                │
├────────────────────────────────────────────────────────────┤
│ Layer 0  Makepad 底层（项目外，前期）                       │
│   octoOs/makepad/                                          │
│   角色：把界面变成原生控件和真实交互（点击、状态、resize）│
│   - View/Label/Button/CandlestickChart 等原生控件           │
│   - widget、shader、动画、window、touch、GPU               │
│   时间分层：前期（Phase 1）                                │
└────────────────────────────────────────────────────────────┘

持续支撑：
- octoscode（开发者与编码 Agent 协作的终端入口）
- OctoSense-App-Hub（持续开发的应用商店 + 模板 + 卡片设计工具）
```

---

## 3. finance-brief 在新分层下的形态

```
finance-brief/                                          # 独立 git repo
│
├── bundle/                                             # OctoSense app bundle
│   ├── main.octoscript                                 # L0 卡片入口（11 屏 declarative UI）
│   ├── workflow.octoscript                             # 数据流编排（capability 调用）
│   ├── capabilities.toml                               # 能力声明（host.call 名 + schema）
│   ├── schema/                                         # 数据契约
│   │   ├── news.schema.json
│   │   ├── quote.schema.json
│   │   ├── candle.schema.json
│   │   ├── research.schema.json
│   │   ├── stream.schema.json
│   │   ├── settings.schema.json
│   │   └── fav.schema.json
│   ├── assets/
│   │   └── icon.svg
│   ├── screenshots/                                    # 真实抓帧（card-host 渲染）
│   ├── listing.json
│   └── manifest.json                                   # app 元数据 + 能力清单
│
├── adapters/                                           # Rust adapters（Layer 1 桥）
│   ├── Cargo.toml
│   ├── src/
│   │   ├── lib.rs                                      # 入口 + 工具 contract 注册
│   │   ├── news_sina.rs                                # Sina RSS adapter
│   │   ├── quote_tencent.rs                            # Tencent 行情 adapter
│   │   ├── quote_hyperliquid.rs                        # Hyperliquid adapter
│   │   ├── quote_frankfurter.rs                        # Frankfurter adapter
│   │   ├── quote_nasdaq.rs                             # NASDAQ 美股 adapter
│   │   └── synth_candles.rs                            # 合成 K 线（demo 数据）
│   └── tests/
│       └── *.rs                                        # 单测（每个 adapter）
│
├── sources/                                            # 已抓取的样本（验证用）
│   ├── sina_2026-09-29.json
│   ├── tencent_*.txt
│   ├── hyperliquid_*.json
│   ├── frankfurter_*.json
│   └── nasdaq_*.json
│
├── fonts/                                              # NotoSansSC 中文字体
│
├── docs/                                               # 项目文档
│   ├── MVP-TODO.md                                     # 顶层规划
│   ├── ARCHITECTURE.md                                 # 本文档
│   ├── DATA-SOURCES.md                                 # 5 个数据源
│   ├── R-1-us-stocks.md                                # 美股实时源调研
│   ├── R-2-kline-feasibility.md                        # K 线可行性
│   ├── R-3-octoscript-platform.md                      # 【新】octoscript 平台调研
│   ├── R-4-l0-cards.md                                 # 【新】L0 卡片语法调研
│   └── screenshots/                                    # 文档截图
│
├── scripts/
│   ├── run-octosense.sh                                # 启动 OctoSense shell + load finance-brief
│   ├── verify.sh                                       # octoscript check + cargo test + L0 check
│   └── install-as-system-app.sh                        # 同步到 OctoSense/apps/finance-brief/
│
├── .todo-octoscript-rewrite-2026-10-02.md              # 本地协调（不入库）
└── README.md
```

---

## 4. 模块边界

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
> - adapter 是实现（实际 fetch + parse + cache），用 JsonToolContract 校验
> - shell 负责挂载 + 切换，不实现业务

---

## 5. L0 卡片 ↔ Rust adapters 通讯接口

### 5.1 capability 清单（capabilities.toml）

| capability 名 | 类型 | 输入 schema | 输出 schema | 说明 |
|--------------|------|------------|------------|------|
| `news.refresh` | fetch | `news.refresh.input.schema.json` | `news.schema.json` (rows) | 拉 Sina 要闻 RSS |
| `news.read` | fetch | `news.read.input.schema.json` | `news.schema.json` (single) | 单条新闻详情 |
| `quote.snapshot` | fetch | `quote.snapshot.input.schema.json` | `quote.schema.json` (rows) | 多源汇总行情 |
| `quote.candles` | fetch | `quote.candles.input.schema.json` | `candle.schema.json` (rows) | K 线 OHLC |
| `research.list` | fetch | `research.list.input.schema.json` | `research.schema.json` (rows) | 研究卡列表 |
| `research.read` | fetch | `research.read.input.schema.json` | `research.schema.json` (single) | 研究卡详情 |
| `stream.mock` | fetch | `stream.mock.input.schema.json` | `stream.schema.json` (rows) | 事件流 stub |
| `fav.list` | fetch | `fav.list.input.schema.json` | `fav.schema.json` (rows) | 收藏列表 |
| `fav.toggle` | call | `fav.toggle.input.schema.json` | `fav.schema.json` (single) | 收藏 toggle |
| `settings.load` | fetch | `settings.load.input.schema.json` | `settings.schema.json` | 读设置 |
| `settings.save` | call | `settings.save.input.schema.json` | `settings.schema.json` | 写设置 |

> 所有 capability 都通过 **JsonToolContract** 校验输入输出（per octoscript-schema 子集）

### 5.2 数据契约（schema/*.schema.json）

每个 schema 是 JSON Schema 子集（per `octoscript-schema` 实现限制）：

- types: null / boolean / number / integer / string / array / object
- object: properties / required / additionalProperties
- array: items / minItems / maxItems
- scalar: minimum / maximum / minLength / maxLength / enum
- **不支持**：$ref、allOf/anyOf/oneOf/not、regex、conditional schemas
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

### 5.3 L0 卡片调用 capability

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

> **注意**：L0 卡片在 realize 阶段才把 `{{state.x}}` 替换成实际数据；编译期不存在 data flow

---

## 6. workflow 编排（workflow.octoscript）

```
use mod.cap.news
use mod.cap.quote
use mod.cap.fav
use mod.cap.settings

let plan = workflow.plan([
    // 步骤 1: 加载设置
    settings.load(),

    // 步骤 2: 加载收藏
    fav.list(),

    // 步骤 3: 并行刷新各数据源
    workflow.parallel([
        news.refresh({limit: 30}),
        quote.snapshot({tab: "a"}),
        quote.snapshot({tab: "us"}),
        quote.snapshot({tab: "crypto"}),
        quote.snapshot({tab: "fx"}),
    ]),

    // 步骤 4: 写日志（可选）
    workflow.log({msg: "refresh complete"}),
])

plan.execute()
```

> workflow 由 octoscript-workflow engine 调度；每个 step 输出 schema 校验；checkpoint + rollback

---

## 7. Rust adapters 实现（adapters/src/）

每个 adapter 实现 `JsonToolContract`：

```rust
// adapters/src/quote_tencent.rs
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

async fn handle(args: serde_json::Value) -> Result<serde_json::Value, String> {
    let tab = args.get("tab").and_then(|v| v.as_str()).unwrap_or("a");
    let url = match tab {
        "a" => "https://qt.gtimg.cn/q=sh000001,sz399001,...",
        "us" => "https://qt.gtimg.cn/q=usAAPL,usMSFT,...",
        _ => return Err(format!("unknown tab {}", tab)),
    };
    let body = reqwest::get(url).await.map_err(|e| e.to_string())?.text().await.map_err(|e| e.to_string())?;
    let rows = parse_tencent(&body).map_err(|e| e.to_string())?;
    Ok(json!({"rows": rows}))
}
```

---

## 8. 渲染管线（octoscript-makepad）

```
Octoscript DSL (main.octoscript)
    ↓ octoscript-render（VM 评估 → UiNode tree）
UiNode tree (backend-agnostic)
    ↓ octoscript-makepad（纯翻译 to_makepad_ui）
makepad dialect string (View{Label{...}})
    ↓ makepad Splash widget set_text()
Live native widgets (Makepad View/Label/Button/CandlestickChart)
    ↓
GPU 渲染 / 触摸事件 → widget 状态 → VM 重新评估
```

> **关键**：octoscript-render 的 VM 是 octoscript 核心 VM；octoscript-makepad 只是翻译层；makepad Splash 是 host widget

---

## 9. shell 集成（OctoSense + Octos）

### 9.1 OctoSense shell（Phase 4）

- 启动 `OctoSense/desktop/` 或 `OctoSense/phone/` 二进制
- shell 加载 `OctoSense/apps/finance-brief/bundle/`（由 `install-as-system-app.sh` 同步）
- 卡片在 card-host 渲染（用 octoscript-makepad 管线）
- 用户操作 → widget event → VM 重新评估 → 卡片更新
- 应用切换：shell 在 OctoSense/apps/finance-brief/ ↔ 其它应用

### 9.2 Octos Agent（Phase 5）

- finance-brief 启动 agent 任务（如"分析 A 股 K 线趋势"）
- Octos 内核调度 agent（模型、工具、会话）
- 执行进展 → 通过 `fb.action` API 反映到界面（per Octoscript-AppCard）
- 任务失败 → 通过 capability error 反映到界面（状态栏 + toast）

---

## 10. octoscode 入口（Phase 6）

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

---

## 11. 数据源表

| # | 源 | URL | 格式 | 限频 | API key | 备注 |
|---|----|-----|------|------|---------|------|
| 1 | Sina 财经要闻 | `https://feed.mix.sina.com.cn/api/rollout?...` | JSON（伪 RSS） | 无 | 否 | 主新闻源 |
| 2 | Tencent 行情快照 | `https://qt.gtimg.cn/q={symbols}` | 文本（GBK） | 无 | 否 | A 股 / 港股 / 美股昨收 |
| 3 | Hyperliquid 加密币 | `https://api.hyperliquid.xyz/info` | JSON | 无 | 否 | POST `{type:"allMids"}` |
| 4 | Frankfurter 外汇 | `https://api.frankfurter.dev/v1/latest?base=USD&symbols=...` | JSON | 无 | 否 | ECB 日级汇率 |
| 5 | NASDAQ 美股 | `https://api.nasdaq.com/api/quote/{SYMBOL}/info?assetclass=stocks` | JSON | 无 | 否 | 美股实时（R-1 调研首选） |

> 所有 5 个源已声明在 `bundle/manifest.json#network.hosts`，无需新增白名单

---

## 12. 下一步（按 Phase）

1. **Phase 0（持续）**：Octoscript-AppCard 文档 / 模板管理
2. **Phase 1（前期）**：R-3 / R-4 调研 → M-1/M-2/O-4/O-5/O-6/O-7 验证
3. **Phase 2（前期→中）**：F-CAP/F-L0-{1..11}/F-WF/F-ADP 设计 + 实现
4. **Phase 3（中）**：C-TEST/C-RENDER 验证（card-host）
5. **Phase 4（中→后期）**：S-SHELL 集成（OctoSense shell）
6. **Phase 5（后期）**：A-AGENT 集成（Octos Agent）
7. **Phase 6（持续）**：D-CODE 集成（octoscode）

每个 Phase 完成后派 review agent 验收。

---

## 13. 变更日志

| 日期 | 变更 |
|------|------|
| 2026-10-01 | v1 初版（splash DSL 直写，1689 行） |
| 2026-10-02 | v2 重写（octoscript 平台分层；L0 卡片 + workflow + capability + adapter） |

---

## 附录 A：v3 决策附录（2026-10-02 用户确认）

| 决策 | 内容 | 影响 |
|---|---|---|
| Q-A | 9 L0 + 3 L1（K线 / 事件流 / 数据源状态页 state 部分） | UI 表达层；3 屏需 L1 expression |
| Q-B | + `stream.subscribe` / `stream.unsubscribe` / `stream.frequency.set` | capability catalog 从 14 → 17 |
| Q-C | adapters 集成 `bundle/native/src/adapters/` | 不新建独立 crate |
| Q-D | 1s/快讯 + 500ms/symbol + settings 可改 | #11 默认频率 + #9 加频率切换 UI |
| Q-E | #11 共用 hyperliquid/orderbook + mock 快讯 | 不再独立 mock；adapter `stream_tick.rs` |
| Q-R1 | 美股用 Stooq CSV（替换 nasdaq） | `quote_stooq.rs` 替换 `quote_nasdaq.rs` |
| Q-R2 | makepad CandlestickChart + L1 expression | K 线渲染路径 |
| Q-F | #12 = 5×8 字段（默认） | 数据源状态页表格 |

完整决策理由见 `.todo-octoscript-rewrite-2026-10-02.md` §2 + §2.1。
