use finance_brief_scaffold::*;

#[test]
fn smoke_version() {
    assert!(!version().is_empty());
}

#[test]
fn smoke_synth_candles_100() {
    assert_eq!(synth::synth_candles(100).len(), 100);
}

#[test]
fn smoke_parse_sina_empty_ok() {
    // parse 占位返回 Ok 或 Err 都是 OK 的 — 只验证调用与编译
    let _ = parse::parse_sina_news("");
}

