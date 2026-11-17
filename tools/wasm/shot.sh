#!/bin/bash
# 给 wasm 页面截图，并可先点几个按钮（按文字匹配），用于交互冒烟。
# 用法：tools/wasm/shot.sh <输出png> ["追加" "清空" ...]
# 依赖：out-wasm.html（先跑 ./wasm.sh）
set -e
OUT="${1:-/tmp/wasm-shot.png}"
shift || true
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
PAGE="$ROOT/out-wasm.html"
TMP="$(mktemp -d)/probe.html"

python3 - "$PAGE" "$TMP" "$@" <<'PY'
import sys, json
page, out = sys.argv[1], sys.argv[2]
clicks = sys.argv[3:]
html = open(page, encoding='utf-8').read()
script = """
<script>
const clicks = %s;
function clickText(text) {
  const el = [...document.querySelectorAll('button')].find(
    (b) => b.textContent.trim() === text
  );
  if (el) el.click();
}
clicks.forEach((text, index) => setTimeout(() => clickText(text), 60 * (index + 1)));
</script>
""" % json.dumps(clicks, ensure_ascii=False)
open(out, 'w', encoding='utf-8').write(html.replace('</body>', script + '</body>'))
PY

"/Applications/Google Chrome.app/Contents/MacOS/Google Chrome" \
  --headless=new --disable-gpu --hide-scrollbars \
  --window-size=412,900 --virtual-time-budget=4000 \
  --screenshot="$OUT" "file://$TMP" 2>/dev/null
echo "$OUT"
