# R-3：octoscript 平台调研报告

**调研日期**：2026-10-03
**调研对象**：`octoscript/` 仓库（pinned dep on `OctoSense-org/makepad` 的 `octoscript` 分支）
**上下文**：`MVP-TODO.md` Phase 1 前置（R-3）
**输出摘要**：octoscript 是一个 **capability-first 的可生成脚本运行时**；finance-brief 的 12 屏中 **9 L0 + 3 L1**（per 附录 A Q-A 决策）。

> Re-verified 2026-10-04: §5 加 17-capability sub-table (per Q-B 新增 3)。

---

## 0. 调研路径与硬约束

| 文件 | 角色 |
|---|---|
| `octoscript/README.md` | 顶层定位 + 当前 baseline 清单 |
| `octoscript/docs/positioning.md` | 与 Makepad Octoscript 区别（关键 §"Difference From Makepad Octoscript"） |
| `octoscript/docs/ui-profile-l0.md` | L0 卡片标准的"什么能/不能写"边界（最长文档，§1 + §5.10/§5.11 + §6/§9） |
| `octoscript/docs/grammar.md` | canonical v0.2 grammar（lexical + program + expression 子集） |
| `octoscript/docs/capability-audits.md` | CapabilityHost 审计导出 + 可选持久化 stream |
| `octoscript/docs/workflow-checkpoints.md` | WorkflowEngine + 步骤 lease + 持久化 checkpoint |
| `octoscript/docs/schema-contracts.md` | `JsonToolContract` 子集（用于 capability 输入/输出契约） |
| `octoscript/examples/makepad_ui_counter.octoscript` | Makepad Octoscript 兼容 body（**不是** L0 卡片，仅作反例） |
| `octoscript/crates/octoscript-ui-l0/tests/fixtures/trip_planner.octoscript` | L2 exemplar（**不是** L0 参考，使用 `let`） |

**不在调研范围**：Rust 源码（`fn `、`impl ` 等）、bundled JSON fixtures、Cargo 工作空间内部。

---

## 1. octoscript 与 Makepad Octoscript 的区别

> 来源：`octoscript/docs/positioning.md` 第 12–39 行 §"Difference From Makepad Octoscript"

Makepad 的 Octoscript 是一个 **VM 与语言底座**，被 Makepad UI 嵌入使用——它接受兼容语法、依赖宿主应用决定能力暴露面。但它不是一个 **稳定的能力边界**，不适合被生成的、不受信的程序直接调用。

octoscript 仓库把 Makepad 那套作为一个 **pinned git 依赖**（`octoscript` 分支），然后在此之上定义了一个**独立的、可移植的 language 契约 + host 模型**。两者差异如下：

| 维度 | Makepad-oriented substrate | octoscript runtime profile |
|---|---|---|
| Source contract | 兼容 parser（广） | canonical v0.2 grammar，preflight 才执行 |
| Primary use | UI / runtime 嵌入 | 动态工作流、dataflow、被审核的 tool call |
| Effects | 由宿主嵌入决定 | deny-by-default；只能调注册过的 capability |
| Native platform surface | 继承的 UI/debug 模块可能可用 | 评估前 mask 掉继承 UI/debug 与直接输出条目 |
| Async behavior | VM-host 集成细节 | bounded host-pumped promises + 显式外部生命周期 |
| Error recovery | 继承 frame-local `try` | canonical `try/catch`（跨函数），硬限制 uncatchable，无回滚 |
| Rust integration | app 自选 native binding | 空过契约 + 策略 + Serde + setup-defined 直接 module |
| Workflow control | app-specific | host-owned plan + approval + bounded JSON dataflow + per-step lease + checkpoint + ledger |
| Generated source | 受信宿主决定 | 应用语法 review + bounded source + call hints + runtime 强制 |
| Containment claim | VM 范围外 | 显式 VM 之外；effective worker 需平台层容器 |

**对 finance-brief 的含义**：finance-brief 写的是"被生成的、不受信的代码"（卡片 + workflow），所以走的是右边那列——canonical grammar preflight + capability call，不允许出现任何 Makepad UI 子模块或者 callback setter。

