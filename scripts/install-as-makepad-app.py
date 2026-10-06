#!/usr/bin/env python3
# install-as-makepad-app.py — register finance-brief with OctoSense desktop shell.
#
# Per OctoSense/desktop/README.md §"Developer programs and the catalog":
#   Catalog lookup: --apps <file> if given,
#                   else ~/.octosense/apps.json if it exists,
#                   else config/apps.json.
# This script uses the user-level file so it touches no project file
# (the dependency rule from OctoSense/AGENTS.md applies; see also
# apps/AGENTS.md: "If you are building a new OctoSense app rather than
# changing these, you are in the wrong place").
#
# Usage:
#   python scripts/install-as-makepad-app.py             # build + register + verify
#   python scripts/install-as-makepad-app.py --uninstall # remove the entry
#   python scripts/install-as-makepad-app.py --dry-run   # echo only, no writes
#   python scripts/install-as-makepad-app.py --help
import argparse
import json
import os
import re
import subprocess
import sys
from pathlib import Path

from _catalog_constants import ENTRY_ID, LABEL, POLICY


def parse_args(argv=None):
    parser = argparse.ArgumentParser(
        prog="install-as-makepad-app.py",
        description=(
            "Default: builds finance-brief and registers it with OctoSense shell "
            "(writes ~/.octosense/apps.json; --dry-run skips the build and write)."
        ),
    )
    group = parser.add_mutually_exclusive_group()
    group.add_argument(
        "--uninstall",
        action="store_true",
        help="remove the finance_brief entry from the catalog",
    )
    group.add_argument(
        "--dry-run",
        action="store_true",
        help="echo only, do not build or write the catalog",
    )
    return parser.parse_args(argv)


def main():
    args = parse_args()
    uninstall = args.uninstall
    dry = args.dry_run

    here = Path(__file__).resolve().parent
    app = here.parent
    ws = app.parent

    # finance-brief 项目的 catalog 入口（按"所有文件在 OctoSenseorg 内"约束）。
    # shell 通过 --apps 显式读它（scripts/run-on-octosense.py）。
    catalog_path = app / "catalog" / "apps.json"

    # .exe on Windows (Git Bash / WSL); make the catalog entry reflect the actual
    # file we built. cargo writes the same name on every host.
    exe_base = app / "apps" / "desktop" / "target" / "release" / "finance-brief"
    is_windows = sys.platform.startswith("win") or os.name == "nt"
    exe = exe_base.with_suffix(".exe") if is_windows else exe_base

    manifest = app / "apps" / "desktop" / "Cargo.toml"

    # --- 1. Build (skip on --uninstall; echo on --dry-run) --------------------
    if not uninstall:
        if dry:
            print(f"[dry-run] would: cargo build --release --manifest-path {manifest}")
        else:
            try:
                subprocess.run(
                    [
                        "cargo", "build", "--release",
                        "--manifest-path", str(manifest),
                    ],
                    check=True,
                    cwd=str(app),
                )
            except subprocess.CalledProcessError as e:
                # Preserve cargo's own exit code (shell `set -e` behavior).
                raise SystemExit(e.returncode)

    # --- 2. Verify the produced binary exists ----------------------------------
    # POSIX: must be executable; Windows: must at least exist (no -x there).
    if not dry and not uninstall:
        if is_windows:
            if not exe.is_file():
                print(f"missing binary: {exe}", file=sys.stderr)
                raise SystemExit(4)
        else:
            if not os.access(exe, os.X_OK):
                print(f"missing or non-executable binary: {exe}", file=sys.stderr)
                raise SystemExit(4)

    # --- 3. Verify makepad rev alignment (OctoSense/AGENTS.md §2) -------------
    # finance-brief pins makepad-widgets at a rev in its Cargo.toml; OctoSense
    # pulls the same crates from $WS/makepad. The shell's hosting protocol is
    # not stable across revisions, so we fail loud instead of silently booting
    # a mismatched binary.
    ws_makepad = ws / "makepad"
    if not dry and ws_makepad.is_dir():
        text = manifest.read_text(encoding="utf-8")
        m = re.search(
            r'makepad-widgets\s*=\s*\{\s*git\s*=\s*"[^"]+"\s*,\s*rev\s*=\s*"([0-9a-f]+)"',
            text,
        )
        fb_rev = m.group(1) if m else ""
        git_proc = subprocess.run(
            ["git", "-C", str(ws_makepad), "rev-parse", "HEAD"],
            capture_output=True, text=True,
        )
        octo_rev = git_proc.stdout.strip() if git_proc.returncode == 0 else "unknown"
        if fb_rev and octo_rev != "unknown" and fb_rev != octo_rev:
            print("makepad rev mismatch:", file=sys.stderr)
            print(f"  finance-brief Cargo.toml pins {fb_rev}", file=sys.stderr)
            print(f"  {ws_makepad} HEAD        is {octo_rev}", file=sys.stderr)
            print(
                "see OctoSense/AGENTS.md §2 'One revision per external dependency'.",
                file=sys.stderr,
            )
            raise SystemExit(3)

    # --- 4. Write the catalog entry (idempotent) ------------------------------
    catalog_path.parent.mkdir(parents=True, exist_ok=True)
    if dry:
        print(f"[dry-run] would write to {catalog_path}:")
        # Mirror the live-write line below: relative-to-catalog-parent path
        # with forward slashes, so the dry-run and live run show the same
        # catalog content.
        exe_rel = os.path.relpath(str(exe), str(catalog_path.parent)).replace("\\", "/")
        print(
            f'  [{{"id":"{ENTRY_ID}","label":"{LABEL}",'
            f'"executable":"{exe_rel}","policy":"{POLICY}"}}]'
        )
    else:
        if catalog_path.exists():
            with open(catalog_path, "r", encoding="utf-8") as f:
                arr = json.load(f)
        else:
            arr = []
        arr = [e for e in arr if e.get("id") != ENTRY_ID]
        # Preserve the Python bool trap safeguard from the original .sh heredoc:
        # a string "0" is truthy in Python, so only bool True / str "1" / int 1
        # are treated as "uninstall" sentinels.
        if uninstall not in (True, "1", 1):
            arr.append({
                "id": ENTRY_ID,
                "label": LABEL,
                # Path is RELATIVE to the catalog file's parent directory
                # (`finance-brief/catalog/`). OctoSense's `parse_catalog`
                # joins `executable` onto the catalog parent and uses that
                # as `app.bin`; `spawn_client` then sets `cmd.current_dir`
                # to that same parent so `Command::new(bin)` resolves the
                # relative path correctly. See crates/shell/src/octosense/catalog.rs:64
                # (`base.join(value)`) and crates/shell/src/clients.rs:1404
                # (`cmd.current_dir(&app.dir)`). Using an absolute path here
                # would also work (Path::join preserves absolute children),
                # but the relative form is portable and matches the
                # convention in `OctoSense/desktop/config/apps.json`.
                "executable": os.path.relpath(str(exe), str(catalog_path.parent)).replace("\\", "/"),
                "policy": POLICY,
            })
        with open(catalog_path, "w", encoding="utf-8") as f:
            json.dump(arr, f, ensure_ascii=False, indent=2)

    # --- 5. Done ---------------------------------------------------------------
    print()
    print(f"registered: {catalog_path}")
    print("next: python scripts/run-on-octosense.py")
    print("      (the shell reads --apps <catalog> instead of ~/.octosense/)")


if __name__ == "__main__":
    main()
