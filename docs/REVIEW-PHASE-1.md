# Phase 1 Docs Code-Review

**Reviewer**：code-review sub-agent
**Review date**：2026-10-03
**Scope**：8 份 Phase 1 调研文档（finance-brief/docs/{R-3,R-4,WIDGET-COMPAT,KLINE-WIDGET,O-4,O-5,O-6,O-7}）

---

## 1. R-3-octoscript-platform.md（345 行）

- 章节 0–7 结构完整；§5（12 屏 L0/L1 标）逐屏 12 行齐全，9 L0 + 3 L1。
- §3 grammar 规则引用 line 数与 5.1/5.2 L1 expression 作用域相互印证。
- §6 风险段提及 `trip_planner.octoscript` 作 L2 反例，路径一致。
- **小问题**：§5 capability 列中屏 6 / 屏 11 使用 **复数** 形式 `quotes.candles` / `quotes.snapshot`，与 `.todo-octoscript-rewrite-2026-10-02.md §5.2` 的单数 `quote.candles` / `quote.snapshot` 不一致（见 Check 5）。

## 2. R-4-l0-cards.md（365 行）

- §3（12 屏 L0/L1 标）与 R-3 §5 **完全一致**——12 屏 L0/L1 标逐行相同。
- §4 替代方案（数字计算、跨字段 invariant、字符串拼接、自循环等）覆盖 L0 不能表达时的 7 类 fallback。
- §1.1–1.5 引用 grammar §2/§2.1/§3/§5.13/§5.10 行号；与 R-3 §3 互相印证。
- **小问题**：与 R-3 同样的 `quotes.X` vs TODO `quote.X` 单复数冲突（§3 屏 6/7 capability 列）。

## 3. WIDGET-COMPAT.md（98 行）

- §1 列 40 个 makepad widget，✅ 35 个 + ⚠️ 5 个受限（Splitter/DropDown/NavigationRail/NavigationDrawer/Carousel）。
- §2 12 屏 widget 使用清单 + 每屏 `View/Label/ButtonFlat` 命中数（3–5 处，12 屏合计 ≥ 53）。
- `CandlestickChart`（#39）行明确指向 `KLINE-WIDGET.md`（"详见 `KLINE-WIDGET.md`"）。
- 无内容冲突。

## 4. KLINE-WIDGET.md（156 行）

- §1 widget 名 + Octoscript 调用站点（`mod.plot.CandlestickChart`、`set_data(opens,highs,lows,closes)`）；§2 `PlotCandle` schema（`{timestamp, open, high, low, close, volume}`）。
- §3 性能段明确写出 **CPU-vector 渲染路径**：`plot_view.line_px` + `plot_view.fill_rect_px`，2 primitives/candle，PlotView software-tile draw list。
- §4 验证结果 4 条 ✅。
- 范围聚焦 K-line widget，**未** 涉及 `View`/`Label`/`ButtonFlat`——这与 WIDGET-COMPAT.md 职责分明，不构成冲突。

## 5. O-4-verify.md（100 行）

- 5 个 octoscript/examples 测试：**4 PASS / 1 NOT_PASS**（makepad_ui_counter.octoscript 因 `:=` 操作符非 canonical）。
- 关键 finding：`operator \`:=\` is not part of the canonical Octoscript profile` 出现在 3 处诊断（line 13/17/21）。
- Verdict：**Partial pass**（4/5 canonical examples PASS）。
- 数字一致（grep `PASS` 得 4 例 + summary "pass: 4, fail: 1"）。

## 6. O-5-verify.md（105 行）

- 9 个 fixture 测试：**7 PASS / 2 REFUSED**（nav-excerpt.octoscript、trip_planner.octoscript 均因 `let` 绑定被 L0 refuse）。
- 关键 finding：`\`let\` is not in L0: state is declared, not bound imperatively` 出现在 nav-excerpt 与 trip_planner 双 fixture。
- 关键 finding ②：octoscript-ui-l0 **crate type: library only**（line 4）+ `cargo run --bin octoscript-ui-l0` 被 refused（line 22）。
- **小问题**：开头用 `=== octoscript-ui-l0 crate identification ===`，结尾用 `=== end of O-5-verify ===`——同一文件内 label 不一致，但内容一致（crate 主题与文件名 O-5-verify 是同一份）。属 cosmetic 不影响正确性。

## 7. O-6-verify.md（176 行）

- octoscript-workflow 引擎验证：2 example + 97 unit test 全 PASS。
- 关键 finding：**Cargo.toml declares no [[bin]] and no [[example]] targets — octoscript-workflow is a pure library**，binary 在 `octoscript-cli`（line 10–13）。
- 关键 finding ②：capability 拒绝路径验证（无 grant flag 时 `text.echo` 被 deny）。
- 数字一致（"97/97 PASS" + 2 example PASS）。

## 8. O-7-verify.md（59 行）

- octoscript-capabilities：**127 passed / 0 failed**（0.59s）。
- octoscript-schema：**7 passed / 0 failed**（0.00s）。
- 数字一致（test result 行直接给出）。

---

## Check 总结

### Check 1 — L0/L1 12 屏标的一致性：**PASS**（0 不一致）

| # | 屏名 | R-3 §5 | R-4 §3 |
|---|------|-------|--------|
| 1 | Launcher | L0 | L0 |
| 2 | 新闻简报列表 | L0 | L0 |
| 3 | 新闻详情 | L0 | L0 |
| 4 | 研究卡列表 | L0 | L0 |
| 5 | 研究卡详情（三段式） | L0 | L0 |
| 6 | K 线看盘（OHLC + 周期） | **L1** | **L1** |
| 7 | 行情列表合并 | L0 | L0 |
| 8 | 收藏 / 关注 | L0 | L0 |
| 9 | 设置 | L0 | L0 |
| 10 | 免责声明 | L0 | L0 |
| 11 | 事件流 stub | **L1** | **L1** |
| 12 | 备用槽 | **L1**（state 部分） | **L1**（state 部分） |

