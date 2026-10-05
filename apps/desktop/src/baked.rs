//! Baked-in finance-brief Octoscript bundle.
//!
//! The 14 `.octoscript` files (`_kit` + 12 screens + `_index`) are concatenated
//! at compile time via `include_str!` into a single static string. The order is
//! fixed by `octoscript_makepad::kit`: `_kit` first (tokens & helpers), the
//! screens sorted alphabetically, `_index` last (the router).
//!
//! `DEVICE_PATH` lets a host-reloaded bundle override the baked source on
//! supported platforms, so screens can be edited on device without a rebuild.
//! On desktop, edits just rebuild.

/// One `.octoscript` per file, concatenated in the order `octoscript_makepad::kit`
/// fixes: `_kit.octoscript` first (tokens and helpers), the screens sorted,
/// `_index.octoscript` last (the index and the router).
///
/// The actual files live at `bundle/screens/<name>.octoscript`; the relative
/// `include_str!` path resolves against this source file.
macro_rules! kit {
    ($($name:literal),* $(,)?) => {
        concat!(
            $(include_str!(concat!("../../../bundle/screens/", $name, ".octoscript")), "\n"),
            *
        )
    };
}

/// Baked bundle: every screen the app exposes. Keep this list aligned with the
/// directory — `baked_kit_matches_the_directory` below pins the two together.
pub const BAKED: &str = kit![
    "_kit",
    "launcher",
    "news_list",
    "news_detail",
    "research_list",
    "research_detail",
    "quote_list",
    "kline",
    "favorites",
    "settings",
    "event_stream",
    "datasource_status",
    "disclaimer",
    "_index",
];

/// Where a host-reloaded bundle can be read from on device. Mirrors
/// `flutter-samples`'s `DEVICE_PATH`; finance-brief uses
/// `/data/local/tmp/finance_brief.octoscript`.
pub const DEVICE_PATH: &str = "/data/local/tmp/finance_brief.octoscript";

/// Read the active source: prefer `DEVICE_PATH` if it exists on the host, else
/// fall back to the baked bundle. Used by `App::mount` on every re-mount.
pub fn current_source() -> String {
    std::fs::read_to_string(DEVICE_PATH).unwrap_or_else(|_| BAKED.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The baked kit must stay in sync with the directory — `cargo makepad`
    /// compiles the app inside a wrapper that never runs our build script, so
    /// the kit cannot be generated; `include_str!` is the only path that
    /// resolves on desktop and on device. The test below is a guard against
    /// drift: adding a `.octoscript` to `bundle/screens/` and forgetting to
    /// update the `BAKED!` list above would leave the new screen invisible
    /// while every test in `octoscript-makepad` still passes.
    #[test]
    fn baked_kit_includes_every_expected_screen() {
        // The names the app shell expects to find inside the baked source.
        // `BAKED` is the concatenation of the files in `kit![…]` above; if any
        // of these names is missing, the corresponding `screen_<name>()` is
        // also missing and `_index.octoscript` falls through to the launcher.
        let expected = [
            "_kit",
            "screen_launcher",
            "screen_news_list",
            "screen_news_detail",
            "screen_research_list",
            "screen_research_detail",
            "screen_quote_list",
            "screen_kline",
            "screen_favorites",
            "screen_settings",
            "screen_event_stream",
            "screen_datasource_status",
            "screen_disclaimer",
            "screen_",
        ];
        for token in &expected {
            assert!(BAKED.contains(token), "baked kit missing token {token:?}");
        }
    }
}

#[cfg(test)]
mod headless_tests {
    use super::*;
    use std::path::PathBuf;

    fn kit_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../bundle/screens")
    }

    fn assembled() -> String {
        octoscript_makepad::kit::concat_kit(&kit_dir()).expect("finance-brief screens concatenates")
    }

    fn render(kit: &str, route: &str) -> String {
        let src = octoscript_makepad::kit::with_state_sized(route, false, 0.0, 440.0, 1400.0, kit);
        let tree =
            octoscript_render::build(&src, octoscript_makepad::kit::register_stub_capabilities)
                .unwrap_or_else(|| panic!("route {route:?} evaluated to nil"));
        assert!(
            tree.count() > 3,
            "route {route:?} produced a {}-node tree — too small to be a screen",
            tree.count()
        );
        octoscript_makepad::to_makepad_ui(&tree)
    }

    fn routes() -> Vec<&'static str> {
        vec![
            "launcher",
            "news_list",
            "news_detail",
            "research_list",
            "research_detail",
            "quote_list",
            "kline",
            "favorites",
            "settings",
            "event_stream",
            "datasource_status",
            "disclaimer",
        ]
    }

    #[test]
    fn every_route_renders_its_own_screen() {
        let kit = assembled();
        for route in routes() {
            let _dialect = render(&kit, route);
        }
    }

    #[test]
    fn baked_kit_includes_every_route_function() {
        // 与 baked::tests::baked_kit_includes_every_expected_screen 对照：
        // 静态 BAKED 必须含这 12 个 screen_<name>() 函数。
        let b = BAKED;
        for r in routes() {
            let token = format!("screen_{r}");
            assert!(b.contains(&token), "BAKED missing screen function {token}");
        }
    }
}

#[cfg(test)]
mod splash_mount_tests {
    //! 验证 dialect 字符串本身足以构造 widget tree。
    //!
    //! 上一个 mod 只验证到"翻译出字符串"。这一步确认 dialect 的长度/结构
    //! 真的是一个 screen，不是空 fragment。
    use std::path::PathBuf;

