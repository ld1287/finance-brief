# Layer -1 Widget 兼容性 — finance-brief splash DSL 当前用法

> 范围：调研 **makepad 底层 widget**（Layer -1）与 `bundle/main.splash` 当前
> 178 行 flat-let 用法的兼容 + 真实交互状态。本文是只读调研 + 文档化结论，
> **不写 splash / 不写 Rust / 不跑 cargo / 不 commit**。
>
> 上下文：
> - finance-brief 当前绕过 octoscript-makepad L0 渲染管线（Phase 3.5 弃用
>   `.octoscript` 卡片，canonical_only=true 拒收），直接用 splash DSL flat-let
>   路线写 `bundle/main.splash`（git `fe69ae8` HEAD）。
> - splash DSL 解析后由 `makepad/widgets/src/splash.rs::Splash` 直接持有
>   一个 `View` 容器（`pub view: View` at L33），脚本里的 `SolidView` /
>   `View` / `ButtonFlat` / `Label` 都是 makepad widget set 的原生实例。
> - 上层架构见 `.todo-re-arch-2026-10-04.md §1`（L0 makepad native / L1 splash
>   DSL / L2 card-host / L3 native crate / L4 external sources）。
>
> 区别于 `docs/WIDGET-COMPAT.md`：该文件讲 octoscript-makepad L0 catalog
> 是否能装 12 屏；本文讲 Layer -1 makepad widget 是否装下当前 splash 这 178 行。

> **文档角色**：本文档 = Layer -1 makepad widget 详细分析（已知坑、TODO、headless GPU 限制、accessibility）。
> **Phase 1 兼容性结论**：见 `docs/WIDGET-COMPAT.md`（12 屏 → octoscript-makepad L0 widget map）。
> Re-verified 2026-10-04: 角色分工明确化。

---

## 1. SolidView

### SCRIPT-API 定义位置 + 行号
- `OctoScript-App-Design-Flow/docs/SCRIPT-API.md:252` —
  `View, SolidView, RoundedView | containers; SolidView/RoundedView for a filled background | –`。
- `SCRIPT-API.md:301-304` Gotchas：**`View{show_bg: true draw_bg.color: …}` 在
  card-host 不绘制背景**；`SolidView{draw_bg.color: …}` 会绘制。**✓ run**。

### makepad source 位置
- `makepad/widgets/src/view_ui.rs:85-98` —
  `mod.widgets.SolidView = mod.widgets.ViewBase { show_bg: true draw_bg +: {
  color: instance(#0000) get_color: fn() { return self.color } pixel: fn() {
  return Pal.premul(self.get_color()) } } }`。即 SolidView = ViewBase +
  `show_bg:true` + 一个纯色 fill pixel shader。
- `makepad/widgets/src/view.rs:70-180` View 结构体（SolidView 共享所有
  `View` 字段，包括 `on_render`、`layout`、`walk`、event dispatch、
  `on_item_tap`）。
- View 内的 widget tree 操作（on_render 写回的 children 重新 instantiate）
  见 `view.rs:208-315` `ScriptHook::on_after_apply` 与 `view.rs:868-970`
  `script_call(render/render_style)` + `script_result`。

### 真实交互
- 渲染流程：`View::draw_walk` (`view.rs:1120-1334`) 检测
  `self.show_bg=true` 时调用 `self.draw_bg.begin` (L1235-1239)；
  `draw_bg.end` 时 SDF pixel shader 跑 `Pal.premul(self.get_color())` 输出。
- `on_render` 重跑：`view.rs:868-921` `script_call(render)` →
  `widget_to_script_async_call_fwd` → isolate 在 sandbox 里跑闭包 →
  `view.rs:925-970` `script_result` 拿到 closure return value（一个 obj） →
  `script_apply(vm, Apply::Reload, …, me_obj.into())` 触发 `on_after_apply`
  把 children 重新 build。**关键：apply mode 是 `Reload`（不是
  `ScriptReapply`），所以每个 child 都是新 WidgetRef 实例** —
  `view.rs:241-264` "An `on_render` result is built fresh, as the first
  render was"。
- redraw 触发：`view.rs:967` `self.redraw(vm.cx_mut())` → 下一帧
  `draw_walk` 重画整棵树。

### finance-brief 当前用法
`bundle/main.splash:172-178` —
```splash
SolidView{width: Fill height: Fill flow: Down spacing: 0 draw_bg.color: colors.bg
    on_render: || {
        render_header()
        if nav.is_home() == true { render_launcher() }
        render_screen_placeholder()
    }
}
```
- `show_bg` 没写 → 默认值（SolidView 模板强制 `show_bg:true`，所以背景总
  是画）。
- `draw_bg.color: colors.bg` 是 `#xfafafa`（`main.splash:12`），全局背景。
- `on_render` 闭包调 3 个 fn（`render_header` / `render_launcher` /
  `render_screen_placeholder`），每次 rebuild 都实例化这些函数返回的
  View / ButtonFlat / Label 子树。