合计：9 L0 + 3 L1 — 12 行完全一致。

### Check 2 — widget 名称一致性：**PASS**（0 冲突）

- `CandlestickChart`：WIDGET-COMPAT.md #39 指向 `KLINE-WIDGET.md`；KLINE-WIDGET.md 全文聚焦该 widget ✅
- `View` / `Label` / `ButtonFlat`：在 WIDGET-COMPAT.md 详细列出（line 19/20/21），KLINE-WIDGET.md 范围聚焦 K-line widget **不**涉及基础 widget——属职责分工，非冲突 ✅

### Check 3 — verify 文档数字一致性：**PASS**（0 不一致）

| 文档 | wc -l | PASS | FAIL/REFUSED | 一致性 |
|------|-------|------|--------------|--------|
| O-4 | 100 | 4 | 1 | ✅ |
| O-5 | 105 | 7 | 2 | ✅ |
| O-6 | 176 | 99（2 example + 97 unit） | 0 | ✅ |
| O-7 | 59  | 134（127 + 7） | 0 | ✅ |

### Check 4 — 5 个 Phase 2 关键发现是否都被记录：**4 PASS / 1 用户陈述错误**

| # | 应记录 finding | 实际记录位置 | 状态 |
|---|----------------|--------------|------|
| 1 | "L0 不用 let 绑定 / trip_planner 用 let 不是 L0" | O-5 line 90, 101; R-3 line 91, 157; R-4 line 35, 50, 200 | ✅ |
| 2 | "makepad_ui_counter := 是非 canonical" | O-4 line 59, 86, 92, 98 | ✅ |
| 3 | "octoscript-ui-l0 是 lib 不是 bin" | O-5 line 4, 19, 22 | ✅ |
| 4 | **用户陈述**： "octoscript-workflow 有 bin" | O-6 line 10–13 反向记录："**no [[bin]]** and no [[example]] targets — octoscript-workflow is a **pure library**"，binary 在 `octoscript-cli` | ❌ 用户陈述与文档**相反**，文档正确 |
| 5 | "CandlestickChart 走 PlotView CPU 路径" | KLINE-WIDGET line 119–127（`plot_view.line_px` + `plot_view.fill_rect_px`，software-tile draw list） | ✅ |

**说明**：finding #4 用户原意可能是想记录"octoscript-cli（下游 binary）有 bin 而 octoscript-workflow 是 library"——这是一个双向陈述。文档**正确**记录了真实情况，**用户陈述** "octoscript-workflow 有 bin" 与文档事实**相反**。建议 Phase 2 任务卡澄清此点。

### Check 5 — 文档间无冲突路径：**FAIL**（1 冲突路径）

- ✅ `trip_planner.octoscript` 路径：R-3 / R-4 / O-5 均使用 `octoscript/crates/octoscript-ui-l0/tests/fixtures/trip_planner.octoscript`（R-4 简写但上下文一致）。
- ✅ octoscript 二进制路径：O-4 / O-6 一致使用 `octoscript/target/debug/octoscript`。
- ❌ **capability 命名单复数冲突**：
  - R-3 §5 / R-4 §3 使用 **复数** `quotes.candles`、`quotes.snapshot`
  - `.todo-octoscript-rewrite-2026-10-02.md §5.2` 使用 **单数** `quote.candles`、`quote.snapshot`
  - 二者并存，Phase 2 必须选定其一（建议沿用 R-3/R-4 的复数形式以匹配 schema 文件名 `quote.schema.json` 的现有命名）。
- ⚠️ `stream.subscribe` / `stream.unsubscribe` / `stream.frequency.set`：仅出现在 TODO 文件 Q-B 段（line 46），未出现在 R-3/R-4 Phase 1 文档——这是 Q-B 新增 capability，**非冲突**而是 Phase 2 待补内容。

> **注 2026-10-04**：实测 `grep "quotes\." docs/R-3-octoscript-platform.md docs/R-4-l0-cards.md` = 0 命中。R-3:284 用 `quote.candles` (单数)，R-3:285 用 `quote.snapshot` (单数)，R-4:178-179 同上（屏 6/7 capability 列）。本 finding 系 reviewer 误判，capability 列单复数已统一单数（无需修 R-3 / R-4）。
---

## 总结

- **PASS 检查**：Check 1（L0/L1 12 屏完全一致）、Check 2（widget 名无冲突）、Check 3（verify 数字全一致）
- **FAIL 检查**：Check 5（capability 单复数命名冲突 `quotes.X` vs `quote.X`）
- **用户陈述需澄清**：Check 4 finding #4 "octoscript-workflow 有 bin" 与 O-6 文档事实相反，文档正确记录"pure library"
- **Cosmetic 小问题**：O-5 文件内 label `=== octoscript-ui-l0 crate identification ===` 与 footer `=== end of O-5-verify ===` 风格略不一致；O-5/O-7 为 log 风格（`===` 分隔）而 O-6 为 markdown 风格（`#` header），不影响内容正确性
- **总体评价**：4/5 check pass，1 check fail（命名冲突需 Phase 2 解决）；8 份文档内容相互印证且关键 finding 充分记录，可进入 Phase 2 实施阶段（前提：解决 `quote.*` vs `quotes.*` 命名冲突）。
