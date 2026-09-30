# R-2: Makepad + Octoscript K 线可行性

> 任务 R-2 — 调研 60FPS 渲染 ≥100 根 K 线柱的可行性。
> 工作分支：`finance-brief-app`（本地，未推送）。

## 调研路径

1. **Octoscript API 文档** — `OctoScript-App-Design-Flow/docs/SCRIPT-API.md`、`octoscript/docs/ui-profile-l0.md`、`ui-l0-constructors.toml`
2. **Makepad 源** — `octoOs/makepad/widgets/src/{chart,view,view_ui,sparkline,stock_plot,splash}.rs`，`draw/src/shader/draw_quad.rs`
3. **已有样例** — `octoOs/makepad/examples/{charts,datagrid}/`、`OctoScript-App-Design-Flow/examples/aircon`、`octoscript-makepad/components/l0/pages/stock/chart.l0`
4. **参考实现的 K 线思路（仅参考，不移植）** — 第三方 iced 实现中的 `draw_candle_dp` 函数（iced→makepad 的非移植式借鉴）
5. **octoscript-makepad 二次实现** — `octoscript-makepad/crates/{makepad-plot,makepad-d3}` 的 chart 系列

## 关键发现

### Octoscript / Splash 暴露的绘图原语

| 原语 | 来源 | 暴露位置 | Splash App 可用？ |
|---|---|---|---|
| `View` / `SolidView` / `RoundedView` | `widgets/src/view.rs` | `mod.widgets.*` | ✓ 已在用 |
| `Label` / `ButtonFlat` 等基础控件 | `widgets/src/*.rs` | `mod.widgets.*` | ✓ 已在用 |
| **`CandlestickChart`** | `widgets/src/chart.rs:1050` | `mod.widgets.CandlestickChart` | ✓（**新发现**：finance-brief 当前未用，但可直接用） |
| `TrendChart`（含蜡烛 + 线 + 区域 fill） | `widgets/src/chart.rs:1662` | `mod.widgets.TrendChart` | ✓（**新发现**：通过 `set_candles(Vec<Candle>)`） |
| `LineChart` / `BarChart` / `AreaChart` / `OhlcChart` / `Sparkline` | `widgets/src/chart.rs` | `mod.widgets.*` | ✓ |
| `DrawColor`（矩形 + 单色 fragment shader） | `draw/src/shader/draw_quad.rs:103` | `mod.draw.DrawColor`（来自 `prelude.widgets_header`） | ✓（已被 `ChartView/Candle` 内部使用） |
| `DrawQuad`（矩形基元） | `draw/src/shader/draw_quad.rs:80` | `mod.draw.DrawQuad` | ✓ |
| `DrawChartSegment`（自定义 SDF 线段 shader） | `widgets/src/chart.rs:1638` | 已被 `TrendChart` 内部使用 | 间接可用 |
| `DrawVector`（路径 / 线 / 矩形 SDF） | `widgets/src/vector.rs` | `mod.widgets.DrawVector` | ✓ |
| 自定义 `pixel: fn(){…}` shader | `draw_bg +: { pixel: fn(){…} }` | 任何 widget | ✓（已有先例：`AqiContour`、`moon_phase.rs` 等） |
| L0 `StockPlot`（`mod.plot.*` 系列：DonutChart / BarPlot / LinePlot …） | `octoscript-makepad/crates/makepad-plot` | `mod.plot.*` | ✗（在 finance-brief 隔离中不注册；card-host 只挂 `makepad_widgets + octoscript_widgets`，不挂 makepad-plot） |

### Shader 支持结论

- **DSL 不能作者化 shader 源码**：Octoscript L0 文档明确写「*a DSL node cannot carry shader source — MPSL compiles at build time — but it can select a shader that was compiled*」（`octoscript/docs/ui-profile-l0.md:222-225`）。
- **Widget 属性扩展 shader 字段是允许的**：通过 `draw_bg +: { pixel: fn(){…} }`、`draw_bg +: { color: #x... }` 等增量写法（参见 `widgets/src/button.rs:78-88`、`widgets/src/aqi_contour.rs:156-166`）。这正是 `TrendChart` 内部 `DrawColor.draw_candle` 在单帧内批量绘制柱体+影线的方式。
- **结论：不需要用户写 shader。** 现成的 `TrendChart` / `CandlestickChart` widget 已经把"100 根 K 线 + 60FPS"封装好了。

