# `finance-brief/scripts` — Script Reference

> Eight Python scripts that cover the full lifecycle of `finance-brief` on the OctoSense desktop: registration, launch, remote-bridge (headless) verification, and shell-state diagnostics/cleanup. Run `python foo.py --help` on any of them for the full argument list.

> This is the long-form companion to [`README.md` §4](../README.md#4-脚本-scriptspy) in the project root. §4 is a 30-second scan; this file is for when you actually need to run something.

## Contents

| § | Script | One-liner |
|---|--------|-----------|
| 1 | [`clean-shell-pollution.py`](#1-clean-shell-pollutionpy) | Clean finance-brief's leftover pollution from the OctoSense shell (3 sites) |
| 2 | [`diagnose-shell-state.py`](#2-diagnose-shell-statepy) | Read-only: scan 5 paths, print JSON, never write |
| 3 | [`drive-test.py`](#3-drive-testpy) | Drive the launcher UI via remote bridge — click 5 tabs, verify label sequence |
| 4 | [`install-as-makepad-app.py`](#4-install-as-makepad-apppy) | **Recommended path**: build + write `~/.octosense/apps.json` |
| 5 | [`install-as-system-app.py`](#5-install-as-system-apppy) | Legacy Page-format installer (supports `--uninstall`) |
| 6 | [`run-octosense.py`](#6-run-octosensepy) | Run `install-as-makepad-app.py` + `cargo run -p octosense` |
| 7 | [`verify.py`](#7-verifypy) | Verify 5 quote tabs + refresh button labels via remote bridge |
| 8 | [`register-with-shell.py`](#8-register-with-shellpy) | Register finance-brief with OctoSense shell user catalog (no rebuild) |

See also: [`README.en.md` §4](../../README.en.md) in the project root for the 30-second overview.

## Common prerequisites

- **Python**: 3.8+ runs fine (stdlib only); the docstrings use `dict | None` annotations, so 3.10+ is what linters expect.
- **Working directory**: run from the repo root `finance-brief/`. The scripts use `Path(__file__).resolve().parent` to navigate, so you don't have to `cd scripts/` first.
- **`OCTOSENSE_HOME` env var**: optional. When set, user-side paths (catalog, cache) live under `$OCTOSENSE_HOME/.octosense/...` instead of `~/`. Honoured on Windows / macOS / Linux. Read by diagnostic + cleanup scripts.
- **Cross-platform**: every script depends only on stdlib (`argparse` / `subprocess` / `urllib.request` / `pathlib` / `shutil` / `json`). On Windows there is no need for Git Bash or WSL — plain `python foo.py` works.
- **OctoSense checkout**: `install-as-system-app.py` requires a sibling `OctoSense/` repository (it writes `system-apps.json` and `apps/finance-brief/`). `install-as-makepad-app.py` does not need it.

## Exit code conventions

| Code | Meaning | Raised from |
|------|---------|-------------|
| `0` | Success | All |
| `2` | argparse failure | All scripts using argparse |
| `7` | remote bridge unreachable (`URLError`) | `drive-test.py` / `verify.py` |
| `10` | Pollution detected | `diagnose-shell-state.py` / `clean-shell-pollution.py --diagnose-only` |
| `1` | Internal error / partial cleanup | `clean-shell-pollution.py` (post-clean still polluted) / `drive-test.py` (other exceptions) |
| `3` | makepad rev mismatch | `install-as-makepad-app.py` |
| `4` | binary missing / not executable | `install-as-makepad-app.py` |

Other codes: `install-as-makepad-app.py` propagates cargo's exit code verbatim; `run-octosense.py` propagates subprocess failures.

## `.sh` vs `.py`

Five legacy `.sh` files still live in this directory alongside their `.py` replacements:

| `.sh` (legacy) | `.py` (preferred) |
|----------------|-------------------|
| `install-as-makepad-app.sh` | `install-as-makepad-app.py` |
| `install-as-system-app.sh` | `install-as-system-app.py` |
| `run-octosense.sh` | `run-octosense.py` |
| `drive-test.sh` | `drive-test.py` |
| `verify.sh` | `verify.py` |

The `.sh` files are pre-2026-10-05 versions. They were kept (rather than deleted) to avoid breaking older callers, but the root `README.en.md §4` already declares them removed. Use the `.py` versions for new work:

- `.py` uses stdlib only, so it runs natively on Windows without Git Bash or WSL.
- `.sh` needs Git Bash or WSL on Windows and bash 4+ features.
- `.py` adds two new scripts (`clean-shell-pollution.py` / `diagnose-shell-state.py`) that have no `.sh` counterpart.

---

## 1. `clean-shell-pollution.py`

**Purpose:** diagnose and clean the three sites where finance-brief may have polluted the OctoSense shell. The pollution occurs when the legacy `install-as-system-app.py` is run without `--uninstall`.

**Prerequisites**:

- Python 3.10+ (uses `dict | None` annotations)
- Optional: `OCTOSENSE_HOME` env var (controls the user-side base path)

**Usage**:

```sh
python scripts/clean-shell-pollution.py [options]
```

**Arguments** (from `--help`):

- `--diagnose-only` (flag): print the diagnostic JSON and exit — no writes. Exits `10` if pollution is found, `0` if clean.
- `--dry-run` (flag): show what would be cleaned (paths + `status: "would-clean"`) without touching anything.
- `--user-cache-only` (flag, mutually exclusive with `--octosense-only`): clean only `~/.octosense/apps/.system/os.finance-brief/`.
- `--octosense-only` (flag, mutually exclusive with `--user-cache-only`): clean only `OctoSense/desktop/system-apps.json` and `OctoSense/apps/finance-brief/`.

**The three pollution sites**:

1. `OctoSense/desktop/system-apps.json` — the `apps` array contains a `"finance-brief"` string entry.
2. `OctoSense/apps/finance-brief/` — the entire directory (a partial bundle copy left by the legacy installer).
3. `~/.octosense/apps/.system/os.finance-brief/<hash>/` — the shell's cached pack from startup.

**Exit codes**:

- `0` — clean. Cleanup ran and the final diagnostic shows no pollution.
- `1` — partial. Cleanup ran but the final diagnostic still reports pollution (a path was locked / read-only / busy).
- `2` — argparse failure.
- `10` — `--diagnose-only` mode and pollution was found (no writes occurred).

**Examples**:

```sh
# Look but don't touch
python scripts/clean-shell-pollution.py --diagnose-only
echo $?     # 10 = polluted, 0 = clean

# Show what would happen
python scripts/clean-shell-pollution.py --dry-run

# Clean everything
python scripts/clean-shell-pollution.py

# Clean only the shell cache (leaves OctoSense repo untouched)
python scripts/clean-shell-pollution.py --user-cache-only

# Clean only OctoSense-side pollution (rare)
python scripts/clean-shell-pollution.py --octosense-only
```

**Typical errors**:

- `JSONDecodeError` (caught, becomes a warning): `system-apps.json` is corrupted. The script treats it as `None` and falls into the "missing (no-op)" branch for the catalog.
- `PermissionError` / `OSError` (propagates to exit `1`): `OctoSense/apps/finance-brief/` is locked or read-only. Stop the processes that reference it, then re-run.
- `FileNotFoundError`: never raised. This script does not require `apps/desktop/target/release/finance-brief(.exe)` to exist.

**Related scripts**:

- Pair with [`diagnose-shell-state.py`](#2-diagnose-shell-statepy): diagnose first to see the full picture, then choose `--user-cache-only` or `--octosense-only`.
- Alternative: [`install-as-system-app.py --uninstall`](#5-install-as-system-apppy) cleans the same three sites too, but its semantics are "undo an install". Prefer this script when the cached pack is the main problem.

---

## 2. `diagnose-shell-state.py`

**Purpose:** read-only diagnostic. Scans five paths relevant to finance-brief, prints a JSON report, and never touches anything.

**Prerequisites**:

- Python 3.10+ (uses `dict | None` annotations)
- Optional: `OCTOSENSE_HOME` env var (controls the user-side base path)

**Usage**:

```sh
python scripts/diagnose-shell-state.py
```

**Arguments:** none. This script does not call `argparse`; passing `--help` still runs the scan and returns `0` (clean) or `10` (polluted). To check cleanliness, look at the exit code only.

**Five scanned paths**:

| Key | Path | Check |
|-----|------|-------|
| `octosense_system_apps` | `OctoSense/desktop/system-apps.json` | Is `"finance-brief"` in the JSON array? |
| `octosense_apps_json` | `OctoSense/desktop/config/apps.json` | Is `"finance-brief"` in the JSON array? |
| `octosense_bundle` | `OctoSense/apps/finance-brief/` | Does the directory exist? |
| `user_cache` | `~/.octosense/apps/.system/os.finance-brief/` | Does the directory exist? If so, list hash subdirs. |
| `user_catalog` | `~/.octosense/apps.json` | Is `"finance_brief"` (underscore) in the JSON array? |

**Exit codes**:

- `0` — clean. No pollution source exists (all of `octosense_system_apps.found`, `octosense_bundle.exists`, `user_cache.exists` are false).
- `10` — pollution found.
- `1` — an uncaught exception during scanning (e.g. JSON unreadable). Writes `{"error": "..."}` to stderr.

**Examples**:

```sh
# Verify after install
python scripts/install-as-makepad-app.py && python scripts/diagnose-shell-state.py
echo $?     # 0 = clean, 10 = polluted

# Use with shell automation
if python scripts/diagnose-shell-state.py > /dev/null; then
    echo "shell state clean"
else
    case $? in
        10) python scripts/clean-shell-pollution.py ;;
        *)  echo "diagnostic error" ;;
    esac
fi
```

**Typical errors**:

- Path missing → JSON field `exists: false`, doesn't count as pollution.
- JSON corrupted → field `exists: true, error: "<details>"`. Doesn't count as pollution, doesn't change the exit code (only `octosense_system_apps.found`, `octosense_bundle.exists`, `user_cache.exists` are inspected).
- Note: `octosense_system_apps` expects a JSON **array** at the top level. If the top level is a dict, the report shows `schema: "dict"` and `found: false`.

**Related scripts**:

- The "safe preamble" to [`clean-shell-pollution.py`](#1-clean-shell-pollutionpy): run diagnose first, then decide what to clean.
- Scans more paths than `install-as-system-app.py --uninstall` (adds `octosense_apps_json` and `user_catalog`).

---

## 3. `drive-test.py`

**Purpose:** drive the launcher UI via the card-host remote bridge. Clicks through five launcher tabs in sequence, prints the visible label sequence after each step, and validates the widget tree + on_click routing.

**Prerequisites**:

- finance-brief is built: `cd apps/desktop && cargo build --release` (or run `cargo run --release` once).
- finance-brief was launched with `--remote=0` (binds an ephemeral port — read the port from the log).
- The remote bridge accepts `/snap` and `/m?k=click&...` on that port.

**Usage**:

```sh
python scripts/drive-test.py --port <port>
python scripts/drive-test.py <port>           # positional form
```

**Arguments** (from `--help`):

- `--port PORT` (int, mutually exclusive with positional `port_pos`; one is required): remote bridge port.
- `port_pos` (positional, int, optional): same as `--port`. The flag form wins.

**Six-step flow**:

1. `snap` — print the initial screen's labels.
2. `click_text("加密")` → snap.
3. `click_text("BTC")` (first row) → snap (enters detail).
4. `click_text("收藏 / 取消")` → snap (favourite).
5. `click_text("返回列表")` (no failure guard) → `click_text("收藏")` → snap.
6. `click_text("要闻")` → snap (live news).

If `rect(name)` cannot find a matching widget, the step prints `[no widget] <name>` and skips to the next — it does not abort.

**Exit codes**:

- `0` — completed all steps.
- `2` — argparse failure (no port given).
- `7` — `urllib.error.URLError`; bridge unreachable (wrong port, bridge not started, process crashed).
- `1` — other exceptions (written to stderr).

**Example**:

```sh
# Start finance-brief in the background with the remote bridge
nohup ./apps/desktop/target/release/finance-brief --remote=0 > /tmp/fb.log 2>&1 &
disown
sleep 5
PORT=$(grep -oE 'listening on 127.0.0.1:[0-9]+' /tmp/fb.log | grep -oE '[0-9]+$' | head -1)

# Auto-click through launcher 5 tabs
python scripts/drive-test.py --port "$PORT"
```

**Typical errors**:

- `URLError` (exit 7): bridge port is not listening / the process crashed / the log has no `listening on 127.0.0.1:PORT`. Sanity-check with `curl http://127.0.0.1:$PORT/status` first.
- `[no widget] 加密` printed: a widget label has changed. The script locates widgets by hard-coded strings — update the source to match.
- A step stalls and never comes back: the click did not update the UI. Check whether the splash `on_click` is wired up.

**Related scripts**:

- Sibling: [`verify.py`](#7-verifypy) — verifies the **5 quote tabs** by hard-coded coordinates; this script verifies the **5 launcher tabs** by widget label. Complementary, not overlapping.

---

## 4. `install-as-makepad-app.py`

**Purpose:** **the recommended install path**. Build finance-brief, then write the binary into `~/.octosense/apps.json` (the user-level catalog; does not touch the dep repo). The shell launcher shows a "财经简报" tile. Includes makepad rev alignment check, UTF-8 writing, and cross-platform binary suffix (`.exe` / nothing).

**Prerequisites**:

- Python 3.10+ (uses PEP 604 syntax).
- `cargo` on `PATH` (for `cargo build --release`).
- Optional: a sibling `../makepad` git repo (for the rev alignment check). The check is skipped when the repo is absent.
- Optional: `OCTOSENSE_HOME` env var (controls where `user_apps` is written).

**Usage**:

```sh
python scripts/install-as-makepad-app.py [--uninstall | --dry-run]
```

**Arguments** (from `--help`):

- `--uninstall` (flag, mutually exclusive with `--dry-run`): remove the `finance_brief` entry from `~/.octosense/apps.json`. Skips the build and the rev check.

- `--dry-run` (flag, mutually exclusive with `--uninstall`): echo what would happen (print the `cargo build` command + the JSON line that would be written). No build, no writes.

**Four-step flow**:

1. `cargo build --release --manifest-path apps/desktop/Cargo.toml` (skipped on `--uninstall` / `--dry-run`).
2. Verify the produced binary exists and is executable (Windows: existence only; POSIX: `-x`). Failure → exit `4`.
3. Verify `../makepad` HEAD matches the `makepad-widgets` rev pinned in `apps/desktop/Cargo.toml` (only runs when `../makepad` exists). Failure → exit `3`.
4. Write `~/.octosense/apps.json` (idempotent): drop the old `finance_brief` entry, append the new one (unless `--uninstall`).

**Exit codes**:

- `0` — success.
- `2` — argparse failure.
- `3` — makepad rev mismatch (see `OctoSense/AGENTS.md §2 "One revision per external dependency"`).
- `4` — binary missing / not executable.
- Cargo's exit code — propagated verbatim (`subprocess.run(..., check=True)` → `SystemExit(e.returncode)`).

**Examples**:

```sh
# Recommended path: register finance-brief with the OctoSense launcher
python scripts/install-as-makepad-app.py
cd ../OctoSense && cargo run --release -p octosense
# → "财经简报" tile in the launcher spawns finance-brief as its own window

# See what would happen
python scripts/install-as-makepad-app.py --dry-run

# Unregister
python scripts/install-as-makepad-app.py --uninstall

# Full v8 flow (summary of README §3.2)
python scripts/install-as-makepad-app.py \
    && python scripts/diagnose-shell-state.py \
    && python scripts/drive-test.py --port <PORT>
```

**Typical errors**:

- `subprocess.CalledProcessError` (cargo failed): cargo's usual exit codes 1 / 101. Reproduce with `cargo build --release --manifest-path apps/desktop/Cargo.toml` to localise.
- Exit `3` (rev mismatch): check out `../makepad` at the commit pinned by `apps/desktop/Cargo.toml`, then re-run install.
- Exit `4` (missing binary): build produced nothing (disk full / compile error). Inspect cargo output.
- `FileNotFoundError`: `apps/desktop/Cargo.toml` is missing (wrong directory layout).

**Related scripts**:

- Do **not** pair with [`run-octosense.py`](#6-run-octosensepy): that script calls `install-as-makepad-app.py` (default rebuild is heavy — not the right tool if you only want to refresh the catalog). Run `cargo run -p octosense` directly (from the `../OctoSense/` repo) — the shell reads `~/.octosense/apps.json` at startup. If you only want to refresh the catalog without rebuilding, use [`register-with-shell.py`](#8-register-with-shellpy).
- Cleanup of a bad install: see [`clean-shell-pollution.py`](#1-clean-shell-pollutionpy) / [`install-as-system-app.py --uninstall`](#5-install-as-system-apppy).

---

## 5. `install-as-system-app.py`

**Purpose:** **legacy**. Copies the finance-brief bundle into `OctoSense/apps/finance-brief/bundle/` (writes into the dep repo) and was designed for the old Page-format bundle (`*.card` files, `workflow.octoscript`, `kit/`, `assets/`). The current repo uses the Path-1 bundle (`bundle/screens/*.octoscript`), so **running this script installs an empty bundle** and prints a WARNING telling you to use `install-as-makepad-app.py` instead.

**Prerequisites**:

- Python 3.10+.
- A sibling `OctoSense/` repository containing `Cargo.toml` (otherwise exit `1`).
- Legacy files like `bundle/launcher.card`, `bundle/manifest.json`, `bundle/listing.json` (Path-1 has none of these, so the script just warns and skips).

**Usage**:

```sh
python scripts/install-as-system-app.py [--dry-run | --uninstall]
```

**Arguments** (from `--help`):

- `--dry-run` (flag): print every file operation (JSON diff / copy / mkdir), no writes.
- `--uninstall` (flag, mutually exclusive with `--dry-run`): remove `"finance-brief"` from `system-apps.json`, delete the `apps/finance-brief/` tree, and clear `~/.octosense/apps/.system/os.finance-brief/`.

**Five transformations**:

1. `bundle/listing.json` → rewrite `icon` to `"icon.svg"` (the system-app packer does not honour the `assets/` subdir).
2. `bundle/manifest.json` → rewrite `id` to `"os." + <short>` (system packs use the `os.` prefix), clear `integrity.bundle_blake3` (build stamps it).
3. `OctoSense/desktop/system-apps.json` → append `"finance-brief"` to the `apps` array.
4. Copy `*.card` / `launcher.card → page.card` / `schema/` / `workflow.octoscript` / `kit/` / `assets/icon.svg` / `screenshots/*.png` into `OctoSense/apps/finance-brief/bundle/`.
5. Write `OctoSense/apps/finance-brief/bundle/page.data.json` = `{}`.

Every copy step is defensive: a missing source file produces a warning and the step is skipped, never a failure.

**Exit codes**:

- `0` — success (also returned by `--dry-run` once printing is done and by `--uninstall` after all cleanup).
- `1` — no `OctoSense/Cargo.toml` sibling.
- `2` — argparse failure.

**Examples**:

```sh
# Strongly recommended: see what would happen first
python scripts/install-as-system-app.py --dry-run

# Actual install (for Path-1 bundle, warns and exits recommending install-as-makepad-app.py)
python scripts/install-as-system-app.py

# Clean up after a bad legacy install
python scripts/install-as-system-app.py --uninstall
```

**Typical errors**:

- Exit `1`: no sibling `OctoSense/` repository. Check the directory layout.
- After install, the shell logs `page.card 系统找不到指定文件 (os error:2)`: the bundle has no `launcher.card` (Path-1 doesn't), so `page.card` was never created. **Correct path**: use `install-as-makepad-app.py` + `~/.octosense/apps.json`; do not use this script.
- `--uninstall` returns `0` but `system-apps.json` is unchanged: nothing was installed via this script (only the makepad path was used). Normal no-op.

**Related scripts**:

- Replacement: [`install-as-makepad-app.py`](#4-install-as-makepad-apppy) is the only correct install path for the current Path-1 bundle.
- Cleanup: `--uninstall` is a subset of [`clean-shell-pollution.py`](#1-clean-shell-pollutionpy). `clean-shell-pollution.py` scans more sites; `install-as-system-app.py --uninstall` has cleaner "undo an install" semantics.

---

## 6. `run-octosense.py`

**Purpose:** one-shot install + run. Internally calls `install-as-makepad-app.py` + `cargo run --release -p octosense`.

**Prerequisites**:

- Python 3.10+.
- `cargo` on `PATH` (for `cargo run`).
- A sibling `OctoSense/` repository.

**Usage**:

```sh
python scripts/run-octosense.py
```

**Arguments:** none. The script defines `argparse.ArgumentParser` but registers no flags (`-h, --help` is automatic).

**Behaviour**:

1. `subprocess.run([python, install-as-makepad-app.py], check=True)`.
2. `subprocess.run(["cargo", "run", "--release", "-p", "octosense"], check=True, cwd=WS / "OctoSense")`.

**Exit codes**:

- `0` — success.
- Non-zero (cargo's or subprocess'): propagated verbatim.

**Example**:

```sh
# One-shot install + run (Path-1 bundle)
python scripts/run-octosense.py
```

**Typical errors**:

- cargo compile failure: inspect cargo output.
- `install-as-makepad-app.py` errors (rev mismatch / binary missing): exit code is propagated.

**Related scripts**:

- **Recommended for the current Path-1 bundle** (`README §3.2` explicitly recommends `install-as-makepad-app.py` + `cargo run -p octosense`; `run-octosense.py` is a shortcut for exactly those two steps).
- Equivalent manual flow:
  ```sh
  python scripts/install-as-makepad-app.py
  cd ../OctoSense && cargo run --release -p octosense
  ```

---

## 7. `verify.py`

**Purpose:** validate, via remote bridge, that the five quote tabs (`要闻` / `A股` / `美股` / `加密` / `外汇`) + the `收藏` tab + the `刷新` button all render their expected labels. For each entry the script clicks a hard-coded coordinate, fetches `/snap`, and prints the visible `Label` widget text.

**Prerequisites**:

- finance-brief built, running with `--remote=0` (binds an ephemeral port).
- The quote tab uses the `quote_list` screen layout (`bundle/screens/quote_list.octoscript` in this repo's Path-1 bundle).

**Usage**:

```sh
python scripts/verify.py <port>
python scripts/verify.py --port <port>
```

**Arguments** (from `--help`):

- `port` (positional, int, optional): server port (positional form).
- `--port PORT_FLAG` (flag, int, optional): server port (flag form, mutually exclusive with positional). Flag wins.

**Hard-coded coordinates (for the current `quote_list` layout)**:

| Tab | x |
|-----|---|
| 要闻 | `41` |
| A股 | `101` |
| 美股 | `161` |
| 加密 | `223` |
| 外汇 | `287` |
| 收藏 | `348` |
| Refresh button | `(367, 87)` |

Sleep 1s after every click; sleep 6s after the refresh button.

**Exit codes**:

- `0` — completed all steps.
- `2` — `argparse.error("the following arguments are required: port")` (no port given).
- `7` — `urllib.error.URLError`; bridge unreachable.

**Example**:

```sh
PORT=$(grep -oE 'listening on 127.0.0.1:[0-9]+' /tmp/fb.log | grep -oE '[0-9]+$' | head -1)
python scripts/verify.py --port "$PORT"

# Or positional
python scripts/verify.py "$PORT"
```

**Typical errors**:

- `URLError` (exit 7): bridge not listening. Sanity-check with `curl http://127.0.0.1:$PORT/status` first.
- Wrong coordinate / no reaction: the layout has changed. After modifying `apps/desktop/src/app.rs` or `bundle/screens/quote_list.octoscript`, update the coordinates here.
- No labels output: the `/snap` JSON has no `ty == "Label"` field, or the layout is all 0×0. The known v8 Splash-zero-rect bug has been fixed (see `apps/desktop/src/app.rs:60` `View{height: Fit, {ui}}` fix).

**Related scripts**:

- Sibling: [`drive-test.py`](#3-drive-testpy) walks launcher 5-tab routing by widget label; this script walks the quote-tab label sequence by coordinate. Complementary.
- Root-cause notes: `.todo-finance-brief-splash-zero-rect-2026-10-05.md` (v8 Splash-subtree rect 0×0 fix).

---

## 8. `register-with-shell.py`

**Purpose**: light-weight register-only script. Writes `~/.octosense/apps.json`, does NOT rebuild. Differs from [`install-as-makepad-app.py`](#4-install-as-makepad-apppy) (which builds by default).

**Usage**:

```bash
python scripts/register-with-shell.py             # check binary + write entry
python scripts/register-with-shell.py --dry-run   # echo entry, don't write
python scripts/register-with-shell.py --uninstall # remove entry
python scripts/register-with-shell.py --help
```

**Behavior**:

- binary missing → exit code **4** (aligned with [`install-as-makepad-app.py`](#4-install-as-makepad-apppy))
- catalog missing → parent dir auto-created
- catalog not a JSON array → exit code `1`, refuse to overwrite
- `--uninstall` mutex with `--dry-run`; no entry → no-op
- cross-platform: Windows / macOS / Linux / WSL; `OCTOSENSE_HOME` env wins over `Path.home()`; Windows auto-adds `.exe` suffix

**Why this script exists**: user's brief said *"apps.json should add finance-brief entry here, but cannot write directly to dep project"*. Writing `OctoSense/desktop/config/apps.json` directly modifies a dep-repo file; [`install-as-makepad-app.py`](#4-install-as-makepad-apppy) rebuilds by default which is heavy. `register-with-shell.py` is the middle ground: writes user-level catalog, no rebuild.

---

## 9. Cross-platform / Windows notes

All `scripts/*.py` use stdlib only and `pathlib` for portable paths:

| Scenario | Approach |
| --- | --- |
| `/tmp/foo` unavailable (Windows Python) | use `tempfile.gettempdir()` or `Path(os.environ.get('TEMP') or Path.home()) / 'foo'` |
| Binary suffix | `sys.platform.startswith('win')` → `.exe` |
| User home | `OCTOSENSE_HOME` env wins over `Path.home()` |
| PowerShell vs Git Bash | plain `python scripts/foo.py` works in both; no `bash foo.sh`, no `powershell foo.ps1` |

---

## Appendix A: typical v8 end-to-end flow

```sh
cd finance-brief/

# 1. Register (recommended path)
python scripts/install-as-makepad-app.py

# 2. Check state (expect exit 0)
python scripts/diagnose-shell-state.py

# 3. Start finance-brief in the background with the remote bridge
nohup ./apps/desktop/target/release/finance-brief --remote=0 > /tmp/fb.log 2>&1 &
disown
sleep 5
PORT=$(grep -oE 'listening on 127.0.0.1:[0-9]+' /tmp/fb.log | grep -oE '[0-9]+$' | head -1)

# 4. Headless verification
python scripts/drive-test.py --port "$PORT"     # launcher 5-tab routing
python scripts/verify.py --port "$PORT"         # quote 5-tab + refresh labels

# 5. (Optional) Launch shell for the GUI
cd ../OctoSense && cargo run --release -p octosense
```

## Appendix B: pollution recovery (shell reports `page.card 系统找不到指定文件`)

```sh
# 1. See what's polluted
python scripts/diagnose-shell-state.py
# 2. Clean the shell cache + system-apps.json + apps/finance-brief/
python scripts/clean-shell-pollution.py
# 3. Switch to the correct path
python scripts/install-as-makepad-app.py
# 4. Restart the shell
cd ../OctoSense && cargo run --release -p octosense
```

See `README.en.md` §3.2 "If the shell still complains about page.card".