#!/bin/bash
# 一步：IR(wasm32) -> out.wasm -> 自包含 out-wasm.html
set -e
APP="${LUMEN_APP:-${1:-dashboard}}"
LLD=$(ls "$HOME"/.rustup/toolchains/*/lib/rustlib/aarch64-apple-darwin/bin/rust-lld | head -1)
CLANG="${LUMEN_CLANG:-/opt/homebrew/opt/llvm/bin/clang}"

echo "==> [1/3] cargo run -- wasm $APP"
cargo run -- wasm "$APP" > /dev/null

echo "==> [2/3] wasm32 编译 + 链接"
"$CLANG" --target=wasm32-unknown-unknown -c out.ll -o out.wasm.o
"$LLD" -flavor wasm --no-entry --export-all --allow-undefined out.wasm.o -o out.wasm
echo "    out.wasm $(stat -f%z out.wasm) bytes"

echo "==> [3/3] 打包 out-wasm.html"
# wasm 直接 base64 内联：file:// 打开时 fetch 本地文件会被 CORS 挡住
BASE64=$(base64 < out.wasm | tr -d '\n')
{
  echo '<!DOCTYPE html><html lang="zh"><head><meta charset="utf-8">'
  echo '<meta name="viewport" content="width=device-width, initial-scale=1">'
  echo '<title>Lumen (wasm)</title><style>'
  echo 'html,body{margin:0;height:100%;background:#fff}'
  echo 'body{font-family:-apple-system,"PingFang SC","Microsoft YaHei",Roboto,sans-serif}'
  echo '#root{position:relative;width:412px;height:900px;overflow:hidden}'
  echo 'input::placeholder{color:var(--ph,rgb(120,131,155));opacity:1}'
  echo '</style></head><body>'
  echo '<div id="root"></div>'
  echo '<script>'
  cat tools/wasm/bridge.js
  echo '</script>'
  echo '<script>window.lumenStart(window.lumenDecodeBase64("'"$BASE64"'"))</script>'
  echo '</body></html>'
} > out-wasm.html
echo "✅ out-wasm.html ($(stat -f%z out-wasm.html) bytes)"