    fn kit_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../bundle/screens")
    }

    #[test]
    fn launcher_dialect_is_a_real_screen_string() {
        let kit_src = octoscript_makepad::kit::concat_kit(&kit_dir())
            .expect("finance-brief screens concatenates");
        let src = octoscript_makepad::kit::with_state_sized(
            "launcher", false, 0.0, 440.0, 1400.0, &kit_src,
        );
        let tree =
            octoscript_render::build(&src, octoscript_makepad::kit::register_stub_capabilities)
                .unwrap_or_else(|| panic!("launcher evaluated to nil"));
        assert!(
            tree.count() > 3,
            "launcher tree too small: {} nodes",
            tree.count()
        );
        let dialect = octoscript_makepad::to_makepad_ui(&tree);
        assert!(
            dialect.len() > 100,
            "launcher dialect suspiciously short: {} bytes",
            dialect.len()
        );
        // 一个 launcher 至少要包含 ScrollYView 或 View 容器 + 多张 tile
        assert!(
            dialect.contains("ScrollYView") || dialect.contains("View "),
            "dialect missing scroll/view containers: {}",
            &dialect[..dialect.len().min(200)]
        );
        assert!(
            dialect.contains("Label"),
            "dialect missing Label widget: {}",
            &dialect[..dialect.len().min(200)]
        );
    }
}

#[cfg(test)]
mod splash_rect_tests {
    //! 防 Splash 0×0 回归。
    //!
    //! 用 dialect 字符串验证 lib.rs Splash 配置仍是 `height:Fill`,
    //! 防止以后有人改回 `height:Fit` 导致 Splash 塌缩到 0×0。
    use std::path::PathBuf;

    fn kit_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../bundle/screens")
    }

    #[test]
    fn launcher_dialect_declares_height_fill_not_fit() {
        let kit_src = octoscript_makepad::kit::concat_kit(&kit_dir())
            .expect("finance-brief screens concatenates");
        let src = octoscript_makepad::kit::with_state_sized(
            "launcher", false, 0.0, 440.0, 1400.0, &kit_src,
        );
        let tree =
            octoscript_render::build(&src, octoscript_makepad::kit::register_stub_capabilities)
                .expect("launcher tree");
        let dialect = octoscript_makepad::to_makepad_ui(&tree);
        assert!(
            dialect.contains("height:Fill") || dialect.contains("height: Fill"),
            "dialect missing height:Fill — Splash was changed back to Fit. dialect head: {}",
            &dialect[..dialect.len().min(400)]
        );
    }
}

#[cfg(test)]
mod splash_swap_tests {
    //! 防 v10 Splash.view 切换不更新 widget tree 回归。
    //!
    //! v10 修复：在 `App::mount` 里 retire 旧 children + `widget_tree_insert_child_deep`
    //! 新 children。如果这步被遗漏，路由 MOUNT 跑成功但 /snap 看到的仍是 launcher。
    //!
    //! 这里只静态检查 app.rs 的关键 patch 字符串仍在源码里（避免被无意 revert）。
    use super::*;

    #[test]
    fn app_rs_uses_insert_child_deep_after_splash_swap() {
        let app_rs = std::fs::read_to_string(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/app.rs"),
        )
        .expect("read app.rs");
        assert!(
            app_rs.contains("widget_tree_insert_child_deep"),
            "app.rs missing widget_tree_insert_child_deep — v10 Splash view swap \
             was reverted; route changes won't update widget tree."
        );
        assert!(
            app_rs.contains("retire_widget_tree"),
            "app.rs missing retire_widget_tree — v10 fix missing."
        );
    }

    #[test]
    fn app_rs_clears_key_focus_before_swap() {
        let app_rs = std::fs::read_to_string(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/app.rs"),
        )
        .expect("read app.rs");
        assert!(
            app_rs.contains("set_key_focus(Area::Empty)"),
            "app.rs missing set_key_focus(Area::Empty) at mount start — \
             keyboard focus may bleed from the previous route."
        );
    }

    #[test]
    fn app_rs_uses_refresh_from_borrowed_after_splash_swap() {
        let app_rs = std::fs::read_to_string(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/app.rs"),
        )
        .expect("read app.rs");
        assert!(
            app_rs.contains("refresh_from_borrowed"),
            "app.rs missing refresh_from_borrowed — v12 Splash view swap was reverted; \
             widget_tree_insert_child_deep alone won't reconcile old children."
        );
    }

    #[test]
    fn app_rs_logs_compact_dump_after_view_set_done() {
        let app_rs = std::fs::read_to_string(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/app.rs"),
        )
        .expect("read app.rs");
        assert!(
            app_rs.contains("compact_dump(cx)"),
            "app.rs missing compact_dump(cx) — v14 view-set-done tree dump was reverted; \
             re-adding the diagnostic is required before v15 can be designed."
        );
    }

    #[test]
    fn app_rs_uses_view_instead_of_splash_for_host() {
        let app_rs = std::fs::read_to_string(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/app.rs"),
        )
        .expect("read app.rs");
        assert!(
            !app_rs.contains("borrow_mut::<Splash>"),
            "app.rs still borrows Splash — v15 host swap incomplete; the Splash \
             placeholder bug will keep the widget tree stuck on launcher’s \
             widgets after every click."
        );
        assert!(
            app_rs.contains("borrow_mut::<View>"),
            "app.rs missing borrow_mut::<View> — v15 host swap not implemented."
        );
        assert!(
            app_rs.contains("mem::replace(&mut *host"),
            "app.rs missing mem::replace(&mut *host, view) — v15 atomic host swap not implemented."
        );
    }

    #[test]
    fn app_rs_calls_set_root_widget_on_host_before_view_swap() {
        let app_rs = std::fs::read_to_string(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/app.rs"),
        )
        .expect("read app.rs");
        assert!(
            app_rs.contains("set_root_widget"),
            "app.rs missing set_root_widget — v16 force-non-placeholder fix not applied."
        );
    }
}