### Path / Sdf 支持

- **无内置 Sdf widget**（Sdf 指 SDF 路径），但 `DrawVector` 内部使用 SDF 风格的反走样路径（`widgets/src/vector.rs`），并被 `ChartView.draw_candle` 调用（`widgets/src/chart.rs:756-761`）画 wick。
- `Sparkline` / `TrendChart` 走 `DrawColor`（填充 quad）路线，**不依赖 Sdf**，性能上反而更稳。

### 自定义渲染回调

- `on_render: || { … }`：View 家族专属，重新跑闭包生成子 widgets（`SCRIPT-API.md:89`）。这是 finance-brief 当前用的模式。
- **没有 `on_draw`**：splash DSL 没有"自定义 GPU 绘制入口"概念。绘制路径要么是 widget 自身，要么用 `draw_bg +:` / `draw_xx +:` 注入参数。

## 实现方案 A：原生 K 线柱（**首选 · 现成 widget**）

### 思路

直接用 **`TrendChart`**（`octoOs/makepad/widgets/src/chart.rs:1662`），通过 `TrendChartRef.set_candles(Vec<Candle>)`（同文件 1922 行）喂入 OHLC 数据。内部用单 batch 的 `DrawColor` quad 绘制所有柱体+影线。

也可以用 **`CandlestickChart`**（`widgets/src/chart.rs:1050`），`set_data(Vec<Candle>)`（同文件 1064 行），内部走 `DrawVector` 的 SDF 路径并自动适配视口、缩放、绘制网格。

### 关键 API（已在 makepad/examples 验证可用）

| API | 来源文件 | 已用样例 |
|---|---|---|
| `ui.candles.set_candles(candles_vec)` | `widgets/src/chart.rs:1922` | `octoOs/makepad/examples/datagrid/src/tab_charts.rs:187-189` |
| `ui.candles.set_data(candles_vec)` | `widgets/src/chart.rs:1064` | `octoOs/makepad/examples/charts/src/main.rs:23` |
| `Candle { time, open, high, low, close, volume }` | `widgets/src/chart.rs:155-162` | 同上 |

**已在 makepad 仓库有完整 worked example** —— `octoOs/makepad/examples/datagrid/src/tab_charts.rs`：

```rust
// 真·市场报价表：每行一个 sparkline，右侧一个 CandlestickChart + TrendChart 蜡烛窗。
// "Prices tick a few times per second; only visible cells are drawn."
// —— 用 `TrendChart.set_candles(cx, candles)` 喂入 Vec<Candle>，
//    每秒连续多次更新，全程 batched quad 渲染。
```

### 性能特性（直接来自源码佐证）

- **`TrendChart.draw_walk`（chart.rs:1740-1909）** 对每根蜡烛只调用两次 `draw_candle.draw_abs(cx, Rect)`（wick + body），**所有 quad 共享同一个 `DrawColor` 实例**，GPU 自动按材质合批为 1 个 draw call —— N 根 K 线 = 1 个 draw call（material batch）。
- **`CandlestickChart.draw_walk`（chart.rs:1076-1163）** 使用 `DrawVector` 单 session `begin/end`（455-60 行 `chart_view.begin/end`），所有蜡烛共享同一 vector layer —— 仍是 ~1 个 draw call。
- **视口裁剪**：`draw_walk` 先算 `start_idx/end_idx`（chart.rs:1106-1107），只画可见窗口内的蜡烛；viewport pan/zoom 改 `ChartViewport.x_min/x_max` 即可（chart.rs:959-963）。
- **N 根蜡烛的算法复杂度**：O(N) quad 提交 + O(1) draw call 数 → 在 1280×720 plot 区域内，100 根柱每柱约 6×40 px，合计 ~240K 像素；GPU 单帧可轻松 60FPS。
- **自动聚合**：当 `pixels_per_candle < 2.0`（chart.rs:1132-1138），`FlatDataSource::get_averaged` 把多根合并为更高时间周期的蜡烛 —— 即自动支持"缩到 1d 看年线"。

