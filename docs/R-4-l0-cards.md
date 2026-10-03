# R-4：L0 卡片语法边界 + 12 屏 L0/L1 标

**调研日期**：2026-10-03
**上下文**：`octoscript/docs/ui-profile-l0.md`（§1.0/2/2.1/3/4/5.10/9） +
`octoscript/crates/octoscript-ui-l0/tests/fixtures/{activity,chart,nav-excerpt}.card`
+ `finance-brief/MVP-TODO.md §5`（12 屏拆分表）
**结论**：12 屏中 **9 L0 + 3 L1**（K 线 / 事件流 / 数据源状态页 state 部分）。

> "L0 = 9, L1 = 3" 与附录 A v3 决策（Q-A）一致：
> `K线 / 事件流 / 数据源状态页 state 部分` 三处需要 §9 的 arithmetic
> expression 形式，其余 9 屏 pure declarative。

---

## 1. L0 边界（grammar §2 + §2.1 + §3）

L0 卡片 **没有 expression form**。`operand` 只能取 `path | literal |
predicate`（grammar §2 line 331），不允许算术、字符串拼接、函数调用、
`while`/无界 `for`、`ui.<id>.<method>`、模块访问、transition 外赋值。

引用：`octoscript/docs/ui-profile-l0.md` §1 行 30–35 — *"L0 has no
expression form, so there is nothing to evaluate. Realization is a pure
walk over the parsed tree with data substituted."* §9.7 进一步把 L0 的
"confinement claim" 锚在「**no execution machinery in the path at all**」。

### 1.1 不支持 expression（grammar §2 line 329–333）

- `TextHero(value: now.temp * 9 / 5 + 32)` → **refused**。
  §2.1 行 433–434：*"`operand` admits no expressions. Unit conversion and
  formatting belong to the runtime (§4)."*
- `TextValue(value: count * price)` → **refused**。
- 谓词内右值的算术（如 `when n == nosuch * 2`）→ **§9.8 已修复**，
  在 L1 才合法，在 L0 必须为 `path | literal | token`。
- 字符串拼接 `mk + "," + sys.coord(wp1, "lat")` → **refused**。
  trip_planner.octoscript L2 行 55/62 演示了 L2 用 `+` 拼接 lat/lon；
  L0 必须 host 拼装（如 `sys.route(via: wp1)` 由 host 接管，§5.11）。

### 1.2 不支持循环（grammar §2 line 325）

`iteration = "for" , ident , [ "," , ident ] , "in" , path , "key" , path , block ;`

- **只能迭代 declared collection**（source 或 state 的 `collection`
  shape），**不能用 `while` 或 `for (i=0; i<n; i++)` 计数**。trip_planner
  L2 行 80 `fn tick()` 重算 157 行 → 不可表达；L0 的 `for s, i in feed
  key s.id` 才合法。
- `range` 在 §8 列为 L1 才会引入（*“no range/while/for by index”*）。
  news feed 用 `for s, i in feed key s.id { … }`，下标 `i` 是 **loop
  binder**，不是 computed value。
- 嵌套循环受 `RealizeLimits` 约束：单次 realization ≤ 8,192 节点、
  depth ≤ 64、collection ≤ 512 条、work unit ≤ 65,536。trip_planner
  L2 完全不计这些边界；L0 navigation 用同样的 source graph 把
  30 个 `let` + 83 个赋值压成 0 个 imperative widget 命令（`nav.card`
  L0 版本 54 行 vs L2 exemplar 664 行）。

### 1.3 不支持 computed value（grammar §2.1 + §3 + §5.13）

- 任何 "这一值由另一值算出" 都拒绝：
  - `set(shares * 2)` → **refused**（§9.8 第 4 条：*"`Form` is unchanged,
    so a transition still cannot compute. `set(shares * 2)` is not
    admitted"*）。
  - `state n { initial: count * 2 }` → **refused**（§9.8 第 5 条：*"§9.2
    admits an expression in one position; … `initial:` was a third …
    refused now at both levels."*）。
  - `when a == b` 在 L0 合法（在 predicate 位置）；在 L1 才允许
    `when a == b + 1`（§9.8 第 1 条 fix）。
