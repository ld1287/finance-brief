#!/usr/bin/env python3
"""Install the finance-brief bundle as an OctoSense desktop system app.

Note: this script targets the legacy Page-format bundle (``*.card`` files,
``launcher.card``, ``screenshots/*.png``). The current repo's bundle is in
Path 1 format (``bundle/screens/*.octoscript``); for that, use
``install-as-makepad-app.py`` instead, which registers the binary in
``~/.octosense/apps.json`` and does not need a pre-built card bundle.
This script is kept defensive so a missing source file prints a warning
and is skipped instead of crashing.
"""
import argparse
import json
import shutil
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
APP = HERE.parent                                      # this repo's root
WS = APP.parent                                        # workspace root (sibling of this repo)
OCTO = WS / "OctoSense"
DEST_APP = OCTO / "apps" / "finance-brief"
DEST = DEST_APP / "bundle"
SEL = OCTO / "desktop" / "system-apps.json"


def _deep_copy(obj):
    return json.loads(json.dumps(obj))


def _diff_keys(old, new, path=""):
    """Yield (path, old, new) for changed leaves between two JSON-like values.

    Dicts are walked key-by-key; lists are compared by equality only.
    """
    if isinstance(old, dict) and isinstance(new, dict):
        for k in sorted(set(old) | set(new)):
            sub = f"{path}.{k}" if path else str(k)
            if k not in old:
                yield sub, "<missing>", new[k]
            elif k not in new:
                yield sub, old[k], "<missing>"
            elif old[k] != new[k]:
                yield from _diff_keys(old[k], new[k], sub)
    elif old != new:
        yield path, old, new


def _print_diff(label, old, new):
    print(f"  {label}")
    changes = list(_diff_keys(old, new))
    if not changes:
        print("    (no change)")
        return
    for path, before, after in changes:
        bs = json.dumps(before, ensure_ascii=False)
        af = json.dumps(after, ensure_ascii=False)
        print(f"    {path}: {bs} -> {af}")


