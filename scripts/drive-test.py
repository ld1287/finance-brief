#!/usr/bin/env python3
"""Drive the finance-brief app over card-host's remote bridge and print what is
on screen after each step.

Usage:
    python drive-test.py --port <port>
    python drive-test.py <port>
"""
import argparse
import json
import sys
import time
import urllib.error
import tempfile
import urllib.request
from pathlib import Path

# Mirrors the .sh version, which writes the snap body to a temp file so the
# heredoc python can read it. Kept here for parity; nothing else consumes it.
SNAP_TMP = Path(tempfile.gettempdir()) / "finance-brief-drive.json"


def _fetch_snap(port):
    """GET /snap and persist the body to SNAP_TMP."""
    url = f"http://127.0.0.1:{port}/snap"
    with urllib.request.urlopen(url, timeout=8) as resp:
        body = resp.read()
    SNAP_TMP.write_bytes(body)
    return body


def snap(port):
    """Print visible non-Splash text labels on the current screen."""
    data = json.loads(_fetch_snap(port))
    out = []
    for s in data["s"]:
        t = s.get("t")
        if t and s["ty"] != "Splash":
            out.append(str(t)[:44])
    print("   ", " | ".join(out))


def rect(port, name):
    """Return (x, y) center of the widget whose text matches `name`, or None."""
    data = json.loads(_fetch_snap(port))
    for s in data["s"]:
        if str(s.get("t", "")) == name:
            r = s["r"]
            return int(r[0] + r[2] / 2), int(r[1] + r[3] / 2)
    return None


def click(port, x, y):
    """Send a click at (x, y) and wait 1s for the UI to settle."""
    url = f"http://127.0.0.1:{port}/m?k=click&x={x}&y={y}&wait=1"
    with urllib.request.urlopen(url, timeout=5):
        pass
    time.sleep(1)


def click_text(port, name):
    """Click the widget whose text matches `name`. Returns False if missing."""
    xy = rect(port, name)
    if not xy:
        print(f"   [no widget] {name}")
        return False
    click(port, xy[0], xy[1])
    return True


def _run(port):
    print("=== 1. initial screen ===")
    snap(port)
    print("=== 2. click tab 加密 ===")
    if click_text(port, "加密"):
        snap(port)
    print("=== 3. open first row detail (BTC) ===")
    if click_text(port, "BTC"):
        snap(port)
    print("=== 4. favourite it ===")
    if click_text(port, "收藏 / 取消"):
        snap(port)
    print("=== 5. back to list ===")
    click_text(port, "返回列表")
    print("=== 6. click tab 收藏 ===")
    if click_text(port, "收藏"):
        snap(port)
    print("=== 7. click tab 要闻 (live news) ===")
    if click_text(port, "要闻"):
        snap(port)


def main(argv=None):
    parser = argparse.ArgumentParser(
        prog="drive-test.py",
        description=(
            "Drive the finance-brief app over card-host's remote bridge and "
            "print what is on screen after each step."
        ),
    )
    group = parser.add_mutually_exclusive_group(required=True)
    group.add_argument(
        "port_pos",
        nargs="?",
        type=int,
        help=argparse.SUPPRESS,
    )
    group.add_argument(
        "--port",
        dest="port",
        type=int,
        help="bridge port (e.g. the port card-host listens on)",
    )
    args = parser.parse_args(argv)
    port = args.port if args.port is not None else args.port_pos

    try:
        _run(port)
    except urllib.error.URLError as e:
        print(f"bridge unreachable: {e}", file=sys.stderr)
        return 7
    except SystemExit:
        raise
    except Exception as e:
        print(f"error: {e}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