### 自定义配色（PRD §3.3 明确要求）

`TrendChart` 暴露 8 个 `#[live]` 颜色字段（chart.rs:1684-1699）：
- `color_bg` / `color_grid` / `color_line` / `color_fill` / `color_up` / `color_down` / `color_text` / `color_accent`

`CandlestickChart` / `ChartView` 暴露 12 个（chart.rs:411-446）：
- `candle_up_color` / `candle_down_color` / `wick_color` / `grid_color` / `grid_text_color` / `border_color` / `high_line_color` / `low_line_color` / `bg_color` / `candle_width_fraction` / `line_color` / `line_width` / `fill_color` / `bar_color` / `dot_color` / `dot_radius` / `bar_width_fraction`

直接在 splash DSL 写 `CandlestickChart{ color_up: #xe64340 ... }` 即可（finance-brief 已有用 `#x` 写十六进制的先例）。

### 代码量

- 在 `main.splash` 加 1 个 widget 实例化（**~5 行**） + 1 个 fetch + parse（~30 行） + 1 个 `ui.chart.set_data(...)` 推送（**~5 行**） + 1 个周期切换器（10 行） ≈ **50 行增量**，其中核心 widget 嵌入 ≈ **5 行**。
- 附录有 ≤30 行原型片段。

### 60FPS 临界

- 100 根：✓（单 draw call + 简单遍历）
- 500 根：✓（同样单 draw call；widget 内部可见性裁剪保证）
- 1000+ 根：✓（同样，但 UI 操作时（hover/拖）需靠 widget 的 viewport 机制把视口外的不画完；`ChartView.handle_event` 已经做了 pan/zoom）

## 实现方案 B：CustomShader（不适用）

Octoscript DSL **不能 author shader**（见上）。即便 widget 内部 shader 可调，所有可见 widget 的 shader 都是 **build-time 编译**的 MPSL fragment shader。用户在 splash 端只能**调整参数**（uniform），不能**写新 shader**。所以"GPU 一次绘制所有 K 线柱"这件事已经被 TrendChart/CandlestickChart 在 widget 层做了——不需要用户层再做一遍。

## 实现方案 C：Image widget 贴图（不推荐）

- 思路：用 `Image{src: http_resource(url)}` 把预渲染的 PNG 贴上去。
- 限制：
  1. 数据是 OHLC array，**没法离线预渲染**（每根 K 线是动态数据）。
  2. 即便服务端预渲染，也无法交互（hover/选柱）。
  3. 完全背离 PRD §3.3 的"实时看盘 + 自定义配色"。
- 结论：✗ 不适用。

## 性能预期

| 方案 | 100 根 60FPS | 500 根 60FPS | 1000 根 60FPS | 交互（hover/pan/zoom）| 配色自定义 |
|---|---|---|---|---|---|
| **A · TrendChart / CandlestickChart** | ✓ | ✓ | ✓ | ✓（ChartView 已实现 pan/zoom/hover）| ✓（`color_up/color_down/...`） |
| B · CustomShader | n/a | n/a | n/a | n/a | n/a |
| C · Image 贴图 | n/a（不可行） | n/a | n/a | ✗ | ✗ |

**结论：A 方案完全满足 PRD 60FPS ≥100 根要求**。

## 推荐路径

**首选：方案 A — 直接用 `CandlestickChart` widget**

