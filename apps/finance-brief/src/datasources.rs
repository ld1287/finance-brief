//! Bridge `host.fetch` from the Octoscript DSL to the finance-brief native
//! adapters under `finance-brief/native/src/adapters/`.
//!
//! The `bundle/workflow.octoscript` already names every capability in
//! `workflow.plan([…])` form. The screens' `sget(key, default)` and
//! `tapto: "set:…"` actions trigger `host.fetch` invocations; this module
//! exposes a `mod.fb.<capability>` for each of the 17 capabilities declared
//! in `finance-brief/native/capabilities.toml`.
//!
//! This file is the ONE place the screens call into: it dispatches to the
//! nine Rust adapter crates, each of which talks to its upstream data source
//! (sina / tencent / stooq / hyperliquid / frankfurter) and writes to a
//! cache that the next `sget` reads. Per `docs/SPLASH-STYLE.md §4.4`,
//! `host.fetch` is fire-and-forget; reply data writes to the cache and the
//! next screen re-render picks it up.
//!
//! The 17 capabilities (per `finance-brief/native/capabilities.toml`):
//!
//! | name                    | adapter              |
//! |-------------------------|-------------------------|
//! | `news.refresh`          | `news_sina`           |
//! | `news.read`             | `news_sina`           |
//! | `quote.snapshot` (a/hk) | `quote_tencent`       |
//! | `quote.snapshot` (us)   | `quote_stooq`         |
//! | `quote.snapshot` (crypto) | `quote_hyperliquid` |
//! | `quote.snapshot` (fx)   | `quote_frankfurter`   |
//! | `quote.candles`         | `synth_candles`       |
//! | `research.list`         | (static sample)       |
//! | `research.read`         | (static sample)       |
//! | `stream.subscribe`      | `stream_subscribe`    |
//! | `stream.unsubscribe`    | `stream_subscribe`    |
//! | `stream.frequency.set`  | `stream_subscribe`    |
//! | `stream.tick`           | `stream_tick`         |
//! | `datasource.status`     | `datasource_status`   |
//! | `fav.list`              | (local storage)       |
//! | `fav.toggle`            | (local storage)       |
//! | `settings.load`         | (local storage)       |
//! | `settings.save`         | (local storage)       |
//!
//! Total: 18 names listed; `quote.snapshot` has 4 tab variants, so the
//! actual distinct `capability_name`s in `capabilities.toml` are 17.

use makepad_widgets::*;

/// Register every `mod.fb.<cap>` as a host-call handler in the app's main VM.
/// Called from `App::script_mod`, which fires once at app startup before
/// any `.octoscript` is evaluated.
pub fn register_capability_handlers(vm: &mut ScriptVm) {
    // News: sina RSS, GBK encoded
    vm.new_fn(crate_ns!(fb), "news_refresh", news_refresh);
    vm.new_fn(crate_ns!(fb), "news_read", news_read);

    // Quote snapshot: tab dispatches to the right adapter
    vm.new_fn(crate_ns!(fb), "quote_snapshot_a", quote_snapshot_a);
    vm.new_fn(crate_ns!(fb), "quote_snapshot_us", quote_snapshot_us);
    vm.new_fn(
        crate_ns!(fb),
        "quote_snapshot_crypto",
        quote_snapshot_crypto,
    );
    vm.new_fn(crate_ns!(fb), "quote_snapshot_fx", quote_snapshot_fx);

    // Candles (synthesized)
    vm.new_fn(crate_ns!(fb), "quote_candles", quote_candles);

    // Research cards (static sample for MVP)
    vm.new_fn(crate_ns!(fb), "research_list", research_list);
    vm.new_fn(crate_ns!(fb), "research_read", research_read);

    // Stream
    vm.new_fn(crate_ns!(fb), "stream_subscribe", stream_subscribe);
    vm.new_fn(crate_ns!(fb), "stream_unsubscribe", stream_unsubscribe);
    vm.new_fn(crate_ns!(fb), "stream_frequency_set", stream_frequency_set);
    vm.new_fn(crate_ns!(fb), "stream_tick", stream_tick);

    // Datasource status
    vm.new_fn(crate_ns!(fb), "datasource_status", datasource_status);

    // Local storage (favorites + settings)
    vm.new_fn(crate_ns!(fb), "fav_list", fav_list);
    vm.new_fn(crate_ns!(fb), "fav_toggle", fav_toggle);
    vm.new_fn(crate_ns!(fb), "settings_load", settings_load);
    vm.new_fn(crate_ns!(fb), "settings_save", settings_save);
}

// ── capability stubs ─────────────────────────────────────────────────
//
// Each is a thin shim that, in the MVP, returns an empty result so the
// screens render with the seeded sample data baked into the .octoscript
// files. The actual native adapter calls live in
// `finance-brief/native/src/adapters/<x>.rs`; this host wiring is where
// they are re-exposed to the Octoscript VM after the migration from
// splash host.call to host.fetch on the octoscript-makepad VM.
//
// Wiring these up against the real adapters requires that
// `finance-brief/native/` be a workspace member or sibling crate of
// `apps/finance-brief/`; that is the next step (host-service crate in
// `.todo-c-path3-reality-2026-10-04.md §4`). For this MVP commit we ship
// the dispatcher + screen wiring, and leave the adapter call sites as
// TODO markers with the exact name they will dispatch to.

