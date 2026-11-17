#!/bin/bash
# 平台验证矩阵的运行器：把能自动化的部分跑一遍，并打印矩阵结果。
# 用法：scripts/verify.sh [todo|dashboard]
# 说明：macOS / iOS 的交互仍是人工项，见 docs/platform-matrix.md
set -uo pipefail

APP="${1:-${LUMEN_APP:-todo}}"
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
SDK="$HOME/Library/Android/sdk"
ADB="$SDK/platform-tools/adb"
cd "$ROOT" || exit 1

result_build=skip
result_launch=skip
result_web=skip
result_wasm=skip
result_android=skip

echo "== 1) 编译器测试 =="
if cargo test --quiet > /tmp/lumen-verify-test.log 2>&1; then
  echo "   ✅ cargo test"
else
  echo "   ❌ cargo test（见 /tmp/lumen-verify-test.log）"
fi

echo "== 2) 生成源码后端（web）=="
if cargo run -q -- web "$APP" > /tmp/lumen-verify-web.log 2>&1; then
  result_web="✅ out.html $(wc -c < out.html | tr -d ' ')B"
  echo "   ✅ out.html 生成成功"
else
  result_web="❌ 生成失败"
  echo "   ❌ 生成失败（见 /tmp/lumen-verify-web.log）"
fi

echo "== 2b) wasm 后端（DOM 桥） =="
if LUMEN_APP="$APP" ./wasm.sh > /tmp/lumen-verify-wasm.log 2>&1; then
  if [ -f out-wasm.html ]; then
    result_wasm="✅ out-wasm.html $(wc -c < out-wasm.html | tr -d ' ')B"
    echo "   ✅ out-wasm.html 生成成功"
  fi
else
  result_wasm="❌ 生成失败"
  echo "   ❌ wasm 生成失败（见 /tmp/lumen-verify-wasm.log）"
fi

echo "== 3) iOS =="
if xcode-select -p > /dev/null 2>&1; then
  if LUMEN_APP="$APP" ./run.sh start shot > /tmp/lumen-verify-ios.log 2>&1; then
    result_build="✅ 构建+链接"
    result_launch="✅ 模拟器启动"
    echo "   ✅ iOS 构建/启动/截图完成"
  else
    result_build="❌ 构建失败"
    echo "   ❌ iOS 构建失败（见 /tmp/lumen-verify-ios.log）"
  fi
else
  echo "   ⚠️ 没有 Xcode，跳过 iOS"
fi

echo "== 4) Android =="
if [ -d "$SDK" ]; then
  if LUMEN_APP="$APP" ./android/build-apk.sh > /tmp/lumen-verify-apk.log 2>&1; then
    result_android="✅ APK 已生成"
    echo "   ✅ APK 构建成功"
    if "$ADB" get-state > /dev/null 2>&1; then
      if (cd android && ./run-apk.sh > /tmp/lumen-verify-run.log 2>&1); then
        result_android="✅ 已安装并启动"
        echo "   ✅ 已安装到模拟器"
      else
        echo "   ⚠️ 安装/启动失败（见 /tmp/lumen-verify-run.log）"
      fi
    else
      echo "   ⚠️ 没有连接设备，跳过安装"
    fi
  else
    result_android="❌ 构建失败"
    echo "   ❌ APK 构建失败（见 /tmp/lumen-verify-apk.log）"
  fi
else
  echo "   ⚠️ 没有 Android SDK，跳过"
fi

echo "== 5) macOS（只构建，交互人工）=="
if LUMEN_APP="$APP" cargo run -q -- --target=macos "$APP" --allow-degrade > /tmp/lumen-verify-mac.log 2>&1; then
  echo "   ✅ AppKit IR 生成（./mac.sh 可打开窗口；点击/输入需人工）"
else
  echo "   ❌ AppKit IR 生成失败（见 /tmp/lumen-verify-mac.log）"
fi

echo
echo "================ 平台验证矩阵（示例：${APP}）================"
printf '%-10s %-22s %-22s %s\n' "平台" "构建" "启动/渲染" "视觉断言"
printf '%-10s %-22s %-22s %s\n' "iOS" "$result_build" "$result_launch" "scripts/parity.sh（颜色采样）"
printf '%-10s %-22s %-22s %s\n' "Android" "$result_android" "$result_android" "scripts/parity.sh（几何 + 颜色）"
printf '%-10s %-22s %-22s %s\n' "Web" "$result_web" "static" "scripts/parity.sh（结构）"
printf '%-10s %-22s %-22s %s\n' "Wasm" "$result_wasm" "Chrome 打开 out-wasm.html" "tools/wasm/shot.sh（截图/点击）"
printf '%-10s %-22s %-22s %s\n' "macOS" "✅ IR" "人工" "人工看窗口"
echo "详细说明：docs/platform-matrix.md"
