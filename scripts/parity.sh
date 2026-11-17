#!/bin/bash
# 三端等价度检查（脚本化，不用肉眼看截图）：
#   1. 生成源码型后端：--target=web 产出的 HTML 结构/样式
#   2. Android：视图树里「徽标文字垂直居中」+ 截图采样「占位符颜色」
#   3. iOS：模拟器截图采样「占位符颜色」
#   4. macOS：需要窗口截图接口，暂列为人工项
#
# 用法：scripts/parity.sh [todo|dashboard]
# 退出码：0 全部通过；1 有失败；2 全部跳过（环境不具备）
set -uo pipefail

APP="${1:-${LUMEN_APP:-todo}}"
SDK="$HOME/Library/Android/sdk"
ADB="$SDK/platform-tools/adb"
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
OUT="${TMPDIR:-/tmp}/lumen-parity"
mkdir -p "$OUT"
cd "$ROOT" || exit 1

pass=0
fail=0
skip=0
record() {
  case "$1" in
    pass) pass=$((pass + 1)) ;;
    fail) fail=$((fail + 1)) ;;
    skip) skip=$((skip + 1)) ;;
  esac
  printf '  [%-4s] %s\n' "$1" "$2"
}

echo "== 1) 生成源码后端（web）=="
if cargo run -q -- web "$APP" > "$OUT/web.log" 2>&1; then
  if python3 tools/parity.py html-has out.html "<!DOCTYPE html>" "display: flex" "::placeholder" > "$OUT/web-check.log" 2>&1; then
    record pass "web HTML 结构：$(tail -1 "$OUT/web-check.log")"
  else
    record fail "web HTML 结构：$(tail -1 "$OUT/web-check.log")"
  fi
  cp out.html "$OUT/out-$APP.html"
else
  record fail "web 生成失败：$(tail -2 "$OUT/web.log" | tr '\n' ' ')"
fi

echo "== 2) Android（视图树 + 截图采样）=="
if "$ADB" get-state > /dev/null 2>&1; then
  # 模拟器屏幕可能已经休眠，先点亮并解除锁屏，否则截出来是全黑
  "$ADB" shell input keyevent 224 > /dev/null 2>&1
  "$ADB" shell wm dismiss-keyguard > /dev/null 2>&1
  sleep 1
  # 应用没在跑就先拉起来（重启模拟器后经常是这种情况）
  if [ -z "$("$ADB" shell pidof com.lumen.app 2>/dev/null | tr -d '\r')" ]; then
    "$ADB" shell am start -n com.lumen.app/.MainActivity > /dev/null 2>&1
    sleep 4
  fi
  "$ADB" exec-out screencap -p > "$OUT/android.png" 2>/dev/null
  if python3 tools/parity.py is-dark "$OUT/android.png" > /dev/null 2>&1; then
    # 还是黑屏（模拟器休眠）：重启一次应用再截
    "$ADB" shell am force-stop com.lumen.app > /dev/null 2>&1
    "$ADB" shell am start -n com.lumen.app/.MainActivity > /dev/null 2>&1
    sleep 4
    "$ADB" exec-out screencap -p > "$OUT/android.png" 2>/dev/null
  fi
  # 应用刚启动时 dump 可能拿不到内容，最多重试 3 次
  for attempt in 1 2 3; do
    "$ADB" shell uiautomator dump /data/local/tmp/ui.xml > /dev/null 2>&1
    "$ADB" shell cat /data/local/tmp/ui.xml > "$OUT/ui.xml" 2>/dev/null
    if [ -s "$OUT/ui.xml" ]; then
      break
    fi
    sleep 2
  done

  if [ "$APP" = "todo" ]; then
    python3 tools/parity.py centered "$OUT/ui.xml" "待办" 2 > "$OUT/center.log" 2>&1
    case $? in
      0) record pass "徽标文字垂直居中：$(tail -1 "$OUT/center.log")" ;;
      2) record skip "徽标居中：$(tail -1 "$OUT/center.log")" ;;
      *) record fail "徽标文字垂直居中：$(tail -1 "$OUT/center.log")" ;;
    esac
  else
    record skip "当前示例没有徽标，跳过居中检查"
  fi

  rect=$(python3 - "$OUT/ui.xml" <<'PY'
import sys
import xml.etree.ElementTree as ET

try:
    root = ET.parse(sys.argv[1]).getroot()
except ET.ParseError:
    sys.exit(0)
for node in root.iter("node"):
    if node.get("class", "").endswith("EditText"):
        parts = node.get("bounds", "").replace("][", ",").strip("[]").split(",")
        if len(parts) == 4:
            x1, y1, x2, y2 = (int(value) for value in parts)
            print(x1, y1, x2 - x1, y2 - y1)
            break
PY
)
  if [ -n "$rect" ]; then
    python3 tools/parity.py placeholder "$OUT/android.png" 120 131 155 12 $rect > "$OUT/android-ph.log" 2>&1
    case $? in
      0) record pass "Android 占位符颜色：$(tail -1 "$OUT/android-ph.log")" ;;
      2) record skip "Android 占位符：$(tail -1 "$OUT/android-ph.log")" ;;
      *) record fail "Android 占位符颜色：$(tail -1 "$OUT/android-ph.log")" ;;
    esac
  else
    record skip "视图树里没有输入框，跳过占位符颜色检查"
  fi
else
  record skip "没有连接 Android 设备/模拟器"
fi

echo "== 3) iOS（模拟器截图采样）=="
if xcrun simctl list devices booted 2>/dev/null | grep -q Booted; then
  if xcrun simctl io booted screenshot "$OUT/ios.png" > /dev/null 2>&1; then
    python3 tools/parity.py placeholder "$OUT/ios.png" 120 131 155 12 > "$OUT/ios-ph.log" 2>&1
    case $? in
      0) record pass "iOS 占位符颜色：$(tail -1 "$OUT/ios-ph.log")" ;;
      2) record skip "iOS 占位符：$(tail -1 "$OUT/ios-ph.log")" ;;
      *) record fail "iOS 占位符颜色：$(tail -1 "$OUT/ios-ph.log")" ;;
    esac
  else
    record skip "iOS 截图失败"
  fi
else
  record skip "没有启动的 iOS 模拟器"
fi

echo "== 4) macOS =="
record skip "macOS 需要窗口截图（用 mac.sh 打开后人工核对，见 docs/platform-matrix.md）"

total=$((pass + fail + skip))
echo
echo "== 结果：${pass} 通过 / ${fail} 失败 / ${skip} 跳过（共 ${total} 项，示例 ${APP}）=="
echo "   中间产物：$OUT"
if [ "$fail" -gt 0 ]; then
  exit 1
fi
if [ "$pass" -eq 0 ]; then
  exit 2
fi
exit 0