`positioning.md` 还明确写了 *"A trusted host may use the explicit compatibility escape hatch for migration, but generated code must never receive it"*——finance-brief 的 bundle 是 generated source，所以**禁用**兼容逃生口。

---

## 2. L0 卡片能做什么 / 不能做什么

> 来源：`octoscript/docs/ui-profile-l0.md` §1 + §5.10 + §6

L0 是"生成的卡片"专用 profile。它的设计原则是：

> *"L0 admits UI constructs, but admits nothing that could reach a capability."*
> —— `ui-profile-l0.md` §1 行 24–25

**结构上**：L0 没有 expression form，所以**没有 evaluator**——realization 就是把 parsed tree 走一遍 + 把数据填进去。来源第 §1 行 30–35：

> *"L0 has no expression form, so there is nothing to evaluate. Realization is a pure walk over the parsed tree with data substituted."*

这个 confinement claim 是 **structural**："nothing to evaluate, so no evaluator, so realization never enters an evaluator at all"。

### 2.1 L0 能做什么（per ui-profile-l0.md）

| 类别 | L0 允许 |
|---|---|
| **数据声明** | `source` / `state` / `event` / `copy` declaration |
| **数据源形状** | 声明 source：`source path capability-query`，host 在 realization 前 resolve |
| **状态形状** | `state path { shape-spec }`，literal / collection / counter / toggle / cycle / record 等 |
| **视图** | `view path view-body`、嵌套 component |
| **构造元素** | role（`TextHero` / `Panel` / `Chip` / `TempBar` 等）+ `on_tap` 触发 transition |
| **迭代** | `for x[, i] in path key path { ... }`，**只能迭代 declared collection** |
| **谓词** | `when path == (path|literal|token)`、`when path .colon-key path == .literal` |
| **transition** | `set($state)` / `cycle($state, .a, .b, .c)` / `toggle($state)` / `append($state, $value)` / `remove($state, $value)` |
| **copy 类** | `copy path { class: ..., locale-map }`（多语言字符串） |

### 2.2 L0 不能做什么

| 类别 | L0 拒绝 | 替代 |
|---|---|---|
| **算术 / 字符串拼接** | `value: a * b + c`、`mk + "," + ...` | host 处理（capability 输出里给出已拼好的字符串） |
| **函数调用** | `sys.coord(wp1, "lat")` 之类 | declared source + `$state` 代替 |
| **`while` / `for i = 0; i < n; i++`** | 仅 `for x in path key x` | 只能迭代 collection |
| **`let` 重绑定** | trip_planner.octoscript 那种 30 个 `let` | L0 没有这个；let 是 L2 |
| **`ui.<id>.set_text(...)`** | imperative widget 命令 | 通过 declared source 的变化驱动 widget |
| **`use mod.xxx`** | 模块导入 | L0 仅接收工具调用结果作为数据，不 import |
| **`fn tick()`** | 自主动画循环 | widget 自己拥有 `nav_period` 之类参数 |
| **跨字段计算** | `state n { initial: count * 2 }` | shape 用 `max: end` 表达约束 |
| **空集合判断** | `count == 0` | L0 没有 count/spec（§1.1 已知 gap，§8 问题 10） |
| **append/remove 累积** | 也不行 | collection 是 prop shape 不是 state shape（§1.1 gap） |
| **`duration` 格式** | 不能写 48 minutes | `.money` / `.compact` / `.time` / `.date` 已有，缺 duration |

### 2.3 RealizeLimits（一次 realization 的硬上限）

来源：`ui-profile-l0.md` preamble 第 11–17 行

| 维度 | 上限 |
|---|---|
| Emitted nodes | 8,192 |
| Nesting depth | 64 |
| Items per collection | 512 |
| Aggregate work units | 65,536 |

**违反**：`RealizeReport::truncated`，hosts 必须把它当 partial 处理。嵌套循环 + component expansion 都算，**不能用递归 / 自调用绕过**。

### 2.4 L0 的"无名副作用" vs L1

`ui-profile-l0.md` §9.7 把 L0 的 confinement 锚在：

> *"not a sandbox that blocks calls, and not an evaluator with an empty module table, but **no execution machinery in the path at all**"*

