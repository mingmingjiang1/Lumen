# 平台验证矩阵

这张表说明每个平台**怎么验证、哪些是自动化的、哪些只能人工**。改动涉及编译器后端时，
至少跑一遍 `scripts/verify.sh`；改 UI 语义时再跑 `scripts/parity.sh`。

## 矩阵

| 平台 | 构建 | 启动/渲染 | 视觉断言 | 交互 | 自动化程度 |
| --- | --- | --- | --- | --- | --- |
| iOS（UIKit） | `cargo run -- <app>` + `run.sh shot` | 模拟器启动 + 截图 | `scripts/parity.sh`：占位符颜色采样 | 需人工（模拟器里点） | 构建/启动/截图/颜色自动，交互人工 |
| macOS（AppKit） | `mac.sh`（`--allow-degrade`） | 打开窗口 | 需人工看窗口 | 需人工 | 构建自动，其余人工 |
| Android（View） | `android/build-apk.sh` | `run-apk.sh` 装到模拟器 | `scripts/parity.sh`：视图树几何 + 占位符颜色 | `uiautomator` 坐标点击可脚本化 | 构建/启动/几何/颜色/交互全自动 |
| Web（生成源码） | `cargo run -- web <app>` → `out.html` | Chrome 打开生成页面 | `scripts/parity.sh`：HTML 结构断言 | 标量状态、条件渲染、按钮/输入/焦点/滚动事件 | 构建/结构自动，交互可在浏览器验证 |
| Wasm（DOM 桥） | `LUMEN_APP=<app> ./wasm.sh` → `out-wasm.html` | Chrome headless 打开页面 | 截图对照 web 参考页（`tools/wasm/shot.sh`） | 按钮/列表重建/滚动/透明度已接 | 全自动（截图 + 可选点击冒烟） |

## 每条平台命令

```bash
# 1) 编译器测试（含两个内置示例 × 三个原生后端）
cargo test

# 2) 生成源码型后端：交互式 HTML
cargo run -- web todo        # 写 out.html
open out.html

# 3) iOS
LUMEN_APP=todo ./run.sh start shot
xcrun simctl io booted screenshot /tmp/ios.png

# 4) Android
LUMEN_APP=todo android/build-apk.sh
android/run-apk.sh
adb shell uiautomator dump /sdcard/ui.xml && adb shell cat /sdcard/ui.xml > /tmp/ui.xml
adb exec-out screencap -p > /tmp/android.png

# 5) macOS
LUMEN_APP=todo ./mac.sh      # 启动后人工点击「完成 / 展开 / 输入框」
```

## 哪些只能人工，以及为什么

| 项 | 原因 | 现在的替代手段 |
| --- | --- | --- |
| macOS 交互 | 本机 `osascript`/合成鼠标事件受辅助功能权限限制，无法稳定驱动 | 人工点击；窗口截图可用 `screencapture -l <窗口id>` |
| iOS 交互 | `simctl` 只能截图/安装，没有坐标点击接口 | 人工点击；几何/颜色靠截图采样 |
| Web 像素级对照 | Chrome headless 的最小窗口是 500x500，无法直接模拟 412 宽的机型 | 目前只做 HTML 结构断言（`tools/parity.py html-has`） |
| 字体渲染 | 三端字体族不同（SF / Roboto / 系统无衬线） | 允许差异，只对比字号与行高 |

## 已知不等价（能力表已登记）

`src/capabilities.rs` 里每个后端登记了例外项，编译时会校验：

| 能力 | iOS | macOS | Android | Web | Wasm |
| --- | --- | --- | --- | --- |
| `onScroll`（滚动事件） | ✅ `scrollViewDidScroll:` | ✅ `NSViewBoundsDidChange` 通知 | ✅ `nativeOnScroll` | ✅ DOM scroll 事件 | ✅ `dom.scrollCallback` → `lumen_scroll` |
| `opacity`（绑定数值单元） | ✅ `setAlpha:` | ✅ `setAlphaValue:` | ✅ `setAlpha` | ✅ 状态变化同步到 CSS | ✅ `dom.alpha` |
| `ScrollView` | ✅ | ✅ | ✅ | ✅ | ✅（内容层 + `overflow-y:auto`） |
| `onClick` | ✅ | ✅ | ✅ | ✅ DOM click 事件 | ✅ `dom.callback` → `lumen_action` |
| `onInput` / `onFocus` | ✅ | ❌ 未实现 | ✅ | ✅ DOM input / focus 事件 | ⚠️ 只渲染外观 |
| `List`（运行期建行） | ✅ | ✅ | ✅ | ⚠️ 只输出初始行，不能动态增删/更新 | ✅ 整表重建 |
| `Image` | ✅ | ✅ | ⚠️ 只占位 | ⚠️ 只占位 | ⚠️ 只占位 |
| `ignoresSafeArea` / `backgroundExtendsTo` | ✅ | ⚠️ 按普通背景 | ⚠️ 宿主补状态栏内边距 | ⚠️ 忽略 | ⚠️ 忽略 |
| `NestedList`（树形行表） | ✅ 平铺 + 深度缩进 | ✅ 同 iOS | ✅ 平铺 + 深度缩进 | ⚠️ 只输出初始行 | ⚠️ 平铺 + 深度缩进 |
| `RecursiveComponent`（运行期递归槽位） | ✅ 子树内嵌 | ✅ 同 iOS | ✅ 子树内嵌 | ⚠️ 只输出初始树 | ⚠️ 子树内嵌（未逐项验证） |

`--allow-degrade` 会把 ❌ 降级成 warning（`mac.sh` 就是这么跑两个示例的）。

## 新增平台/能力时要做的事

1. 在 `src/capabilities.rs` 的 `KNOWN_KEYS` 里登记新的能力键；
2. 在后端的 `exceptions` 里声明例外（支持就不写）；
3. 在 `scripts/parity.sh` 里加一条能自动断言的检查（几何或颜色采样）；
4. 更新本文档的矩阵与「已知不等价」表。
