# GitHub Issue: Splash/View host-slot swap doesn't refresh widget tree graph

> **Title suggestion**: `widget_tree: refresh_from_borrowed does not refresh host slot after mem::replace on Splash or View`
>
> **Labels**: `bug`, `area::widget-tree`, `severity::blocker`
>
> **Affects**: `makepad/widgets/src/{widget_tree.rs, splash.rs, view.rs, widget.rs}`
>
> **Reproducer**: finance-brief (OctoSense app) — `apps/desktop/src/app.rs` mount() function

---

## Summary

After `mem::replace(&mut *host, view)` (host is a `Splash` or a plain `View`), calling `refresh_from_borrowed(host_uid, ...)` does not update the widget tree graph to reflect the new view's children. The graph continues to show the old route's widgets, `compact_dump` does not change, `/d` and `/g` outputs are byte-identical before and after the swap.

This is a blocker for any SPA / multi-route / launcher-style application that swaps the root view per navigation.

## Reproduction

Repository: <https://github.com/OctoSense-org/finance-brief> (HEAD `5bef74ea`)
makepad rev: `975c5630e01b0f3f3ae16cafd2e0fce3c9a5f7d9`

```sh
# 1. Build finance-brief desktop exe
cd finance-brief
cargo build --release --manifest-path apps/desktop/Cargo.toml --offline

# 2. Run it with the remote bridge
./apps/desktop/target/release/finance-brief.exe --remote=0

# 3. In another shell: capture /d before click
PORT=$(grep -oE 'listening on 127.0.0\.1:[0-9]+' /tmp/fb.log | grep -oE '[0-9]+$' | head -1)
curl http://127.0.0.1:$PORT/d > /tmp/before.txt

# 4. Click a launcher tile (e.g. 新闻简报 at x=120 y=204)
curl 'http://127.0.0.1:$PORT/click?x=120&y=204'

# 5. Capture /d after click
curl http://127.0.0.1:$PORT/d > /tmp/after.txt

# 6. Compare
diff /tmp/before.txt /tmp/after.txt   # ← EMPTY (bug)
sha256sum /tmp/before.txt /tmp/after.txt  # ← identical
```

The host calls `cx.widget_tree().refresh_from_borrowed(host_uid, visit)` (widget_tree.rs:1065-1119) after `mem::replace`; both before and after SHA256 hashes are byte-identical.

## Expected

After the click and the subsequent mount(route=news_list):
- `/d` shows `ScrollYView` + `news_label` widgets, and the 11 `OctoscriptTap` launcher tiles are gone.
- `/g` returns a PNG of the `news_list` screen, not the launcher.

## Actual

`/d` and `/g` keep showing the launcher. SHA256 hashes are byte-identical before and after the click.

## Root cause (with line references)

1. **`refresh_from_borrowed` (widget_tree.rs:1065-1119)** seeds the host uid with `widget: WidgetWeakRef::default()` and `placeholder: true` (L1083-1102).
2. **`refresh_node_children` (widget_tree.rs:1612-1654)** has `if parent_placeholder { return true }` (L1622-1626) — placeholder nodes are NEVER re-walked by `flush_dirty`.
3. For `Splash` host (struct-level stable uid, widget.rs:17-26): graph key is stable but node is permanently `placeholder = true`. Splash never gets promoted.
4. For `View` host (per-instance uid, view.rs:72): each `mem::replace` allocates a fresh uid from `WIDGET_UID_COUNTER` (widget.rs:17). The old uid points to a dropped `View`; `remove_subtree` (widget_tree.rs:1627-1631, L1873-1891) deletes the route A subtree; the new route B's uid is NEVER inserted because nothing calls `set_root_widget` (L957) or `seed_from_widget` (L894) on it.

finance-brief verified all three failure modes (v10, v12, v15). v15 is most diagnostic: `Splash` → `View` host + `mem::replace(&mut *host, view)` + `refresh_from_borrowed(host_uid, ...)` with new test `app_rs_calls_set_root_widget_on_host_before_view_swap`. Adding `cx.widget_tree().set_root_widget(host_ref.clone())` before the replace did NOT fix it — host_uid is still the OLD view's uid (because `WidgetRef::widget_uid()` returns the View struct's own uid, widget.rs:820-833, and the struct has been swapped out from under the WidgetRef).

## Proposed fix (RFC has 3 options, recommend A1 + B)

Full RFC at `finance-brief/docs/rfc-splash-view-swap.md`.

**Option A1 (preferred, no API change)**: introduce `#[slot_uid] uid: WidgetUid` for widgets intended as host slots. `WidgetRef::widget_uid()` prefers `slot_uid` when present. Default `View` could ship with `#[slot_uid]` since most live in long-lived host slots. `mem::replace(&mut *host, view)` no longer changes the graph key.

**Option B (more defensive)**: change `refresh_from_borrowed(uid: WidgetUid, ...)` to `refresh_from_borrowed(widget_ref: WidgetRef, ...)`. Internally read the **current** uid via `widget_ref.try_widget_uid()`, and if `graph.contains_key` is true, promote `placeholder = false` + reassign `node.widget = widget_ref.downgrade()`. This makes the API resilient against forgotten swaps.

**Recommendation**: ship A1 + B together. A1 makes the uid stable, B catches forgotten swaps.

**Option C2 (Splash-only hardening)**: have `Splash::set_text` (splash.rs:678-692) call `cx.widget_tree().seed_from_widget(self.as_widget_ref())` after every body change. Promotes the placeholder on every re-eval. Does NOT fix the View case — but harmless.

## Environment

- makepad rev `975c5630e01b0f3f3ae16cafd2e0fce3c9a5f7d9`
- octoscript-widgets rev (per Octoscript-Makepad/runtime.json)
- finance-brief commit `5bef74ea00129267754c4e405db8e0106b1e52e9`
- Platform: Windows (verified); macOS / Linux should reproduce identically

## Impact

- **Blocker** for SPA / multi-route / launcher-style applications
- **Non-issue** for single-page static hosts
- Affected apps in the wild: finance-brief (OctoSense), any future Octoscript app using `octoscript_widgets::widgets_mod` with route navigation

---

## Cross-references in finance-brief

- `.todo-finance-brief-v15-host-view-2026-10-05.md` — v15 Splash→View DBG traces
- `.todo-finance-brief-v16-final-attempt-2026-10-05.md` — v16 set_root_widget attempt (also failed)
- `docs/rfc-splash-view-swap.md` — full RFC with 3 fix options + line-number index

## Suggested next steps for dep maintainer

1. Decide between A1 (slot_uid attribute) vs B (refresh_from_borrowed by WidgetRef).
2. If both: ship A1 as the primary fix, B as belt-and-suspenders.
3. Add cargo tests per `docs/rfc-splash-view-swap.md §2.8` (3 templates).
4. Verify against `octoscript-widgets` test suite (`cargo test -p octoscript-widgets`).

Filed from the finance-brief side; happy to coordinate a PR if any option above is acceptable.