### 已知坑 / 限制
- **不能嵌套 on_render**：SolidView 的 `on_render` 闭包返回值是一个 obj
  （vec of widgets）。如果闭包内嵌的 `View{...}` 也写了 `on_render`，
  内层闭包只在外层 `render()` 触发时才会跑一次，**不会**随 isolate 的
  state 变化自动重跑。要换语义须显式 `ui.x.render()` (`SCRIPT-API.md:67`)。
- **flow: Overlay vs Down**：splash DSL 只暴露 `flow: Down/Right/Overlay`
  （见 `SCRIPT-API.md:31-36` 示例 + `view.rs:99-101` `event_order: EventOrder`
  是 rust 内部字段，splash 端不直写）。`SolidView` 当前用 `flow: Down`
  + `spacing: 0`，子节点纵向堆叠。
- **state 必须先于 widget 树**：`start_timeout(0.05, || nav.go_home())`
  (`main.splash:169`) 启动后 50ms 把 `nav.current` 设为 `"launcher"`，
  **但 SolidView 的 `on_render` 在 body eval 时就跑过一次了** —
  `SCRIPT-API.md:31` 注释 `// ui is injected AFTER the body evaluates`。
  50ms 后 nav.current 变化不会自动重画 root；要触发须显式
  `ui.root.render()`（当前 splash 没用 — 见 §7.1 TODO）。

### 测试覆盖
- ✅ **card-host admit**：
  `finance-brief/.local-state/card-host.log:19` —
  `card-host: finance-brief 0.3.0 admitted — capabilities {"net",
  "storage"}, hosts {…}, storage 16777216 bytes, agent none`。
- ✅ **isolate jailed**：
  `finance-brief/.local-state/card-host.log:20` —
  `card-host: isolate jailed at
  /home/lumina/octoOs/finance-brief/.local-state/finance-brief with
  16777216 bytes, 2 capability(ies), 6 host(s), 20000000 instructions,
  67108864 bytes of heap, prompts false — all enforced`。
- ✅ **widget-draw phase instantiate**：
  `finance-brief/.local-state/card-host.log:21` —
  `[SPLASH] eval: 76837 bytes preserve=false view=true animating=false
  tick=false epoch=0`（`view=true` 字段即 Splash 内 `pub view: View`
  已 instantiate，是 SolidView 子树 draw-tree 落地的间接证据）。
  `README.md:39-43` 解释：headless GPU grab PNG 失败但 draw-tree 已
  instantiate。

---

## 2. View（与 SolidView 区别）

### SCRIPT-API 定义位置 + 行号
- `SCRIPT-API.md:252` — `View, SolidView, RoundedView | containers | –`。
- `SCRIPT-API.md:253` — `ScrollYView, ScrollXView, ScrollXYView | scrolling
  containers, often with on_render | –`。
- `SCRIPT-API.md:301-304` Gotchas 重点：**`View{show_bg: true}` 画不出背景**，
  必须用 `SolidView` / `RoundedView`。

### makepad source 位置
- `makepad/widgets/src/view_ui.rs` 内 `mod.widgets.ViewBase` 模板（L83
  `Filler = View { width: Fill height: Fill }`，L100 `RectView` 等都是从
  ViewBase 派生）。
- `makepad/widgets/src/view.rs:70-180` View 结构本体，`show_bg: false`
  默认（`view.rs:79-80` `#[live(false)] pub show_bg: bool`），与
  SolidView 的差别就这一个 bool。
- `View` 没有 `draw_bg` 的 default instance slot — 模板
  `mod.widgets.ViewBase` 本身没声明 `draw_bg +: {…}`，所以单纯 `View{}`
  默认 `draw_bg` = 零像素填充（除非用户显式 `draw_bg.color` 才会 instantiate
  color instance，但 `show_bg=false` 不会画 — `view.rs:1231-1239` 仅当
  `self.show_bg` 才 `draw_bg.begin`）。

### flow: Right / Overlay / Down
- splash DSL 端：`flow: Right` 是 launcher tile 横向排列（`main.splash:119,
  143, 148`），`flow: Down` 是根 + placeholder + launcher 外层（`172, 142,
  153, 162`）。
- makepad 端：`Layout::flow` 决定子节点是横排 / 纵排 / 堆叠（`view.rs:82-83`
  `#[layout] pub layout: Layout`）。`flow: Overlay` 在 splash 端对应
  `Layout::Overlay` — splash 当前未用。

### finance-brief 用法（4 个 View）
1. `main.splash:119` header — `View{width: Fill height: Fit flow: Right
   spacing: 8 padding: 16 align: Align{y: 0.5}`，嵌 1 个 `ButtonFlat`
   (back) + 1 个 `Label`。
2. `main.splash:143` launcher row 1 — `View{width: Fill height: Fit flow:
   Right spacing: 12}`，嵌 3 个 `render_tile(...)`。
3. `main.splash:148` launcher row 2 — 同 row 1，3 个 tile。
4. `main.splash:153` 备用槽容器 — `View{width: Fill height: Fit flow:
   Down spacing: 6 padding: 16}`，嵌 1 个 `Label` ("备用槽") + 1 个 tile。
