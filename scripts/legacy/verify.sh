#!/bin/sh
# Print what each tab shows. Usage: sh verify.sh <port>
P="$1"
snap() {
    curl -s -m 8 "127.0.0.1:$P/snap" -o /tmp/v.json
    python3 - <<'PY'
import json
d = json.load(open('/tmp/v.json'))
out = []
for s in d['s']:
    t = s.get('t')
    if t and s['ty'] == 'Label' and not str(t).startswith('//'):
        out.append(str(t)[:38])
print('   ', ' | '.join(out)[:520])
PY
}
click() { curl -s -m 5 "127.0.0.1:$P/m?k=click&x=$1&y=$2&wait=1" > /dev/null; sleep 1; }

for pair in "要闻 41" "A股 101" "美股 161" "加密 223" "外汇 287" "收藏 348"; do
    set -- $pair
    echo "### $1"
    click "$2" 139
    snap
done
echo "### 刷新"
click 367 87
sleep 6
snap
