#!/bin/bash
set -e

# ---------- 配置 ----------
APP_NAME="Demo"
BUNDLE_ID="com.example.demo"
SIM_DEVICE="iPhone 16 Pro"
RESOURCES_DIR="resources"

# ---------- 解析参数 ----------
START_SIM=0
DO_SHOT=0
for arg in "$@"; do
    case "$arg" in
        start) START_SIM=1 ;;
        shot|screenshot) DO_SHOT=1 ;;
        *)
            echo "用法: $0 [start] [shot]"
            echo "  start  启动模拟器 ${SIM_DEVICE}"
            echo "  shot   安装并启动应用后截图"
            exit 1
            ;;
    esac
done

# ---------- 1. 生成 out.ll ----------
echo "==> [1/5] cargo run (生成 out.ll, app=${LUMEN_APP:-dashboard})"
cargo run -- "${LUMEN_APP:-dashboard}"

# ---------- 2. 链接出可执行文件 ----------
echo "==> [2/5] clang 链接 ${APP_NAME}"
clang -arch arm64 \
  -isysroot "$(xcode-select -p)/Platforms/iPhoneSimulator.platform/Developer/SDKs/iPhoneSimulator.sdk" \
  out.ll -o "${APP_NAME}" \
  -lobjc -framework UIKit \
  -fno-objc-msgsend-selector-stubs

# ---------- 3. 打包 .app ----------
echo "==> [3/5] 打包 ${APP_NAME}.app"
mkdir -p "${APP_NAME}.app"
rm -f "${APP_NAME}.app/${APP_NAME}"   # 只删可执行文件
# Info.plist 和其他资源保留

cp "${APP_NAME}" "${APP_NAME}.app/${APP_NAME}"
chmod +x "${APP_NAME}.app/${APP_NAME}"

# 首次运行才生成默认 Info.plist
if [ ! -f "${APP_NAME}.app/Info.plist" ]; then
    cat > "${APP_NAME}.app/Info.plist" <<EOF
    ...
EOF
fi

if [ -d "${RESOURCES_DIR}" ]; then
    echo "    拷贝资源: ${RESOURCES_DIR}/*"
    cp "${RESOURCES_DIR}"/* "${APP_NAME}.app/" 2>/dev/null || true
fi

# ---------- 4. 启动模拟器（可选） + 安装 ----------
if [ "${START_SIM}" -eq 1 ]; then
    echo "==> [4/5] 启动模拟器 ${SIM_DEVICE}"
    xcrun simctl boot "${SIM_DEVICE}" 2>/dev/null || echo "    模拟器可能已启动，继续"
    open -a Simulator
    xcrun simctl bootstatus booted -b
else
    echo "==> [4/5] 跳过启动模拟器（未传 start），直接安装到已启动的 booted 设备"
fi

echo "==> 安装 ${APP_NAME}.app"
xcrun simctl uninstall booted "${BUNDLE_ID}" 2>/dev/null || true
xcrun simctl install booted "${APP_NAME}.app"

echo "==> 启动应用"
xcrun simctl launch booted "${BUNDLE_ID}" > /dev/null
echo "    PID: $(xcrun simctl launch booted "${BUNDLE_ID}" 2>/dev/null | awk '{print $2}' || echo 'running')"

# ---------- 5. 截图（可选） ----------
if [ "${DO_SHOT}" -eq 1 ]; then
    echo "==> [5/5] 截图"
    sleep 1   # 等 UI 渲染一帧
    SHOT_FILE="demo_$(date +%H%M%S).png"
    xcrun simctl io booted screenshot "${SHOT_FILE}"
    echo "    已保存: ${SHOT_FILE}"
    open "${SHOT_FILE}"
else
    echo "==> [5/5] 跳过截图（未传 shot）"
fi

echo ""
echo "✅ 完成"