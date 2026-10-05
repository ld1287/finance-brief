# RFC: Stable uid for Splash/View host slot when swap-and-reroute

- **Status**: Draft
- **Date**: 2026-10-05
- **Reporter**: finance-brief (OctoSense app) — sub-agent Q4 research
- **Affected crates**: `octoscript-widgets` (splash.rs / view.rs) + upstream `makepad/widgets` (widget_tree.rs / widget.rs)
- **Severity**: Blocker for any application that swaps the root view per route. SPA / multi-route / launcher-style hosts are all affected. Single-page static hosts are unaffected.

> Scope note: this RFC is an **issue-ready draft** for the upstream dep maintainers. It does **not** propose a PR; finance-brief is contributing root-cause analysis and reproduction.

---

## §2.2 Problem summary (≤ 200 字)

**Repro**: a host app spawns a `Splash` (or plain `View`) as the root widget, mounts route A, and later mounts route B by `mem::replace(&mut *host, view)`. The widget-tree graph keys on `WidgetRef::widget_uid()`. For `Splash` that uid is **struct-level** (set once at struct creation by `WIDGET_UID_COUNTER`, widget.rs:17-26), but `refresh_from_borrowed` seeds the uid with `placeholder: true` and an empty `WidgetWeakRef` (widget_tree.rs:1083-1102), so `flush_dirty` short-circuits via the `if parent_placeholder { return true }` guard (widget_tree.rs:1622-1626) and the graph never re-walks the host's children. For `View` the uid is **per-instance** (also `WIDGET_UID_COUNTER`-allocated, but no slot identity), so after `mem::replace` the graph's old uid points to a dropped `View` and `remove_subtree` (widget_tree.rs:1627-1631) deletes the entire route A subtree — but the new route B's `View` is never inserted, because nothing calls `set_root_widget` / `seed_from_widget` for it.

**Expected**: after `mem::replace` + a fresh `refresh_from_borrowed`, the graph's host slot should reflect the new view's children, and `compact_dump` (widget_tree.rs:2780) should show route B.

**Actual**: `/d` and `/g` keep showing route A's widgets (or an empty graph), and the screen never updates. finance-brief reproducer below (§2.7) demonstrates this with launcher → news-list swap.

---

## §2.3 已尝试的 workarounds (finance-brief 侧)

| Version | Strategy | cargo test | Runtime route swap |
|---------|----------|------------|-------------------|
| v10 | `Cx2d::widget_tree_insert_child_deep(host_uid, ...)` after `mem::replace` to register the new view's subtree manually. Sets `manual = true` on the new nodes (widget_tree.rs:717, 747). | PASS (splash_swap_tests) | FAIL — `manual=true` permanently retains the old route's widgets; graph.children[host_uid] grows on every swap; `remove_subtree` skips `manual` nodes (refresh_node_children_from_discovered L1816-1822); eventually OOM and stale draws. |
| v12 | Call `refresh_from_borrowed(host_uid, visit)` (widget_tree.rs:1065-1119) after `mem::replace`. Inserts `placeholder = true` for unknown uids (L1083-1102) and walks the discovered children. | PASS (refresh_from_borrowed unit tests) | FAIL — `mark_structure_dirty` is hardcoded `false` (L1110), and the host node is seeded as `placeholder = true` (L1090) so `flush_dirty` short-circuits (L1622-1626). The host's `children()` is never re-walked after the first mount; only direct children are discovered. |
| v15 | Replace `host := Splash{...}` with `host := View{...}` and use `mem::replace(&mut *host, view)`. The host `View.uid` is per-instance (`#[uid]` macro, view.rs:72), so each swap allocates a fresh uid from `WIDGET_UID_COUNTER` (widget.rs:17). | PASS (splash_swap_tests, app_rs_uses_view_instead_of_splash_for_host) | FAIL — `refresh_from_borrowed` is still called with the **old** host uid (the one whose `View` struct is now dropped). `node.widget.upgrade()` returns `None` (widget_tree.rs:1619), the code falls through to `remove_subtree(old_uid)` (L1629), and the new View's uid is never inserted because nothing calls `set_root_widget` (L957) / `seed_from_widget` (L894) on it. |

All three pass unit tests because no test simulates the full host-slot-swap path: the failing assertions live at runtime, in `compact_dump` (widget_tree.rs:2780) and `snapshot`/`flat_tree` outputs.

