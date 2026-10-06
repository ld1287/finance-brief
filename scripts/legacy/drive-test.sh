#!/bin/sh
# Drive the finance-brief app over card-host's remote bridge and print what is
# on screen after each step. Usage: sh drive-test.sh <port>
P="$1"

snap() {
    curl -s -m 8 "127.0.0.1:$P/snap" -o /tmp/drive.json
    python3 - <<'PY'
import json
d = json.load(open('/tmp/drive.json'))
out = []
for s in d['s']:
    t = s.get('t')
    if t and s['ty'] != 'Splash':
        out.append(str(t)[:44])
print('   ', ' | '.join(out))
PY
}

rect() {
    curl -s -m 8 "127.0.0.1:$P/snap" -o /tmp/drive.json
    python3 - "$1" <<'PY'
import json, sys
d = json.load(open('/tmp/drive.json'))
name = sys.argv[1]
for s in d['s']:
    if str(s.get('t', '')) == name:
        r = s['r']
        print(int(r[0] + r[2] / 2), int(r[1] + r[3] / 2))
        break
PY
}

click() {
    curl -s -m 5 "127.0.0.1:$P/m?k=click&x=$1&y=$2&wait=1" > /dev/null
    sleep 1
}

click_text() {
    xy="$(rect "$1")"
    if [ -z "$xy" ]; then echo "   [no widget] $1"; return 1; fi
    click $xy
}

echo "=== 1. initial screen ==="
snap
echo "=== 2. click tab 加密 ==="
click_text 加密 && snap
echo "=== 3. open first row detail (BTC) ==="
click_text BTC && snap
echo "=== 4. favourite it ==="
click_text "收藏 / 取消" && snap
echo "=== 5. back to list ==="
click_text 返回列表
echo "=== 6. click tab 收藏 ==="
click_text 收藏 && snap
echo "=== 7. click tab 要闻 (live news) ==="
click_text 要闻 && snap
