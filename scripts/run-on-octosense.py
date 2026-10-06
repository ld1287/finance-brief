#!/usr/bin/env python3
# run-on-octosense.py — launch OctoSense shell with the finance-brief catalog.
#
# Per OctoSense/desktop/README.md, the shell reads:
#   1. --apps <path> if given,
#   2. else ~/.octosense/apps.json if it exists,
#   3. else config/apps.json.
# Per finance-brief policy, every file lives under C:\Code\OctoSenseorg\,
# so we pass --apps and never touch the user-level catalog.
import argparse
import os
import subprocess
import sys
import time
from pathlib import Path
from urllib.request import urlopen

DEFAULT_PORT = 57450


def parse_args():
    parser = argparse.ArgumentParser(
        prog="run-on-octosense.py",
        description=(
            "Launch OctoSense shell with the finance-brief catalog "
            "(--apps finance-brief/catalog/apps.json)."
        ),
    )
    parser.add_argument(
        "--port", type=int, default=DEFAULT_PORT,
        help=f"makepad remote instrument port (default {DEFAULT_PORT})",
    )
    parser.add_argument(
        "--no-wait", action="store_true",
        help="return as soon as the shell is spawned, without waiting for /s",
    )
    return parser.parse_args()


def get(port, route):
    with urlopen(f"http://127.0.0.1:{port}/{route}", timeout=2) as r:
        return r.read().decode("utf-8", "replace")


def main():
    args = parse_args()
    here = Path(__file__).resolve().parent
    finance_brief = here.parent
    workspace = finance_brief.parent
    catalog = finance_brief / "catalog" / "apps.json"
    shell_bin = (
        workspace / "OctoSense" / "target" / "release" / "octosense.exe"
    )

    if not catalog.is_file():
        print(f"missing catalog: {catalog}", file=sys.stderr)
        print(
            "run `python scripts/install-as-makepad-app.py` first",
            file=sys.stderr,
        )
        raise SystemExit(2)
    if not shell_bin.is_file():
        print(f"missing shell binary: {shell_bin}", file=sys.stderr)
        raise SystemExit(3)

    runtime_home = finance_brief / ".octosense"
    runtime_home.mkdir(parents=True, exist_ok=True)

    port = args.port
    env = os.environ.copy()
    # Per finance-brief policy, every file lives under C:\Code\OctoSenseorg\.
    # OCTOSENSE_HOME redirects shell runtime data (approvals/, apps/, secrets/,
    # wm/) here; see OctoSense/desktop/README.md "OCTOSENSE_HOME override".
    # migrate-octosense-home.py one-time moves ~/.octosense/ contents over.
    env["OCTOSENSE_HOME"] = str(runtime_home)
    env.pop("MAKEOS_HOME", None)
    env["RUSTUP_TOOLCHAIN"] = "stable-x86_64-pc-windows-msvc"
    # MAKEPAD_REMOTE / MAKEPAD_HIDE_WINDOWS MUST NOT be in subprocess env:
    # OctoSense shell's spawn_client (clients.rs:1427 scrub_env) preserves
    # MAKEPAD_* prefixed env vars for children, so a finance-brief.exe
    # launched from the launcher drawer would try to bind MAKEPAD_REMOTE's
    # port itself and collide with the shell (os error 10048). Pass these
    # settings to the shell only via argv so subprocess children inherit a
    # clean env. See makepad/platform/src/remote.rs:413-426 (--remote PORT).
    env.pop("MAKEPAD_REMOTE", None)
    env.pop("MAKEPAD_HIDE_WINDOWS", None)

    print(f"launching: {shell_bin}")
    print(f"  --apps {catalog}")
    print(f"  --remote {port} --hide-windows")
    print(f"  OCTOSENSE_HOME={runtime_home}")
    proc = subprocess.Popen(
        [str(shell_bin), "--apps", str(catalog),
         "--remote", str(port), "--hide-windows"],
        cwd=str(workspace / "OctoSense"),
        env=env,
        creationflags=subprocess.CREATE_NO_WINDOW,
    )
    if not args.no_wait:
        for _ in range(60):
            try:
                get(port, "s")
                break
            except Exception:
                time.sleep(0.5)
        else:
            print("warning: /s timeout", file=sys.stderr)
    print(f"octosense pid={proc.pid} port={port}")
    print("press Ctrl-C to stop")
    try:
        proc.wait()
    except KeyboardInterrupt:
        proc.terminate()
        proc.wait()


if __name__ == "__main__":
    main()
