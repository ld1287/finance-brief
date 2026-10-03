#!/bin/sh
# Install this bundle as a system app of the OctoSense desktop shell.
#
# Why a system app: the shell packs the system apps named in its selection file
# (`OCTOSENSE_SYSTEM_APPS`, set in the shell's `.cargo/config.toml`) straight
# into the binary, by digest and without publisher signing - so it runs in the
# shell today. The store path is not usable here: `hub check` refuses a bundle
# whose listing names a screenshot that is not in it, so `hub publish` cannot
# produce the signed local catalog a store install needs.
#
# The workspace layout this expects is the one the design flow documents:
#   <workspace>/finance-brief/   (this repo)
#   <workspace>/OctoSense/       (the shell)
#
# Usage: sh scripts/install-as-system-app.sh
set -e

APP="$(cd "$(dirname "$0")/.." && pwd)"   # this repo's root
WS="$(cd "$APP/.." && pwd)"               # workspace root (sibling of this repo)
OCTO="$WS/OctoSense"
DEST_APP="$OCTO/apps/finance-brief"
DEST="$DEST_APP/bundle"
SEL="$OCTO/desktop/system-apps.json"

if [ ! -f "$OCTO/Cargo.toml" ]; then
    echo "no OctoSense checkout at $OCTO" >&2
    exit 1
fi

mkdir -p "$DEST" "$DEST/screenshots"
# Page format bundle: 12 *.card screens + page.card entry + page.data.json + schema/caps/workflow.
# The runtime looks up `card_source` = page.card; without it the install falls back to the stock
# 809-byte splash. The packer walks the bundle dir recursively, so every referenced asset must
# live here.
cp "$APP/bundle"/*.card "$DEST/"
cp "$APP/bundle/launcher.card" "$DEST/page.card"
echo '{}' > "$DEST/page.data.json"
cp -r "$APP/bundle/schema" "$DEST/"
cp "$APP/bundle/capabilities.toml" "$DEST/"
cp "$APP/bundle/workflow.octoscript" "$DEST/"
cp -r "$APP/bundle/kit" "$DEST/"
# Launcher art: the packer looks for icon.svg/icon.png at the bundle root.
cp "$APP/bundle/assets/icon.svg" "$DEST/icon.svg"
# listing.json: rewrite the icon path to the root copy (the store listing
# keeps it under assets/, which the system-app packer does not honour).
python3 - "$APP/bundle/listing.json" "$DEST/listing.json" <<'PY'
import json, sys
src, dst = sys.argv[1], sys.argv[2]
d = json.load(open(src))
d["icon"] = "icon.svg"
json.dump(d, open(dst, "w"), ensure_ascii=False, indent=2)
PY
# Real screenshots (1..6, taken with the shell's --test-action capture:
# GPU readback) - the store path also references them in listing.json.
if [ -d "$APP/bundle/screenshots" ]; then
    cp "$APP/bundle/screenshots"/*.png "$DEST/screenshots/" 2>/dev/null || true
fi

# A system app's id is under os. (the gate reserves the prefix for them) and its
# digest is left empty: the build stamps the packed bytes.
python3 - "$APP/bundle/manifest.json" "$DEST/manifest.json" <<'PY'
import json, sys
src, dst = sys.argv[1], sys.argv[2]
m = json.load(open(src))
short = m["id"].split(".")[-1]
m["id"] = "os." + short
m["integrity"]["bundle_blake3"] = ""
json.dump(m, open(dst, "w"), ensure_ascii=False, indent=2)
print("installed manifest:", m["id"], "version", m["version"])
PY

# Add it to the desktop's system-app selection (idempotent), keeping the rest.
python3 - "$SEL" <<'PY'
import json, sys
p = sys.argv[1]
d = json.load(open(p))
if "finance-brief" not in d["apps"]:
    d["apps"].append("finance-brief")
    json.dump(d, open(p, "w"), ensure_ascii=False, indent=2)
print("desktop system apps:", d["apps"])
PY

echo
echo "installed to $DEST"
echo "run it with:"
echo "  cd $OCTO && cargo run --release -p octosense"
