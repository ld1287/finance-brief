//! App shell — single root struct, owns the mounted body, drives re-mounts.

use makepad_widgets::*;

use crate::baked;
use crate::nav;

/// Single root struct mirroring `flutter-samples/src/lib.rs`. Holds the mounted
/// body in `Splash.view` and the routing signal in `nav_signal`. Re-mounts on
/// every NAV emission from the bundle (or on theme toggle / state apply).
#[derive(Script, ScriptHook)]
pub struct App {
    #[live]
    ui: WidgetRef,
    #[rust]
    next_frame: NextFrame,
    #[rust]
    last_src: String,
    #[rust]
    route: String,
    #[rust]
    dark: bool,
    /// Viewport in vp, from WindowGeomChange. 0 until the first one arrives.
    #[rust]
    vw: f64,
    #[rust]
    vh: f64,
    #[rust]
    started: bool,
}

impl App {
    /// Translate + mount the active route.
    fn mount(&mut self, cx: &mut Cx) {
        cx.set_key_focus(Area::Empty);
        let src = baked::current_source();
        self.last_src = src.clone();
        let route = if self.route.is_empty() {
            "launcher"
        } else {
            &self.route
        };
        // `Event::Startup` 调 mount 时 WindowGeomChange 还没来过, vw/vh 仍是 0.0;
        // 此时 fillw/fillh 在 0×0 下塌缩, 窗口停留空白帧。Fallback 到 lib.rs:39
        // 声明的 inner_size 兜底, 与 Window{window.inner_size: vec2(440, 1400)} 一致。
        let (vw, vh) = if self.vw > 0.0 && self.vh > 0.0 {
            (self.vw, self.vh)
        } else {
            (440.0, 1400.0)
        };
        let full = octoscript_makepad::kit::with_state_sized(route, self.dark, 0.0, vw, vh, &src);
        let built =
            octoscript_render::build(&full, octoscript_makepad::kit::register_stub_capabilities);
        let mut eval_ok = false;
        let mut view_set = false;
        if let Some(ref node) = built {
            let ui = octoscript_makepad::to_makepad_ui(node);
            // On the APP VM, with THIS crate as the module identity — not as
            // text into the `Splash` isolate. Two reasons, both documented in
            // flutter-samples/src/lib.rs:250-260.
            let code = format!("use mod.prelude.widgets.*\nView{{height:Fill, {ui}");
            let script_mod = ScriptMod {
                cargo_manifest_path: env!("CARGO_MANIFEST_DIR").to_string(),
                module_path: module_path!().to_string(),
                file: file!().to_string(),
                line: 1,
                column: 0,
                code,
                values: Vec::new(),
            };
            let built_view = cx.with_vm(|vm| {
                vm.eval_checked(script_mod, 2_000_000)
                    .map(|value| View::script_from_value(vm, value))
            });
            if built_view.is_none() {
                log!("finance-brief MOUNT eval returned error (route={})", route);
            }
            eval_ok = built_view.is_some();
            let host_ref = self.ui.widget(cx, ids!(host));
            let host_uid = host_ref.widget_uid();
            // v16: Force the host View's GraphNode to a non-placeholder state so
            // flush_dirty can walk host's children after swap. set_root_widget installs
            // a valid WidgetWeakRef + placeholder=false into graph[host_uid] (per
            // widget_tree.rs:957-1014). Without this, refresh_from_borrowed's children
            // update never gets picked up by walk (Splash path had same bug; v15 made it
            // visible by switching host to View). Safe to call repeatedly: after first
            // mount, root_uid is non-zero, so this only inserts a fresh GraphNode.
            cx.widget_tree().set_root_widget(host_ref.clone());
            // Save the old view so we can retire its children after the swap.
            // `host.view = view` only mutates the struct field; without
            // `widget_tree_insert_child_deep` the widget tree still hangs on to
            // the previous route's children and `/snap` keeps reporting the
            // launcher even though MOUNT succeeded (v10 fix).
            if let Some(view) = built_view {
                // ATOMIC REPLACE: View struct swap includes children + walk + layout.
                // Drop borrow BEFORE snapshotting new children / walking tree.
                let retired = {
                    let mut host = match host_ref.borrow_mut::<View>() {
                        Some(h) => h,
                        None => {
                            log!("finance-brief MOUNT host borrow_mut::<View>() failed");
                            return;
                        }
                    };
                    let pre_host_uid = host.widget_uid();
                    log!(
                        "finance-brief DBG mount route={} pre_host_uid={:?} new_view_widget_uid={:?}",
                        route,
                        pre_host_uid,
                        view.widget_uid()
                    );
                    let r = std::mem::replace(&mut *host, view);
                    let post_host_uid = host.widget_uid();
                    log!(
                        "finance-brief DBG mount route={} post_host_uid={:?} retired.uid={:?}",
                        route,
                        post_host_uid,
                        r.widget_uid()
                    );
                    log!(
                        "finance-brief DBG mount route={} host_uid={:?} retired.children.len={}",
                        route,
                        host_uid,
                        r.children.len()
                    );
                    r
                };
                // Snapshot the NEW view's children (after replace, host's children
                // are the freshly-scripted ones).
                let mut new_children: Vec<(LiveId, WidgetRef)> = Vec::new();
                host_ref.children(&mut |id, child| new_children.push((id, child)));
                log!(
                    "finance-brief DBG mount route={} new_children.len={} first_child_uid={:?}",
                    route,
                    new_children.len(),
                    new_children.first().map(|(_, w)| w.widget_uid())
                );
                // Retire the OLD view's overlay draw lists and walk its subtree to
                // release any per-widget resources (mirrors calendar L185-187 /
                // beauty L185-188).
                for (_, child) in retired.children.iter() {
                    retire_widget_tree(cx, child);
                }
                log!(
                    "finance-brief DBG mount route={} about-to-insert children count={}",
                    route,
                    new_children.len()
                );
                // Re-register the new subtree under host_uid. View is NOT a
                // placeholder (Splash was), so flush_dirty will walk host's live
                // children and pull the entire new screen's widgets into the graph
                // (Splash's subtree was never re-walked after the first mount —
                // see .todo-v14 dump analysis).
                cx.widget_tree().refresh_from_borrowed(host_uid, |visit| {
                    for (id, child) in &new_children {
                        visit(*id, child.clone());
                    }
                });
                cx.widget_tree_mark_dirty(host_uid);
                view_set = true;
                log!("finance-brief DBG host_uid={:?} view_set_done", host_uid);
                // v14 diagnostic — keep until v16 confirms /d shows the new widgets.
                let dump = cx.widget_tree().compact_dump(cx);
                log!(
                    "finance-brief DBG mount route={} compact_dump ({} bytes):\n{}",
                    route,
                    dump.len(),
                    dump
                );
            }
            cx.redraw_all();
        }
        log!(
            "finance-brief MOUNT route={route} src_len={} built={} eval_ok={} view_set={}",
            src.len(),
            built.is_some(),
            eval_ok,
            view_set
        );
    }
}

