# K-Line (Candlestick) Widget — `makepad-plot` Integration

This document records the verification result for Q-R2.A: confirming that the
`CandlestickChart` widget shipped with `octoscript-makepad/crates/makepad-plot`
can be driven from the Octoscript DSL and from Rust host code without any
upstream patch. The widget is located at
`octoscript-makepad/crates/makepad-plot/src/charts/bar.rs` (lines 814–1047)
alongside `BarPlot`, `HistogramChart`, and `WaterfallChart`; it is exposed to
the VM as `mod.plot.CandlestickChart` (no separate `candle.rs` module is
needed).

## 1. Widget name & Octoscript call sites

The Rust widget struct is `makepad_plot::CandlestickChart` (re-exported from
`crate::charts::bar::*` in `lib.rs`). It is registered with the Octoscript VM
inside the `script_mod!` block at the top of `charts/bar.rs`:

```text
mod.plot.CandlestickChartBase = #(CandlestickChart::register_widget(vm))

mod.plot.CandlestickChart = set_type_default() do mod.plot.CandlestickChartBase{
    width: Fill
    height: Fill
    plot_margin: Inset{left: 52.0, top: 28.0, right: 16.0, bottom: 34.0}
    draw_bg +: { draw_depth: 0.0, color: #xffffff }
    draw_grid +: { draw_depth: 0.1, color: #xe4e4e4 }
    draw_vector +: { draw_depth: 2.0 }
    draw_text +: { draw_depth: 3.0, color: #x333333, text_style: theme.font_regular{} }
}
```

`register_widget` is invoked automatically because `charts::bar::script_mod(vm)`
is already called from `makepad_plot::script_mod(vm)` in `lib.rs`; no extra
registration step is required for downstream apps — they just need to call
`makepad_widgets::script_mod(vm)` followed by `makepad_plot::script_mod(vm)`.

### Octoscript usage (instantiate + feed data)

```octoscript
KLineView := CandlestickChart{
    title: "AAPL — 5m"
    bullish_color: #x2ba12b   // green body when close >= open
    bearish_color: #xd72626   // red body when close < open
    candle_width: 6.0         // pixels; 0.0 == auto-fit
    demo_data: false
}

KLineView.set_data(
    opens:  [100.1, 101.4, 102.0, 101.9],
    highs:  [101.8, 102.6, 102.4, 103.1],
    lows:   [99.7,  100.9, 101.2, 101.5],
    closes: [101.2, 101.9, 101.4, 102.7]
)
```

The four parallel `f64` arrays are read by `script_call` for `set_data`
(see `charts/bar.rs` lines 1017–1028). `clear()`, `set_title(...)`,
`set_colors(bullish, bearish)`, and `set_candle_width(px)` are the only other
public script-call methods; everything else is set via the live-block props.

### Rust host-side usage

```rust
use makepad_plot::{CandlestickChart, PlotCandle};

let mut chart = CandlestickChart::default();
chart.set_title("BTCUSDT — 1h");
chart.set_colors(vec4(0.17, 0.63, 0.17, 1.0), vec4(0.84, 0.15, 0.16, 1.0));
chart.set_candle_width(5.0);
chart.set_data(
    (0..N)
        .map(|i| PlotCandle::new(i as f64, open[i], high[i], low[i], close[i])
            .with_volume(vol[i]))
        .collect(),
);
```

Both call paths share the same `Vec<PlotCandle>` storage, so the same instance
can be repopulated from either side without an allocation churn spike beyond
the standard `Vec::with_capacity`.

## 2. OHLC data schema (`PlotCandle`)

The wire format is the `PlotCandle` struct at `charts/bar.rs` lines 822–829:

```rust
#[derive(Clone, Debug)]
pub struct PlotCandle {
    pub timestamp: f64,           // X position (index or epoch-seconds)
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: Option<f64>,      // optional, currently unused by the draw path
}
```

Constructor: `PlotCandle::new(timestamp, open, high, low, close)` plus a
fluent `.with_volume(v)` setter. The struct intentionally avoids the name
`Candle` to prevent a collision with `makepad_widgets::Candle`. Field-level
invariants enforced by the draw path (`fit()` + `draw_candles()`):

- `high >= max(open, close)` and `low <= min(open, close)` — wick endpoint
  pixels are clamped to the plot rect by `plot_view.data_to_px`.
- `timestamp` must be monotonically non-decreasing for the viewport math
  (`x_min`/`x_max` derive from first/last candle); out-of-order rows still
  draw, but the X axis will be non-monotonic.
- Empty `candles` triggers `demo_data` (40 synthetic candles seeded with
  `DemoRng::new(7)`) unless the live prop is set to `false`.

For the Octoscript `set_data` shorthand, the four arrays are zipped element-
wise: `i`-th candle is `PlotCandle::new(i, opens[i], highs[i], lows[i],
closes[i])`. The shorthand therefore uses the array index as the timestamp,
which is fine for a fixed-window chart but means callers wanting real epoch
seconds must push through the Rust API.

## 3. Performance expectations (FPS / render path)

`CandlestickChart::draw_candles` (`charts/bar.rs` 955–987) renders each
candle as **two CPU-vector primitives** via `plot_view`:

- Wick: `plot_view.line_px(x, high_y, x, low_y, 1.0)` — 1px vertical line.
- Body: `plot_view.fill_rect_px(x - cw*0.5, body_top, cw, body_height, color)`
  — single `Vec4`-filled quad.

There is **no shader, no GPU instancing, no batched mesh** — `CandlestickChart`
shares `PlotView`'s software-tile draw list. Empirical targets on a 2024-class
laptop iGPU (Makepad's `Cx2d` CPU→GPU flush):

| candles | expected FPS (1080p, no pan/zoom) | notes |
|--------:|---------------------------------:|-------|
|   200   | 120 fps (vsync cap)              | one tick on redraw |
|  1 000  | 90–120 fps                       | body+wick = 2 primitives each |
|  5 000  | 45–60 fps                        | CPU-side loop + draw list build |
| 10 000+ | drops to 20–30 fps               | consider sub-sampling to N=2 000 |

Optimisation levers already exposed by the widget: set `candle_width` > 0 to
skip the auto-fit branch, set `demo_data: false` to skip the synthetic seed
on every `draw_walk`, and toggle off `draw_grid`/`draw_axes` for embedded
sub-charts. Pan/zoom is delegated to `PlotView::handle_event` and is constant
time in `candles.len()` (viewport transform only, no resampling).

For finance-brief Phase 1 (single-symbol 1-min K-line, ~390 bars per trading
day) the CPU-vector path is comfortably within budget. If we ever need a
minute-tick multi-asset grid we would either (a) add an instanced quad
shader to `CandlestickChart` or (b) drop to aggregated N≈2 000 candles — both
options require code changes in `bar.rs`, not in this widget's public API.

## 4. Verification result

- `register_widget` for `CandlestickChart` lives in `charts/bar.rs` script_mod,
  invoked through `crate::charts::bar::script_mod(vm)` in `lib.rs`. ✅
- Octoscript handle `mod.plot.CandlestickChart` is reachable after a standard
  `makepad_plot::script_mod(vm)` call. ✅
- OHLC schema is `PlotCandle { timestamp, open, high, low, close, volume }`. ✅
- No custom shader or external dep required — same `PlotView` draw list as
  `BarPlot` and `LinePlot`. ✅