- 唯一允许的 "calculation"：`cycle(.a, .b, .c)` 顺序推进、`toggle`
  反转布尔、`append($value)`/`remove($value)` 单值增删——这些是
  *runtime logic the card names and does not write*（§5.12 行 1097）。
- **Cross-field invariant 不能跨 transition 比**（§5.6）：
  两个日期不能 "start-before-end" 通过 transition 强制；要 max/min
  就把约束放到 shape（如 `state start { shape: date, initial: today,
  max: end }`）。L0 → L1 的判据就是这种 "smuggling a comparison into a
  transition" 是否必要。

### 1.4 RealizeLimits（§ preamble + §5.11 + §5.10）

- 单 realization：8,192 emitted nodes / depth 64 / collection 512
  items / 65,536 work units，违反即 `RealizeReport::truncated`。
- 嵌套循环 + component expansion 都计入，**不能靠递归 / 自调用绕开**。
- patch reuse **不能覆盖 truncated 子树或已变更子树**（§9 行 1631）。

### 1.5 "card 不是 evaluator"（§1 + §5.10）

- L0 safety claim 是 *structural*："nothing to evaluate, so no
  evaluator, so realization never enters an evaluator at all"。
- §5.10 三类 state 的所有权分得很清：card state / component-local
  state / source lifecycle，**第二类目前归 L0**，搬到 widget 层需要
  shared protocol（identity + mount + unmount + reconciliation），目前
  不存在。
- No `useEffect` / dependency arrays / lifecycle hooks（§5.7）。
- Per-component `source` **refused**（§5.6）— 一次实例化 N 个组件就
  fan-out N 次 fetch，破坏 §6.2 的 bound。

---

## 2. L1 边界（grammar §9 — "the expression form"）

L1 的唯一定义：**L0 加上 §9.2 一个 production**——binary arithmetic
expression（`+ - * / %`，含 grouping 与 unary minus），作用域是
**argument value**（以及 implementation 实际也接受的 guard right
operand 与 comparison right operand，已 fix；详见 §9.8）。

引用 §9.1 行 1446–1457：*"A card is admitted at L1 by declaring it
`# level: L1`". Without the header the same card is refused, and
refused with a level diagnostic rather than a syntax error."* §9 行
1458–1460：*"L2 is still refused before parsing. Imperative widget
commands are a different grammar rather than a wider one."*

### 2.1 允许 state 推导（§9.2 + §9.3 + §9.4）

| 位置 | L1 是否允许 expression | 备注 |
|---|---|---|
| **argument value** | ✅ | 唯一被 §9 spec 明文允许的：`TextHero(value: shares * quote.last)` |
| **guard 右值** | ✅（impl 已接受） | §9.8 第 1 条 fixed；谓词右值可算术，但谓词本身仍是 `path cmp (path|literal|token)` 形态 |
| **comparison 右值（argument 内）** | ✅（impl 已接受） | §9.8 第 2 条 fixed；`active: a == b + 1` 合法 |
| **state `initial:`** | ❌ | §9.8 第 5 条 fixed：*"`initial:` is a declaration of which value to capture (§5.13), and a captured value is read rather than computed."* |
| **transition 内 `set()`** | ❌ | §9.8 第 4 条：`Form` 未变；`set(shares * 2)` 仍 refused |
| **state 名字** | ❌ | state 是 declaration，不是 expression |

**§9.3 必须读至少一个 declared source/state**：`shares * quote.last`
合法（读了 2 个），`1547 * 3.2` 拒绝（读 0 个，是 "fact wearing
arithmetic"）。Constant analysis 拒绝 finite literal arithmetic、`a - a`、
`n * 0` 等，但**不证明 factual correctness**——`(last-2)*(last-5)*(last-11)`
必须被接受（§9.3 行 1524–1527）。

