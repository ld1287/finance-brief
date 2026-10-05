#!/usr/bin/env python3
"""diagnose-shell-state.py — read-only diagnostic for OctoSense shell state.

Inspects 5 locations relevant to finance-brief:

  1. OctoSense/desktop/system-apps.json  (system app catalog)
  2. OctoSense/desktop/config/apps.json    (user/developer catalog in dep repo)
  3. OctoSense/apps/finance-brief/         (partial bundle if polluted)
  4. ~/.octosense/apps/.system/os.finance-brief/   (shell-cached pack)
  5. ~/.octosense/apps.json                (user-level catalog; finance_brief)

Output is JSON to stdout. Exit codes:
  0  = clean (no pollution, finance_brief user entry may or may not exist)
  10 = pollution found
  1  = error reading files

This script NEVER writes anything.
"""
import json
import os
import sys
from pathlib import Path

# --- Paths ---
WS = Path(__file__).resolve().parent.parent.parent
OCTO = WS / "OctoSense"
OCTO_SEL = OCTO / "desktop" / "system-apps.json"
OCTO_CFG = OCTO / "desktop" / "config" / "apps.json"
OCTO_BUNDLE = OCTO / "apps" / "finance-brief"

USER_BASE = Path(os.environ.get("OCTOSENSE_HOME") or Path.home())
USER_SYS = USER_BASE / ".octosense" / "apps" / ".system" / "os.finance-brief"
USER_CATALOG = USER_BASE / ".octosense" / "apps.json"

SYSTEM_ID = "finance-brief"   # for system-apps.json + .system dir (hyphen)
USER_ID = "finance_brief"     # for user catalog (underscore)


def _scan_json_array(path: Path, key: str, value: str) -> dict:
    """If path is a JSON array, check whether any element has key=value."""
    if not path.is_file():
        return {"path": str(path), "exists": False}
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except (json.JSONDecodeError, OSError) as e:
        return {"path": str(path), "exists": True, "error": str(e)}
    if not isinstance(data, list):
        return {"path": str(path), "exists": True, "schema": type(data).__name__}
    matches = [e for e in data if isinstance(e, dict) and e.get(key) == value]
    return {
        "path": str(path),
        "exists": True,
        "count": len(data),
        "found": bool(matches),
        "match": matches[0] if matches else None,
    }


def main() -> int:
    report: dict = {
        "octosense_system_apps": _scan_json_array(OCTO_SEL, "id", SYSTEM_ID),
        "octosense_apps_json":   _scan_json_array(OCTO_CFG, "id", SYSTEM_ID),
        "octosense_bundle": {
            "path": str(OCTO_BUNDLE),
            "exists": OCTO_BUNDLE.exists(),
        },
        "user_cache": {
            "path": str(USER_SYS),
            "exists": USER_SYS.exists(),
        },
        "user_catalog": _scan_json_array(USER_CATALOG, "id", USER_ID),
    }
    if USER_SYS.exists():
        hash_dirs = sorted([p for p in USER_SYS.iterdir() if p.is_dir()])
        report["user_cache"]["hash_subdirs"] = [p.name for p in hash_dirs]

    report["any_pollution"] = (
        report["octosense_system_apps"].get("found", False)
        or report["octosense_bundle"]["exists"]
        or report["user_cache"]["exists"]
    )
    report["user_app_registered"] = report["user_catalog"].get("found", False)

    print(json.dumps(report, ensure_ascii=False, indent=2))
    return 10 if report["any_pollution"] else 0


if __name__ == "__main__":
    try:
        sys.exit(main())
    except Exception as e:
        print(
            json.dumps({"error": str(e)}, ensure_ascii=False),
            file=sys.stderr,
        )
        sys.exit(1)