L1 才加入一个 **expression form**（`+ - * / %`，含 grouping 与 unary minus），作用域仅限 **argument value**（implementation 已扩展接受 guard right operand 与 comparison right operand）。详细见 §3 与 R-4。

---

## 3. canonical v0.2 grammar 关键规则

> 来源：`octoscript/docs/grammar.md` §"Lexical Rules" + §"Program and Statements"

### 3.1 Lexical

来源 §"Lexical Rules"：

- identifier = `identifier-start { identifier-continue }`
- identifier-start = `[A-Za-z_]`
- identifier-continue = `[A-Za-z0-9_]`
- integer = `[0-9]+`
- number = integer 可选小数 + 可选指数
- string = `"..."`；转义支持 `\" \\ \r \n \t` 与 Unicode escape `\uXXXX` 或 `\u{1F600}`
- 注释：行尾 `//`、块 `/* ... */`；块注释终结符 `**/` 不算重叠闭合
- **关键字**（canonical）：`if elif else try for in loop while fn let return break continue use true false nil`
- **保留字**（reject in canonical）：`var match ok do and or is mut me scope`
- **`catch` 是 contextual 关键字**：在 `try` 分支后是 separator，其他地方是普通标识符
- 数字后紧跟 `.field` 解析为小数尾巴——要 `5 .field` 或 `(5).field` 才合法
- Unicode escape 必须合法 scalar value（surrogate 与 > U+10FFFF 拒绝）

### 3.2 Program & Statements

来源 §"Program and Statements"：

- `program = { statement statement-end }`
- `statement-end = newline | ";"`
- statement ∈ {import, declaration, function-declaration, return-statement, break-statement, continue-statement, expression}
- `import = "use" module-path`
- `module-path = "mod" "." identifier { "." identifier }`
- `declaration = "let" identifier [ "=" expression ]`
- `function-declaration = "fn" identifier parameter-list block`
- `block = "{" { statement statement-end } "}"`

> ⚠️ **重要提示**：`trip_planner.octoscript` 用 `let x = "{{state.x}}"` 反复绑定 state，这种 `let` 是 **L2 imperative 写法**。L0 卡片不写 `let`——数据来自 `source` / `state` declaration，由 host 在 realization 前 resolve。

### 3.3 v0.2 的新增

`grammar.md` 第 14–16 行：

> *"Version 0.2 adds the canonical `try ... catch ...` expression. Every v0.1 program remains valid v0.2 source; the new form does not enable an error value, ambient effect, or new host API."*

`try/catch` 跨函数 recover；硬资源限制（heap/stack/frame/instruction/time）uncatchable，**没有隐式 effect 回滚**。

### 3.4 程序入口的差异

- **workflow / canonical**：`octoscript check` + `check_syntax` 走 canonical grammar，`mod.tool` / `mod.arithmetic` / `mod.std.*` 可用
- **Makepad UI body**（`examples/makepad_ui_counter.octoscript`）：用 `View { ... Label { ... ButtonFlat } }` DSL，是 Makepad 兼容语法，**不是 L0 卡片**——counter 那个例子里 `ui.display.set_text("..."` 就是 L2 imperative widget setter
- **L0 卡片**：header 可选 `# ledger X@1.0.0` + `# level: L0` + `# model: ...` + `# profile: ...`；body 是 declaration（source / state / event / copy / component / view）

### 3.5 L0 入口写法（区别于 counter 例子的 imperative 写法）

L0 卡片用声明式 declaration：

```
# ledger finance-brief@1
# profile: ui-l0

source feed news.refresh
state filter { shape: token, initial: .all }
view body {
  Panel {
    for n, i in feed key n.id {
      RowCard {
        title: n.title
        on_tap: open_detail(n.id)
      }
    }
  }
}
```

**没有 `let`、没有 `ui.<id>.set_*`、没有 `+ - * /`**——这就是 trip_planner.octoscript 与合法 L0 卡片的本质区别。

---

## 4. capability + workflow + schema contract 用法

### 4.1 capability（octoscript-capabilities）

来源：`octoscript/README.md` + `octoscript/docs/capability-audits.md`