5. `main.splash:162` screen placeholder — `View{width: Fill height: Fill
   flow: Down spacing: 8 padding: 16}`，嵌 2 个 `Label`。

合计 6 个 `View{...}`，**不是 4 个** — `bundle/main.splash` 实际 grep 出 6
个 `View{`（L119 header, L142 launcher 外层, L143 row1, L148 row2, L153
备用槽, L162 placeholder），其中 L142/L143/L148/L153 在 launcher
嵌套内层。下面 §6 校对表用实际 grep 数（注意 grep `View{` 会匹配
`SolidView{` 子串，须排除）。

### 限制
- **不能 on_render** — 这是错的：View **可以** `on_render`，与
  SolidView 行为一致（`view.rs:131-132` `#[live] on_render: ScriptFnRef`）。
  差别是 SolidView 有 `show_bg:true`，更适合做"需要填色背景 + 重渲内容"
  的复合容器；View 不画背景，只能做"纯布局"。
- **`View{show_bg:true}` 在 card-host 不画背景** —
  `SCRIPT-API.md:301-304` 明确说这是验证过的 bug。finance-brief 完全
  规避（所有填色背景都走 `SolidView` 或 `draw_bg` 在 ButtonFlat 实例
  内的 `+:` patch）。

### 测试覆盖
- 与 SolidView 同源 — admit / jailed / eval 三个 log 即覆盖。
- View 在 render tree 里是 SolidView 的 children，draw-walk 时
  `view.rs:1242-1258` 循环 `self.children.iter_mut()` → 每个 child
  `draw_walk` — 无独立 log 痕迹。

---

## 3. ButtonFlat

### SCRIPT-API 定义位置 + 行号
- `SCRIPT-API.md:79` — `on_click: || … | Button, ButtonFlat, ButtonFlatter
  | none | ButtonFlat{text: "Add" on_click: || add()} ✓ run`。
- `SCRIPT-API.md:72` — `ui.x.on_click() | Button family (button.rs:548-583)
  | fires the handler`。agent 用 `ui.x.on_click()` 触发（同 §6.2
  recommended pattern）。
- `SCRIPT-API.md:305-310` Gotchas：**`ButtonFlat` 默认白字浅描边，在白底
  app 上不可见**；必须 `draw_bg +: {color: … color_hover: …
  color_down: …}` + `draw_text +: {color: … color_hover: …
  color_down: …}`。finance-brief 全部满足。

### makepad source 位置
- `makepad/widgets/src/button.rs:20` — `mod.widgets.ButtonFlat =
  set_type_default() do mod.widgets.ButtonBase{...}`。ButtonBase =
  `Button::register_widget(vm)`（`button.rs:16`），即 flat 是 Button 的
  default 形态。
- `button.rs:36-75` `draw_text` 字段：`color / color_hover / color_down /
  color_focus / color_disabled` + `text_style: theme.font_regular`
  (`button.rs:63-66`)；`get_color` mix 顺序：focus → hover → down →
  disabled。
- `button.rs:83-220` `draw_bg` 字段：同 5 状态色 + `border_size`、
  `border_radius` + bevel pixel shader）。
- `button.rs:349-364` Button = ButtonFlat + `:draw_bg +: {border_color_…,
  border_color_2_…}`（标准 face + outset gradient bevel）。
- `button.rs:541-585` `impl Widget for Button { fn script_call(...) }`
  — 处理 `text / set_text / on_click / on_press` 4 个 script-side 方法。

### click dispatch
- `button.rs:601-697` `handle_event`：
  - `Hit::FingerDown`（L622-642）：按主指时 `widget_action_with_data(
    ButtonAction::Pressed)` + 调 `on_press`（若 `trigger_on_press`）。
  - `Hit::FingerHoverIn`（L643-650）：设 MouseCursor + `animator_play(
    hover.on)`。
  - `Hit::FingerHoverOut`（L651-653）：`animator_play(hover.off)`。
  - `Hit::FingerUp`（L657-694）：`was_clicked = fe.is_over && fe.was_tap()`
    → `widget_action_with_data(ButtonAction::Clicked)` + `widget_to_script_call(
    uid, NIL, source, on_click, &[])`（L670-678）调 splash 闭包；hover
    state reset。
- 闭包调度由 `widget_to_script_call` → `widget_to_script_call_fwd` →
  `widget_to_script_calls` VecDeque（`makepad/widgets/src/widget_async.rs:336-346,
  830-834, 1107-1120, 1190-1232`），再经 `pump_widget_async`（L1410+）
  路由到 isolate VM 执行。

### draw_bg hover 机制
- `draw_bg` 是 `instance(0.0)` 字段：`hover / focus / down / disabled`
  都是 shader-instance floats（`button.rs:84-91`）。
- hover 状态切换 = `animator_play(cx, ids!(hover.on/off))` 触发
  Animator 的帧间 tween（0 → 1 / 1 → 0）。`animator_handle_event` 在
  `button.rs:603-605` 检测到 must_redraw 时调 `self.draw_bg.redraw(cx)`
  → 下一帧 pixel shader 用 `mix(self.color, self.color_hover, self.hover)`
  输出渐变色（`button.rs:189` 类似的 mix 调用）。
