# Splash Style Guide — finance-brief

> Convention for writing `bundle/main.splash` (the single-file flat-let DSL
> that card-host loads). All rules here are derived from working code in
> `bundle/main.splash` and the runtime spec at
> `OctoScript-App-Design-Flow/docs/SCRIPT-API.md`.
>
> Last verified: 2026-10-04, against `bundle/main.splash` at `de89355`
> (178 lines, B-0 flat-let rewrite landed in `871f7ea`, `ui.root.render()`
> dead code reverted in `de89355`).

## 1. State declaration

### 1.1 Use top-level `let`, not nested modules

```splash
let colors = {bg: #xfafafa ink: #x1c1c1e}
let nav    = {current: "launcher" history: []}
let host_news_refresh = fn() { host.call("news.refresh", {limit: 30}) }
```

**Why**: splash L0 does not support `mod.X.Y` nested assignments. Every
read of `mod.lib.X.Y` walks the prototype chain at parse time and dies
with `property lib not found` before the bundle renders anything.
History: `4a30bb5 → 871f7ea → fe69ae8` documented this regression.

**Don't**:
```splash
mod.lib.colors = {bg: #xfafafa}   // property lib not found
use mod.lib.colors.*              // use is a reserved word
```

### 1.2 Object literals cannot host `fn` bodies

splash object literals (`{k: v k: v}`) accept values only. Methods must
be assigned via property assignment on a separate line:

```splash
let nav = {current: "launcher" history: []}

nav.push = fn(name){
    nav.history.push(nav.current)
    nav.current = name
}

nav.back = fn(){
    if nav.history.len() == 0 { return }
    nav.current = nav.history.pop()
}
```

**Don't**:
```splash
let nav = {
    current: "launcher"
    push: fn(name) { ... }   // fn inside object literal — not supported
}
```

### 1.3 Reserved words cannot be variable names

From `OctoScript-App-Design-Flow/docs/SCRIPT-API.md:333-335`:

| Reserved | | | |
|---|---|---|---|
| `me` | `scope` | `self` | `nil` |
| `true` | `false` | `ok` | `let` |
| `var` | `mut` | `fn` | `if` |
| `elif` | `else` | `for` | `in` |
| `while` | `loop` | `match` | `return` |
| `break` | `continue` | `and` | `or` |
| `is` | `do` | `try` | `use` |

**Don't**: `let try = 1`, `let ok = true`. Pick distinct names.

### 1.4 Implicit 1 Hz interval on `fn tick(`

Naming a top-level function `tick(...)` triggers an implicit 1 Hz
interval calling it (`splash.rs:381-397`). Rename to avoid surprise:
`fn tick_news()` not `fn tick()`.

## 2. Widget ID naming

### 2.1 Use `name := Widget{…}` for any widget you need to reach

Per `SCRIPT-API.md:49-50`:

> `name := Widget{…}` makes a widget addressable as `ui.name`. A `:=` id
> is a field of its direct parent only; name every wrapper you need to
> reach through.

**Pattern**:
```splash
<!-- top-level wrapper: id it so any handler can reach it -->
list := ScrollYView{width: Fill height: Fill flow: Down
    on_render: || { ... }
}

<!-- nested widget inside a parent: id the parent first -->
root := SolidView{width: Fill height: Fill flow: Down
    header := View{ ... }          <!-- ui.root.header works -->
    body := View{ on_render: || ui.body.render() }
}
```

### 2.2 ID prefix conventions

| Prefix | Use | Example |
|---|---|---|
| (no prefix) | top-level app state (single instance) | `colors`, `nav`, `settings` |
| noun-only | widget with one canonical instance at root | `header`, `body`, `tile` |
| noun with screen suffix | widgets scoped per screen | `news_list_view`, `kline_chart` |
| `handler_` | event handlers (optional; only if not a fn) | `handler_news_refresh` |

### 2.3 Don't ID the root widget unless you have to

The root widget in `finance-brief/bundle/main.splash` is anonymous
(`SolidView{...}` at L172). ID-ing it requires every nav mutation to
end with `ui.<root>.render()` — and that path is currently
**runtime-dead** (`MethodNotFound`; see `docs/WIDGET-COMPATIBILITY.md
§7.2`). Leave it anonymous; rely on the `on_render` closure to
re-evaluate on `let` mutation.

## 3. Section comment convention

`finance-brief/bundle/main.splash` partitions the file with:

```splash
// ── section name ─────────────────────────────────────
```

Rules:
- 3 em-dash (`─`) blocks before and after the section name
- Total width: 60 columns (matches file width)
- Section names match the `render_*` function names when possible
- Top-to-bottom: state declarations → handlers → render fns → boot → root