impl MatchEvent for App {}

impl AppMain for App {
    fn script_mod(vm: &mut ScriptVm) -> ScriptValue {
        // The theme the M3 widgets resolve against must match the surface the
        // host paints; finance-brief renders a near-white background, so light.
        crate::makepad_widgets::theme_mod(vm);
        script_eval!(vm, {
            mod.theme = mod.themes.light
        });
        // Fork-free themed widgets (Material 3), against upstream makepad.
        octoscript_widgets::widgets_mod(vm);
        // K-line + future plot widgets under `mod.plot.*`.
        makepad_plot::script_mod(vm);
        // Taps from the mounted body — `tapto:` strings land here.
        nav::register_nav(vm);
        // Expose `mod.fb.*` so the screens' `use mod.fb.*` resolves on the app VM.
        crate::datasources::register_capability_handlers(vm);
        super::script_mod(vm)
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event) {
        self.match_event(cx, event);
        if let Event::WindowGeomChange(geom) = event {
            let g = &geom.new_geom;
            let (vw, vh) = (g.inner_size.x, g.inner_size.y);
            if (vw - self.vw).abs() > 0.5 || (vh - self.vh).abs() > 0.5 {
                self.vw = vw;
                self.vh = vh;
                self.last_src.clear();
                self.mount(cx);
            }
        }
        if let Event::Startup = event {
            if !self.started {
                self.started = true;
                // Arm it BEFORE the first mount so the same handler also drives
                // subsequent re-mounts from NAV taps (kit-host's pattern).
                self.next_frame = cx.new_next_frame();
                self.mount(cx);
            }
            return;
        }
        if self.next_frame.is_event(event).is_some() {
            // Drive re-mounts from NAV taps emitted by the mounted body.
            let nav_raw = nav::take_tap().unwrap_or_default();
            let nav = nav_raw.trim();
            if !nav.is_empty() {
                if nav == "theme:toggle" {
                    self.dark = !self.dark;
                    self.last_src.clear();
                    self.mount(cx);
                } else if octoscript_render::state::apply(nav) {
                    self.last_src.clear();
                    self.mount(cx);
                } else {
                    self.route = nav.to_string();
                    self.last_src.clear();
                    self.mount(cx);
                }
                cx.redraw_all();
            }
            // Re-arm so the next tap also lands here.
            self.next_frame = cx.new_next_frame();
        }
        // Forward every event to the Root widget so Draw, WindowGeomChange,
        // Mouse, Touch etc. reach the tree; without this, Root's `draw_all`
        // never fires and Splash's draw_walk never runs (kit-host L887-891,
        // flutter-samples L476-480 — finance-brief originally omitted it).
        self.ui.handle_event(cx, event, &mut Scope::empty());
    }
}

/// Recursively retire a widget subtree's overlay draw lists and children.
/// finance-brief's bundle doesn't use DesignOverlay/DesignGlassSvg, so
/// `retire_overlay` is a no-op for our screens; we still walk the subtree
/// so any retained draw lists in descendant widgets get cleared.
fn retire_widget_tree(cx: &mut Cx, widget: &WidgetRef) {
    octoscript_widgets::kit::retire_overlay(cx, widget);
    let mut children: Vec<WidgetRef> = Vec::new();
    widget.children(&mut |_, child| children.push(child));
    for child in children {
        retire_widget_tree(cx, &child);
    }
}
