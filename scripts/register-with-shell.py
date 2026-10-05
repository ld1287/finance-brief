#!/usr/bin/env python3
"""register-with-shell.py — register finance-brief with OctoSense shell (no rebuild).

Per OctoSense/desktop/README.md §"Developer programs and the catalog", the
shell's catalog lookup is:
  1. --apps <file>          (CLI flag, wins)
  2. ~/.octosense/apps.json (user catalog — this script writes here)
  3. config/apps.json       (project default)

This script touches the project file at none — only the user-level catalog.
Differs from install-as-makepad-app.py:
  - does NOT rebuild the binary; call install-as-makepad-app.py for that.
  - smaller surface: just upserts the catalog entry.

Usage:
  python scripts/register-with-shell.py             # check binary + write entry
  python scripts/register-with-shell.py --dry-run   # echo entry, don't write
  python scripts/register-with-shell.py --uninstall # remove the entry
  python scripts/register-with-shell.py --help
"""
import argparse
import json
import os
import sys
from pathlib import Path

from _catalog_constants import ENTRY_ID, LABEL, POLICY


def _user_base() -> Path:
    """OCTOSENSE_HOME (if set) wins over $HOME; works on Win/macOS/Linux/WSL."""
    return Path(os.environ.get("OCTOSENSE_HOME") or Path.home())


def _exe_path(app_root: Path) -> Path:
    is_windows = sys.platform.startswith("win") or os.name == "nt"
    base = app_root / "apps" / "desktop" / "target" / "release" / "finance-brief"
    return base.with_suffix(".exe") if is_windows else base


def _load_catalog(path: Path) -> list:
    if not path.is_file():
        return []
    try:
        arr = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as e:
        print(f"unreadable {path}: {e}", file=sys.stderr)
        raise SystemExit(1)
    if not isinstance(arr, list):
        print(f"{path} is not a JSON array; refusing to corrupt it", file=sys.stderr)
        raise SystemExit(1)
    return arr


def main() -> int:
    here = Path(__file__).resolve().parent
    app_root = here.parent
    exe = _exe_path(app_root)
    apps_json = _user_base() / ".octosense" / "apps.json"

    ap = argparse.ArgumentParser(
        prog="register-with-shell.py",
        description=(
            "Upsert a finance_brief entry in ~/.octosense/apps.json "
            "(OctoSense shell user catalog). Does NOT rebuild."
        ),
    )
    group = ap.add_mutually_exclusive_group()
    group.add_argument("--uninstall", action="store_true",
                       help="Remove the finance_brief entry from the catalog.")
    group.add_argument("--dry-run", action="store_true",
                       help="Echo the resulting catalog and exit without writing.")
    args = ap.parse_args()

    if args.uninstall:
        if not apps_json.is_file():
            print(f"no catalog at {apps_json} (no-op)")
            return 0
        arr = _load_catalog(apps_json)
        before = len(arr)
        arr = [e for e in arr if e.get("id") != ENTRY_ID]
        if len(arr) == before:
            print(f"'{ENTRY_ID}' not in {apps_json} (no-op)")
            return 0
        if args.dry_run:
            print(f"[dry-run] would write {apps_json}:")
            print(json.dumps(arr, ensure_ascii=False, indent=2))
        else:
            apps_json.write_text(json.dumps(arr, ensure_ascii=False, indent=2),
                                 encoding="utf-8")
            print(f"removed '{ENTRY_ID}' from {apps_json}")
        return 0

    # Install (default) or dry-run with default action.
    if not exe.is_file():
        print(f"missing binary: {exe}", file=sys.stderr)
        print("run `python scripts/install-as-makepad-app.py` first "
              "(it builds + registers in one step).", file=sys.stderr)
        return 4

    arr = _load_catalog(apps_json)
    arr = [e for e in arr if e.get("id") != ENTRY_ID]
    arr.append({
        "id": ENTRY_ID,
        "label": LABEL,
        "executable": os.path.abspath(str(exe)),
        "policy": POLICY,
    })
    if args.dry_run:
        print(f"[dry-run] would write {apps_json}:")
        print(json.dumps(arr, ensure_ascii=False, indent=2))
        return 0
    apps_json.parent.mkdir(parents=True, exist_ok=True)
    apps_json.write_text(json.dumps(arr, ensure_ascii=False, indent=2),
                         encoding="utf-8")
    print(f"registered: {apps_json}")
    print("next: cargo run --release -p octosense  (in $WS/OctoSense)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