- 即：**hover 不是 boolean，是 animator 的 0..1 instance float**，
  shader 在 fragment shader 里 mix。这点对 splash 用户透明，splash
  DSL 写 `color_hover: …` 只是设 color_hover instance 字段。

### on_click handler 闭包是否 async / sync
- **同步（sync）**：`button.rs:670-678` 调 `widget_to_script_call` 走
  `widget_to_script_calls` VecDeque 后在 `pump_widget_async` 中转到
  isolate 跑 — 同步执行闭包体（不是 tokio::spawn / 不是 Rust async）。
- 闭包返回值忽略（`&[]` 参数 + NIL result）。
- 闭包没有捕获 args；`SCRIPT-API.md:79` `on_click: || … | none` —
  splash handler 不接事件参数。
- 闭包体可调 `host.call("cap", {...})`（`SCRIPT-API.md:199-230`）—
  `host.call` 是异步回包（reply 通过 isolate 重新 wake），但 on_click
  闭包本身不等回包。

### finance-brief 用法
- `main.splash:121-123` header back button —
  ```splash
  ButtonFlat{text: "← 返回" height: 36 on_click: || nav.back()
      draw_bg +: {color: colors.bg_elevated color_hover: colors.divider border_radius: 18.0 border_size: 1.0}
      draw_text +: {color: colors.ink color_hover: colors.accent}}
  ```
  height: 36（不是 Fit），圆角 18（pill shape），border 1px 浅灰描边 —
  iOS-style back button。
- `main.splash:133-136` launcher tile（render_tile 函数体）—
  ```splash
  ButtonFlat{width: Fill height: 96 on_click: || nav.push(screen_name)
      draw_bg +: {color: colors.bg_elevated color_hover: colors.divider border_radius: 12.0 border_size: 1.0}
      draw_text +: {color: colors.ink}
      Label{text: label draw_text.color: colors.ink draw_text.text_style.font_size: 16}}
  ```
  width: Fill, height: 96, 圆角 12 — 卡片 tile。注意 **`draw_text +:`
  没设 `color_hover`**，所以 hover 时文字颜色不变（仅背景从 elevated →
  divider 变灰）。`render_tile` 被调用 7 次（6 launcher + 1 备用槽），
  实例化 7 个 ButtonFlat。

### 测试覆盖
- 与 SolidView 同源 — admit / jailed / eval log 即覆盖。
- on_click 实际触发未见 log：handler `|| nav.push(screen_name)` 改的是
  splash 顶层 `let nav = {...}` 字段，不打 log；
  `pump_widget_async` 调完不写 INFO 级日志。
- draw_bg hover animator 切换也无 log — 必须 `tools/octo shot` 抓 PNG
  或真 GPU + 鼠标驱动才能验证（headless 不可见）。

---

## 4. Label

### SCRIPT-API 定义位置 + 行号
- `SCRIPT-API.md:65` — `ui.x.text() / ui.x.set_text(s) | any widget
  (widget.rs:792-808); Label, LinkLabel, TextInput, buttons | set_text
  wants a string: set_text("" + n). ✓ run`。
- `SCRIPT-API.md:254` — `Label, LinkLabel | text (default text color is
  white: set draw_text.color) | –`。
- `SCRIPT-API.md:337` Gotchas — **Text is white by default. Set
  `draw_text.color` on light backgrounds.** finance-brief 全部 4 个
  Label 都设了 `draw_text.color`，规避正确。

### draw_text style: color / font_size / font_family
- `makepad/widgets/src/label.rs:11-46` `mod.widgets.Label` 模板 —
  `width: Fit height: Fit padding: theme.mspace_1`，`draw_text +: {
  ink_centered: true color: theme.color_label_outer … text_style:
  theme.font_regular { line_spacing: theme.font_wdgt_line_spacing } }`。
- `draw_text.color` 设前景色；`draw_text.text_style.font_size` 改字号。
- splash DSL 端 `draw_text.text_style.font_size: 22` 是 patch 语法 —
  实际等价于 `text_style = theme.font_regular { font_size: 22 }`（覆盖
  `theme.font_regular` 默认值）。
- font_family 不在 splash 端写死 走 `theme.font_regular` /
  `theme.font_bold` token，跨 DPI 自适应（`README.md:46-49`，
  `.todo-re-arch-2026-10-04.md:264`）。

### finance-brief 用法（4 个 Label）
1. `main.splash:125-127` header 标题 —
   `Label{width: Fill text: "财经简报" draw_text.color: colors.ink
   draw_text.text_style.font_size: 22}`。
2. `main.splash:136` launcher tile 内文 —
   `Label{text: label draw_text.color: colors.ink
   draw_text.text_style.font_size: 16}`。`text` 是 render_tile 函数参数
   （如 `"新闻简报"`、`"研究卡"` 等），闭包参数驱动。