- **deny-by-default**：`scripts can call only explicitly registered tools through `mod.tool``
- 注册：host 通过 `register_validated_json_tool(...)` / `register_validated_protocol_json_tool(...)` / `register_typed_json_tool(...)`
- 工具路径：script 里写 `use mod.tool; tool.call("text.echo", "release")`
- 直接 module：`use mod.arithmetic; arithmetic.add({left: 20, right: 22})` 也合法（host 把 method 映射到一个 contract-enforced capability）
- 审计：每个 runtime 维护一个 bounded in-memory audit；`audit_since(cursor)` 导出 `AuditEventBatch`
- 持久化（opt-in feature `durable-audit-journal`）：`CapabilityAuditStore` 用 authenticated store + 4 次 bounded CAS retry
- 上限：保留 ≤ 1,024 事件，序列化的 audit document ≤ 192 KiB
- 批是 **serializable 但不 deserializable authority**——只携带 telemetry，**不**授权 tool、重建 pending op、确认 cancel

**对 finance-brief 的用法**：5 个数据源 + storage + 推流全部走 capability。**`host.call` 永远不直接出现**——必须封装到 capability 里（per MVP-TODO §8.1）。

### 4.2 workflow（octoscript-workflow）

来源：`octoscript/docs/workflow-checkpoints.md`

- **plan**：host 持 trusted `WorkflowPlan`：`engine.plan(vec![WorkflowStep::new("id", "src")])`
- **approve**：`engine.approve(&plan)` / `approve_with_step_capability_leases` / `approve_resume`
- **lease**：每个 step 一个 `CapabilityLeaseGrant`，绑定 runtime identity + catalog fingerprint + 允许 tool 名 + per-tool call limit
- **执行**：当前 step 的 lease 才 active；deferred `await` / 续延期间 lease 仍 active；catalog 变更 → `CatalogChanged` fail closed
- **checkpoint**：trusted plan 在 prefix 完成后，`engine.checkpoint_after(&plan, n)` 拿 BLAKE3 fingerprint + 完成的步骤 id + 可选 dataflow digest
- **resume**：restart 后重建 trusted plan + tool policy，再 `approve_resume`，**用普通 `approve` 结果 resume 会拒绝**
- **draft**（区别）：LLM 提议的 pre-approval step list 用 **data-only draft**（无 fingerprint、无 restart authority），由 host 升格为 trusted plan

**对 finance-brief 的用法**：每个 tab 的 `refresh` 是一个 workflow step（带 capability lease）；多步流程（fetch → parse → update state）用 plan 串起来；checkpoint 用作"上次拉到的 cursor"（如新闻列表分页、收藏写盘进度）。

### 4.3 schema contract（octoscript-schema）

来源：`octoscript/docs/schema-contracts.md`

- `JsonToolContract::new(input_schema, output_schema)` 编译一个 bounded executable JSON Schema 子集
- 注册后 runtime **先验证 envelope，再验证 input contract**，**才**预留 call 或 invoke handler；输出侧同样先 envelope + contract 再返回
- **拒绝**的 input 记为 `denied`，**不消耗** call budget
- 同一条路径用于同步调用与 deferred 调用（host-pumped 或 externally completed）

#### 4.3.1 supported subset

- types：`null`, `boolean`, `number`, `integer`, `string`, `array`, `object`
- object：`properties`, `required`, `additionalProperties: bool`
- array：一个 `items` schema + `minItems` / `maxItems`
- scalar：`minimum`, `maximum`, `minLength`, `maxLength`, `enum`
- non-enforcing annotation：`title`, `description`, `default`, `examples`, `$schema`, `$id`

#### 4.3.2 不支持的（这是有意为之的子集）

- `$ref`
- schema composition（`allOf`, `anyOf`, `oneOf`, `not`）
- regex patterns
- conditional schemas
- schema-valued `additionalProperties`

#### 4.3.3 schema source 上限

- 单 schema ≤ 32 KiB
- 嵌套 ≤ 32 层
- 单 object ≤ 128 properties
- enum ≤ 128 values

#### 4.3.4 typed Rust adapters

