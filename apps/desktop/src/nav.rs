//! Navigation bridge — taps from the mounted body land here.
//!
//! The emitted `on_click` handler in every mounted screen calls the `NAV`
//! global with the route name (e.g. `NAV(t: "news_list")`). `register_nav`
//! wires that global to push the route into a thread-safe queue
//! (`TAPS`), which `App::handle_event` drains on each `NextFrame` to decide
//! whether to re-mount.
//!
//! This is the makepad-script mirror of `kit-host`'s `nav_signal` polling
//! pattern; the body evaluates on the APP VM (not the Splash isolate), so
//! the global must be registered here.

use makepad_widgets::*;
use octoscript_render::makepad_script::live_id;

/// FIFO queue of pending route taps from the mounted body. Static so the
/// closure passed to `add_global_fn` (which has no captured state) can still
/// reach it.
static TAPS: std::sync::Mutex<Vec<String>> = std::sync::Mutex::new(Vec::new());

/// Drain the oldest tap off the queue, if any.
pub fn take_tap() -> Option<String> {
    TAPS.lock().ok().and_then(|mut q| {
        if q.is_empty() {
            None
        } else {
            Some(q.remove(0))
        }
    })
}

/// Register the `NAV(t: "…")` global on `vm`. Called from `App::script_mod`
/// once per VM init; the closure runs on every tap the mounted body emits.
pub fn register_nav(vm: &mut ScriptVm) {
    let f_nav = octoscript_render::add_global_fn(
        vm,
        &[(
            live_id!(t),
            octoscript_render::makepad_script::ScriptValue::NIL,
        )],
        |vm, a| {
            let t = octoscript_render::string_prop(vm, a, live_id!(t)).unwrap_or_default();
            if let Ok(mut q) = TAPS.lock() {
                q.push(t);
            }
            octoscript_render::makepad_script::ScriptValue::NIL
        },
    );
    vm.set_injected_global(live_id!(NAV), f_nav);
}