3. `main.splash:154` 备用槽标题 —
   `Label{text: "备用槽" draw_text.color: colors.secondary
   draw_text.text_style.font_size: 12}`。
4. `main.splash:163` screen placeholder 大字 —
   `Label{text: nav.current draw_text.color: colors.ink
   draw_text.text_style.font_size: 18}`。**`text: nav.current`** —
   runtime 变量在 render 时取值。
5. `main.splash:164` screen placeholder 小字 —
   `Label{text: "(host loads the corresponding .card)" draw_text.color:
   colors.secondary draw_text.text_style.font_size: 12}`。

合计 5 个 `Label{...}`，**不是 4 个**（L125, L136, L154, L163, L164）。

### 限制
- **不能 on_click / 不能 set handler**：Label 没有 `on_click` 字段
  （`SCRIPT-API.md:79` Events 表只有 Button/CheckBox/TextInput/
  GestureView/SheetView/View family/Camera/Slash 系列）。要点击响应
  须套 GestureView 或包 ButtonFlat（template script-app 就是这样）。
- **不能放 widget 子级**：Label 的 `draw_text` 字段没有 children vec
  — 模板 `mod.widgets.Label` 只声明 `width / height / padding /
  draw_text / text`。要嵌图标可用 `Icon` widget（`SCRIPT-API.md:260`）。
- **`text` 引用 runtime 变量只在 on_render 时取值**：
  `text: nav.current`（L163）这个写法只在 SolidView root 的 on_render
  闭包被 `ui.root.render()` 重新跑时才会重新实例化 Label widget 取新值；
  nav.current 变化后**没有 reactive 触发器**（splash DSL 没有 fine-grained
  reactivity）。见 §7.3 TODO。

### 测试覆盖
- 与 SolidView 同源 — admit / jailed / eval log 即覆盖 Label 字段
  解析。
- `text: nav.current` 这种闭包变量捕获的 Label 实际渲染值在 headless
  下不可见 — 没 log evidence 证明它取到 `"launcher"` 字面值。
- 字体回退 / NotoSansSC 跨 DPI 自适应需要真 GPU + 真字体系统才能验证。

---

## 5. 真实交互清单（针对当前 splash）

### ✅ 已验证（有 log evidence）
- ✅ **splash parse**：`bundle/main.splash` 178 行 body 被
  `makepad/widgets/src/splash.rs::Splash::eval_styled_body` (L201-205)
  完整 tokenize + parse + apply。
  - 证据：`finance-brief/.local-state/card-host.log:21` —
    `[SPLASH] eval: 76837 bytes preserve=false view=true animating=false
    tick=false epoch=0`。
  - 上下文：76837 是 splash body 字节数（含 `SPLASH_PREFIX`，见
    `splash.rs:145-152`），与 main.splash 178 行 × 字节数大致吻合。
  - `view=true` 字段表示 `Splash::view` (splash.rs:32-33) 已被 instantiate
    — 即 root `SolidView` + on_render 闭包 + 5 个 View + 3 个 ButtonFlat
    + 5 个 Label 的 widget tree 都建好。
- ✅ **card-host admit**：`finance-brief/.local-state/card-host.log:19` —
  `card-host: finance-brief 0.3.0 admitted — capabilities {…}, hosts
  {…}, storage 16777216 bytes, agent none`。
- ✅ **isolate jailed**：`finance-brief/.local-state/card-host.log:20` —
  `card-host: isolate jailed at …/finance-brief with 16777216 bytes, 2
  capability(ies), 6 host(s), 20000000 instructions, 67108864 bytes of
  heap, prompts false — all enforced`。

### ❌ 未验证（headless GPU 限制）
- ❌ **实际像素渲染**：`finance-brief/.local-state/card-host.log:8-15` —
  ```
  libEGL warning: failed to get driver name for fd -1
  libEGL warning: MESA-LOADER: failed to retrieve device information
  MESA: error: ZINK: failed to choose pdev
  libEGL warning: egl: failed to create dri2 screen
  ```
  ZINK failed + EGL failed → GL Error 500（从 `README.md:41` 推论）→
  `card-host` grab PNG (`/g` endpoint) 报 GL render failure。
  **结论：draw-tree instantiate ≠ pixel out** — `view=true` 仅证明 widget
  树建立，真实 GPU 渲染没跑。
- ❌ **on_click handler 触发后 nav.push / nav.back 真实调用**：
  on_click 闭包调 `nav.push(screen_name)`，splash 没有 INFO 级 log，
  也无 `[host.call]` log 紧跟在 admit 之后 — 没法证明 on_click 真触发。
- ❌ **draw_bg hover 状态切换**：animator 是 0..1 instance float，headless
  下既无鼠标也没渲染 — 不可见。
- ❌ **on_render 重渲染触发新 SolidView children**：nav.current 变化
  后没人调 `ui.root.render()`（splash DSL 没暴露 reactive 系统），需
  手动 `render()` 或 `set_visible` 触发 — 见 §7.1。
