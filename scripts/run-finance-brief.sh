#!/bin/sh
# finance-brief one-shot launcher
#
# Runs finance-brief via card-host (OctoSense-App-Hub Path 1).
# This is the only path currently working end-to-end (OctoSense#317
# blocks path 2; install-as-system-app.sh is broken on path 3).
#
# Usage:
#   ./scripts/run-finance-brief.sh [port]
#
# Env overrides:
#   OCTOSENSE_APP_HUB  default /home/lumina/octoOs/OctoSense-App-Hub
#   APP_DATA           default /tmp/ch-X
#
# After start the script prints card-host PID and exits; card-host
# stays in the background. Stop with:  kill $PID

set -e

PORT="${1:-8180}"
HERE="$(cd "$(dirname "$0")" && pwd)"
APP="$(cd "$HERE/.." && pwd)"
APP_HUB="${OCTOSENSE_APP_HUB:-/home/lumina/octoOs/OctoSense-App-Hub}"
APP_DATA="${APP_DATA:-/tmp/ch-X}"
LOG="/tmp/card-host-finance-brief.log"

# --- pre-flight ---
if [ ! -x "$APP_HUB/target/debug/card-host" ]; then
  echo "ERROR: card-host binary not found at $APP_HUB/target/debug/card-host" >&2
  echo "Build with:  cd $APP_HUB && cargo build -p card-host" >&2
  exit 1
fi

if [ ! -f "$APP/bundle/main.splash" ]; then
  echo "ERROR: finance-brief bundle missing main.splash at $APP/bundle/" >&2
  exit 1
fi

# --- clean app-data ---
rm -rf "$APP_DATA"
mkdir -p "$APP_DATA"

# --- launch card-host in background ---
cd "$APP_HUB"
echo "Starting card-host on port $PORT ..."
nohup env LIBGL_ALWAYS_SOFTWARE=1 GALLIUM_DRIVER=softpipe MESA_LOADER_DRIVER_OVERRIDE=softpipe \
  ./target/debug/card-host \
    --bundle "$APP/bundle" \
    --app-data "$APP_DATA" \
    --allow-unsigned --stamp --remote "$PORT" \
    > "$LOG" 2>&1 &
PID=$!

# --- wait for splash eval ---
sleep 12

# --- sanity ---
if ! ps -p "$PID" > /dev/null 2>&1; then
  echo "ERROR: card-host (PID $PID) died before splash eval. Last 30 lines of log:" >&2
  tail -30 "$LOG" >&2
  exit 1
fi

echo "card-host alive (PID $PID)."
if grep -q "SPLASH.*eval" "$LOG"; then
  echo "PASS: splash parsed and evaluated"
  grep "SPLASH.*eval" "$LOG" | head -1
else
  echo "WARN: splash eval not yet in log (may still be loading)"
fi

# --- optional verification scripts ---
if [ -x "$HERE/verify.sh" ]; then
  echo ""
  echo "Running scripts/verify.sh on port $PORT ..."
  sh "$HERE/verify.sh" "$PORT" || echo "(verify.sh exited non-zero, continuing)"
fi

if [ -x "$HERE/drive-test.sh" ]; then
  echo ""
  echo "Running scripts/drive-test.sh on port $PORT ..."
  sh "$HERE/drive-test.sh" "$PORT" || echo "(drive-test.sh exited non-zero, continuing)"
fi

# --- done ---
cat <<EOF

card-host is running in the background.
  PID:     $PID
  Port:    $PORT
  Log:     $LOG
  AppData: $APP_DATA
  URL:     http://127.0.0.1:$PORT

To stop:           kill $PID
To clean app-data: rm -rf $APP_DATA
To follow log:     tail -f $LOG
EOF