---

## §2.4 根因分析 (with line-number references)

### §2.4.1 Placeholder 节点为何被 flush_dirty 跳过

`refresh_from_borrowed` seeds a borrowed uid by inserting a `GraphNode` with `widget: WidgetWeakRef::default()` and `placeholder: true` (widget_tree.rs:1083-1102):

```rust
// widget_tree.rs:1083-1102 (refresh_from_borrowed)
let mut inner = self.inner.borrow_mut();
if !inner.graph.contains_key(&uid) {
    inner.graph.insert(uid, GraphNode {
        name: LiveId(0),
        widget: WidgetWeakRef::default(),
        placeholder: true,        // ← always placeholder on first borrow
        skip_search: false,
        parent: None,
        children: Vec::new(),
        nesting_depth: 0,
        manual: false,
    });
    if inner.root_uid == WidgetUid(0) {
        inner.root_uid = uid;
    }
    inner.structure_dirty = true;
}
```

This is intentional: a borrowed `WidgetRef` cannot be borrowed again to read its `children()` (it would deadlock — see the `try_borrow` chain in `WidgetRef::try_widget_uid` at widget.rs:828-833), so the graph node is left as a placeholder until a real `WidgetRef` is seeded via `seed_from_widget` (widget_tree.rs:894-955) or `set_root_widget` (L957-1063).

The placeholder guard then lives in `refresh_node_children` (widget_tree.rs:1612-1654):

```rust
// widget_tree.rs:1612-1654
fn refresh_node_children(
    inner: &mut WidgetTreeInner,
    uid: WidgetUid,
    pending: &mut Vec<WidgetUid>,
    mark_structure_dirty: bool,
) -> bool {
    let (parent_widget, parent_placeholder) = match inner.graph.get(&uid) {
        Some(node) => (node.widget.upgrade(), node.placeholder),
        None => return true,
    };
    if parent_placeholder {
        // Placeholder node (seeded from borrowed context without a WidgetRef):
        // keep existing child edges until a concrete WidgetRef is seeded.
        return true;                       // ← widget_tree.rs:1622-1626
    }
    let Some(parent_widget) = parent_widget else {
        // Concrete widget no longer exists; remove stale subtree.
        Self::remove_subtree(inner, uid);   // ← widget_tree.rs:1629
        return true;
    };
    // ... continues with try_children(...)
}
```

The early-return at L1622-1626 means **any node still flagged `placeholder = true` is never re-walked by `flush_dirty` (L1152-1163)**. Once a uid has been seeded as placeholder, the only way to flip `placeholder = false` is:
- `seed_from_widget` (L908-919): clears `placeholder` if `node.placeholder || node.widget != widget`,
- `set_root_widget` (L985-989): same condition,
- `insert_child` (L722-726): same condition when the child is later discovered,
- `refresh_node_children_from_discovered` (L1706-1710): same condition for discovered children.

**None of these paths apply to a host slot that was swapped via `mem::replace` without a follow-up `seed_from_widget` / `set_root_widget` call from the host code** — which is exactly the finance-brief mount() pattern. The graph node for the old host uid stays `placeholder = true` forever.

A second wrinkle: even when the **child** nodes are correctly discovered, the `parent_is_placeholder` flag in `refresh_node_children_from_discovered` (widget_tree.rs:1676, used at L1722-1724) actively prevents adoption of children when the parent is a placeholder, so the host's children stay unlinked from the host uid in the graph:

```rust
// widget_tree.rs:1675-1678
let (old_children, parent_is_placeholder) = match inner.graph.get_mut(&uid) {
    Some(node) => (std::mem::take(&mut node.children), node.placeholder),
    None => return true,
};
// widget_tree.rs:1722-1724 (kept_real_parent)
let keep_real_parent = parent_is_placeholder
    && child_node.parent.is_some_and(|p| p != uid);
if child_node.parent != Some(uid) && !keep_real_parent {
    child_node.parent = Some(uid);
    child_parent_changed = true;
}
```

This is the "FoldHeader anchors its children but must not adopt them" pattern (comment at L1715-1721). For a host Splash, the result is that **discovered children keep their real parent** (somewhere else in the tree) and the host's `graph.children` vec is never populated, so `rebuild_dense` (L1893-1946, walks `node.children` at L2007-2010) cannot reach them from the host root.

### §2.4.2 `WidgetRef::widget_uid()` 在不同 widget 上的语义