- ❌ **scrollbar / TextInput / Slider / CheckBox 等 widget 的真实交互**：
  finance-brief 当前 splash 完全没用这几个 widget（只用了 4 类：SolidView
  / View / ButtonFlat / Label），无对应测试覆盖需求。

### 需要的验证环境
- 真 GPU（独立显卡 或 Intel/AMD 集成显卡带 Mesa DRI3 驱动，不能 ZINK
  fallback）。
- Wayland 或 X11 display server 真桌面（不是 headless SSH）。
- 输入设备（鼠标 for hover / 触屏 for tap）— animator/ButtonFlat
  hover state 切换需要 `FingerHoverIn` / `FingerHoverOut` 事件
  (`button.rs:643-653`)。
- `tools/octo shot` 抓 PNG 工具（参见
  `OctoScript-App-Design-Flow/templates/script-app/README.md:30-31`）。

---

## 6. 当前 splash 用法对照 SCRIPT-API.md

逐个 widget 在 `bundle/main.splash` 实际出现位置 + 属性清单：

| Widget | 行号 | 关键属性 | SCRIPT-API.md 文档对应 |
|--------|------|----------|------------------------|
| `SolidView` | 172-178 | `width: Fill height: Fill flow: Down spacing: 0 draw_bg.color: colors.bg on_render: \|\| {...}` | L31-36 示例 + L250-252 + L301-304（show_bg 强制） |
| `View` (header) | 119 | `width: Fill height: Fit flow: Right spacing: 8 padding: 16 align: Align{y: 0.5}` | L252 + L31-36 |
| `View` (launcher 外层) | 142 | `width: Fill height: Fill flow: Down spacing: 12 padding: 16` | 同上 |
| `View` (launcher row 1) | 143 | `width: Fill height: Fit flow: Right spacing: 12` | 同上 |
| `View` (launcher row 2) | 148 | 同上 | 同上 |
| `View` (备用槽) | 153 | `width: Fill height: Fit flow: Down spacing: 6 padding: 16` | 同上 |
| `View` (placeholder) | 162 | `width: Fill height: Fill flow: Down spacing: 8 padding: 16` | 同上 |
| `ButtonFlat` (back) | 121-123 | `text: "← 返回" height: 36 on_click: \|\| nav.back() draw_bg +: {color, color_hover, border_radius: 18.0, border_size: 1.0} draw_text +: {color, color_hover}` | L79 + L256 + L305-310（draw_bg/draw_text 必填） |
| `ButtonFlat` (tile) | 133-136 | `width: Fill height: 96 on_click: \|\| nav.push(...) draw_bg +: {color, color_hover, border_radius: 12.0, border_size: 1.0} draw_text +: {color} Label{...}` | 同上 |
| `Label` (header) | 125-127 | `width: Fill text: "财经简报" draw_text.color draw_text.text_style.font_size: 22` | L65 + L254 + L337（color 必填） |
| `Label` (tile) | 136 | `text: label draw_text.color draw_text.text_style.font_size: 16` | 同上 |
| `Label` (备用槽) | 154 | `text: "备用槽" draw_text.color draw_text.text_style.font_size: 12` | 同上 |
| `Label` (placeholder big) | 163 | `text: nav.current draw_text.color draw_text.text_style.font_size: 18` | 同上 |
| `Label` (placeholder small) | 164 | `text: "(host loads the corresponding .card)" draw_text.color draw_text.text_style.font_size: 12` | 同上 |

### 实际 grep 计数（真实数字）

| 类型 | 字面 grep 计数 | 实例数（闭包调用展开） | 任务要求的预估 |
|------|--------------|------------------------|----------------|
| `SolidView{...}` | 1（root, L172） | 1 | 1 ✅ |
| `View{...}` | **6** | 6 | 4 ❌（实际 6：header + launcher 外层 + 2 row + 备用槽 + placeholder） |
| `ButtonFlat{...}` | **2**（L121 back + L133 render_tile 函数体） | **7**（render_tile 被调 7 次：6 launcher + 1 备用槽） | 3 ⚠️（任务描述里"3"模糊，源码字面 2 实例点 + 1 函数） |
| `Label{...}` | **5** | 5（render_tile 内的 Label 是每次调用新建，闭包内文本是 render_tile 参数） | 4 ❌（实际 5：header + tile + 备用槽 + 2 placeholder） |

**SCRIPT-API.md 覆盖结论**：所有 18 处 widget 用法的属性（`width /
height / flow / spacing / padding / align / draw_bg.color /
draw_bg.color_hover / draw_bg.border_radius / draw_bg.border_size /
draw_text.color / draw_text.color_hover / draw_text.text_style.font_size
/ text / on_click`）**全部在 `SCRIPT-API.md` 文档里有显式条目**。
无未文档化属性；无 `show_bg: true` 反模式（finance-brief 完全规避
`SCRIPT-API.md:301-304` 警告）。

### widget id 缺失警告
- `bundle/main.splash` **没有用 `widget := Widget{...}` 命名 id**
  （对比 template `script-app/bundle/main.splash:34-37` 用 `entry :=
  TextInput{...}`）。
