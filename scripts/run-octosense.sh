#!/bin/sh
# Install this bundle as an OctoSense desktop system app and run the shell.
# Usage: sh scripts/run-octosense.sh
set -e

HERE="$(cd "$(dirname "$0")" && pwd)"
APP="$(cd "$HERE/.." && pwd)"                 # this repo's root
WS="$(cd "$APP/.." && pwd)"                   # workspace root (sibling of this repo)

sh "$HERE/install-as-system-app.sh"

cd "$WS/OctoSense"
exec cargo run --release -p octosense
