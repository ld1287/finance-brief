# finance-brief

## OctoSense app — single Octoscript path

finance-brief is an OctoSense app rendered **entirely from `.octoscript`
files** through the octoscript-makepad pipeline. There is one path; the
splash DSL entry (`bundle/main.splash`) and the 12 `.card` glance tiles
were removed in the `feat/one-octoscript` migration on 2026-10-04.

| | |
|---|---|
| **UI** | 12 `.octoscript` screens under `bundle/screens/` |
| **Entry** | `apps/finance-brief/src/lib.rs` (mirror of `Octoscript-Makepad/apps/flutter-samples/src/lib.rs`) |
| **Data** | 9 Rust adapters under `native/src/adapters/`, exposed to the DSL as `mod.fb.<cap>` via `apps/finance-brief/src/datasources.rs` |
| **Workflow** | `bundle/workflow.octoscript` (7 functions, already existed; now actually wired) |
| **Pipeline** | `octoscript-render` (VM → UiNode) → `octoscript-makepad::to_makepad_ui` → `Splash.view` → native makepad widgets |
| **Hot reload** | `/data/local/tmp/finance_brief.octoscript` on device (mirrors flutter-samples' DEVICE_PATH) |

## 1. Project

An OctoSense controlled-script app (`bundle/` is the only submit unit;
`apps/finance-brief/` is the desktop / phone packaging, mirroring
`Octoscript-Makepad/apps/flutter-samples/`). Aggregates public, keyless
APIs for finance news and quotes across five tabs (要闻 / A股 / 美股 / 加密 /
外汇), with favorites and offline sample-data fallback. All data is for
demonstration only and is **not investment advice**.

## 2. Data sources

Five sources, six capabilities. TTLs are defined in `native/capabilities.toml`.

| name | URL | content | TTL |
|------|-----|---------|-----|
| Sina 要闻 | `feed.mix.sina.com.cn` | Chinese finance RSS (GBK) | `news.refresh` 300s; `news.read` 600s |
| Tencent 行情 | `qt.gtimg.cn` | A股 / 港股 snapshots | 5s |
| Stooq 美股 | `stooq.com` | US equities realtime CSV | 5s |
| Hyperliquid 加密 | `api.hyperliquid.xyz` | crypto (BTC/ETH/SOL, `allMids`) | 5s |
| Frankfurter 外汇 | `api.frankfurter.dev` | ECB daily fix rates | 3600s |

## 3. Run

The single command sequence:

```sh
# 1) Native adapter unit / smoke tests
cargo test --manifest-path native/Cargo.toml      # 65/65 PASS

# 2) Run the Octoscript-driven app (desktop, mirrors flutter-samples)
cd apps/finance-brief && cargo run --release

# Outputs:
#   finance-brief MOUNT route=launcher src_len=NNNNN built=true
# = the 12 screens evaluate through octoscript-makepad and mount on
#   Splash.view as native widgets.

# 3) Phone build (mirrors flutter-samples)
cargo makepad android run -p finance-brief --release
```

`$OCTO` points at `OctoScript-App-Design-Flow/tools/octo`. This repository
no longer relies on it for the interactive UI.

## 4. Repository layout

```
finance-brief/
├── apps/
│   └── finance-brief/                      # Rust app entry (mirror of flutter-samples)
│       ├── Cargo.toml
│       └── src/{main.rs, lib.rs, datasources.rs}
├── bundle/                                 # Submit unit (mirrors OctoSense app structure)
│   ├── screens/                            # 14 .octoscript files (kit + 12 screens + index)
│   ├── workflow.octoscript                 # 7 workflow functions (already existed)
│   ├── capabilities.toml                   # capability whitelist (per capabilities.toml)
│   ├── schema/                             # 17 JSON schemas
│   ├── kit/                                # 144 palette / axis token files
│   ├── assets/  listing.json  screenshots/
│   └── manifest.json                       # app metadata
├── native/                                 # Rust adapters (5 sources + cache + 9 modules)
│   ├── src/adapters/
│   └── capabilities.toml
├── scripts/                                # boot / install scripts
├── docs/                                   # platform layered docs
├── .todo-*.md                              # gitignored local coordination docs
└── README.md
```

## 5. Twelve launcher screens

12 screens per `bundle/workflow.octoscript` mapping. Each screen is a
single `.octoscript` file under `bundle/screens/`. The router lives in
`bundle/screens/_index.octoscript`.

| # | screen | file |
|---|--------|------|
| 1 | Launcher (home) | `screens/launcher.octoscript` |
| 2 | 新闻简报 list | `screens/news_list.octoscript` |
| 3 | 新闻详情 | `screens/news_detail.octoscript` |
| 4 | 研究卡 list | `screens/research_list.octoscript` |
| 5 | 研究卡详情 | `screens/research_detail.octoscript` |
| 6 | K线看盘 (`mod.plot.CandlestickChart`) | `screens/kline.octoscript` |
| 7 | 行情 list (4 tabs: A / US / crypto / FX) | `screens/quote_list.octoscript` |
| 8 | 收藏 | `screens/favorites.octoscript` |
| 9 | 设置 | `screens/settings.octoscript` |
| 10 | 免责声明 | `screens/disclaimer.octoscript` |
| 11 | 事件流 | `screens/event_stream.octoscript` |
| 12 | 数据源状态 | `screens/datasource_status.octoscript` |

Per `docs/R-4-l0-cards.md §5`: 9 screens are pure L0 declarative;
3 (K-line, event_stream, datasource_status) admit L1 arithmetic.

## 6. Test

```sh
cargo test --manifest-path native/Cargo.toml
…
test result: ok. 62 passed; 0 failed   # unit
test result: ok.  3 passed; 0 failed   # smoke
                             ─────
                             65 / 65
```

App-level smoke: launch `cargo run -p finance-brief` and verify the
launcher renders 11 tiles. Tapping a tile routes via `_index.octoscript`
into the corresponding screen (verified by visual QA; no automated
screenshot harness in this MVP).

## 7. Known issues

| item | status |
|------|--------|
| 9 native adapters not yet wired to `apps/finance-brief/src/datasources.rs` | `host.fetch` shims log a stub; data still flows through `native/capabilities.toml` for an MVP path |
| `kline.octoscript` `chart_candlestick` widget registration | depends on `octoscript_widgets::script_mod(vm)` + `makepad_plot::script_mod(vm)` being called on the app VM; not in `datasources.rs` yet |
| `bundle/listing.json` `publisher.privacy_policy_url` | current is the repo GitHub URL; needs an independent privacy doc before public submission |
| `screens/*.octoscript` not yet walked through `octoscript_render::build` on a real device | verify on Android / desktop before tagging v0.4.0 |

## 8. Git info

- Branch: `feat/one-octoscript` (off `b19656c`)
- Tracking: `origin/main`
- HEAD on main: `b19656c` "review, re-arch ,update docs"
- Migration baseline: 2026-10-04

## 9. License

Apache-2.0 (see `LICENSE`).