fn news_refresh(_vm: &mut ScriptVm, _args: Vec<ScriptValue>) -> ScriptValue {
    // TODO: dispatch to finance_brief_native::adapters::news_sina::handle
    //       with args[0] = {limit: i32}. The adapter returns a Vec<NewsItem>
    //       shaped per `finance-brief/bundle/schema/news.schema.json`.
    log!("fb.news_refresh: stub — wire to finance_brief_native::adapters::news_sina");
    ScriptValue::NIL
}

fn news_read(_vm: &mut ScriptVm, _args: Vec<ScriptValue>) -> ScriptValue {
    // TODO: dispatch to news_sina with args[0] = {key: String}.
    log!("fb.news_read: stub");
    ScriptValue::NIL
}

fn quote_snapshot_a(_vm: &mut ScriptVm, _args: Vec<ScriptValue>) -> ScriptValue {
    // TODO: dispatch to quote_tencent (tab=a).
    log!("fb.quote_snapshot_a: stub — wire to finance_brief_native::adapters::quote_tencent");
    ScriptValue::NIL
}

fn quote_snapshot_us(_vm: &mut ScriptVm, _args: Vec<ScriptValue>) -> ScriptValue {
    // TODO: dispatch to quote_stooq (tab=us, per Q-R1).
    log!("fb.quote_snapshot_us: stub — wire to finance_brief_native::adapters::quote_stooq");
    ScriptValue::NIL
}

fn quote_snapshot_crypto(_vm: &mut ScriptVm, _args: Vec<ScriptValue>) -> ScriptValue {
    // TODO: dispatch to quote_hyperliquid (tab=crypto).
    log!("fb.quote_snapshot_crypto: stub — wire to finance_brief_native::adapters::quote_hyperliquid");
    ScriptValue::NIL
}

fn quote_snapshot_fx(_vm: &mut ScriptVm, _args: Vec<ScriptValue>) -> ScriptValue {
    // TODO: dispatch to quote_frankfurter (tab=fx).
    log!("fb.quote_snapshot_fx: stub — wire to finance_brief_native::adapters::quote_frankfurter");
    ScriptValue::NIL
}

fn quote_candles(_vm: &mut ScriptVm, _args: Vec<ScriptValue>) -> ScriptValue {
    // TODO: dispatch to synth_candles with args[0] = {symbol, period}.
    log!("fb.quote_candles: stub — wire to finance_brief_native::adapters::synth_candles");
    ScriptValue::NIL
}

fn research_list(_vm: &mut ScriptVm, _args: Vec<ScriptValue>) -> ScriptValue {
    // Static samples baked into the Octoscript screens.
    ScriptValue::NIL
}

fn research_read(_vm: &mut ScriptVm, _args: Vec<ScriptValue>) -> ScriptValue {
    // Static samples: per-id lookup against the baked-in set.
    ScriptValue::NIL
}

fn stream_subscribe(_vm: &mut ScriptVm, _args: Vec<ScriptValue>) -> ScriptValue {
    // TODO: dispatch to stream_subscribe with args[0] = {symbols, freq_ms}.
    ScriptValue::NIL
}

fn stream_unsubscribe(_vm: &mut ScriptVm, _args: Vec<ScriptValue>) -> ScriptValue {
    // TODO: dispatch to stream_subscribe.
    ScriptValue::NIL
}

fn stream_frequency_set(_vm: &mut ScriptVm, _args: Vec<ScriptValue>) -> ScriptValue {
    // TODO: dispatch to stream_subscribe.
    ScriptValue::NIL
}

fn stream_tick(_vm: &mut ScriptVm, _args: Vec<ScriptValue>) -> ScriptValue {
    // TODO: dispatch to stream_tick with args[0] = {kind, limit}.
    ScriptValue::NIL
}

fn datasource_status(_vm: &mut ScriptVm, _args: Vec<ScriptValue>) -> ScriptValue {
    // TODO: dispatch to datasource_status with args[0] = {capabilities}.
    ScriptValue::NIL
}

fn fav_list(_vm: &mut ScriptVm, _args: Vec<ScriptValue>) -> ScriptValue {
    // Local storage; the actual read happens on the next `sget("fav")`
    // call. Stub keeps the surface minimal.
    ScriptValue::NIL
}

fn fav_toggle(_vm: &mut ScriptVm, _args: Vec<ScriptValue>) -> ScriptValue {
    ScriptValue::NIL
}

fn settings_load(_vm: &mut ScriptVm, _args: Vec<ScriptValue>) -> ScriptValue {
    ScriptValue::NIL
}

fn settings_save(_vm: &mut ScriptVm, _args: Vec<ScriptValue>) -> ScriptValue {
    ScriptValue::NIL
}
