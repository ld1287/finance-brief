#!/usr/bin/env python3
"""End-to-end verification for v10 Splash.view swap.

Boots finance-brief.exe with --remote=0, captures /snap?all=1 before and after
a synthetic tap on the launcher tile, and asserts the widget tree actually
changed (route → UI update). One-shot helper — not part of the permanent
scripts/ catalog.
"""
from __future__ import annotations

import json
import os
import re
import subprocess
import sys
import time
import urllib.error
import urllib.request
from pathlib import Path
from collections import Counter
from tempfile import gettempdir

EXE = Path(
    r"C:\Code\OctoSenseorg\finance-brief\apps\desktop\target\release\finance-brief.exe"
)
LOG = Path(gettempdir()) / "fb_swap.log"


def kill_existing() -> None:
    subprocess.run(
        ["powershell", "-NoProfile", "-Command",
         "Get-Process finance-brief -ErrorAction SilentlyContinue | Stop-Process -Force"],
        check=False, capture_output=True,
    )
    time.sleep(0.5)


def boot() -> int:
    kill_existing()
    env = os.environ.copy()
    env["MAKEPAD_HIDE_WINDOWS"] = "1"
    env["RUST_BACKTRACE"] = "1"
    with open(LOG, "wb") as fh:
        proc = subprocess.Popen(
            [str(EXE), "--remote=0"],
            env=env,
            stdout=fh, stderr=subprocess.STDOUT,
            creationflags=0x00000008 | 0x00000200,  # DETACHED + NEW_PGRP
        )
    # Wait for the HTTP instrument to come up.
    deadline = time.time() + 25
    port = None
    while time.time() < deadline:
        if LOG.exists():
            txt = LOG.read_text(errors="ignore")
            m = re.search(r"listening on 127\.0\.0\.1:(\d+)", txt)
            if m:
                port = int(m.group(1))
                break
        time.sleep(0.3)
    if port is None:
        print(f"BOOT FAIL — no port in {LOG}", file=sys.stderr)
        sys.exit(2)
    return port


def get_snap(port: int) -> dict:
    url = f"http://127.0.0.1:{port}/snap?all=1"
    with urllib.request.urlopen(url, timeout=10) as r:
        return json.loads(r.read())


def post_click(port: int, x: int, y: int) -> None:
    url = f"http://127.0.0.1:{port}/click?x={x}&y={y}"
    try:
        with urllib.request.urlopen(url, timeout=5) as r:
            r.read()
    except urllib.error.HTTPError:
        pass  # /click may return 4xx; we only care that the tap landed


def widget_fingerprint(snap: dict) -> dict:
    """Compress a /snap tree to (type_counts, label_texts, total)."""
    def walk(node):
        if isinstance(node, dict):
            yield node
            for v in node.values():
                yield from walk(v)
        elif isinstance(node, list):
            for v in node:
                yield from walk(v)
    types = Counter()
    labels = []
    for n in walk(snap):
        t = n.get("type") or n.get("kind") or "?"
        types[t] += 1
        text = n.get("text") or n.get("label") or n.get("value")
        if isinstance(text, str) and text.strip():
            labels.append(text.strip())
    return {"types": dict(types), "labels": sorted(set(labels)), "total": sum(types.values())}


def main() -> int:
    port = boot()
    print(f"BOOT  port={port}")
    time.sleep(1.5)  # let first mount settle
    snap_before = get_snap(port)
    fp_before = widget_fingerprint(snap_before)
    print(f"BEFORE total={fp_before['total']} OctoscriptTap={fp_before['types'].get('OctoscriptTap', 0)}")
    # Click the first tile in the launcher grid (matches the prior session coords).
    post_click(port, 120, 204)
    time.sleep(1.5)  # let re-mount + insert_child_deep propagate
    snap_after = get_snap(port)
    fp_after = widget_fingerprint(snap_after)
    print(f"AFTER  total={fp_after['total']} OctoscriptTap={fp_after['types'].get('OctoscriptTap', 0)}")

    # Diff.
    types_before, types_after = fp_before["types"], fp_after["types"]
    type_diff = {
        "only_in_after": sorted({k for k in types_after if types_after.get(k, 0) != types_before.get(k, 0)}),
        "counts_changed": {
            k: (types_before.get(k, 0), types_after.get(k, 0))
            for k in set(types_before) | set(types_after)
            if types_before.get(k, 0) != types_after.get(k, 0)
        },
    }
    labels_before, labels_after = set(fp_before["labels"]), set(fp_after["labels"])
    label_diff = {
        "added": sorted(labels_after - labels_before),
        "removed": sorted(labels_before - labels_after),
    }
    print("TYPE_COUNTS_CHANGED:", json.dumps(type_diff["counts_changed"], ensure_ascii=False))
    print("LABELS_ADDED:", label_diff["added"][:10])
    print("LABELS_REMOVED:", label_diff["removed"][:10])

    n_changed_types = len([k for k, (b, a) in type_diff["counts_changed"].items() if b != a])
    n_added_labels = len(label_diff["added"])
    n_removed_labels = len(label_diff["removed"])

    print(
        f"\nSUMMARY: type_counts_changed={n_changed_types} "
        f"labels_added={n_added_labels} labels_removed={n_removed_labels}"
    )

    kill_existing()

    if n_changed_types == 0 and n_added_labels == 0 and n_removed_labels == 0:
        print("FAIL — widget tree did not change after click; v10 swap did not propagate.")
        return 1
    if n_added_labels < 1:
        print("WEAK — type counts changed but no new labels; route may not have re-rendered.")
        return 3
    print("PASS — widget tree updated on route change.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