The `#[uid]` field attribute in both `Splash` and `View` calls `WidgetUid::new()` (widget.rs:22-26), which `fetch_add`s the global `WIDGET_UID_COUNTER` (widget.rs:17):

```rust
// widget.rs:17-26
static WIDGET_UID_COUNTER: AtomicU64 = AtomicU64::new(1);

#[derive(Clone, Debug, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct WidgetUid(pub u64);

impl WidgetUid {
    pub fn new() -> Self {
        Self(WIDGET_UID_COUNTER.fetch_add(1, Ordering::Relaxed))
    }
}
```

Both `Splash.uid` (splash.rs:29) and `View.uid` (view.rs:72) are therefore **per-instance, atomic-counter-allocated**. There is no inherent difference at the language level.

The **operational difference** is lifetime:

- **`Splash.uid` (splash.rs:26-29)**: assigned when the `Splash` struct is constructed (script eval) and lives until `set_text("")` calls `stop(cx)` (splash.rs:678-692, splash.rs:420-434). The Splash struct is designed to be a long-lived container that wraps an isolate VM (`vm_id: SplashVmId`, splash.rs:44). Its uid is therefore effectively **stable for the isolate's lifetime**.
- **`View.uid` (view.rs:70-72)**: assigned when `View { ... }` is constructed in `live_apply` / `on_after_apply`. Plain `View` instances are short-lived layout containers; re-creating a View gives a new uid.

`WidgetRef::widget_uid()` (widget.rs:824-833) does not distinguish — it just delegates to the inner widget's `widget_uid()`:

```rust
// widget.rs:820-833
pub fn widget_uid(&self) -> WidgetUid {
    self.try_widget_uid().unwrap_or(WidgetUid(0))
}

pub fn try_widget_uid(&self) -> Option<WidgetUid> {
    self.0
        .try_borrow()
        .ok()
        .and_then(|r| r.as_ref().map(|w| w.widget.widget_uid()))
}
```

For a `Splash` (splash.rs:585-588), `widget_uid()` returns `self.uid` (the stable struct-level uid). The graph keying is therefore stable, but the graph **node** for that uid is `placeholder = true` (per §2.4.1).

For a `View` (view.rs:748-751), `widget_uid()` returns `self.uid` (per-instance). The graph keying is therefore unstable across swap — every `mem::replace` produces a fresh uid, and the old uid's graph entry now points to a dropped `View`.

### §2.4.3 为什么 `mem::replace(&mut *host, view)` 不更新 graph

The host slot is an `Rc<RefCell<...>>` carrying a `WidgetRef` (or similar). `mem::replace` swaps the underlying widget struct in place; the host slot's **memory address** is unchanged, but the **WidgetUid** of the widget occupying that slot changes:

| Step | host slot content | host slot's `widget_uid()` | graph entry for that uid |
|------|-------------------|---------------------------|---------------------------|
| Pre-mount | empty `View` (initial live-apply) | `View.uid = V0` | not yet in graph |
| Mount A | `View { route A children }` | `V0` (same slot) | `graph[V0]` — placeholder, set by `refresh_from_borrowed` (L1083-1102); later promoted by `seed_from_widget` / `set_root_widget` (L908-919, L985-989) when concrete `WidgetRef` is reachable. |
| `mem::replace(&mut *host, view)` | `View { route B children }` | `view.uid = V1` (NEW, from `WIDGET_UID_COUNTER.fetch_add`) | `graph[V0]` is now dangling — `node.widget.upgrade()` returns `None` (widget_tree.rs:1619); `graph[V1]` does not exist. |
| `refresh_from_borrowed(V0, ...)` | unchanged | `V0` | `graph.contains_key(V0) == true` (L1084 branch skipped), so no new insert. `refresh_node_children_from_discovered` walks route B's children but parents them under `V0` (which is dangling). |

The graph ends up with a dangling `V0` node whose `WidgetWeakRef` no longer upgrades, and zero `V1` nodes. On the next `flush_dirty` (L1152-1163) — triggered by `compact_dump`'s `sync_dense` (L2781), `snapshot`, `flat_tree`, `query_rects`, or the remote bridge's `/snap`/`/d` — `refresh_node_children(V0)` runs:

```rust
// widget_tree.rs:1618-1631
let (parent_widget, parent_placeholder) = match inner.graph.get(&uid) {
    Some(node) => (node.widget.upgrade(), node.placeholder),
    None => return true,
};
// ... not a placeholder (V0 was promoted in mount A) ...
let Some(parent_widget) = parent_widget else {
    // Concrete widget no longer exists; remove stale subtree.
    Self::remove_subtree(inner, uid);   // ← L1629
    return true;
};
```

`remove_subtree` (L1873-1891) wipes `V0` and its (stale) `children` from the graph. The new `V1` is invisible because nobody inserted it.

**This is the core asymmetry**: `WidgetRef::widget_uid()` is stable per-instance for both `Splash` and `View`, but
- for `Splash` the long-lived uid maps to a **stuck `placeholder`** node that `flush_dirty` skips;
- for `View` the short-lived uid maps to a **dangling** node that `flush_dirty` removes on sight.

Either way, after `mem::replace` the new widget is unreachable from the graph, so `compact_dump` / `snapshot` / `flat_tree` / `/d` / `/g` all show stale or empty results, and the screen doesn't redraw from the new route.

---

## §2.5 建议修复

### 方案 A：stable slot uid for host View

Add a **slot identity** that survives `mem::replace`. Two sub-options:

**A1 (preferred, no API change)**: introduce a separate `#[slot_uid] uid: WidgetUid` attribute on widgets intended to be host slots, and have `WidgetRef::widget_uid()` prefer `slot_uid` when present. Currently `#[uid]` always calls `WidgetUid::new()` (widget.rs:22-26); `#[slot_uid]` could be the same implementation but would document intent and allow future caching. The default `View` could ship with `#[slot_uid]`, since most Views live in long-lived host slots.

**A2**: add a `host_slot: Cell<Option<WidgetRef>>` thread-local or `Cx`-local map keyed by memory address of the host slot, populated by `View::on_after_apply` (view.rs:195-316). On `refresh_from_borrowed` (widget_tree.rs:1065-1119), the placeholder insertion uses the slot uid, not the per-instance uid.

**Effect**: `mem::replace(&mut *host, view)` no longer changes the uid seen by the widget tree. The graph key for the host slot is stable, and `refresh_from_borrowed` (L1065-1119) correctly re-walks children after every swap.

**Trade-offs**:
- Requires a new attribute or a small new API; needs careful default handling for `View`s that are NOT host slots (a `View` deep inside a route should still get a per-instance uid so it can be swapped independently).
- Risk of two slots accidentally sharing the same memory address (e.g. a recycled `Box<View>` on a heap allocator) — mitigate by also keying the slot identity on the slot's parent path or a generational counter.

### 方案 B：`refresh_from_borrowed` 按 `WidgetRef` 解析