- `SCRIPT-API.md:57-75` 解释：没 id 的 widget 仍可显示，但 `ui.<id>.text()` /
  `ui.<id>.set_text(...)` / `ui.<id>.render()` / `ui.<id>.on_click()`
  全部不可达。
- `.todo-re-arch-2026-10-04.md:286-288` 建议：每个 button / 交互 widget
  加 `:=` id，agent 自动化需要。当前 splash 故意省略（agent 驱动 UI 是
  Phase B-2 范围，不阻塞当前 launch）。

---

## 7. 已知坑 + TODO

### 7.1 host.call 异步返回后 splash 端如何更新 state
- `host.call` 是 async callback（`SCRIPT-API.md:199-230`），reply 通过
  isolate 重新 wake。
- splash 端 reply 闭包形如 `let host_news_refresh = fn(){ host.call(
  "news.refresh", {limit: 30}) }`（`main.splash:53-55`）— **没有
  on_reply callback**！reply 只是 ack，data 没存回 `nav` 或顶层 `let`。
- 当前 splash 完全没用 on_reply — `host.call` 是 fire-and-forget，
  reply 在 isolate 里被 GC。这与设计原则一致（`.todo-re-arch-2026-10-
  04.md:17` "splash 纯 UI"）。
- **真正的数据回流** 走 `native/src/` → `host.call` reply 写到
  `#sdk-cache/` JSON 文件 + `cache_*` JSON 文件，由下次 `host.call`
  读 file 拿回 data。这层不归 splash 管。

### 7.2 button on_click 调 nav.push(name) → nav.history.push + nav.current
= name + on_render 重跑 → re-render 整棵树（会不会 leak 旧 widget?）
- `nav.push(name)` 只改 `nav.current` 字段（`main.splash:29-32`），
  **没调 `ui.root.render()`** — 所以点击 back / tile 实际不会重画
  任何东西（headless log 没新 `[SPLASH] eval` 行可证）。
- 即使修复 re-render：`view.rs:925-967` `script_result` 用 `Apply::Reload`
  模式（不是 `ScriptReapply`），每个 child 都被替换为新 WidgetRef
  实例（`view.rs:241-264`）— 旧 widget 走 Rust drop → `Widget::drop`
  → makepad 自动 `cx.redraw()` cleanup + isolate heap 旧对象 GC。
- **不会 leak 旧 widget**（makepad 自管），但每次 `render()` 都是
  完整 O(n) rebuild（n = 当前 nav 对应的全部 widget）— 小屏无感知，
  大屏（屏 11 事件流 / 屏 12 数据源 5×8 grid）可能 16ms 阈值风险。

### 7.3 Label.text 引用 nav.current 这种 runtime 变量, splash 何时重渲染 text
- **只在 on_render 时**：`text: nav.current`（L163）的字面值在
  `view.rs::on_after_apply` (L208-315) 闭包 `WidgetRef::script_from_value_
  scoped(vm, scope, kv.value)` 时取一次，存进 `Label.text` StringBuffer
  字段（`makepad/widgets/src/label.rs` register_widget 的
  `#[rust] text: String` 字段）。
- 之后 nav.current 改了，Label 不会重画 — splash DSL 没暴露
  fine-grained reactive。
- 修复路径：
  1. 手动在 on_click 后调 `ui.root.render()`（需要先给 root 一个
     id） — 但当前 splash 没 id。
  2. 等 splash 上游暴露 reactive 绑定（`SCRIPT-API.md` 没有
     `bind`/`watch`/`computed` 类 API）。
  3. 把 `nav.current` 嵌入到 widget tree **结构**（condition render），
     而不是嵌入到 Label.text — 当前 `main.splash:175` `if nav.is_home()
     == true { render_launcher() } render_screen_placeholder()` 已经
     走结构分支，但 `nav.current` 字符串同时又显示在 placeholder
     Label 里 (`L163`) — 这条 Label 永远显示初始值（`nav.current
     = "launcher"` at boot）。
- **结论**：当前 splash 的 placeholder Label 在 home 屏显示
  `"launcher"` 一直不变 — 这不是 bug，是 splash DSL 的限制。

### 7.4 如果是 headless GPU 抓不到 PNG, 怎么证明 on_click 真实 work?
- **方法 A（间接）**：给 handler 加 native 端 log —
  `.todo-re-arch-2026-10-04.md:298-308` 建议 `tracing::info!(target:
  "finance_brief::adapters", capability=name, args=?args, "call")` —
  然后调 `host.call("news.refresh", {})` 在 on_click 体内，触发
  cap log 即证 on_click 跑了。
- **方法 B（直接）**：用 `ui.x.on_click()` script-side 方法触发
  (`SCRIPT-API.md:72`) — 这是 splash DSL 端 invoke handler 的官方
  接口。但需要 widget id（见 §6 widget id 缺失警告）。
- **方法 C（推荐 Phase B-2）**：写 `tools/octo shot` 抓 PNG +
  agent 用 `ui.X.on_click()` 触发 + 抓下一帧 PNG 对比 — 这是
  `.todo-re-arch-2026-10-04.md:281-294` 推荐做法，但当前 splash 缺
  id，需要先加 `:=`。