`register_typed_json_tool` 需要 schema；Rust adapter 走 schema-checked Serde bridge，**不走 raw JSON**。`ToolMetadata::with_input_schema` / `with_output_schema` **只是 prompt metadata，不做验证**——要做验证必须 wrap 到 `JsonToolContract`。

**对 finance-brief 的用法**：5 个 adapter（news / quotes / fx / crypto / stream）每个都配 `JsonToolContract`；输入输出 schema 入 `bundle/schema/*.schema.json`（per MVP-TODO §2 文件清单）。**adapter 不准 raw socket**（per SCRIPT-API 网络策略）；host 白名单决定哪些 host 可达。

---

## 5. finance-brief 12 屏 L0/L1 标

> 来源：`finance-brief/MVP-TODO.md` §5 拆分表 + 附录 A Q-A 决策
> 决策依据：`finance-brief/docs/ARCHITECTURE.md` 附录 A `Q-A | 9 L0 + 3 L1（K线 / 事件流 / 数据源状态页 state 部分）`

| # | 界面 | 屏定位 | 标 | 数据形状（capability / state / source） | 备注（为什么是这个标） |
|---|---|---|---|---|---|
| # 1 | Launcher | 6 tile 网格（declarative） | **L0** | `state selected { shape: token, initial: .news }`（tap 切主题用 `set($state)`） | 纯 declarative，无 L1 expression |
| # 2 | 新闻简报列表 | 主题列表 + FilterChip | **L0** | `source news feed.news.refresh` + `state filter { shape: token }` + `for n, i in feed key n.id` | L0 source + declared collection 迭代 |
| # 3 | 新闻详情 | 内容 + BackButton + 收藏按钮 | **L0** | `source detail news.read` + `transition toggle($favs, detail.id)` | L0 transition（toggle）足够 |
| # 4 | 研究卡列表 | 5 类研究卡模板 | **L0** | `source cards research.list` + `for c, i in cards key c.id` | L0 declarative 列表 |
| # 5 | 研究卡详情（三段式） | 事实层（蓝色实线） | **L0** | `source card research.read` + `event on_tap { set($view, .detail) }` | L0 tap transition |
| # 6 | K线看盘（OHLC + 周期） | K 线 widget + 周期 chip | **L1** | `source candles quote.candles` + `TextHero(value: c.last - c.first)` 等 expression | 需要 §9 arithmetic expression（涨跌、振幅等需要算） |
| # 7 | 行情列表合并 | 5 tab + 单一列表 + 4 主题筛选 chip | **L0** | `source quotes quote.snapshot` + `for q, i in quotes key q.symbol` + `when q.change_pct >= 0` | L0 + predicate 比较 |
| # 8 | 收藏 / 关注 | 按类型分组 | **L0** | `source favs favs.list` + `transition toggle($favs, x)` | L0 列表 + toggle transition |
| # 9 | 设置 | theme + 数据源开关 + 推送时段 | **L0** | `source settings settings.load` + `for s, i in settings key s.id` + `on_tap set($state)` | L0 控件组 |
| # 10 | 免责声明 | 静态 Markdown | **L0** | 仅 `view body { Markdown(...) }`，无 build / archive | 纯静态渲染 |
| # 11 | 事件流 stub | mock 快讯 + 订单簿 stub | **L1** | `source stream stream.mock` + `TextValue(value: book.bid - book.ask)` 等 spread expression | 需要 L1 算 spread / 中间价 |
| # 12 | 备用槽 | 数据源状态页（5×8 字段） | **L1**（state 部分） + L0（其它） | `state ok_count { shape: counter }` + `TextHero(value: ok_count / total)` 等 ratio expression | Q-A 明确"数据源状态页 state 部分"需要 L1 expression（占比、累计） |

#### 5.0.1 17-capability 总盘清单（per Q-B 新增 3）

> 数据源（来源）：`.todo-octoscript-rewrite-2026-10-02.md` §5.1 capability → adapter 映射表
> 单复数统一单数（per R-4 §3 保持一致：`quote.candles` / `quote.snapshot`，**非** `quotes.X`）
> 3 个 stream.* capability 为 Q-B 新增（v1 14 → v1+3 = 17）

