//! `mod.fb.*` capability handlers for finance-brief.
//!
//! Every screen does `use mod.fb.*` then calls e.g. `news.refresh({…})` or
//! `quote.snapshot_us({"AAPL"})`. We register a namespace object on the app VM
//! shaped like `mod.fb.{news, quote, research, stream, datasource, fav,
//! settings}` — each leaf is a function that currently returns NIL.
//!
//! Real adapter wiring (calling into the `native` crate) is deferred — see
//! `.todo-app-desktop-rewrite-2026-10-04.md §2` ("wire 留到下次"). For now
//! the goal is: the bundle compiles, the app launches, every `use mod.fb.*`
//! resolves, and a future PR can fill in the bodies without touching screens.

use octoscript_render::makepad_script::ScriptValue;

pub mod datasource_status;
pub mod local;
pub mod news;
pub mod quote;
pub mod research;
pub mod stream;

/// Build a `ScriptValue` from a no-arg closure that ignores VM/args and
/// returns NIL. We use `add_global_fn` because it returns a `ScriptValue`
/// (callable) that we then attach to the namespace object.
fn nil_callable(vm: &mut octoscript_render::makepad_script::ScriptVm) -> ScriptValue {
    octoscript_render::add_global_fn(vm, &[], |_vm, _a| ScriptValue::NIL)
}

/// Build `mod.fb` on the given VM. Each `fb.<ns>.<cap>` resolves to a stub
/// function returning NIL — screens compile and run; data fetching is a no-op
/// until the adapter wiring lands.
pub fn register_capability_handlers(vm: &mut octoscript_render::makepad_script::ScriptVm) {
    use makepad_widgets::*;

    // news.*
    let news_refresh = nil_callable(vm);
    let news_read = nil_callable(vm);
    let news_obj = vm.bx.heap.new_object();
    vm.bx
        .heap
        .set_value_def(news_obj, makepad_live_id::id!(refresh).into(), news_refresh);
    vm.bx
        .heap
        .set_value_def(news_obj, makepad_live_id::id!(read).into(), news_read);

    // quote.*
    let quote_snapshot_a = nil_callable(vm);
    let quote_snapshot_us = nil_callable(vm);
    let quote_snapshot_crypto = nil_callable(vm);
    let quote_snapshot_fx = nil_callable(vm);
    let quote_candles = nil_callable(vm);
    let quote_obj = vm.bx.heap.new_object();
    vm.bx.heap.set_value_def(
        quote_obj,
        makepad_live_id::id!(snapshot_a).into(),
        quote_snapshot_a,
    );
    vm.bx.heap.set_value_def(
        quote_obj,
        makepad_live_id::id!(snapshot_us).into(),
        quote_snapshot_us,
    );
    vm.bx.heap.set_value_def(
        quote_obj,
        makepad_live_id::id!(snapshot_crypto).into(),
        quote_snapshot_crypto,
    );
    vm.bx.heap.set_value_def(
        quote_obj,
        makepad_live_id::id!(snapshot_fx).into(),
        quote_snapshot_fx,
    );
    vm.bx.heap.set_value_def(
        quote_obj,
        makepad_live_id::id!(candles).into(),
        quote_candles,
    );

    // research.*
    let research_list = nil_callable(vm);
    let research_read = nil_callable(vm);
    let research_obj = vm.bx.heap.new_object();
    vm.bx.heap.set_value_def(
        research_obj,
        makepad_live_id::id!(list).into(),
        research_list,
    );
    vm.bx.heap.set_value_def(
        research_obj,
        makepad_live_id::id!(read).into(),
        research_read,
    );

    // stream.*
    let stream_subscribe = nil_callable(vm);
    let stream_unsubscribe = nil_callable(vm);
    let stream_frequency_set = nil_callable(vm);
    let stream_tick = nil_callable(vm);
    let stream_obj = vm.bx.heap.new_object();
    vm.bx.heap.set_value_def(
        stream_obj,
        makepad_live_id::id!(subscribe).into(),
        stream_subscribe,
    );
    vm.bx.heap.set_value_def(
        stream_obj,
        makepad_live_id::id!(unsubscribe).into(),
        stream_unsubscribe,
    );
    vm.bx.heap.set_value_def(
        stream_obj,
        makepad_live_id::id!(frequency_set).into(),
        stream_frequency_set,
    );
    vm.bx
        .heap
        .set_value_def(stream_obj, makepad_live_id::id!(tick).into(), stream_tick);

    // datasource.*
    let datasource_status = nil_callable(vm);
    let ds_obj = vm.bx.heap.new_object();
    vm.bx.heap.set_value_def(
        ds_obj,
        makepad_live_id::id!(status).into(),
        datasource_status,
    );

    // fav.*
    let fav_list = nil_callable(vm);
    let fav_toggle = nil_callable(vm);
    let fav_obj = vm.bx.heap.new_object();
    vm.bx
        .heap
        .set_value_def(fav_obj, makepad_live_id::id!(list).into(), fav_list);
    vm.bx
        .heap
        .set_value_def(fav_obj, makepad_live_id::id!(toggle).into(), fav_toggle);

    // settings.*
    let settings_load = nil_callable(vm);
    let settings_save = nil_callable(vm);
    let settings_obj = vm.bx.heap.new_object();
    vm.bx.heap.set_value_def(
        settings_obj,
        makepad_live_id::id!(load).into(),
        settings_load,
    );
    vm.bx.heap.set_value_def(
        settings_obj,
        makepad_live_id::id!(save).into(),
        settings_save,
    );

    // Stitch into `fb`.
    let fb = vm.bx.heap.new_object();
    vm.bx
        .heap
        .set_value_def(fb, makepad_live_id::id!(news).into(), news_obj.into());
    vm.bx
        .heap
        .set_value_def(fb, makepad_live_id::id!(quote).into(), quote_obj.into());
    vm.bx.heap.set_value_def(
        fb,
        makepad_live_id::id!(research).into(),
        research_obj.into(),
    );
    vm.bx
        .heap
        .set_value_def(fb, makepad_live_id::id!(stream).into(), stream_obj.into());
    vm.bx
        .heap
        .set_value_def(fb, makepad_live_id::id!(datasource).into(), ds_obj.into());
    vm.bx
        .heap
        .set_value_def(fb, makepad_live_id::id!(fav).into(), fav_obj.into());
    vm.bx.heap.set_value_def(
        fb,
        makepad_live_id::id!(settings).into(),
        settings_obj.into(),
    );

    vm.set_injected_global(makepad_live_id::id!(fb), fb.into());
}