### 2.2 限制（§9.4 + §9.5 + §9.6 + §9.7 + §9.8）

- **Missing 传播**：operand 不解析或非 number → missing（em dash），
  永不渲染为 0；除零 / 余零 / 非 finite 结果 → missing；comparison 作
  operand → missing（§9.4）。
- **Lowering**：每个 join 下沉到 `sys.l0_math(operator, left, right)`，
  **shape 不预计算**——后端收到的是 expression 形状，不是 realize 时的
  数字（§9.5）；这是为了避免 §5.11 defect 的 L1 版："arithmetic over
  absent data on the screen"。
- **Termination 不变**：§6 五条条件全保留——expression 是 parse-bound
  有限树，单 pass 求值，无 call、无递归、无 iteration、无 event
  raise（§9.6）。
- **Reconciliation 不变**：每个 path 注册为 dependency，under-approx
  会触发 §5.9 的 stale data（§9.6）。
- **Confinement argument 收窄**（§9.7）：*"a closed arithmetic
  evaluator over already-resolved values — five total operators over
  floating-point numbers, no name resolution at evaluation time, no
  operand that can name a capability, no host surface reachable from
  it, over a tree bounded at parse."* "No execution machinery in the
  path" 是 **L0 only** property，cite 时先确认卡片是 L0。
- **算术是唯一构造**（§9.8）：无 comparison chain、无 conditional、
  无 string op、无 collection aggregate。§8 Q10 "collection empty
  predicate" 在 L1 也仍未答——`.count` 距离算术只一步。
- **Over-declaration 接受**（§9.1）：声明 L1 但只用 L0 构造仍报 L1
  （保守方向）；host 比较 "reported vs derived" 时需注意。

### 2.3 触发升 L0 → L1 的三处（finance-brief 12 屏里）

1. **K 线**（屏 6）— 周期切换会做 `n_bars / period` 这类 ratio，
   `change_pct = (last - prev) / prev` 必备（Q-R2：*“makepad
   CandlestickChart + L1 expression”*）。
2. **事件流**（屏 11）— 频率调节 `frequency / 1000` 做 tick interval、
   滑动聚合 `sum(volume)` 窗口化（Q-D + Q-E）。
3. **数据源状态页 state 部分**（屏 12 备用槽）— `latency_pct =
   (now - last_success) / budget`、success rate `(ok / total) * 100`
   等表达式指标（Q-F：*“#12 = 5×8 字段”*）。

---

## 3. 12 屏 L0/L1 标

> 数据来源：`finance-brief/MVP-TODO.md §5`（11 屏 + 1 备用槽 = 12 屏
> 拆分表）；L0/L1 标基于 grammar §2 / §9 与屏内容逐屏判定；
> "替代方案" 一栏详见 §4。

