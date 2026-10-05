//! finance-brief — OctoSense app shell that renders the finance-brief
//! bundle as native makepad widgets, via the octoscript-makepad pipeline.
//!
//! Shape is the Octoscript-Makepad flutter-samples app shell:
//! `bundle/screens/_kit.octoscript` is concatenated first, the screens sorted,
//! and `_index.octoscript` last (the router). Hot-reload override lives at
//! `DEVICE_PATH`.
//!
//! Mirror of `Octoscript-Makepad/apps/flutter-samples/src/lib.rs`, simplified
//! for finance-brief: no self-drive, no route override file, no viewport
//! measurement. Single `App` struct, mount on next_frame, `AppMain` hookups.
//!
//! File layout:
//! - `baked.rs`   — `BAKED` const (concatenation of every `.octoscript`),
//!                  `DEVICE_PATH`, `current_source` helper, drift-guard test.
//! - `nav.rs`     — `NAV(t: "…")` global registration + tap queue.
//! - `app.rs`     — `App` struct, `mount`, `AppMain`, event handling.
//! - `datasources/` — `mod.fb.*` namespace registration. Stubs only — real
//!                  adapter wiring into `native/` is post-merge work.

pub use makepad_widgets;
use makepad_widgets::*;

app_main!(App);

mod app;
mod baked;
pub mod datasources;
mod nav;

pub use app::App;

script_mod! {
    use mod.prelude.widgets.*

    startup() do #(App::script_component(vm)){
        ui: Root{
            main_window := Window{
                window.inner_size: vec2(440, 1400)
                body +: {
                    flow: Down
                    ScrollYView{
                        width: Fill
                        height: Fill
                        flow: Down
                        show_bg: true
                        draw_bg +: { color: #xfafafaff }
                        // v15: View instead of Splash. Splash's graph-placeholder behaviour
                        // (refresh_from_borrowed inserts the host with an empty WidgetWeakRef and
                        // `placeholder = true`, so flush_dirty never re-walks it after the first
                        // mount — v14 dump showed the graph stays at the launcher's widgets
                        // forever, and the second click's MOUNT never replaces them). View is a
                        // plain widget that we can mutate children on directly; the routing
                        // semantics are unchanged.
                        host := View{ width: Fill, height: Fill }
                    }
                    // Routing signal the mounted bundle writes; the app reads
                    // it each frame and re-mounts. Mirrors flutter-samples'
                    // nav_signal pattern.
                    nav_signal := Label{ text: "" height: 0 draw_text.text_style.font_size: 1 }
                }
            }
        }
    }
}