Current sections (at `de89374`, 178 lines):
- `// ── colors ──` (L10)
- `// ── nav ──` (L23)
- `// ── host ──` (L48)
- `// ── launcher + header ──` (L113)
- `// ── header ──` (L117)
- `// ── launcher tile ──` (L131)
- `// ── launcher ──` (L140)
- `// ── screen placeholder ──` (L160)
- `// ── boot ──` (L168)
- `// ── root ──` (L171)

## 4. Handler style

### 4.1 `on_click: || expr`

```splash
ButtonFlat{text: "← 返回" on_click: || nav.back()}
ButtonFlat{text: "新闻简报" on_click: || nav.push("news_list")}
```

For complex handlers, prefer extracting a top-level `fn`:

```splash
fn back(){
    if nav.history.len() == 0 { return }
    nav.current = nav.history.pop()
}
ButtonFlat{text: "← 返回" on_click: || back()}
```

### 4.2 `on_render: || { children }`

The closure's widgets become the children (`SCRIPT-API.md:89`).
Re-runs when the closure's referenced `let` variables mutate.

```splash
list := ScrollYView{width: Fill height: Fill flow: Down
    on_render: || {
        if notes.len() == 0 { Label{text: "No notes yet."} }
        for i in notes.len() { Label{text: notes[i]} }
    }
}
```

**Gotcha**: `if … {…} else for …` keeps stale rows when the if branch
draws nothing (SCRIPT-API.md:298-300, ✓ run). Use two parallel
branches: `if cond { Empty } for i in items.len() { Row }`.

### 4.3 `on_change: |text| …` for `TextInput`

The `text` argument is the new value. Don't read `ui.<id>.text()` from
inside the same handler unless you're sure the runtime is on the
post-`d0a9def5` makepad (see `SCRIPT-API.md:311-318`).

### 4.4 No `on_reply` from `host.call`

`host.call(...)` is fire-and-forget. Reply data writes to native cache
files; the next `host.call` reads it. There is **no** callback for
inline `r.data` consumption. Pattern:

```splash
fn refresh_news(){
    host.call("news.refresh", {limit: 30})
    // data is in cache; on_render closure will pick it up
    // on the next let mutation (e.g. a set_timeout tick)
}

start_interval(2.0, || {
    refresh_news()
    ui.body.render()   // see §5 about ui.<id>.render()
})
```

## 5. UI mutation rules

### 5.1 `set_text` and `set_visible` are always available

Per `widget.rs:765-808` (default impl on every widget) and
`SCRIPT-API.md:65-66`:

| Call | Use |
|---|---|
| `ui.x.set_text(s)` | change Label text; `s` must be string, `"" + n` for numbers |
| `ui.x.text()` | read current text |
| `ui.x.set_visible(true/false)` | show / hide |
| `ui.x.visible()` | read visibility |

### 5.2 `render()` is on View-family only

Per `view.rs:877-892` (View family: View, SolidView, RoundedView,
ScrollYView, ScrollXView, ScrollXYView) and `SCRIPT-API.md:67`. Re-runs
the view's `on_render` closure.

**Not** available on: Label, Button, Image, GestureView, TextInput,
CheckBox, etc. — fall through to widget.rs default which has no
`render` method (will hit `widget method render not found for uid …`).

### 5.3 `on_click` fires the handler

`ui.x.on_click()` on a Button-family widget fires its on_click
handler (`button.rs:548-583`). Use this for programmatic clicks
(e.g. agent-driven automation; see `docs/AGENT-INTEGRATION.md`).

## 6. Color and drawing rules

### 6.1 Hex colors must use `#x` prefix

From `SCRIPT-API.md:294-296`:

> Hex colors: write `#x` before any hex color with an `e` next to a
> digit (`#x1e1e2e`, `#x2ecc71`); `#x` is always safe. Otherwise the
> tokenizer reads an exponent.

**Don't**:
```splash
View{draw_bg.color: #fafafa}       // ambiguous if next char is digit
View{draw_bg.color: #xRRGGBB}      // WRONG — literal X
```

**Do**:
```splash
View{draw_bg.color: #xfafafa}      // safe
View{draw_bg.color: #x1c1c1e}      // safe (1e would be exponent)
```

### 6.2 Use `SolidView` or `RoundedView` for filled backgrounds

Per `SCRIPT-API.md:301-304`, `View{show_bg: true draw_bg.color: …}`
does not draw its background in card-host. Use:

```splash
SolidView{width: Fill height: Fill draw_bg.color: #xfafafa
    on_render: || { ... }
}
```

### 6.3 Default text color is white

Set `draw_text.color` on light backgrounds (`SCRIPT-API.md:337`):

```splash
Label{text: "财经简报"
    draw_text.color: colors.ink
    draw_text.text_style.font_size: 22}
```

### 6.4 Default `ButtonFlat` is invisible on white