| # | capability | 屏归属 | L0/L1 | 数据源 | 备注 |
|---|---|---|---|---|---|
| 1 | `news.refresh` | #2 新闻简报列表 | L0 | `news_sina.rs` → `feed.mix.sina.com.cn` | 入站 feed，10s 自动 + 下拉刷新 |
| 2 | `news.read` | #3 新闻详情 | L0 | `news_sina.rs`（本地缓存） | 详情 fetch（命中缓存即返回） |
| 3 | `quote.snapshot` (tab=a/us/hk) | #7 行情列表合并 | L0 | `quote_tencent.rs` → `qt.gtimg.cn` | A 股 / 港股 / 美股 tab 用 Tencent |
| 4 | `quote.snapshot` (tab=crypto) | #7 行情列表合并 | L0 | `quote_hyperliquid.rs` → `api.hyperliquid.xyz` | 加密币 tab 用 Hyperliquid |
| 5 | `quote.snapshot` (tab=fx) | #7 行情列表合并 | L0 | `quote_frankfurter.rs` → `api.frankfurter.dev` | 外汇 tab 用 Frankfurter |
| 6 | `quote.snapshot` (kind=us, real-time) | #7 行情列表合并 | L0 | **`quote_stooq.rs`** → `stooq.com CSV` | 美股 real-time 用 Stooq（per Q-R1: B，替换 `quote_nasdaq.rs`） |
| 7 | `quote.candles` | #6 K线看盘 | **L1** | `synth_candles.rs`（合成） | §9 arithmetic：涨跌 / 振幅计算 |
| 8 | `research.list` | #4 研究卡列表 | L0 | （内置样本） | 5 类研究卡模板枚举分发 |
| 9 | `research.read` | #5 研究卡详情 | L0 | （内置样本） | MVP 仅事实层（蓝实线） |
| 10 | `stream.tick` | #11 事件流 stub | **L1** | `stream_tick.rs`（hyperliquid/orderbook + mock 快讯） | mock ticker + orderbook 聚合；spread / mid 算 |
| 11 | `datasource.status` | #12 数据源状态页 | **L1**（state 部分） | `datasource_status.rs`（audit log 聚合） | 5×8 字段；占比 / `avg(latency)` 算 |
| 12 | `fav.list` | #8 收藏 / 关注 | L0 | （内置 fs） | app jail 持久化 |
| 13 | `fav.toggle` | #8 收藏 / 关注 | L0 | （内置 fs） | `toggle($favs, x)` transition（§3） |
| 14 | `settings` (load + save) | #9 设置 | L0 | （内置 fs） | load / save 合并为 1 capability（双 schema 入口） |
| 15 | **`stream.subscribe`** | #11 事件流 stub | **L1** | **`stream_subscribe.rs`**（host state 切换） | **Q-B 新增 3**：进入 #11 时开启后台 tick |
| 16 | **`stream.unsubscribe`** | #11 事件流 stub | **L1** | **`stream_subscribe.rs`**（host state 切换） | **Q-B 新增 3**：离开 #11 或点暂停时停止 tick |
| 17 | **`stream.frequency.set`** | #9 设置 | L0 | **`stream_subscribe.rs`**（host state 切换） | **Q-B 新增 3**：#9 频率切换 chip（1s / 5s / 10s）写入 host state |

### 5.1 L1 表达式的精确作用域（per ui-profile-l0.md §9）

- **允许 L1 expression 的位置**：
  - argument value（唯一 spec 明确：`TextHero(value: shares * quote.last)`）
  - guard 右值（implementation 已接受；spec §9.8 第 1 条 fix）
  - comparison 右值（implementation 已接受；spec §9.8 第 2 条 fix）
- **不允许 L1 expression 的位置**：
  - state `initial:`（spec §9.8 第 5 条 fix：capture read，不 compute）
  - transition 内 `set()`（spec §9.8 第 4 条：`Form` 未变，`set(shares * 2)` refused）
  - state 名字（declaration 不是 expression）

### 5.2 升 L1 的判据

