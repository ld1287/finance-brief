#!/usr/bin/env python3
# Install finance-brief with the OctoSense desktop shell (catalog + binary)
# and run the shell. Uses install-as-makepad-app.py (Path-1 bundle, catalog
# writes go to ~/.octosense/apps.json; no dep-repo files are touched).
# Usage: python scripts/run-octosense.py
import argparse
import subprocess
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
APP = HERE.parent
WS = APP.parent

def main() -> int:
    parser = argparse.ArgumentParser(
        description="Install finance-brief with the OctoSense desktop shell "
                    "(catalog + binary) and run the shell.",
    )
    # (no extra flags today; reserved for future dry-run / passthrough)
    parser.parse_args()

    subprocess.run(
        [sys.executable, HERE / "install-as-makepad-app.py"],
        check=True,
    )
    subprocess.run(
        ["cargo", "run", "--release", "-p", "octosense"],
        check=True,
        cwd=WS / "OctoSense",
    )
    return 0

if __name__ == "__main__":
    raise SystemExit(main())