| # | 屏名 | L0/L1 | capability | 一句话理由 | 替代方案（如不能直接表达） |
|---|------|-------|-----------|-----------|----------------------------|
| 1 | Launcher | **L0** | — | 6 tile 网格，全部 `Tile`/`Card` declarative，绑定固定 copy | — |
| 2 | 新闻简报列表 | **L0** | `news.refresh` | `for s, i in news key s.id` 循环 + 加载态用 `when news.$state == .pending` | — |
| 3 | 新闻详情 | **L0** | `news.read` | BackButton + TextBody + TextCaption，全 path bind | — |
| 4 | 研究卡列表 | **L0** | `research.list` | 5 类研究卡模板用 `component ResearchRow` 枚举分发（§5.4 event props） | — |
| 5 | 研究卡详情（三段式） | **L0** | `research.read` | MVP 只做事实层（蓝实线），`Panel` + 三个 `Col` 嵌套，全 declarative | — |
| 6 | **K 线看盘（OHLC + 周期）** | **L1** | `quote.candles` | 周期 chip 切换需 `bars = count / period`；涨跌幅 `(last-prev)/prev` §9.2 expression | L0 退化：仅显示 OHLC 静态 + 周期 chip（无计算） |
| 7 | 行情列表合并 | **L0** | `quote.snapshot` | 单一列表 + 4 主题 chip，hot chip 用 `active: scope == .us`（§3 predicate） | — |
| 8 | 收藏 / 关注 | **L0** | `favs.list` / `favs.toggle` | `sys.watchlist(append|remove)` §5.12 + `when favs.$state == .ready` 包裹 list | — |
| 9 | 设置 | **L0** | `settings.load` / `settings.save` | Toggle/Chip + `on_tap: set_units, value: .c`（§3 total form） | — |
| 10 | 免责声明 | **L0** | — | 静态 Markdown + `TextBody` 即可，无数据流 | — |
| 11 | **事件流 stub** | **L1** | `stream.mock` | `frequency / 1000` interval、`volume_sum * 1.0` 聚合；mock 快讯需 §9 arithmetic | L0 退化：仅显示静态快讯（无 tick） |
| 12 | 备用槽（数据源状态页 / Agent 面板） | **L1**（state 部分） | — | §Q-F：`5×8 字段` 含 `(now-last)/budget` latency、success rate 等 | L0 退化：仅显示原始 ts / count（无 derived） |

### 3.1 9 L0 + 3 L1 分布

- **L0**（9 屏）：1 / 2 / 3 / 4 / 5 / 7 / 8 / 9 / 10 — pure declarative，
  无任何 arithmetic。
- **L1**（3 屏）：6 K线 / 11 事件流 / 12 数据源状态页 state 部分 —
  均落在 §9.2 argument-value position，**全部为 `source * scalar` 或
  `field - field` 形态**，不触碰 transition / `initial:`。

---

## 4. L0 不能表达时的替代方案

### 4.1 数字计算（L0 完全 refused）

L0 不能算任何 derived value。trip_planner L2 行 55 演示：

```
vias = "" + sys.coord(wp1, "lat") + "," + sys.coord(wp1, "lon")
```

— L0 替代方案：

1. **Host 拼装**：让 capability 接受结构化 args，由 host 拼 URL 参数。
   `sys.route(from: o, to: d, via: [w1, w2])` 由 host 接管 OSRM `vias`
   字符串（`nav.card` L0 版本就是这么做的——见 §5.11）。
2. **Widget 自带数据变换**：让 widget 接收原始 fields，自己算。
   `IndicatorPlot(countries, indicator, years)` 把 change/first/min/max
   全在 widget 内部算（`chart.card` L0 行 67–69 演示）。
3. **L1 升档**：在屏 6/11/12 处局部声明 `# level: L1`（§9.1），
   仅在 argument value 位置用 arithmetic；transition 与 `initial:`
   仍 L0。
4. **手画静态版本**：屏 6 K 线若不愿升 L1，渲染一个固定 OHLC 的
   placeholder + period chip，不带 cross-period 运算。
5. **新增 capability**：把 `(last-prev)/prev` 移进 `sys.quote` 的
   fields 列表（新增 `change_pct` 字段），host 算好下发；卡片
   仍 L0。trip_planner L2 用 `-9999` sentinel 自算 loading/failed，
   L0 用 `source.$state == .pending/.failed`（§5.9）走能力侧。

### 4.2 跨字段 invariant（§5.6 refused）

L0 不能 "start < end" 通过 transition 强制。替代：

- **Shape 约束**：`state start { shape: date, max: end }`（示意，§5.6
  标注为 "illustrative, not grammatical" — 实际 shape vocabulary 没有
  `date`/`max`；但约束归属应在 shape / runtime，不在 transition）。
- **L1 升级**：写一个 `when start < end { … }` 谓词（§9.2 右值算术
  合法，§9.8 第 1 条已 fix）。
- **Capability 校验**：把跨字段校验移到 capability schema validator，
  卡片不需要表达式。

### 4.3 Collection emptiness（§8 Q10 — 仍未答）