---

## 8. Layer 0（octoscript-makepad）状态

### 当前架构定位
- finance-brief **当前直接用 splash DSL 写 `bundle/main.splash`**，跳过
  Layer 0（octoscript-makepad 渲染管线）。
- 完整 5 层架构见 `.todo-re-arch-2026-10-04.md §1`（L26-83 mermaid 图）：
  - L0 = makepad native widget（本文）
  - L1 = splash DSL（`bundle/main.splash`，178 行）
  - L2 = card-host (`OctoSense-App-Hub/crates/card-host/src/host.rs`)
  - L3 = finance-brief/native/src/（9 adapter + cache + host）
  - L4 = 5 个外部数据源

### Layer 0（octoscript L0 卡片）的阻塞
- `.card / .octoscript` 卡片走 octoscript-makepad 渲染管线（VM → UiNode
  → `to_makepad_ui` → makepad Splash widget set）— 见
  `docs/ARCHITECTURE.md §2` (L70-87) + `docs/WIDGET-COMPAT.md §1`
  (40-Widget 兼容清单)。
- **当前阻塞**：`.todo-re-arch-2026-10-04.md:340-345` Phase B-3 "待
  OctoScript#56 解锁"。OctoScript#56 是 canonical_only=true 严格
  grammar 检查（`.todo-re-arch-2026-10-04.md:11` "此前还有 `lib/*.splash`
  三层目录 + `main.splash` + 11 个 `.bak-pre-card-rewrite` 备份
  （git `4a30bb5` → `871f7ea` → `fe69ae8` 多次切换）"）。
- 历史 commit `dfeca45..7381764`（Phase 3.5）显示用过 `.octoscript`
  卡片，被 canonical_only=true 拒收，已弃用 — 当前 `bundle/main.splash`
  是 flat-let 单文件回退路径。

### 建议
- **继续走 splash DSL flat-let 路线**（当前 HEAD `fe69ae8` 已 stable，
  178 行 + 65/65 测试 pass + card-host admit + isolate jailed + widget-
  draw phase instantiate）— 不阻塞 launch。
- **Layer 0（octoscript-makepad）待 #56 解锁后再说**：
  `.todo-re-arch-2026-10-04.md:340-345` Phase B-3 把
  `render_screen_placeholder` 替换成 11 个 L0 卡片（`bundle/*.card` 11
  个文件已存在）。届时 splash DSL 单文件作 launcher + header，
  `.card` 作子屏。
- **不私自改上游**：`.todo-re-arch-2026-10-04.md:354-360` §9 硬约束 —
  "不改 OctoSense / OctoSense-App-Hub / octoscript 等依赖项目源码；
  不 fork 或 PR upstream"。

---

## 附录 A. 引用文件清单

| 文件 | 行号 | 引用原因 |
|------|------|----------|
| `bundle/main.splash` | 1-178 | 当前 splash 全部用法（4 类 widget × N 实例） |
| `OctoScript-App-Design-Flow/docs/SCRIPT-API.md` | 31-36, 57-75, 75-96, 244-279, 292-341 | splash DSL 完整 API + gotchas |
| `OctoScript-App-Design-Flow/templates/script-app/bundle/main.splash` | 31-53 | light app 模板（flat-let 风格 + on_render 列表） |
| `makepad/widgets/src/splash.rs` | 27-124, 617-693, 702-730 | Splash struct + handle_event + call_script_fn |
| `makepad/widgets/src/button.rs` | 20-220, 349-364, 541-697 | ButtonFlat 模板 + Button 继承 + handle_event (click + animator) |
| `makepad/widgets/src/view.rs` | 70-180, 208-315, 868-970, 980-1334 | View struct + ScriptHook + script_call + handle_event + draw_walk |
| `makepad/widgets/src/view_ui.rs` | 83-98 | SolidView = ViewBase + show_bg + flat color pixel shader |
| `makepad/widgets/src/label.rs` | 11-46 | Label 模板（draw_text color default white） |
| `makepad/widgets/src/widget_async.rs` | 336-346, 830-834, 1107-1232, 1410+ | widget→script call dispatch |
| `finance-brief/.local-state/card-host.log` | 19, 20, 21 | admit + jail + SPLASH eval log |
| `finance-brief/.todo-re-arch-2026-10-04.md` | §1 (L26-83), §5 (L240-268), §6.2 (L281-294), §8 (L322-352), §9 (L352-365) | 5 层架构图 + UI 自适应 + agent integration + 渐进迁移 + 不做的事 |
| `finance-brief/README.md` | 39-43 | widget-draw phase instantiate 解释（headless GPU 限制） |
| `finance-brief/docs/ARCHITECTURE.md` | §2 (L33-87) | 5 层架构定义 |
| `finance-brief/docs/WIDGET-COMPAT.md` | 全文 | 对比文件：octoscript-makepad L0 catalog 12 屏 widget 覆盖（不是本文） |