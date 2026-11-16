#!/bin/bash
# 手工出安卓 APK（不依赖 Gradle）：
#   cargo run -- android -> out.ll --NDK clang--> liblumen.so
#   AndroidManifest + src --aapt2/javac/d8--> classes.dex
#   zip + zipalign + apksigner --> build/lumen.apk
#
# 默认用编译器生成的 IR（liblumen.so 里就是整套界面构建与状态更新）。
# 想看早期手写 JNI 切片时用 FROM_HOST_C=1，它会编译 jni/host.c 那份原型。
set -e

SDK="$HOME/Library/Android/sdk"
BT=$(ls -d "$SDK"/build-tools/* | tail -1)
PLAT=$(ls -d "$SDK"/platforms/* | tail -1)
NDK=$(ls -d "$SDK"/ndk/* | tail -1)
TC="$NDK/toolchains/llvm/prebuilt/darwin-x86_64"

# ---------- 选 JDK ----------
# 机器上常常同时装着 JDK 8（老项目/其它工具链）和 JDK 17+，JAVA_HOME 或 java_home 的
# 默认值不一定指对；而 Android build-tools 的 d8/apksigner 又要求 Java 11+ 的 class 文件，
# 所以这里按候选依次探测，挑一个 >= 11 的，并据此决定 javac 用哪个 --release。
jdk_major() {
  [ -x "$1/bin/javac" ] || return 1
  "$1/bin/javac" -version 2>&1 | sed -n 's/.*javac \([0-9][0-9]*\).*/\1/p'
}

CHOOSE_JAVA_HOME=""
CHOOSE_JAVA_MAJOR=0
for candidate in \
  "${LUMEN_JAVA_HOME:-}" \
  "$(/usr/libexec/java_home -v 17 2>/dev/null)" \
  "$(/usr/libexec/java_home -v 11 2>/dev/null)" \
  "${JAVA_HOME:-}" \
  "/Applications/Android Studio.app/Contents/jbr/Contents/Home" \
  /opt/homebrew/opt/openjdk/libexec/openjdk.jdk/Contents/Home \
  "$(/usr/libexec/java_home 2>/dev/null)"
do
  [ -n "$candidate" ] || continue
  major=$(jdk_major "$candidate") || continue
  if [ "$major" -ge 11 ]; then
    CHOOSE_JAVA_HOME="$candidate"
    CHOOSE_JAVA_MAJOR="$major"
    break
  fi
done

if [ -z "$CHOOSE_JAVA_HOME" ]; then
  echo "❌ 没找到 JDK 11+（Android build-tools 的 d8/apksigner 需要 Java 11+）。"
  echo "   当前 PATH 上的 javac: $(javac -version 2>&1 | head -1)"
  echo "   装一个：brew install openjdk@21"
  echo "   或用环境变量指定：LUMEN_JAVA_HOME=/path/to/jdk ./android/build-apk.sh"
  exit 1
fi

JAVA_HOME="$CHOOSE_JAVA_HOME"
if [ "$CHOOSE_JAVA_MAJOR" -ge 17 ]; then
  JAVAC_FLAGS=(--release 17)
else
  JAVAC_FLAGS=(--release "$CHOOSE_JAVA_MAJOR")
fi
export JAVA_HOME
export PATH="$JAVA_HOME/bin:$PATH"

APP="$(cd "$(dirname "$0")" && pwd)"
OUT="$APP/build"
APK="$OUT/lumen.apk"
KS="$HOME/.android/lumen-debug.keystore"

rm -rf "$OUT"
mkdir -p "$OUT/classes" "$OUT/lib/arm64-v8a"

if [ "${FROM_HOST_C:-0}" = "1" ]; then
  echo "==> [1/5] 编译 jni/*.c -> liblumen.so（手写原型）"
  "$TC/bin/clang" --target=aarch64-linux-android26 --sysroot="$TC/sysroot" \
    -shared -fPIC -I"$TC/sysroot/usr/include" \
    "$APP"/jni/*.c -o "$OUT/lib/arm64-v8a/liblumen.so"
else
  echo "==> [1/5] cargo run -- android ${LUMEN_APP:-dashboard} 生成 out.ll"
  (cd "$APP/.." && cargo run -q -- android "${LUMEN_APP:-dashboard}" > /dev/null)
  echo "    clang 链接 out.ll -> liblumen.so"
  # LLVM 23 会把部分 double 常量打印成 f0x 前缀，Xcode 的 clang 不认
  sed -i '' 's/f0x/0x/g' "$APP/../out.ll"
  "$TC/bin/clang" --target=aarch64-linux-android26 --sysroot="$TC/sysroot" \
    -shared -fPIC "$APP/../out.ll" -o "$OUT/lib/arm64-v8a/liblumen.so"
fi

echo "==> [2/5] aapt2 link"
"$BT/aapt2" link --manifest "$APP/AndroidManifest.xml" -I "$PLAT/android.jar" \
  -o "$OUT/base.apk" --min-sdk-version 26

echo "==> [3/5] javac + d8（JDK $(jdk_major "$JAVA_HOME" || echo '?'), flags: ${JAVAC_FLAGS[*]}）"
"$JAVA_HOME/bin/javac" "${JAVAC_FLAGS[@]}" -classpath "$PLAT/android.jar" \
  -d "$OUT/classes" $(find "$APP/src" -name '*.java')
"$BT/d8" --lib "$PLAT/android.jar" --min-api 26 --output "$OUT" \
  $(find "$OUT/classes" -name '*.class')

echo "==> [4/5] 打包"
cd "$OUT"
zip -q -j base.apk classes.dex
zip -q -r base.apk lib
"$BT/zipalign" -f 4 base.apk aligned.apk

echo "==> [5/5] 签名"
if [ ! -f "$KS" ]; then
  "$JAVA_HOME/bin/keytool" -genkeypair -keystore "$KS" -storepass lumen123 -keypass lumen123 \
    -alias lumen -keyalg RSA -keysize 2048 -validity 3650 -dname "CN=Lumen" > /dev/null 2>&1
fi
"$BT/apksigner" sign --ks "$KS" --ks-pass pass:lumen123 --key-pass pass:lumen123 \
  --out "$APK" aligned.apk
echo "✅ $APK"