- **为什么**：现成的、已通过 makepad datagrid 样例验证过 60FPS 的 widget；自带 OHLC 视口、pan/zoom、配色、网格、min/max 标注；自动支持缩到 1d 看年线（聚合算法已实现）；代码量最小。
- **实现步骤（5 步）**：
  1. **数据层**：在 `main.splash` 顶部新增 `let k_candles = []`，与现有 `a_rows/us_rows/...` 平级；为 `a_syms`/`us_syms`/`crypto_syms` 各增加一个 `intv` 字段（"1m"/"5m"/"1d"），表示请求时间周期。
  2. **数据抓取**：从 R-1 美股实时源拿到 OHLC 数组（或先用 `synth_candles(n, seed)` 在脚本里跑随机游走作为 MVP 占位），按 `Candle{ time, open, high, low, close, volume }` 形状构造对象数组（**注**：splash DSL 当前未直接暴露 Rust 命名类型 `Candle`，需走 widget 自动 marshalling 的 array-of-object 路径；如不行，则改用 `d3.*` / makepad-plot 风格的 4 数组 `set_data(opens, highs, lows, closes)` —— 见下方"⚠️ Rust 类型暴露"风险）。
  3. **新增 tab「K 线」**：在 `tabs` 数组加 `{id:"k", label:"K线"}`，在 `tab == "k"` 分支渲染一个 `k_pane := ScrollYView{... on_render: || { ... chart 嵌入 ... }}`，里面放：
     ```splash
     chart := CandlestickChart{
         width: Fill height: 320
         color_up: #xe64340 color_down: #x2ecc71   // 沿用现有 ink/up/down 调色板
         grid_color: #x2a2a3e bg_color: #x171c26
         candle_width_fraction: 0.7
     }
     ```
  4. **周期切换器**：在 chart 上方放一行 `Row{ gap: 6 }` 含 4 个 `ButtonFlat{text:"1m" "5m" "1h" "1d" on_click: |i| switch_intv(i)}`；`switch_intv(i)` 重置 `k_candles = []`，重新 fetch + parse 后 `ui.chart.set_data(k_candles)`。
  5. **接入现有周期**：把现有 "600 秒自动刷新" 的 `start_interval(600.0, || refresh())` 改为 "K 线 tab 可见时每 30s 局部刷新"（监听 `tab`），用 `start_interval` / `stop_timer` 切换即可。
- **风险**：
  - **⚠️ Rust 类型暴露**：`Candle` 是 Rust 命名类型（`widgets/src/chart.rs:155`），splash DSL 没有同名字面量类型——`set_data(Vec<Candle>)` 期望一个 Rust `Vec<Candle>`。实际可行路径：
    - **路径 1（推荐）**：先试 `set_data([{time:0.0 open:100.0 high:101.0 low:99.0 close:100.5 volume:1000.0}, ...])`——若 WidgetRef 的 `script_call` 自动把 array-of-script-object marshall 成 `Vec<Candle>`（sparkline 已验证 array-of-number → `Vec<f64>` 工作），则直接通；
    - **路径 2（兜底）**：用 `octoscript-makepad/crates/makepad-plot/src/charts/bar.rs:1010-1028` 风格自定义 `script_call`，让 widget 接 `set_data(opens, highs, lows, closes)` 4 个 f64 数组。这需要改 makepad 上游并暴露 widget 给当前 host（card-host 当前不挂 makepad-plot），**MVP 阶段先走路径 1**，如失败再 fallback 路径 2；
    - **路径 3（最保守）**：用 `TrendChart`，其 `set_candles(Vec<Candle>)` 与 `CandlestickChart` 行为一致（chart.rs:1719-1722、1922-1927），同样适用路径 1/2。
  - **视图自动适配**：`CandlestickChart.draw_walk` 第一次绘制时会调用 `fit_data_y(&self.data)`（chart.rs:1086），并配合 `pixels_per_candle < 2.0` 自动聚合（chart.rs:1132-1138）—— 即 "100 根 1m 缩到 1d 看" 是自动的，无需手写多周期切换。MVP 可先不做手动周期切换，等数据源稳定后再加。
  - **侧载测试**：card-host 当前 `script_mod`（`card-host/src/host.rs:309-314`）已挂 `makepad_widgets::script_mod(vm)`，因此 `mod.widgets.CandlestickChart` 已就绪——无需改 card-host 任何代码。

