#!/bin/bash
# 启动模拟器（如未运行）→ 安装 → 启动 → 截图
set -e
SDK="$HOME/Library/Android/sdk"
ADB="$SDK/platform-tools/adb"
AVD="${AVD:-Medium_Phone_API_36}"
APK="$(cd "$(dirname "$0")" && pwd)/build/lumen.apk"

EMU_FLAGS="-avd $AVD -no-audio -no-snapshot -gpu swiftshader_indirect"
# 默认开窗口；后台跑（CI/只截图）时用 HEADLESS=1
[ "${HEADLESS:-0}" = "1" ] && EMU_FLAGS="$EMU_FLAGS -no-window"

if ! "$ADB" get-state > /dev/null 2>&1; then
  echo "==> 启动模拟器 $AVD（HEADLESS=${HEADLESS:-0}）"
  "$SDK/emulator/emulator" $EMU_FLAGS > /tmp/lumen-emulator.log 2>&1 &
  "$ADB" wait-for-device
  for _ in $(seq 1 40); do
    [ "$("$ADB" shell getprop sys.boot_completed 2>/dev/null | tr -d '\r')" = "1" ] && break
    sleep 5
  done
fi

"$ADB" install -r "$APK" | tail -1
"$ADB" shell am force-stop com.lumen.app
"$ADB" shell am start -n com.lumen.app/.MainActivity | tail -1
sleep 3
"$ADB" exec-out screencap -p > /tmp/lumen-android.png
echo "✅ /tmp/lumen-android.png"