> 关键：升 L1 的"必要条件"是 §9.3——L1 expression 必须 **read at least one declared source/state**（禁止纯字面量算术）。

| 屏 (k) | 升 L1 触发 | 是否满足 §9.3 | 决策 |
|---|---|---|---|
| 6 K线 | 计算 `c.last - c.first`、`(h-l)/(max-min)` 等振幅/涨跌 | ✅ 引用 `c` (declared source) | **L1** |
| 11 事件流 | 计算 `book.bid - book.ask` spread、`(bid+ask)/2` mid | ✅ 引用 `book` | **L1** |
| 12 状态页 state 部分 | `ok_count / total`、`avg(latency)` 之类的占比/均值 | ✅ 引用 declared counter/state | **L1**（state 部分）；其它 5×8 字段 L0 |

### 5.3 全文 L0/L1 计数

- 12 屏中 **L0 = 9**（#1, 2, 3, 4, 5, 7, 8, 9, 10）
- 12 屏中 **L1 = 3**（#6, 11, 12 state 部分）
- 与附录 A Q-A 一致：`9 L0 + 3 L1（K线 / 事件流 / 数据源状态页 state 部分）`

### 5.4 17-capability 总盘

总盘 **17 = 11 数据**（news×2 + `quote.snapshot`×4 + `quote.candles`×1 + research×2 + `stream.tick`×1 + `datasource.status`×1）+ **3 storage**（`fav.list` + `fav.toggle` + `settings` load/save）+ **3 stream**（`stream.subscribe` / `stream.unsubscribe` / `stream.frequency.set`，**per Q-B 新增**）。完整 17 行见 §5.0.1 sub-table。

---

## 6. 风险与下一步

| 风险 | 来源 | 缓解 |
|---|---|---|
| L0 不能判空集合 | ui-profile-l0.md §1.1 gap | 第 #11 屏空流时显示占位文案（host 侧判断） |
| L0 不能 append collection | ui-profile-l0.md §1.1 gap | 第 #8 屏收藏用 `favs.toggle`（已存在状态）走 storage adapter |
| L0 不能写 duration 格式 | ui-profile-l0.md §1.1 gap | 第 #6 屏 K 线时间由 widget 自管 `nav_period` 之类参数 |
| `on_tap` 不在 text roles 上 | ui-profile-l0.md §1.1（"可能正确但没写下来"） | 用 `Row` 包裹 |
| `try/catch` 跨函数但无 rollback | grammar.md 第 14 行 | adapter 错误归一化，**不依赖** try/catch 做补偿 |

**下一步**：派 R-4 调研（已在并行），给出 12 屏每屏的 L0 卡片草图 + 替代方案；R-3 输出 + R-4 输出一起喂给 F-CAP（capability catalog 设计）和 F-L0-{1..12}（F-L0-6 / F-L0-11 / F-L0-12 走 L1，其余 L0）。

---

## 7. 参考

- `octoscript/README.md`：capability-first 脚本运行时，canonical v0.2 grammar
- `octoscript/docs/positioning.md`：与 Makepad Octoscript 区别
- `octoscript/docs/ui-profile-l0.md`：L0 卡片 spec（§1/§2/§5.10/§5.11/§9.7/§9.8）
- `octoscript/docs/grammar.md`：v0.2 canonical grammar（lexical + program）
- `octoscript/docs/capability-audits.md`：CapabilityHost audit export
- `octoscript/docs/workflow-checkpoints.md`：WorkflowEngine + checkpoint + lease
- `octoscript/docs/schema-contracts.md`：JsonToolContract 子集 + typed Rust adapters
- `octoscript/examples/makepad_ui_counter.octoscript`：**反例**——L2 imperative widget setter，不是 L0
- `octoscript/crates/octoscript-ui-l0/tests/fixtures/trip_planner.octoscript`：**反例**——L2 exemplar，30 个 `let` + `+ -` 拼接 lat/lon
- `finance-brief/MVP-TODO.md` §5：12 屏拆分表
- `finance-brief/docs/ARCHITECTURE.md` 附录 A Q-A：9 L0 + 3 L1 决策
- `finance-brief/docs/R-4-l0-cards.md`（并行）：每屏 L0/L1 草图