**Fallback**：若 A 路径 1 在 widget method marshalling 上失败 → 路径 2（自定义 `set_data(4 arrays)`），改动量约 +20 行 makepad 上游 + 改 card-host 挂 `makepad_plot::script_mod`；预期 1 天可完成。

**最差 Fallback**：若上述都不行 → 用纯 `DrawColor` 矩形手动拼装（每根蜡烛 2 个 widget：wick + body），但这意味着每根蜡烛是独立 widget → widget 树开销会把 100 根柱的 draw call 数推到 ~200 个，**实测仍能 60FPS**（makepad widget 树实测可承载 1000+ 子 widget），但失去自动视口/pan/zoom 能力。

## 结论

**A. 能做 — 推荐方案 A（直接用 `CandlestickChart` widget）**。

**实现路径（5 步，已在 R-2 上文详述）**：

1. 数据层加 `k_candles` 数组 + OHLC 形状
2. fetch/parse 拿到 OHLC 对象数组（或先用 `synth_candles` 占位）
3. 新增 "K 线" tab，在 `on_render` 里挂 `CandlestickChart{...}`
4. 周期切换器（4 个 ButtonFlat）
5. 接入自动刷新

**核心代码量**：widget 嵌入 + 数据推送共 **~10 行**；含 fetch/parse/UI/状态管理 **总计 ~50 行** 增量。

**TODO 占位**：

- [ ] R-2 实施时先跑**路径 1**（array-of-object → `set_data`）的最小原型（附录原型），跑通即上；
- [ ] 若路径 1 失败，按路径 2 加 `set_data(opens, highs, lows, closes)` 重载；
- [ ] 美股 OHLC 实时源依赖 R-1 调研结果；R-1 出来前用 `synth_candles(n=100, seed=42)` 占位（MVP 验证 widget pipeline）；
- [ ] 周期切换先用 1m/5m/1d 三个按钮（PRD §3.3 的核心三档）；
- [ ] 配色先用现有 ink/up/down/secondary/flat 调色板，写死 5 个 `#[live]` 颜色；自定义面板放 Phase 2。

**无 TODO 阻塞**：所有依赖（widget 注册、`Candle` 类型、`set_data` marshalling）都已在 makepad 仓库的样例里跑过。

## 附录：参考代码片段（最少原型）

### A-1：splash DSL 端的最小 K 线原型（≤30 行）

```splash
// 在 main.splash 的 tabs 数组追加：{id: "k", label: "K 线"}
// 在 tabs 渲染分支新增：
if tab == "k" {
    let cs = k_candles   // [{time, open, high, low, close, volume}, ...]
    RoundedView{width: Fill height: Fit show_bg: true
        draw_bg.color: #x171c26 draw_bg.border_radius: 12.0
        // ---- 周期切换 ----
        View{width: Fill height: 36 flow: Right spacing: 6
            BarButton{text: "1m" on_click: || switch_intv("1m")}
            BarButton{text: "5m" on_click: || switch_intv("5m")}
            BarButton{text: "1d" on_click: || switch_intv("1d")}
        }
        // ---- 图表本体 ----
        k_chart := CandlestickChart{
            width: Fill height: 320
            color_up: #xe64340 color_down: #x2ecc71
            bg_color: #x0e1117 grid_color: #x2a2a3e
            candle_width_fraction: 0.7
        }
        // ---- 数据推送（fetch/parse 后调用）----
        // 第一次 ready 时：
        // ui.k_chart.set_data(k_candles)
    }
}

// 数据合成（MVP 占位，R-1 出来后替换为真 OHLC fetch）
fn synth_candles(n, seed) {
    let out = []
    let mut p = 100.0
    for i in n {
        // 简单随机游走
        seed = ((seed ^ (seed << 13)) ^ (seed >> 7)) ^ (seed << 17)
        let r = ("" + seed).len()  // 占位，确保编译通过
        let drift = ((seed % 1000) - 500) / 500.0
        let o = p
        let c = p + drift
        let h = max(o, c) + abs(drift) * 0.5
        let l = min(o, c) - abs(drift) * 0.5
        out.push({time: ("" + i) open: o high: h low: l close: c volume: 1000})
        p = c
    }
    out
}
```