`ButtonFlat{text: …}` alone draws white text on a light outline
(`SCRIPT-API.md:305-310`). Style it explicitly:

```splash
ButtonFlat{text: "10%" on_click: || set_tip(10)
    draw_bg +: {color: #xffffff color_hover: #xe5e5ea border_radius: 18.0 border_size: 1.0}
    draw_text +: {color: #x1c1c1e color_hover: #x007aff}
}
```

To reuse, bind once and instantiate:
```splash
let Chip = ButtonFlat{height: 40
    draw_bg +: {color: #xffffff color_hover: #xe5e5ea}
    draw_text +: {color: #x1c1c1e color_hover: #x007aff}
}
Chip{text: "10%" on_click: || set_tip(10)}
```

### 6.5 `ButtonFlat` holds no children

Per `SCRIPT-API.md:319-321`, a `Label` inside `ButtonFlat` is not
drawn. Use `text:` for the button's label. For a rich tappable row,
wrap with `GestureView{on_tap: |x, y| …}` around the content.

## 7. Control flow gotchas

### 7.1 No `else for`

Splash has no `else for`; branch with two independent `if` statements
or use the pattern from §4.2:

```splash
if cond { Empty }     // separate branch
for i in items.len() { Row }
```

### 7.2 No dynamic-key-then-read

```splash
let o = {}
o["key"] = "value"     // OK to write
let v = o["key"]        // RISKY: dynamic key read walks prototype chain
```

Per `SCRIPT-API.md:240`, `o.k` and `o[k]` both exist; the latter is
slower and is the only path that allows non-identifier keys. Prefer
static `o.k` access.

### 7.3 No `else if` chained ternary

Use `elif` (splash allows) or nested `if` blocks.

### 7.4 For loops are 0-indexed and inclusive-exclusive

```splash
for i in 3              // i = 0, 1, 2
for v in [10, 20, 30]   // v = 10, 20, 30
for k v in {a:1 b:2}    // k = "a", "b"; v = 1, 2
```

There is **no `range()`**. Use `a..b` ranges (`SCRIPT-API.md:242`).

## 8. Boot sequence

```splash
// ── boot ──
start_timeout(0.05, || nav.go_home())

// ── root ──
SolidView{width: Fill height: Fill flow: Down draw_bg.color: colors.bg
    on_render: || {
        render_header()
        if nav.is_home() == true { render_launcher() }
        render_screen_placeholder()
    }
}
```

`ui` is injected after the body evaluates
(`splash.rs:367-371`; `SCRIPT-API.md:45-48`). **Every System App** uses
`start_timeout(0.05, || boot())` to start work. `start_timeout(0, ...)`
fires before `ui` is ready.

## 9. File layout

finance-brief uses a **single-file** flat splash (`bundle/main.splash`,
178 lines at `de89374`). The splash L0 dialect does not support a
`lib/` subdirectory (card-host only loads `bundle/main.splash`,
`OctoSense-App-Hub/crates/card-host/src/host.rs:117-120`).

Rules:
- All state in one file
- All handlers in one file
- All `render_*` functions in one file
- Cross-file split: only the L0 card route (Phase B-3, blocked on
  OctoScript#56) when one `main.splash` exceeds ~300 lines

## 10. Review checklist

Before committing any change to `bundle/main.splash`:

- [ ] `grep -n "mod\." bundle/main.splash` returns 0 lines
- [ ] `grep -n "^use " bundle/main.splash` returns 0 lines (`use` is reserved)
- [ ] All hex colors use `#x` prefix (no `#` or `0x`)
- [ ] No variable names in §1.3 reserved-word list
- [ ] No `else for` chained statements
- [ ] All widget `id := Widget{…}` ids match SCRIPT-API.md:65-66 method
      surface (`set_text`, `set_visible`, `render` for View family)
- [ ] No `View{show_bg: true …}` (use `SolidView`/`RoundedView` instead)
- [ ] No `Label` inside `ButtonFlat` (use `text:` or wrap with `GestureView`)
- [ ] All `on_click` / `on_render` / `on_change` handlers are top-level
      `fn` bodies, not anonymous if/else branches
- [ ] Single-file ≤ 300 lines; if bigger, open `.card` route (B-3)

## 11. References

- `OctoScript-App-Design-Flow/docs/SCRIPT-API.md` — splash DSL spec
- `OctoScript-App-Design-Flow/docs/CAPABILITIES.md` — capability contract
- `OctoScript-App-Design-Flow/docs/HOST-SERVICES.md` — `host.request` shape
- `finance-brief/docs/WIDGET-COMPATIBILITY.md` — Layer -1 widget audit
- `finance-brief/.todo-re-arch-2026-10-04.md §3` — splash ↔ Rust
  decoupling design
- `finance-brief/bundle/main.splash` — canonical example (178 lines)