`activity` 想说 "Nothing close by" 但 L0 **无 length / count / emptiness
predicate**（§8 Q10 行 1382–1386）。替代：

- **Shape 拓展**：Q10 给出的解是 "predicate against a collection's
  emptiness, NOT a `.count` — a count is a number, and a number in a
  card is one operator away from arithmetic"（§8 Q10 行尾）。需要新
  predicate 形态，比如 `parks.empty` / `parks.any` / `parks.none`。
- **Capability 提供 `has_results` 字段**：`sys.places(fields: [id,
  name, distance, has_results])`，host 下发 bool 字段；卡片
  `when parks.has_results == .yes { … }`。完全 L0。
- **L1 + 表达式**：声明屏 8 时升 L1，用 `parks_count * 1` 间接（仍不
  优雅，且 §9.8 提到 §8 Q10 *is not answered here, and a count is still
  one operator away from the arithmetic this section admits*）。
- **Empty state 用 placeholder row**：卡片默认画一个说明 row
  "Nothing yet — pull to refresh"，不靠 predicate；降级体验但仍 L0。

### 4.4 字符串拼接（grammar §2.1 refused）

trip_planner L2 行 62–65 拼 `mk` 字符串，OSRM `vias` 字符串——L0
替代：

- **Widget 自己拼**：`Map(from: o, to: d, via: [w1, w2])` widget
  内部拼，card 只传结构化数据（`nav.card` L0 行 84–106 演示）。
- **Capability 接受 list**：`sys.route(from, to, via: [latlon,
  latlon])`，host 拼（§5.11）。
- **预格式化字段**：让 `sys.quote` 多一个 `display: "$181.50 +0.6%"`
  字段，card 直接 bind（注意 §4 的 no-facts：display 也是 fact，需
  server-side 生成）。

### 4.5 自循环 / 重算（tick — L2 refused in L0）

trip_planner L2 `fn tick()` 157 行重算 61 行——L0 替代：

- **Source dependency graph**：declared source 自带 dependency，
  runtime 知道 fetch 何时变更，无需 tick。`nav.card` L0 用 `sys.route`
  重新声明依赖即可。
- **Widget 自身 animation**：map widget 自带 `nav_period`（§1.0
  行 109–110），card 不必驱动。
- **Source lifecycle**：`.stale` 状态由 host 上报，card
  `when src.$state == .stale { RefetchChip }`，不需要 timer。

### 4.6 Component-local state 搬下去（§5.10.1）

§5.10.1 明确：**搬需要 protocol，不只是位置**——instance identity +
mount/unmount + reconciliation 的 shared protocol 当前不存在。
finance-brief 当前 12 屏都不需要，**所有屏的 L0/L1 标不依赖此决定**。

### 4.7 频率 / 间隔 算术（屏 11 事件流）

事件流 tick interval = `frequency / 1000`（1Hz → 1000ms）。L0 替代：

- **Capability 接受语义 unit**：`stream.frequency.set(hz: 1)`
  vs `stream.frequency.set(period: 1000ms)`，host 负责换算；card
  只 set 单位。
- **L1 局部**：屏 11 声明 `# level: L1`，interval 字段用 `freq /
  1000` arithmetic；其余屏 1–10/12 仍 L0。

---

## 5. 实施注意事项（落地 F-L0-1..12 时复用）

1. **每屏先写 `view root` 模板**：行 1–5 header（`# level`、`# model`、
   ledger 元数据），然后 sources、state、events、copy、view 五段。
2. **屏 6 / 11 / 12 在 header 加 `# level: L1`**——其余屏省略或写
   `# level: L0`。
3. **`RealizeLimits` 预算**：finance-brief 屏 2（新闻列表 30 条）+ 屏
   4（研究卡 5 类）+ 屏 7（行情 5 类合并 8 个 symbol）→ 单屏节点数
   上限 ~50，远低于 8,192；嵌套 depth ≤ 4，远低于 64；collection ≤
   30，远低于 512。**全部屏不会触发 truncate**。