Change the API from `refresh_from_borrowed(uid: WidgetUid, ...)` to `refresh_from_borrowed(widget_ref: WidgetRef, ...)`. Internally, call `widget_ref.try_widget_uid()` to read the **current** uid, and if `graph.contains_key` is true for that uid, promote the placeholder by reassigning `node.widget = widget_ref.downgrade()` (mirroring `seed_from_widget`'s logic at widget_tree.rs:908-919).

```rust
pub fn refresh_from_borrowed<F>(&self, widget_ref: WidgetRef, mut visit: F)
where
    F: FnMut(&mut dyn FnMut(LiveId, WidgetRef)),
{
    let Some(uid) = widget_ref.try_widget_uid() else { return };
    if uid == WidgetUid(0) { return; }

    let mut inner = self.inner.borrow_mut();
    if let Some(node) = inner.graph.get_mut(&uid) {
        if node.placeholder || node.widget != widget_ref {
            node.widget = widget_ref.downgrade();
            node.placeholder = false;
            // continue with the discovered children walk...
        }
    } else {
        // ... insert new placeholder (unchanged from L1085-1102) ...
    }
    // ... refresh_node_children_from_discovered ...
}
```

**Effect**: every call to `refresh_from_borrowed` with a live `WidgetRef` for the **current** uid promotes the placeholder atomically and re-walks children. The host code's mount() needs only to call `refresh_from_borrowed(host_ref, ...)` instead of passing a stale uid.

**Trade-offs**:
- Breaking API change. Mitigate by adding the new signature alongside the old one and deprecating the uid-based one.
- Still requires the host code to call `refresh_from_borrowed` on every swap — silent breakage if forgotten. (A scheme like a `tracing` hook on `mem::replace` could detect this, but that's out of scope.)

### 方案 C：让 Splash 的 GraphNode 跟随 host slot

`Splash` already has a stable uid. The problem is only that the graph node stays `placeholder = true`. Two sub-options:

**C1**: in `Splash::on_after_apply` (splash.rs:126-131), call `cx.widget_tree().seed_from_widget(self.as_widget_ref())`. The host code that constructs the Splash gets one graph promotion on the first eval, and subsequent `set_text` re-evals don't need to do anything.

**C2**: have `Splash::set_text` (splash.rs:678-692), on every body change, call `cx.widget_tree().seed_from_widget(self.as_widget_ref())` after `eval_body(cx)`. This promotes the placeholder on every re-eval, so `flush_dirty` will re-walk the host's children every time the body changes.

**Effect (C1)**: only fixes first-mount, not route swaps (since route swaps `mem::replace` a Splash in finance-brief's pre-v15 code, which has the same dangling-struct problem as Views).

**Effect (C2)**: every `set_text` triggers a host-graph refresh. But the host slot's `WidgetRef` is the Splash itself — its uid is stable — so the graph key is correct. Subsequent `refresh_node_children` (widget_tree.rs:1612-1654) walks the new children.

**Trade-offs**:
- C1 is partial (only first mount).
- C2 requires Splash to know about `Cx::widget_tree`, which is currently a `CxWidgetExt` trait method (widget_tree.rs:3035-3045, 3084-3113, 3115-3148). Splash is in `widgets/src/splash.rs`, widget_tree is in `widgets/src/widget_tree.rs`, so cross-file plumbing is fine, but it adds a runtime cost on every `set_text` (one graph lookup + one downgrade call).
- Does NOT fix the `View` case (which is exactly what v15 needs). So this is at best a Splash-only workaround.

**Recommendation**: **方案 A1 + 方案 B** in tandem — A1 makes the uid stable across swap, B makes `refresh_from_borrowed` resilient against forgotten swaps. C2 can ship as a Splash-specific hardening but does not address the general bug.

---

## §2.6 影响范围

| Application shape | Affected? | Why |
|-------------------|-----------|-----|
| Single-page static app (one root `View`/`Splash`, no route swap) | NO | The root uid is never changed. `seed_from_widget` / `set_root_widget` runs once, and the graph stays stable. `compact_dump` shows the right widgets. |
| Multi-route app with `host := Splash` and `Splash.set_text(body_for_route_X)` | YES (finance-brief v10-v14) | `Splash.children()` delegates to `self.view.children()` (splash.rs:602-604), but the Splash's graph node is `placeholder = true` forever after first `refresh_from_borrowed` (widget_tree.rs:1622-1626 early-returns). Children re-discovery at L1656-1871 finds the new view's children but `parent_is_placeholder` (L1676, 1722-1724) prevents adoption. |
| Multi-route app with `host := View` and `mem::replace(&mut *host, view)` | YES (finance-brief v15) | Each swap produces a new `View.uid` (view.rs:72 + widget.rs:17-26). `refresh_from_borrowed` is called with the old uid (now dangling). `remove_subtree` (widget_tree.rs:1629, 1873-1891) wipes the stale subtree; the new uid is never inserted. |
| SPA with router (e.g. OctoScript-App-Design-Flow/examples/calendar/native) | NO (verified) | Calendar uses a custom `CalendarView` widget that owns its children via direct mutation; it does not rely on `refresh_from_borrowed` to populate the widget tree. The graph follows the live widget ref because `seed_from_widget` runs whenever the widget is reachable. |
| App with a `PortalList` recycler that re-creates rows | PARTIAL | `mark_dirty` (widget_tree.rs:886-892) re-queues the affected node. As long as the recycler reuses the same `View` instance (same memory address, same uid), the graph follows. But if the recycler `mem::replace`s row widgets, the same bug bites per-row. |
| App with dock-style containers (`Dock`) that own children outside the parent's child vec | NO | `manual = true` (widget_tree.rs:717, 747) preserves such children even across refreshes; L1816-1822 explicitly keeps them. This is the pattern that originally motivated `manual`, and it works correctly. |
| App relying on the remote bridge `/d`, `/g`, `/snap` to inspect the widget tree | YES (all of the above) | These endpoints call `compact_dump` (widget_tree.rs:2780+) / `snapshot` / `flat_tree`, all of which call `sync_dense` → `sync_dirty` → `flush_dirty`. They show the same stale graph. |

**Bottom line**: any app whose host slot's widget identity changes after the first graph promotion is affected. finance-brief is the canary; flutter-samples, calendar's own router variants, and any future SPA will hit the same bug.

---

## §2.7 复现脚本路径 (finance-brief 仓库内)

The bug is reproducible against the current HEAD `5bef74ea00129267754c4e405db8e0106b1e52e9` on Windows with the makepad rev pinned by `Octoscript-Makepad/runtime.json` (`975c5630`).

### Reproduction recipe

1. **App shell** — `finance-brief/apps/desktop/src/app.rs`, function `mount(cx, &mut self, route)` (current v15 code replaces `Splash` with `View` at the `host` slot). The mount function:
   - reads `self.ui.widget(cx, ids!(host))` to get a `WidgetRef` to the host slot,
   - calls `mem::replace(&mut *host, view)` with a freshly built `View` for the target route,
   - calls `cx.widget_tree().refresh_from_borrowed(host_uid, ...)` with the OLD host uid.

2. **Trigger** — `finance-brief/bundle/screens/launcher.octoscript` mounts 11 launcher tiles. Clicking any tile fires `mount(launcher_tile.id)` in `app.rs`, which performs the swap to the corresponding route (e.g. `news-list`, `kline`, `weather`, etc.).

3. **Tree dump verification**:
   - `finance-brief/scripts/_dump_widget_tree.py` — calls the remote bridge endpoint `/d` and writes `widget_tree.json`. After clicking a launcher tile, the JSON's `nodes[].id` should change to the new route's widget ids; in practice it stays at the launcher ids.
   - `finance-brief/scripts/_capture_g.py` — calls `/g` and writes `widget_graph.json`. The `host` slot's `children[]` stays `[launcher_inner_uid, ...]` even after a swap to `news-list`.
   - Compare SHA256 of the two PNG snapshots taken before and after the click. The diff is empty (the screen does not change).

4. **Expected vs actual** (v15):
   - Expected: `widget_graph.json` shows `host.children == [news_list_inner_widget_uid, ...]` after the click; `widget_tree.json` shows news-list widgets; PNG diff is non-empty.
   - Actual: `host.children == [launcher_inner_widget_uid, ...]` (or `[]` after `remove_subtree`); PNG diff is empty.

### Reference files in finance-brief

| Path | Role |
|------|------|
| `finance-brief/apps/desktop/src/app.rs` | `mount(cx, &mut self, route)` — the host-slot-swap site |
| `finance-brief/apps/desktop/src/lib.rs` | `host := View{ width: Fill, height: Fill }` (v15) — the slot declaration |
| `finance-brief/bundle/screens/launcher.octoscript` | 11-tile launcher that triggers the swap |
| `finance-brief/scripts/_dump_widget_tree.py` | Calls `/d`, writes `widget_tree.json` |
| `finance-brief/scripts/_capture_g.py` | Calls `/g`, writes `widget_graph.json` |
| `finance-brief/scripts/_diff_dump.py` | SHA256 diffs two dumps |
| `finance-brief/scripts/verify_swap.py` | End-to-end launcher-click → news-list verification (currently FAIL by design — v15 didn't fix the dep bug) |
| `finance-brief/.todo-finance-brief-v15-host-view-2026-10-05.md` | Detailed v14→v15 root-cause write-up (DBG evidence included) |

---

## §2.8 验证建议 (cargo test templates for dep maintainers)

The following tests should be added to `makepad/widgets/src/widget_tree.rs` `mod tests` (currently L3190-5104) to lock the fix. They do **not** require a Cargo workspace rebuild beyond `cargo test -p makepad-widgets`.

### Test 1: `refresh_from_borrowed` promotes placeholder when called with the live `WidgetRef`

```rust
#[test]
fn refresh_from_borrowed_promotes_placeholder_with_live_widget_ref() {
    // arrange: a TestWidget (L3255-3259) under a host View, refresh_from_borrowed
    //          inserts a placeholder (matches widget_tree.rs:1083-1102).
    let cx = Cx::new();
    let host = make_widget(); // returns WidgetRef to a freshly-constructed TestWidget
    let host_uid = host.widget_uid();
    cx.widget_tree().refresh_from_borrowed(host_uid, &mut |visit| {
        visit(id!(child_a), make_widget());
    });
    // sanity: graph node is placeholder, not yet promoted
    {
        let inner = cx.widget_tree().inner.borrow();
        assert!(inner.graph[&host_uid].placeholder);
    }

    // act: call refresh_from_borrowed again, this time passing the LIVE WidgetRef
    //      so the API can promote the placeholder (方案 B's contract).
    cx.widget_tree().refresh_from_borrowed(host, &mut |visit| {
        visit(id!(child_a), make_widget());
        visit(id!(child_b), make_widget());
    });

    // assert: graph node is promoted; children list updated.
    let inner = cx.widget_tree().inner.borrow();
    let node = &inner.graph[&host_uid];
    assert!(!node.placeholder, "placeholder should be cleared on live WidgetRef refresh");
    assert_eq!(node.children.len(), 2, "should now host child_a and child_b");
}
```

### Test 2: host slot uid is stable across `mem::replace` (方案 A1 contract)

```rust
#[test]
fn host_view_slot_uid_stable_across_mem_replace() {
    use std::mem;
    let cx = Cx::new();
    // arrange: a Box<View> slot, one inner View.
    let mut slot: Box<View> = Box::new(make_view(/*route_a*/));
    let initial_uid = slot.widget_uid();
    cx.widget_tree().seed_from_widget(slot.as_widget_ref());

    // act: swap in a new View (simulates mem::replace for route B)
    let route_b = make_view(/*route_b*/);
    let route_b_uid = route_b.widget_uid();
    let replaced = mem::replace(&mut *slot, route_b);
    drop(replaced); // mimic the old struct being dropped
    cx.widget_tree().refresh_from_borrowed(slot.as_widget_ref(), &mut |visit| {
        visit(id!(route_b_root), make_widget());
    });

    // assert: slot identity survived the swap; graph is rooted at the original uid.
    let inner = cx.widget_tree().inner.borrow();
    assert_eq!(inner.root_uid, initial_uid, "host slot uid must not change on swap");
    assert!(inner.graph.contains_key(&initial_uid), "graph must still key on host slot uid");
    assert!(!inner.graph.contains_key(&route_b_uid), "per-instance route_b uid must NOT pollute the graph");
}
```

### Test 3: `compact_dump` reflects route B after swap

```rust
#[test]
fn compact_dump_reflects_route_after_host_swap() {
    let cx = Cx::new();
    let mut slot: Box<View> = Box::new(make_view(/*route_a*/));
    cx.widget_tree().seed_from_widget(slot.as_widget_ref());
    cx.widget_tree().mark_dirty(slot.widget_uid());

    // mount route B
    let route_b = make_view_with_id("route_b_widget");
    std::mem::replace(&mut *slot, route_b);
    cx.widget_tree().refresh_from_borrowed(slot.as_widget_ref(), &mut |visit| {
        visit(id!(route_b_root), make_widget());
    });

    let dump = cx.widget_tree().compact_dump(&cx);
    assert!(dump.contains("route_b_widget"), "compact_dump must show the new route's widgets");
    assert!(!dump.contains("route_a"), "compact_dump must not show stale route A widgets");
}
```

### Existing tests that should still pass

All tests in `widget_tree.rs mod tests` (L3190-5104), in particular:
- `test_refresh_from_borrowed_discovers_children` (L3632-3647) — old behaviour preserved when caller does NOT swap.
- `test_root_lookup_heals_after_subtree_replaced` (L3752-3796) — confirms the `remove_subtree` path still works when intentionally invoked.
- `test_repeated_refresh_no_spurious_rebuild` (L4029-4071) — confirms `flush_dirty`'s `if parent_placeholder { return true }` (L1622-1626) stays intact for non-host placeholder nodes (e.g. borrowed children inside a parent that hasn't been seeded yet).

Plus any `Splash`-specific tests in `splash.rs mod tests` (L1038-1148, especially `style_reapply_keeps_body_module_state` L1121-1147) to make sure `set_text` (splash.rs:678-692) is not regressed by 方案 C2's proposed `seed_from_widget` call.

---

## Appendix: line-number index

| File | Lines | What |
|------|-------|------|
| `makepad/widgets/src/widget_tree.rs` | 144-180 | `WidgetTreeInner` struct (graph, dirty, root_uid, structure_dirty, dense_stale) |
| `makepad/widgets/src/widget_tree.rs` | 182-187 | `WidgetTreeNode` (dense-index row) |
| `makepad/widgets/src/widget_tree.rs` | 189-205 | `GraphNode` (placeholder + manual fields) |
| `makepad/widgets/src/widget_tree.rs` | 673-842 | `insert_child` (placeholder parent insertion at L687-705; manual=true at L717, 747) |
| `makepad/widgets/src/widget_tree.rs` | 846-873 | `insert_child_deep` (recursive, post-creation only) |
| `makepad/widgets/src/widget_tree.rs` | 886-892 | `mark_dirty` |
| `makepad/widgets/src/widget_tree.rs` | 894-955 | `seed_from_widget` (clears placeholder at L908-919) |
| `makepad/widgets/src/widget_tree.rs` | 957-1063 | `set_root_widget` (clears placeholder at L985-989) |
| `makepad/widgets/src/widget_tree.rs` | 1065-1119 | `refresh_from_borrowed` (placeholder insert at L1083-1102) |
| `makepad/widgets/src/widget_tree.rs` | 1121-1130 | `sync_dirty` |
| `makepad/widgets/src/widget_tree.rs` | 1135-1141 | `sync_dense` |
| `makepad/widgets/src/widget_tree.rs` | 1152-1163 | `flush_dirty` |
| `makepad/widgets/src/widget_tree.rs` | 1612-1654 | `refresh_node_children` (placeholder early-return at L1622-1626; remove_subtree on dangling at L1629) |
| `makepad/widgets/src/widget_tree.rs` | 1656-1871 | `refresh_node_children_from_discovered` (parent_is_placeholder at L1676, kept_real_parent at L1722-1724; manual-keep at L1816-1822) |
| `makepad/widgets/src/widget_tree.rs` | 1873-1891 | `remove_subtree` |
| `makepad/widgets/src/widget_tree.rs` | 1893-1946 | `rebuild_dense` (walks `node.children` at L2007-2010 in `build_dense_from_iterative`) |
| `makepad/widgets/src/widget_tree.rs` | 2780-3009 | `compact_dump` (sync_dense at L2781) |
| `makepad/widgets/src/widget_tree.rs` | 3035-3045 | `CxWidgetExt` trait |
| `makepad/widgets/src/widget_tree.rs` | 3084-3113 | `impl CxWidgetExt for Cx` |
| `makepad/widgets/src/widget_tree.rs` | 3115-3148 | `impl CxWidgetExt for Cx2d` |
| `makepad/widgets/src/widget_tree.rs` | 3190-5104 | `mod tests` (existing unit tests) |
| `makepad/widgets/src/splash.rs` | 26-124 | `Splash` struct (uid at L29, view at L33, body at L35, source at L31, allow_net at L37) |
| `makepad/widgets/src/splash.rs` | 585-605 | `impl WidgetNode for Splash` (widget_uid at L586-588; children delegates to view at L602-604) |
| `makepad/widgets/src/splash.rs` | 665-672 | `Splash::draw_walk` (delegates to `self.view.draw_walk`) |
| `makepad/widgets/src/splash.rs` | 678-692 | `Splash::set_text` (calls eval_body or stop) |
| `makepad/widgets/src/splash.rs` | 420-434 | `Splash::stop` (reclaims isolate) |
| `makepad/widgets/src/view.rs` | 70-180 | `View` struct (uid at L72, source at L74, draw_bg at L77, layout at L83, walk at L86, visible at L106, children at L174) |
| `makepad/widgets/src/view.rs` | 748-865 | `impl WidgetNode for View` (widget_uid at L749-751; children at L774-778; redraw at L764-772) |
| `makepad/widgets/src/view.rs` | 195-316 | `impl ScriptHook for View` (on_before_apply, on_after_apply) |
| `makepad/widgets/src/widget.rs` | 17 | `static WIDGET_UID_COUNTER: AtomicU64 = AtomicU64::new(1)` |
| `makepad/widgets/src/widget.rs` | 19-26 | `WidgetUid` struct + `WidgetUid::new` |
| `makepad/widgets/src/widget.rs` | 28-223 | `WidgetNode` trait (widget_uid at L29; children at L31) |
| `makepad/widgets/src/widget.rs` | 820-833 | `WidgetRef::widget_uid` + `try_widget_uid` |

---

*End of RFC draft. finance-brief will not push a PR; this is intended as input for the upstream maintainers' triage.*