def main():
    ap = argparse.ArgumentParser(
        description="Install this bundle as an OctoSense system app.",
    )
    ap.add_argument(
        "--dry-run",
        action="store_true",
        help="print every file operation and JSON change without writing anything",
    )
    group = ap.add_mutually_exclusive_group()
    group.add_argument(
        "--uninstall",
        action="store_true",
        help="remove the finance-brief entry from OctoSense system-apps.json, delete apps/finance-brief/, and clear ~/.octosense/apps/.system/os.finance-brief/",
    )
    args = ap.parse_args()
    dry = args.dry_run

    if not (OCTO / "Cargo.toml").is_file():
        print(f"no OctoSense checkout at {OCTO}", file=sys.stderr)
        raise SystemExit(1)

    if args.uninstall:
        if SEL.is_file():
            sel = json.loads(SEL.read_text(encoding="utf-8"))
            before_apps = list(sel.get("apps", []))
            sel["apps"] = [a for a in sel.get("apps", []) if a != "finance-brief"]
            if sel["apps"] != before_apps:
                SEL.write_text(json.dumps(sel, ensure_ascii=False, indent=2), encoding="utf-8")
                print(f"removed 'finance-brief' from {SEL}")
            else:
                print(f"'finance-brief' not in {SEL} (no-op)")
        else:
            print(f"missing {SEL}, skipping")
        if DEST_APP.exists():
            shutil.rmtree(DEST_APP)
            print(f"removed {DEST_APP}")
        else:
            print(f"{DEST_APP} not present (no-op)")
        # Also clear any stale shell-side cached pack at ~/.octosense/apps/.system/os.finance-brief/
        # The shell packs system-apps.json entries on startup; without this, a previously
        # cached pack (with no page.card) would linger even after the catalog entry is removed.
        user_system = Path.home() / ".octosense" / "apps" / ".system" / "os.finance-brief"
        if user_system.exists():
            shutil.rmtree(user_system)
            print(f"removed shell-cached pack: {user_system}")
        else:
            print(f"no shell-cached pack: {user_system} (no-op)")
        return

    src = APP / "bundle"
    src_screenshots = src / "screenshots"

    # ---- Plan (read everything we need up front so dry-run is non-mutating) ----
    card_files = sorted(src.glob("*.card"))
    screenshot_files = (
        sorted(src_screenshots.glob("*.png")) if src_screenshots.is_dir() else []
    )

    # listing.json: rewrite icon path to the root copy (the store listing keeps
    # it under assets/, which the system-app packer does not honour).
    listing = json.loads((src / "listing.json").read_text(encoding="utf-8"))
    new_listing = _deep_copy(listing)
    new_listing["icon"] = "icon.svg"

    # manifest.json: a system app's id is under os. (the gate reserves the prefix
    # for them) and its digest is left empty: the build stamps the packed bytes.
    manifest = json.loads((src / "manifest.json").read_text(encoding="utf-8"))
    new_manifest = _deep_copy(manifest)
    short = new_manifest["id"].split(".")[-1]
    new_manifest["id"] = "os." + short
    new_manifest["integrity"]["bundle_blake3"] = ""

    # system-apps.json: add this app to the desktop's selection (idempotent).
    sel_data = json.loads(SEL.read_text(encoding="utf-8"))
    new_sel = _deep_copy(sel_data)
    if "finance-brief" not in new_sel["apps"]:
        new_sel["apps"] = list(new_sel["apps"]) + ["finance-brief"]

    if dry:
        print(f"[dry-run] OCTO = {OCTO}")
        print(f"[dry-run] DEST = {DEST}")
        print(f"[dry-run] SEL  = {SEL}")
        print()
        print("[dry-run] directory ops:")
        print(f"  mkdir -p {DEST}")
        print(f"  mkdir -p {DEST / 'screenshots'}")
        print()
        print("[dry-run] file copies:")
        for c in card_files:
            print(f"  copy {c} -> {DEST / c.name}")
        print(f"  copy {src / 'launcher.card'} -> {DEST / 'page.card'}")
        print(f"  copy -r {src / 'schema'} -> {DEST / 'schema'}")
        print(f"  copy {src / 'workflow.octoscript'} -> {DEST / 'workflow.octoscript'}")
        print(f"  copy -r {src / 'kit'} -> {DEST / 'kit'}")
        print(f"  copy {src / 'assets' / 'icon.svg'} -> {DEST / 'icon.svg'}")
        if screenshot_files:
            for s in screenshot_files:
                print(f"  copy {s} -> {DEST / 'screenshots' / s.name}")
        else:
            print(f"  (no screenshots under {src_screenshots})")
        print()
        print("[dry-run] generated files:")
        print(f"  write {DEST / 'page.data.json'} = {{}}")
        print()
        print("[dry-run] JSON rewrites:")
        _print_diff(
            f"{src / 'listing.json'} -> {DEST / 'listing.json'}",
            listing, new_listing,
        )
        _print_diff(
            f"{src / 'manifest.json'} -> {DEST / 'manifest.json'}",
            manifest, new_manifest,
        )
        _print_diff(str(SEL), sel_data, new_sel)
        print()
        print(f"[dry-run] installed manifest: {new_manifest['id']} version {new_manifest['version']}")
        print(f"[dry-run] desktop system apps: {new_sel['apps']}")
        print()
        print(f"installed to {DEST}")
        print("run it with:")
        print(f"  cd {OCTO} && cargo run --release -p octosense")
        return

    # ---- Real execution ----
    DEST.mkdir(parents=True, exist_ok=True)
    (DEST / "screenshots").mkdir(parents=True, exist_ok=True)

    # Page-format bundle: 12 *.card screens + page.card entry + page.data.json
    # + schema/caps/workflow. The runtime looks up `card_source` = page.card;
    # without it the install falls back to the stock 809-byte splash. Only
    # *.card files are copied: main.splash is Path 1 (card-host interactive UI)
    # and must not enter the shell pack.
    # Each copy is wrapped: missing source → warn + skip, do NOT crash.
    # The current repo's Path-1 bundle has none of these files, so the install
    # gracefully degrades to "installed an empty bundle" with a clear summary.

    def _try_copy(src_path: Path, dst_path: Path, kind: str = "file") -> bool:
        """Copy src→dst if src exists; warn + return False if not."""
        if not src_path.exists():
            print(f"  [skip] missing source: {src_path}  ({kind})", file=sys.stderr)
            return False
        try:
            if src_path.is_dir():
                shutil.copytree(src_path, dst_path, dirs_exist_ok=True)
            else:
                shutil.copy(src_path, dst_path)
            return True
        except (FileNotFoundError, NotADirectoryError, IsADirectoryError) as e:
            print(f"  [skip] copy failed: {src_path} → {dst_path}: {e}", file=sys.stderr)
            return False

    copied: list[str] = []
    skipped: list[str] = []

    # *.card files (legacy Page-format screens)
    for c in card_files:
        if _try_copy(c, DEST / c.name, "card"):
            copied.append(str(c.name))
        else:
            skipped.append(str(c.name))

    # launcher.card → page.card (entry point)
    if _try_copy(src / "launcher.card", DEST / "page.card", "entry card"):
        copied.append("launcher.card→page.card")
    else:
        skipped.append("launcher.card→page.card")

    # schema/, workflow.octoscript, kit/, assets/icon.svg
    for s, label in [
        (src / "schema",              "schema/"),
        (src / "workflow.octoscript", "workflow.octoscript"),
        (src / "kit",                 "kit/"),
        (src / "assets" / "icon.svg", "icon.svg"),
    ]:
        if _try_copy(s, DEST / label.rstrip("/"), "dir" if s.is_dir() or label.endswith("/") else "file"):
            copied.append(label)
        else:
            skipped.append(label)

    # screenshots/*.png
    if screenshot_files:
        for s in screenshot_files:
            if _try_copy(s, DEST / "screenshots" / s.name, "screenshot"):
                copied.append(f"screenshots/{s.name}")
            else:
                skipped.append(f"screenshots/{s.name}")
    else:
        print("  [skip] no screenshots/ directory under bundle/", file=sys.stderr)
        skipped.append("screenshots/*.png (none)")

    (DEST / "page.data.json").write_text(
        json.dumps({}, ensure_ascii=False), encoding="utf-8"
    )

    (DEST / "listing.json").write_text(
        json.dumps(new_listing, ensure_ascii=False, indent=2), encoding="utf-8"
    )

    (DEST / "manifest.json").write_text(
        json.dumps(new_manifest, ensure_ascii=False, indent=2), encoding="utf-8"
    )
    print("installed manifest:", new_manifest["id"], "version", new_manifest["version"])

    if new_sel != sel_data:
        SEL.write_text(
            json.dumps(new_sel, ensure_ascii=False, indent=2), encoding="utf-8"
        )
    print("desktop system apps:", new_sel["apps"])

    print()
    print(f"installed to {DEST}")
    print(f"  copied ({len(copied)}): {copied if copied else '(none)'}")
    print(f"  skipped ({len(skipped)}): {skipped if skipped else '(none)'}")
    if copied:
        print("run it with:")
        print(f"  cd {OCTO} && cargo run --release -p octosense")
    elif not copied:
        print()
        print("WARNING: nothing was installed (current repo uses Path 1 bundle).")
        print("         Use scripts/install-as-makepad-app.py instead — it registers")
        print("         the binary via ~/.octosense/apps.json without needing *.card files.")


if __name__ == "__main__":
    main()