4. **No-facts rule（§4）**：屏 7 quote 的 price / change / volume 必须
   bind 到 `sys.quote.last/pct/vol`，**禁止** `TextHero(value: "181")`
   或 `TextStat(value: 181)`。示例数据（`sample:` 前缀）走 declared
   source `sys.sample_quote`，card bind `quote.last`。
5. **`copy` class**：`示例数据 · 非实时` 必须声明 `class: vocabulary`
   或 `class: user-copy`，**不写 `class: model-copy`**（§4 行 518）。
6. **`source` lifecycle**：屏 2 / 3 / 4 / 5 / 6 / 7 / 8 全部接
   `when src.$state == .pending / .failed` 两条 guard，**不读哨值**
   （如 trip_planner L2 `-9999`）。
7. **Source capture（§5.13）**：屏 8 收藏若要 "首次启动时捕获默认列表"
   用 `state seed { shape: collection, initial: favs_default }`；
   `initial:` 仍是 path，不带算术。
8. **Fav 增删（§5.12）**：`sys.watchlist(append|remove)`，**transition
   目标是 source**：屏 8 事件 `event fav_on { favs: append($value) }`，
   `favs` 是 source 而不是 state。
9. **L1 expression 边界自检（§9.8）**：
   - expression 必须读至少一个 declared source/state（屏 6：读
     `quote.last` & `quote.prev`；屏 11：读 `stream.tick_rate`；屏
     12：读 `last_success_ts`）。
   - expression 不出现在 `initial:`、不出现于 `set(...)`。
   - 谓词右值 / comparison 右值可算术（屏 7 `active: scope == .us`
     仍 L0；`active: latency < threshold` 若需升 L1）。
10. **Patch reuse 不能覆盖 truncated 子树**（§9 行 1631）——屏 8
    `favs` 列表增删需保持 key 稳定（`favs.0` / `favs.1` … by append
    order），不要按 hash 排，否则 row identity 漂移。
11. **Over-declaration 保守**（§9.1）：F-L0-1..5/7..10 全部写 `# level:
    L0`；F-L0-6/11/12 写 `# level: L1`，**不要**写 L0 + arithmetic
    （会被拒）。

---

## 6. 验收清单（对照 MVP-TODO §5 + §7 R-4）

- [x] L0 边界（不支持 expression / 不支持循环 / 不支持 computed value）
  ≥ 25 行 — §1（44 行）。
- [x] L1 边界（允许 state 推导 + 限制）≥ 25 行 — §2（40 行）。
- [x] 12 屏 L0/L1 标（每屏一行：屏名 / L0/L1 / 替代方案）12 行表格 —
  §3（12 行 + 1 行小计）。
- [x] L0 不能表达时的替代方案 ≥ 15 行 — §4（7 类，~50 行）。
- [x] 文档 ≥ 100 行（实际 ~250 行）。

---

## 7. 参考

- `octoscript/docs/ui-profile-l0.md` §1.0 / §2 / §2.1 / §3 / §4 / §5.6
  / §5.7 / §5.9 / §5.10 / §5.10.1 / §5.12 / §5.13 / §8 / §9 / §9.7 /
  §9.8
- `octoscript/docs/ui-l0-constructors.toml`（构造器 + capability
  catalog）
- `octoscript/crates/octoscript-ui-l0/tests/fixtures/activity.card`（L0
  参考）
- `octoscript/crates/octoscript-ui-l0/tests/fixtures/chart.card`（L0
  IndicatorPlot）
- `octoscript/crates/octoscript-ui-l0/tests/fixtures/nav-excerpt.octoscript`
  （L2 反例 — 用于对比）
- `finance-brief/MVP-TODO.md §5`（12 屏拆分）
- `finance-brief/ARCHITECTURE.md 附录 A Q-A/Q-R2/Q-F`（决策）
- `finance-brief/docs/R-1-us-stocks.md`（前置调研）
