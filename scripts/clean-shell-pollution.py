#!/usr/bin/env python3
"""clean-shell-pollution.py — clear finance-brief pollution from OctoSense shell.

Three pollution sources may exist after accidentally running
install-as-system-app.py (legacy Page-format script) without --uninstall:

  1. OctoSense/desktop/system-apps.json (catalog entry)
  2. OctoSense/apps/finance-brief/        (partial bundle copy)
  3. ~/.octosense/apps/.system/os.finance-brief/<hash>/  (shell-cached pack)

This script diagnoses all three, cleans them, and verifies a clean state.
Use --diagnose-only to inspect without modifying anything.

Exit codes:
  0  = clean (all three locations cleared)
  1  = partial state (ran with --*--only flag, others untouched)
  10 = diagnostic-only found pollution (with --diagnose-only)
  2  = bad CLI args
"""
import argparse
import json
import os
import shutil
import sys
from pathlib import Path

# --- Paths ---
WS = Path(__file__).resolve().parent.parent.parent
OCTO = WS / "OctoSense"
OCTO_SEL = OCTO / "desktop" / "system-apps.json"
OCTO_BUNDLE = OCTO / "apps" / "finance-brief"

USER_BASE = Path(os.environ.get("OCTOSENSE_HOME") or Path.home())
USER_SYS = USER_BASE / ".octosense" / "apps" / ".system" / "os.finance-brief"

ENTRY_ID = "finance-brief"


def _read_octo_sel() -> dict | None:
    """Return OctoSense/desktop/system-apps.json as dict, or None if missing/unreadable.

    Uses ``utf-8-sig`` so a UTF-8 BOM (which some Windows tools, e.g. PowerShell
    ``Set-Content -Encoding UTF8``, prepend) is stripped transparently.
    """
    if not OCTO_SEL.is_file():
        return None
    try:
        return json.loads(OCTO_SEL.read_text(encoding="utf-8-sig"))
    except (json.JSONDecodeError, OSError) as e:
        print(f"warning: cannot parse {OCTO_SEL}: {e}", file=sys.stderr)
        return None


def _diagnose() -> dict:
    """Inspect all three locations; no writes."""
    diag: dict = {"octosense": {}, "user_cache": {}}

    sel = _read_octo_sel()
    if sel is None:
        diag["octosense"]["catalog"] = {"path": str(OCTO_SEL), "exists": False}
    else:
        apps = sel.get("apps", [])
        diag["octosense"]["catalog"] = {
            "path": str(OCTO_SEL),
            "exists": True,
            "apps": apps,
            "polluted": ENTRY_ID in apps,
        }
    diag["octosense"]["bundle"] = {
        "path": str(OCTO_BUNDLE),
        "exists": OCTO_BUNDLE.exists(),
    }

    diag["user_cache"]["path"] = str(USER_SYS)
    diag["user_cache"]["exists"] = USER_SYS.exists()
    if USER_SYS.exists():
        hash_dirs = sorted([p for p in USER_SYS.iterdir() if p.is_dir()])
        diag["user_cache"]["hash_subdirs"] = [p.name for p in hash_dirs]

    diag["any_pollution"] = (
        diag["octosense"].get("catalog", {}).get("polluted", False)
        or diag["octosense"]["bundle"]["exists"]
        or diag["user_cache"]["exists"]
    )
    return diag


def _clean_octosense(dry: bool) -> dict:
    """Clean OctoSense-side pollution (catalog + bundle). Returns a report dict."""
    report: dict = {"catalog": {}, "bundle": {}}
    sel = _read_octo_sel()
    if sel is None:
        report["catalog"] = {"path": str(OCTO_SEL), "status": "missing (no-op)"}
    else:
        apps = sel.get("apps", [])
        before = list(apps)
        apps[:] = [a for a in apps if a != ENTRY_ID]
        if before != apps:
            status = "would-clean" if dry else "cleaned"
            if not dry:
                OCTO_SEL.write_text(
                    json.dumps(sel, ensure_ascii=False, indent=2) + "\n",
                    encoding="utf-8",
                )
            report["catalog"] = {
                "path": str(OCTO_SEL),
                "before": before,
                "after": apps,
                "status": status,
            }
        else:
            report["catalog"] = {
                "path": str(OCTO_SEL),
                "apps": apps,
                "status": "already-clean",
            }
    if OCTO_BUNDLE.exists():
        status = "would-clean" if dry else "cleaned"
        if not dry:
            shutil.rmtree(OCTO_BUNDLE)
        report["bundle"] = {"path": str(OCTO_BUNDLE), "status": status}
    else:
        report["bundle"] = {"path": str(OCTO_BUNDLE), "status": "missing (no-op)"}
    return report


def _clean_user_cache(dry: bool) -> dict:
    """Clean user-side shell-cached pack. Returns a report dict."""
    if USER_SYS.exists():
        status = "would-clean" if dry else "cleaned"
        if not dry:
            shutil.rmtree(USER_SYS)
        return {"path": str(USER_SYS), "status": status}
    return {"path": str(USER_SYS), "status": "missing (no-op)"}


def main() -> int:
    ap = argparse.ArgumentParser(
        prog="clean-shell-pollution.py",
        description=(
            "Diagnose + clean finance-brief pollution from OctoSense shell. "
            "By default cleans all three locations and verifies."
        ),
    )
    ap.add_argument(
        "--diagnose-only",
        action="store_true",
        help="print diagnostic JSON and exit (no writes)",
    )
    ap.add_argument(
        "--dry-run",
        action="store_true",
        help="show what would be cleaned without writing",
    )
    scope = ap.add_mutually_exclusive_group()
    scope.add_argument(
        "--user-cache-only",
        action="store_true",
        help="only clean ~/.octosense/apps/.system/os.finance-brief/",
    )
    scope.add_argument(
        "--octosense-only",
        action="store_true",
        help="only clean OctoSense/desktop/system-apps.json + apps/finance-brief/",
    )
    args = ap.parse_args()

    diag = _diagnose()
    print("=== diagnose (before) ===")
    print(json.dumps(diag, ensure_ascii=False, indent=2))

    if args.diagnose_only:
        return 10 if diag["any_pollution"] else 0

    print()
    print("=== clean ===")
    if not args.user_cache_only:
        r = _clean_octosense(args.dry_run)
        print(json.dumps(r, ensure_ascii=False, indent=2))
    if not args.octosense_only:
        r = _clean_user_cache(args.dry_run)
        print(json.dumps(r, ensure_ascii=False, indent=2))

    print()
    print("=== diagnose (after) ===")
    final = _diagnose()
    print(json.dumps(final, ensure_ascii=False, indent=2))

    if final["any_pollution"]:
        print()
        print("WARNING: pollution still detected after clean.", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())