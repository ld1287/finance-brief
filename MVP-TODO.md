# Finance Brief — OctoSense Market MVP TODO (v2: Octoscript Platform)

> Phase 1 MVP 实现计划，落地在 `finance-brief/`。
> 工作流：调研 → 架构 → UI 卡片 → adapters → shell 集成（全部本地，不推远端）。
> 重写日期：2026-10-02（v1 是 splash 直接写法，v2 切到 octoscript 平台分层）

## 0. ⏸️ Upstream 阻塞（2026-10-03）

| 项 | 详情 |
|---|---|
| Issue | [OctoSense-org/OctoScript#56](https://github.com/OctoSense-org/OctoScript/issues/56) |
| 标题 | `octoscript-ui-l0`: enable hosts to register additional `sys.*` helpers |
| 状态 | open (0 comments) |
| 影响 | finance-brief 17 capability 接不进 L0 屏（catalog::ANSWERS 硬编码 32 条） |
| 阻塞范围 | Phase 4 (S-RESOLVE-SYS-X) / Phase 5 (S-REWRITE-ALL-CARDS) / Phase 6 (S-LOWER-AND-RUN) |
| 不阻塞 | Phase 1 调研 ✅ / Phase 2 adapters + schemas ✅ / Phase 3 C-TEST ✅ |
| 解锁条件 | upstream 合并 `register(Contract)` runtime API |
| 当前动作 | 等 OctoSense-org 反馈；finance-brief 不私自改 octoscript-ui-l0 |
| 详细编排 | `.todo-octoscript-rewrite-2026-10-02.md` §0.1 + §10 batch 4+ |

## 0. 决策纪要（Q1–Q8，与用户对齐）

| # | 问题 | 决策 |
|---|------|------|
| Q1 | 在哪扩展？ | `finance-brief/` 直接演进，不开新目录 |
| Q2 | Agent 面板（右侧栏） | 后置，Phase 5（Octos 集成）才上 |
| Q3 | 数据源 | 保留 5 个免费源（sina / tencent / hyperliquid / frankfurter / **stooq**（Q-R1: B））；金十/Wind/付费 WS **不接** |
| Q4 | K 线实现 | L0 卡片 declarative + Makepad shader widget（60FPS 由 R-2 调研确定） |
| Q5 | 范围 | 只做 **Phase 1**（11 屏 + 1 备用槽 = 12 屏） |
| Q6 | 提交策略 | 独立仓库 octoOs/finance-brief（main），commit 后推送 origin/main |
| Q7 | 谁写代码 | sub-agent 实现；主对话负责 TODO / review / 汇报 |
| Q8 | 改哪个仓库 | octoOs/finance-brief；同步到 OctoSense/apps/finance-brief/bundle/ 以便在 shell 里运行 |
| **Q9（新）** | UI 表达层 | 用 **octoscript L0 卡片**（declarative, data-only），不是 splash DSL 直写 |
| **Q10（新）** | 数据层 | 用 **octoscript workflow + capability + schema contract**，不是 splash 手写 fetch/parse |
| **Q11（新）** | 渲染管线 | 通过 **octoscript-makepad**（octoscript DSL → makepad native widgets） |
| **Q12（新）** | 平台分层 | **前期** = Makepad + Octoscript；**后期** = OctoSense + Octos；**持续** = Octoscript-AppCard |
| **Q13（新）** | adapters 路径 | **集成** `bundle/native/src/adapters/`（per Q-C: B） |
| **Q14（新）** | 事件流频率 | **1s/快讯 + 500ms/symbol**（per Q-D: B），#9 settings 加频率切换 UI |
| **Q15（新）** | 美股源 | **Stooq CSV**（per Q-R1: B），不用 nasdaq API |

git 作者：仓库发布者（用 `GIT_AUTHOR_*` / `GIT_COMMITTER_*` 环境变量设置）。

## 1. 平台组件分层（核心架构）

```
┌────────────────────────────────────────────────────────┐
│ Layer 4 (后期)  OctoSense shell                        │
│   用户打开 / 操作 / 切换应用；卡片在同一应用环境中     │
│   出现；状态与结果可检查                                │
├────────────────────────────────────────────────────────┤
│ Layer 3 (后期)  Octos Agent 内核                        │
│   执行模型、工具、会话、执行进展；任务失败如何反映     │
│   到界面                                                │
├────────────────────────────────────────────────────────┤
│ Layer 2 (前期→持续)  Octoscript-AppCard                │
│   把意图路由给应用 Agent；管理生成卡片的应用层；       │
│   请求、生成、校验、数据绑定、交互更新                  │
├────────────────────────────────────────────────────────┤
│ Layer 1 (前期)  Octoscript 核心                        │
│   受约束应用表达 + capability + workflow + L0 卡片    │
│   - canonical v0.2 grammar                              │
│   - octoscript-ui-l0（check_syntax / realize）         │
│   - octoscript-workflow（dataflow + 步骤编排）         │
│   - octoscript-capabilities（capability + audit）      │
│   - octoscript-schema（数据契约 + JSON tool contract） │
├────────────────────────────────────────────────────────┤
│ Layer 0 (前期)  Octoscript-Makepad 渲染管线             │
│   octoscript DSL → UiNode → makepad dialect string     │
│   → makepad Splash widget → live native widgets        │
├────────────────────────────────────────────────────────┤
│ Layer -1 (前期)  Makepad 底层                           │
│   View/Label/Button/CandlestickChart 等原生控件        │
│   真实交互：点击是否改变状态，窗口变化后能否继续用     │
└────────────────────────────────────────────────────────┘

持续支撑：
- octoscode 开发者终端入口（交代任务、看修改、运行验证、退回修正）
- OctoSense-App-Hub 模板 / 卡片设计工具 / hub 商店（持续维护）
```

## 2. finance-brief 在新分层下的形态

```
finance-brief/                                      # 独立 git repo
├── bundle/                                         # OctoSense app bundle
│   ├── main.octoscript                             # L0 卡片入口（11 屏 declarative UI）
│   ├── workflow.octoscript                         # 数据流编排（capability 调用）
│   ├── capabilities.toml                           # 能力声明（host.call 名 + schema）
│   ├── schema/                                     # 数据契约（JSON Schema 子集）
│   │   ├── news.schema.json                        # NewsRow 契约
│   │   ├── quote.schema.json                       # QuoteRow 契约
│   │   ├── candle.schema.json                      # Candle 契约
│   │   ├── research.schema.json                    # ResearchCard 契约
│   │   ├── stream.schema.json                      # StreamItem 契约
│   │   └── settings.schema.json                    # Settings 契约
│   ├── assets/                                     # 图标等
│   │   └── icon.svg
│   ├── screenshots/                                # 真实抓帧（octoscript-makepad 渲染）
│   └── manifest.json                               # app 元数据 + 能力清单
│
├── adapters/                                       # Rust adapters（Layer 1 桥）
│   ├── Cargo.toml                                  # 独立 crate（不进 bundle）
│   └── src/
│       ├── lib.rs                                  # 入口
│       ├── news_sina.rs                            # Sina RSS adapter
│       ├── quote_tencent.rs                        # Tencent 行情 adapter
│       ├── quote_hyperliquid.rs                    # Hyperliquid adapter
│       ├── quote_frankfurter.rs                    # Frankfurter adapter
│       └── quote_nasdaq.rs                         # NASDAQ 美股 adapter
│
├── docs/
│   ├── MVP-TODO.md                                 # 本文档
│   ├── ARCHITECTURE.md                             # 详细架构说明
│   ├── DATA-SOURCES.md                             # 5 个数据源调研结论
│   ├── R-1-us-stocks.md                            # 美股实时源调研
│   ├── R-2-kline-feasibility.md                    # K 线可行性调研
│   ├── R-3-octoscript-platform.md                  # 【新】octoscript 平台调研
│   └── R-4-l0-cards.md                             # 【新】L0 卡片语法调研
│
├── scripts/
│   ├── run-octosense.sh                            # 启动 OctoSense shell + load finance-brief
│   ├── verify.sh                                   # cargo test + octoscript check + L0 check
│   └── install-as-system-app.sh                    # 同步到 OctoSense/apps/finance-brief/
│
├── fonts/                                          # NotoSansSC 中文字体
│
├── .todo-octoscript-rewrite-2026-10-02.md          # 本地协调（不入库）
└── README.md
```

## 3. 数据源现状（v0.1.0 已工作）

| 来源 | URL | 内容 | 状态 |
|------|-----|------|------|
| Sina 要闻 | `feed.mix.sina.com.cn` | 中文财经要闻 RSS | OK（GBK） |
| Tencent 行情 | `qt.gtimg.cn` | A股 / **HK 美股上一交易日收盘** | OK；美股不是实时，需换源 |
| Hyperliquid | `api.hyperliquid.xyz` | 加密货币实时（BTC/ETH/SOL 等） | OK（POST JSON） |
| Frankfurter | `api.frankfurter.dev` | ECB 外汇汇率 | OK |
| NASDAQ 美股 | `api.nasdaq.com` | 美股实时（盘中分钟级 tick） | OK（R-1 调研结论） |

## 4. 调研任务（先派 2 个并行 agent）

### R-1：美股实时源
- 测：可达 / 是否 key / 延迟字段（秒级实时 vs 收盘价）/ 编码 / JSON 格式
- 输出：可用清单 + 推荐首选 + manifest hosts 片段
- 验收：返回 ≥3 个可达候选 + 1 个首选（含为什么）
- **本轮默认**：首选 `api.nasdaq.com`（per R-1 调研）

### R-2：Makepad Shader + Octoscript 可行性（K 线 60FPS ≥100 根）
- 读 `octoscript/docs/makepad-ui-compatibility.md`（widget 兼容性）
- 读 `makepad/` `draw/src/` `widgets/src/` 找最少原型代码量
- 读 `octoscript-makepad/crates/makepad-d3` 和 `makepad-plot` 找 chart 例子
- 读 `octoscript-ui-l0/tests/fixtures/trip_planner.octoscript` 找 L0 declarative chart 用法
- 输出：能做 / 不能做 / 有条件做 + 性能预期 + 实现路径
- 验收：明确结论 + 推荐路径或 TODO 占位

### R-3（新增）：octoscript 平台调研
- 读 `octoscript/README.md` + `octoscript/docs/positioning.md`
- 读 `octoscript/docs/ui-profile-l0.md`（L0 卡片标准）
- 读 `octoscript/docs/grammar.md`（v0.2 canonical grammar）
- 读 `octoscript/examples/*.octoscript`（workflow / L0 / tool 用法）
- 读 `octoscript-ui-l0/tests/fixtures/trip_planner.octoscript`（L0 卡片完整例子）
- 读 `octoscript-makepad/README.md`（渲染管线）
- 输出：L0 卡片能不能表达 finance-brief 11 屏；workflow 怎么编；capability 怎么声明；schema contract 怎么写
- 验收：明确结论 + 11 屏每屏 L0 卡片草图

### R-4（新增）：L0 卡片语法调研
- 读 `octoscript-ui-l0/src/lower_l0.rs`（L0 编译器源码，**仅读语法**）
- 读 `octoscript/docs/ui-profile-l0.md` §1.0（什么适合 L0 / 什么不适合 L0）
- 输出：finance-brief 11 屏哪些适合 L0（declarative）；哪些需要 L1（带 expression）
- 验收：每屏标 L0/L1 + 替代方案

## 5. 界面拆分（1–12）按 octoscript L0 卡片

| # | 界面 | 来源 | L0/L1 | 数据来源（capability） | 任务 | 依赖 | 验收 | 状态 |
|---|------|------|-------|-----------------------|------|------|------|------|
| 1 | Launcher | 6.1 | **L0** | — | I-1 | — | 6 个 tile 网格（declarative） | TBD |
| 2 | 新闻简报列表 | 3.1 | **L0** | `news.refresh` (capability) | I-2 | — | declarative RowCard × N + FilterChip | TBD |
| 3 | 新闻详情 | detail | **L0** | `news.read` (capability) | I-3 | I-2 | BackButton + 内容（declarative） | TBD |
| 4 | 研究卡列表 | 3.2 | **L0** | `research.list` (capability) | I-4 | — | 5 类研究卡模板 | TBD |
| 5 | 研究卡详情（三段式） | 3.2 | **L0** | `research.read` (capability) | I-5 | I-4 | MVP 只做事实层（蓝色实线） | TBD |
| 6 | K线看盘（OHLC + 周期） | 3.3 | **L1** | `quotes.candles` (capability) | I-6 | R-2 | K 线 widget + 周期 chip（带 expression） | TBD |
| 7 | 行情列表合并 | 5 tab | **L0** | `quotes.snapshot` (capability) | I-7 | R-1 | 单一列表 + 4 个主题筛选 chip | TBD |
| 8 | 收藏 / 关注 | favs | **L0** | `favs.list` / `favs.toggle` (capability) | I-8 | — | 按类型分组 | TBD |
| 9 | 设置 | 6.3 + 7 | **L0** | `settings.load` / `settings.save` | I-9 | — | theme + 数据源开关 + 推送时段 | TBD |
| 10 | 免责声明 | 7.3 | **L0** | — | I-10 | — | 静态 Markdown | TBD |
| 11 | 事件流 stub | 3.1 + 3.3 | **L1** | `stream.mock` (capability) | I-11 | I-2/I-6 | mock 快讯 + 订单簿 stub（条件做） | TBD |
| 12 | 备用槽 | — | **L0** | — | — | — | 数据源状态页 / Agent 面板占位 | TBD |

> L0（declarative, data-only）占 9 屏；L1（带 expression，state 推导）占 2 屏（K线、事件流）；其余是 L0 widget 组合

## 6. 任务分层（按用户指定的"前期/持续/后期"）

### Phase 0（持续）：Octoscript-AppCard
- O-1：octoscript-AppCard 文档 / 模板管理
- O-2：finance-brief bundle 在 hub 里的发布/版本
- O-3：app card designer 工具集成

### Phase 1（前期）：Makepad + Octoscript 核心（依赖：Octoscript 平台调研）
- M-1：makepad widget 兼容性调研（哪些 widget 能用）
- M-2：makepad CandlestickChart widget 验证
- O-4：octoscript grammar / VM / parser verify
- O-5：octoscript-ui-l0 check_syntax + realize 验证
- O-6：octoscript-workflow engine + dataflow 验证
- O-7：octoscript-capabilities + schema contracts 验证
- R-3：octoscript 平台调研（前置）
- R-4：L0 卡片语法调研（前置）

### Phase 2（前期→中）：finance-brief 11 屏 L0 卡片设计
- F-1：capability catalog 设计（11 个 capability + schema contract）
- F-2：11 屏 L0 卡片模板（每个屏一张卡片）
- F-3：workflow 编排（数据流步骤）
- F-4：adapters Rust 实现（5 个数据源 adapter）

### Phase 3（中）：finance-brief 在 card-host 上验证
- C-1：adapters 单元测试 + 集成测试
- C-2：L0 卡片在 card-host 渲染验证（用 `tools/octo shot` 抓帧）
- C-3：capability 调通验证（splash → host.call → adapter → 返回）

### Phase 4（中→后期）：OctoSense shell 集成（依赖：OctoSense shell 启动）
- S-1：finance-brief bundle 安装到 OctoSense/apps/finance-brief/
- S-2：OctoSense shell 启动加载 finance-brief 验证
- S-3：状态可检查（卡片环境、应用切换）

### Phase 5（后期）：Octos Agent 集成（依赖：Octos 内核）
- A-1：finance-brief 启动 agent 任务（如"分析 K 线趋势"）
- A-2：执行进展 → 反映到界面
- A-3：任务失败 → 反映到界面
- A-4：右侧栏 Agent 面板（Phase 2 之前的占位）

### Phase 6（持续）：octoscode 开发者终端
- D-1：octoscode 接 finance-brief 任务
- D-2：开发者看修改 + 运行验证 + 退回修正流程
- D-3：CI / pre-commit hooks

## 7. 实现任务（agent 拆分）

按 Phase 分组，每组并行/串行策略不同：

| Agent | 范围 | 输入 | 输出 | Phase |
|-------|------|------|------|-------|
| **R-3** | octoscript 平台调研 | splash + 5 个 host | 报告 + L0 卡片语法摘要 | Phase 1 |
| **R-4** | L0 卡片语法调研 | R-3 + 11 屏清单 | 11 屏 L0/L1 标 + 替代方案 | Phase 1 |
| **F-CAP** | capability catalog | R-3/R-4 + 11 屏 | capabilities.toml + schema/*.json | Phase 2 |
| **F-L0-1..11** | 11 屏 L0 卡片 | capability catalog | 11 个 L0 卡片模板 | Phase 2 |
| **F-WF** | workflow 编排 | capability catalog | workflow.octoscript | Phase 2 |
| **F-ADP** | 5 个 Rust adapters | 5 个数据源 | adapters/src/*.rs | Phase 2 |
| **C-TEST** | 单元 + 集成测试 | adapters + capability | tests/* + cargo test | Phase 3 |
| **C-RENDER** | card-host 渲染验证 | 11 L0 卡片 | screenshots/*.png | Phase 3 |
| **S-SHELL** | OctoSense shell 集成 | bundle + shell | install + load 验证 | Phase 4 |
| **A-AGENT** | Octos Agent 集成 | workflow + agent | 启动 + 进展 + 失败反映 | Phase 5 |
| **D-CODE** | octoscode 集成 | 全部 | CI + hooks + 文档 | Phase 6 |

**Agent 拆分原则**：
1. 写权限完全独立（不重叠文件）
2. Phase 内并行；Phase 间串行（前置 Phase 通过再启后置）
3. 每个 agent 完成后由主对话收集 + 派 review agent

## 8. Code-review 检查点（每个 agent 后强制）

按 skill 规则 4，每次 commit 后派独立 review agent，检查：

### 8.1 L0 卡片硬约束（替代之前的 splash 硬约束）
- L0 卡片用 `let x = "{{state.x}}"` 占位符注入数据
- L0 卡片**没有 expression**（不是 if/else 计算；只是 declarative 树）
- L1 卡片允许 expression（state 推导）—— 用于 K 线、事件流
- 数据契约严格按 `schema/*.schema.json`（octoscript-schema 子集）
- capability 名严格按 `capabilities.toml`（octoscript-capabilities）
- workflow 步骤严格按 `octoscript-workflow` 规范
- host.call 永远不直接出现 —— 必须用 capability

### 8.2 适配器（Rust）硬约束
- 每个 adapter 实现 `JsonToolContract`（输入输出 schema）
- 错误归一化（HTTP 错 / 解析错 / 超时 → 统一错误格式）
- 不准 raw socket（per SCRIPT-API 网络策略）
- 不准直连白名单外的 host

### 8.3 manifest
- hosts 完整（5 个数据源 host）
- capabilities 含 `storage / net / workflow / ui-l0`
- version bump（v0.2.0 → v0.x）
- publisher 字段（name / support / privacy）由人工填写

### 8.4 privacy
- 不含本机路径 / 用户名 / 平台字符串
- 不含 API key / 凭证

### 8.5 gate
- `octoscript check` PASSED（语法）
- `octoscript check_ui_l0` PASSED（L0 卡片）
- `cargo test -p finance-brief-adapters` PASSED

### 8.6 截图
- `bundle/screenshots/` 含真实 PNG（card-host 抓帧）

## 9. 验收标准（Phase 1 整体）

- [ ] R-1 美股实时源接入 + 延迟验证（秒级实时）
- [ ] R-2 K 线 60FPS 或 TODO 占位
- [ ] R-3 octoscript 平台调研完成 + L0 卡片语法摘要
- [ ] R-4 11 屏 L0/L1 标 + 替代方案
- [ ] capability catalog 设计完成（11 个 capability + schema）
- [ ] 11 屏 L0 卡片模板全部完成
- [ ] workflow 编排完成
- [ ] 5 个 Rust adapter 实现 + 单测
- [ ] card-host 渲染 11 屏 OK
- [ ] OctoSense shell 加载 finance-brief OK
- [ ] `bundle_blake3` 重算 + `octoscript check` PASSED
- [ ] 8 张以上真实截图入库
- [ ] README 同步更新（含平台分层、capability 清单、Phase 1 完成清单）
- [ ] 本地分支 commit 完毕，**未推远端**

## 10. 时间线（建议）

1. **Step 1**（now）：重写 MVP-TODO + ARCHITECTURE + .todo ← 当前
2. **Step 2**：派 R-3 + R-4 调研 agent（并行，Phase 1 前置）
3. **Step 3**：派 M-1/M-2/O-4/O-5/O-6/O-7 验证 agent（Phase 1，并行）
4. **Step 4**：派 F-CAP/F-L0-{1..11}/F-WF/F-ADP agent（Phase 2，分组并行）
5. **Step 5**：派 C-TEST/C-RENDER agent（Phase 3，并行）
6. **Step 6**：派 S-SHELL agent（Phase 4）
7. **Step 7**：派 A-AGENT agent（Phase 5）
8. **Step 8**：每步后派 review agent
9. **Step 9**：派 D-CODE agent（Phase 6）
10. **Step 10**：主对话收集 + 写最终汇报

## 11. 参考

- **PRD**：`OctoSense Market Agent - MVP产品需求文档.md`
- **Octoscript 平台**：
  - `octoscript/README.md`（核心定位）
  - `octoscript/docs/positioning.md`（与 Makepad Octoscript 区别）
  - `octoscript/docs/ui-profile-l0.md`（L0 卡片标准）
  - `octoscript/docs/grammar.md`（v0.2 canonical grammar）
  - `octoscript/examples/*.octoscript`（workflow / L0 / tool 用法）
  - `octoscript-ui-l0/tests/fixtures/trip_planner.octoscript`（L0 完整例子）
- **Octoscript-Makepad 渲染**：`octoscript-makepad/README.md`
- **Makepad 底层**：`makepad/`
- **平台分层组件**：`OctoSense/` / `OctoSense-App-Hub/` / `octos/` / `octoscode/`
- **Octoscript 旧文档**：`OctoScript-App-Design-Flow/docs/SCRIPT-API.md`（splash 方言参考）
- **K 线思路**：参考 iced 实现的 K 线项目（仅思路，不是移植）
- **Hub 模板**：`OctoScript-App-Design-Flow/templates/script-app/`
- **当前 splash**：`apps/finance-brief/bundle/main.splash`（1437 行，v1 实现，本轮要拆解）
- **当前 hosts**：`feed.mix.sina.com.cn, qt.gtimg.cn, api.hyperliquid.xyz, api.frankfurter.dev, api.nasdaq.com`
