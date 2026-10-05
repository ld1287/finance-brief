# finance-brief

## OctoSense app — single Octoscript path

finance-brief is an OctoSense app rendered **entirely from `.octoscript`
files** through the octoscript-makepad pipeline. There is one path; the
splash DSL entry (`bundle/main.splash`) and the 12 `.card` glance tiles
were removed in the `feat/one-octoscript` migration on 2026-10-04.

| | |
|---|---|
| **UI** | 12 `.octoscript` screens under `bundle/screens/` |
| **Entry** | `apps/desktop/src/lib.rs` (mirror of `Octoscript-Makepad/apps/flutter-samples/src/lib.rs`) |
| **Data** | 9 Rust adapters under `native/src/adapters/`, exposed to the DSL as `mod.fb.<cap>` via `apps/desktop/src/datasources.rs` |
| **Workflow** | `bundle/workflow.octoscript` (7 functions, already existed; now actually wired) |
| **Pipeline** | `octoscript-render` (VM → UiNode) → `octoscript-makepad::to_makepad_ui` → `Splash.view` → native makepad widgets |
| **Hot reload** | `/data/local/tmp/finance_brief.octoscript` on device (mirrors flutter-samples' DEVICE_PATH) |

## 1. Project

An OctoSense controlled-script app (`bundle/` is the only submit unit;
`apps/desktop/` is the desktop / phone packaging, mirroring
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

Four modes. All CLI lives in `scripts/*.py` (ported from `.sh` on 2026-10-05; stdlib only; cross-platform on Windows / macOS / Linux).

### 3.1 Standalone (desktop window)

```sh
# 1) Native adapter unit / smoke tests
cargo test --manifest-path native/Cargo.toml      # 65/65 PASS

# 2) Desktop window (mirror of flutter-samples)
cd apps/desktop && cargo run --release
#   Output: finance-brief MOUNT route=launcher src_len=NNNNN built=true
# = the 12 screens evaluate through octoscript-makepad and mount on
#   Splash.view as native widgets.
```

### 3.2 Register with the OctoSense shell launcher

```sh
# User-level registration (does NOT touch the dep repo; writes
# ~/.octosense/apps.json — see OctoSense/AGENTS.md §"Developer programs and
# the catalog")
python scripts/install-as-makepad-app.py

# The shell reads ~/.octosense/apps.json before config/apps.json
cd ../OctoSense && cargo run --release -p octosense
# → the "财经简报" tile in the shell launcher spawns finance-brief.exe
```

`install-as-makepad-app.py` does: ① `cargo build --release` ② verify the local `../makepad` HEAD matches the rev pinned in `apps/desktop/Cargo.toml` (OctoSense/AGENTS.md §2 "One revision per external dependency") ③ write the `~/.octosense/apps.json` entry (label=财经简报, executable=absolute path). Supports `--dry-run` / `--uninstall` / `--help`.

> **v6 fix (2026-10-05)**: a previous session accidentally ran `install-as-system-app.py` (the legacy Page-format script), which registered `finance-brief` in `OctoSense/desktop/system-apps.json` and copied the bundle into `OctoSense/apps/finance-brief/bundle/`. Because the source bundle has no `launcher.card`, no `page.card` was ever created; the shell then loaded `system-apps.json` as a system-app catalog and reported `page.card 系统找不到指定文件 (os error:2)`.
>
> **Correct path**: `finance-brief` is a developer program, not a system app — it must go through the `executable=` field of `~/.octosense/apps.json`. `install-as-makepad-app.py` already implements that path.
>
> **If the shell still reports `page.card`**:
> 1. `python scripts/install-as-system-app.py --uninstall` (added in v6) — clears the OctoSense-side pollution
> 2. Delete `~/.octosense/apps/.system/os.finance-brief/` if present — clears the shell-side cache
> 3. Re-run `python scripts/install-as-makepad-app.py` — rewrites the user catalog

> **Why not write `OctoSense/desktop/config/apps.json` directly?**  
> `config/apps.json` is an `OctoSense`-owned file; finance-brief is not its owner. Modifying a dep-repo file pulls in a sync burden (finance-brief bumps → OctoSense must follow → PR flow). Instead we write the registration into `~/.octosense/apps.json` — a user-level catalog the shell looks at before `config/apps.json`. `register-with-shell.py` is that entry point; `install-as-makepad-app.py` is its build+register one-step sibling.

### 3.3 Headless verification (remote bridge)

No window needed; verify widget tree / pixels remotely:

```sh
# Start finance-brief in the background with the remote bridge attached
nohup ./apps/desktop/target/release/finance-brief.exe --remote=0 > /tmp/fb.log 2>&1 &
disown
sleep 5

# Extract the port (--remote=0 means makepad picks an ephemeral port)
PORT=$(grep -oE 'listening on 127.0.0.1:[0-9]+' /tmp/fb.log | grep -oE '[0-9]+$' | head -1)

curl http://127.0.0.1:$PORT/status     # window state + size
curl http://127.0.0.1:$PORT/snap       # widget tree
curl http://127.0.0.1:$PORT/g          # grab PNG (capture_kind=backend_png)
```

Or use `python scripts/drive-test.py <port>` to drive the 5 launcher tab clicks automatically, or `python scripts/verify.py <port>` to verify the 5 quote tabs + refresh button.

### 3.4 Phone

```sh
cargo makepad android run -p finance-brief --release
```

`$OCTO` points at `OctoScript-App-Design-Flow/tools/octo`. This repository no longer relies on it for the interactive UI.

## 4. Scripts (`scripts/*.py`)

All scripts were ported from `.sh` on 2026-10-05. stdlib only (`argparse` / `subprocess` / `urllib.request` / `pathlib` / `shutil`); runs on Windows without Git Bash / WSL — `python foo.py` is enough. Every script supports `--help`; bad args → exit 2; bridge unreachable → exit 7.

| script | purpose |
|--------|---------|
| `install-as-makepad-app.py` | **Current recommendation.** Builds + registers finance-brief with the OctoSense shell user-level catalog (`~/.octosense/apps.json`). Includes makepad-rev alignment check + UTF-8 + Python-bool fixes (Windows + Chinese-label safe). |
| `register-with-shell.py` | Register finance-brief with the OctoSense shell user-level catalog (`~/.octosense/apps.json`), without rebuilding. Documented in [scripts/README.en.md](scripts/README.en.md). |
| `install-as-system-app.py` | **Legacy.** Copies the bundle into `OctoSense/apps/finance-brief/bundle/` (writes to the dep repo) — designed for the old Page-format bundle (`*.card` files). Now defensive: missing source files print a warning and are skipped instead of crashing. Under the current Path-1 bundle this installs an empty bundle and prints a WARNING pointing at `install-as-makepad-app.py`. (Supports `--uninstall`, added in v6.) |
| `run-octosense.py` | Calls `install-as-makepad-app.py` then `cargo run --release -p octosense`. Under the current Path-1 bundle, prefer `cargo run -p octosense` directly after running `install-as-makepad-app.py` once. |
| `drive-test.py <port>` | Drives the 5 launcher tabs via the remote bridge and prints the screen label sequence. |
| `verify.py <port>` | Drives the 5 quote tabs + refresh button via the remote bridge (hardcoded coordinates for the `quote_list` layout). |

Old `.sh` files were removed.

## 5. Repository layout

```
finance-brief/
├── apps/
│   └── desktop/                            # Rust app entry (mirror of flutter-samples)
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

## 6. Twelve launcher screens

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

## 7. Test

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

## 8. Known issues

| item | status |
|------|--------|
| 9 native adapters not yet wired to `apps/desktop/src/datasources.rs` | `host.fetch` shims log a stub; data still flows through `native/capabilities.toml` for an MVP path |
| `kline.octoscript` `chart_candlestick` widget registration | depends on `octoscript_widgets::script_mod(vm)` + `makepad_plot::script_mod(vm)` being called on the app VM; not in `datasources.rs` yet |
| `bundle/listing.json` `publisher.privacy_policy_url` | current is the repo GitHub URL; needs an independent privacy doc before public submission |
| `screens/*.octoscript` not yet walked through `octoscript_render::build` on a real device | verify on Android / desktop before tagging v0.4.0 |
| Splash subtree is 0×0 in headless `/snap?all=1` | **Fixed (v8, 2026-10-05)**. Root cause: `apps/desktop/src/app.rs:60`'s `View{height:Fit, {ui}}` wrap + `view.walk = host.walk` walk freeze + missing `WindowGeomChange` remount; flutter-samples uses the same Fit wrap but its kit content uses plain View+px heights, unlike finance-brief's `fb_page` ScrollYView+fillh:1. See `.todo-finance-brief-splash-zero-rect-2026-10-05.md` §3 for the 5 changes. Verified: `/snap?all=1` Splash r=[0,29,440,997], 11 tiles r=[24,167,192,96]/[224,167,192,96]/..., `/g` PNG captures the full launcher UI |
| `install-as-system-app.py` is a legacy Page-format script; ineffective on the Path-1 bundle | Do not run it. If it was already run and polluted the OctoSense state, clean up with `python scripts/install-as-system-app.py --uninstall` (added in v6). |

## 9. Git info

- Branch: `feat/one-octoscript` (off `b19656c`)
- Tracking: `origin/main`
- HEAD on main: `b19656c` "review, re-arch ,update docs"
- Migration baseline: 2026-10-04

## 10. License

Apache-2.0 (see `LICENSE`).