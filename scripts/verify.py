#!/usr/bin/env python3
"""Print what each tab shows. Usage: python verify.py <port>"""

import argparse
import json
import sys
import time
import urllib.error
import urllib.request


def snap(port):
    url = f"http://127.0.0.1:{port}/snap"
    with urllib.request.urlopen(url, timeout=8) as resp:
        data = json.loads(resp.read())
    out = []
    for s in data["s"]:
        t = s.get("t")
        if t and s["ty"] == "Label" and not str(t).startswith("//"):
            out.append(str(t)[:38])
    print("   ", " | ".join(out)[:520])


def click(port, x, y):
    url = f"http://127.0.0.1:{port}/m?k=click&x={x}&y={y}&wait=1"
    urllib.request.urlopen(url, timeout=5).read()
    time.sleep(1)


def main():
    parser = argparse.ArgumentParser(
        description="Print what each tab shows.")
    parser.add_argument("port", nargs="?", type=int,
                        help="server port (positional)")
    parser.add_argument("--port", dest="port_flag", type=int, default=None,
                        help="server port (flag form)")
    args = parser.parse_args()
    port = args.port_flag if args.port_flag is not None else args.port
    if port is None:
        parser.error("the following arguments are required: port")

    tabs = [("要闻", 41), ("A股", 101), ("美股", 161),
            ("加密", 223), ("外汇", 287), ("收藏", 348)]

    try:
        for name, x in tabs:
            print(f"### {name}")
            click(port, x, 139)
            snap(port)
        print("### 刷新")
        click(port, 367, 87)
        time.sleep(6)
        snap(port)
    except urllib.error.URLError as e:
        print(f"URLError: {e}", file=sys.stderr)
        sys.exit(7)


if __name__ == "__main__":
    main()