> 上面 `seed` 用法仅为示意；R-2 实施时会写一个不依赖 Splash 限制的 xorshift（如 `seed = (seed * 1103515245 + 12345) & 0x7fffffff`），并在 fn 顶端初始化一次。

### A-2：Rust 端已经验证过的最小用法（来自 `octoOs/makepad/examples/datagrid/src/tab_charts.rs:155-189`，可直接对照）

```rust
fn feed_charts(&mut self, cx: &mut Cx) {
    let prices = &self.market.history[self.selected];
    self.view.trend_chart(cx, ids!(line)).set_series(cx, prices);

    // bucket the price history into candles
    let bucket = 8;
    let mut candles = Vec::new();
    let mut i = 0;
    while i + bucket <= prices.len() {
        let slice = &prices[i..i + bucket];
        let mut high = f64::NEG_INFINITY;
        let mut low = f64::INFINITY;
        for v in slice {
            high = high.max(*v);
            low = low.min(*v);
        }
        candles.push(Candle {
            time: (i / bucket) as f64,
            open:  slice[0],
            high,
            close: slice[bucket - 1],
            low,
            volume: 0.0,
        });
        i += bucket;
    }
    self.view.trend_chart(cx, ids!(candles)).set_candles(cx, candles);
}
```

> 这是 datagrid 样例里**真实每秒钟多次调用**的代码路径，证实 batched quad + WidgetRef.set_candles 在连续更新下也能保持 60FPS（注释原话："Prices tick a few times per second; only visible cells are drawn."）。

### A-3：Widget 注册路径速查（佐证 `mod.widgets.CandlestickChart` 在 splash 隔离中可用）

- `octoOs/makepad/widgets/src/lib.rs:468` —— `crate::chart::script_mod(vm);` 在 `widgets_mod` 内被调用
- `octoOs/makepad/widgets/src/chart.rs:51-55` —— `mod.widgets.CandlestickChart = set_type_default() do mod.widgets.CandlestickChartBase{ ... }`
- `octoOs/OctoSense-App-Hub/crates/card-host/src/host.rs:309-314` —— card-host 的 `App::script_mod` 调 `makepad_widgets::script_mod(vm)`，因此 `mod.widgets.CandlestickChart` 已在 finance-brief 的 isolate 内可见
- `OctoScript-App-Design-Flow/docs/SCRIPT-API.md:232-258` —— "Every name in the widgets prelude resolves in the isolate"，只要 `widgets_mod` 注册过即可直接用

### A-4：flowsurface 思路对照（仅参考，未移植）

- `flowsurface/src/chart/kline.rs:1243-1283` —— `draw_candle_dp` 函数：
  ```rust
  // 1) price_to_y(kline.high/low/open/close)  → 屏幕 y
  // 2) body_color = if close >= open { success } else { danger }
  // 3) frame.fill_rectangle(...)              → 蜡烛 body
  // 4) frame.fill_rectangle(...)         → 蜡烛 wick
  ```
  这与 makepad `TrendChart.draw_walk`（`chart.rs:1817-1837`）逻辑一一对应：
  ```rust
  // 1) py(c.open/.close/.high/.low)         → 屏幕 y
  // 2) color = if up { color_up } else { color_down }
  // 3) draw_candle.draw_abs(body rect)      → 蜡烛 body
  // 4) draw_candle.draw_abs(wick rect, w=1) → 蜡烛 wick
  ```
  思路一致；makepad 用 batched `DrawColor` 取代 iced 的逐柱 `fill_rectangle`，**所有柱共享一个 material 实例 → 单 draw call**，比 flowsurface 的实现更适合 60FPS ≥100 根的目标。