#!/bin/bash
set -e

# ---------- 生成 AppKit IR ----------
echo "==> [1/3] cargo run -- macos ${LUMEN_APP:-dashboard}（--allow-degrade：macOS 的 onFocus 还没接）"
cargo run -- --target=macos "${LUMEN_APP:-dashboard}" --allow-degrade

# LLVM 23 会把部分 double 常量打印成 f0x 前缀，Xcode 的 clang 不认，链接前归一化
sed -i '' 's/f0x/0x/g' out.ll

# ---------- 链接 ----------
echo "==> [2/3] clang 链接 Lumen-mac"
clang -arch arm64 \
  -isysroot "$(xcrun --show-sdk-path)" \
  out.ll -o Lumen-mac \
  -lobjc -framework Cocoa \
  -fno-objc-msgsend-selector-stubs

# ---------- 运行 ----------
echo "==> [3/3] 启动 Lumen-mac"
./Lumen-mac
