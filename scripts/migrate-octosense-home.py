#!/usr/bin/env python3
# migrate-octosense-home.py — copy ~/.octosense/ contents to
# finance-brief/.octosense/, the new shell runtime home.
#
# Per finance-brief policy (see TODO .todo-finance-brief-runtime-home-inside-
# 2026-10-05.md), every file lives under C:\Code\OctoSenseorg\. Shell runtime
# data (approvals/, apps/, secrets/, wm/) used to live at
# C:\Users\<USER>\.octosense\ (= ~/.octosense/); run-on-octosense.py now
# redirects it via OCTOSENSE_HOME=<finance-brief>/.octosense.
#
# This script is a one-time migration: copy existing state over so approvals,
# weights, app metadata and theme settings survive. After running, the user
# manually removes ~/.octosense/.
#
# Usage:
#   python scripts/migrate-octosense-home.py            # copy
#   python scripts/migrate-octosense-home.py --dry-run  # list only, no copies
import shutil
import sys
from pathlib import Path


def parse_args():
    import argparse
    p = argparse.ArgumentParser(
        prog="migrate-octosense-home.py",
        description=(
            "Copy ~/.octosense/ contents into finance-brief/.octosense/. "
            "One-time migration; user deletes the source afterwards."
        ),
    )
    p.add_argument(
        "--dry-run", action="store_true",
        help="list what would be copied, don't actually copy",
    )
    return p.parse_args()


def main():
    args = parse_args()
    src = Path.home() / ".octosense"
    here = Path(__file__).resolve().parent
    dst = here.parent / ".octosense"

    if not src.is_dir():
        print(f"source missing: {src}", file=sys.stderr)
        print(
            "(nothing to migrate; run-on-octosense.py will create the new one)",
            file=sys.stderr,
        )
        return 0
    if dst.exists() and any(dst.iterdir()):
        print(f"destination already populated: {dst}", file=sys.stderr)
        print(
            "remove it first if you want a fresh copy "
            "(rmdir /s /q .octosense)",
            file=sys.stderr,
        )
        return 1

    files = [f for f in src.rglob("*") if f.is_file()]
    if args.dry_run:
        print(f"[dry-run] would copy {src} -> {dst}")
        print(f"[dry-run] {len(files)} files total")
        for f in files:
            rel = f.relative_to(src)
            print(f"  {f.stat().st_size:>10}  {rel}")
        return 0

    dst.mkdir(parents=True, exist_ok=True)
    n_files = 0
    n_bytes = 0
    for f in files:
        rel = f.relative_to(src)
        target = dst / rel
        target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(f, target)
        n_files += 1
        n_bytes += f.stat().st_size
    print(f"copied {n_files} files ({n_bytes} bytes): {src} -> {dst}")
    print()
    print("next: verify shell works with the new home, then remove the old:")
    print(f"  rmdir /s /q \"{src}\"")
    return 0


if __name__ == "__main__":
    sys.exit(main())