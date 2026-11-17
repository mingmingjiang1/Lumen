**我的建议：采用 React 熟悉的 JSX/声明式写法，但不要把框架做成 React 的复刻。**内部更适合走 Svelte 式编译时更新 + 细粒度状态，而不是依赖 Virtual DOM。这样既能降低用户的学习成本，也更贴合你现在“解析代码、生成各平台 UI”的编译器路线。
Flex 的位置与间距目前仍由框架计算后转成 UIKit 约束
风格	优点	代价
React	JSX 普及、生态和开发者熟悉；组件组合直观	如果照搬 hooks、生命周期和 Virtual DOM，需要维护复杂运行时；各端还得实现一致的运行语义
SwiftUI	声明简洁，和 Apple 原生 UI 很贴合	API 和心智模型偏 Apple；跨到 Android、Web、Harmony、Windows 时难以保持一致
Flutter	Widget 组合统一，布局行为可控	更像自带 UI 引擎；若用各平台原生控件实现，同一套组件语义不容易完全对齐
Svelte	编译时处理更新，运行时相对轻；适合代码生成	语法与生态认知度不如 JSX；若直接采用 Svelte 语法，也未必适合你的 Rust/TSX 工具链
我会把这几个层次分开设计：

如果后续增加 Android、Web 等后端，建议共享状态语义，各后端分别实现状态更新，不必要求每个平台都用同一种状态 API。

共享层规定行为：状态的初值、赋值、事件触发、哪些 UI 随状态变化而更新。
后端层按目标平台落地：UIKit 可像现在这样生成存储和事件回调；Android 可生成 Kotlin/原生控件更新逻辑；Web 可生成 JavaScript/DOM 更新逻辑。若目标是 SwiftUI、Jetpack Compose 等声明式框架，也可以选择接入它们的状态机制。
这样不同后端内部实现可以不同，但同一份程序的状态行为应一致。当前 UIKit 用全局变量保存单个状态并更新关联标签，适合作为原型；扩展前最好把这套状态语义从 UIKit 代码生成中抽出来，避免 Android、Web 后端各自定义一套不一致的规则。当前实现位于 src/main.rs。

**开发语法：**保留 JSX，支持组件、属性、事件和清晰的状态 API；这只是表层语法，不代表要兼容 React。
**公共 UI 语义：**定义平台无关的组件、布局、样式和交互规则。布局最好明确采用 CSS Flexbox 的规则，而不是让不同后端各自猜测。
**状态更新：**用编译器识别状态依赖，只更新受影响的控件。先不实现 React hooks 的完整规则，也不必先引入 Virtual DOM。
**平台后端：**分别生成 UIKit、Android、ArkUI、DOM/CSS、AppKit、WinUI 等实现；共享 API 和布局规则，但允许平台有明确记录的差异。
也就是说，**借 React 的语法熟悉度，借 Svelte 的编译思路；不要把 SwiftUI 或 Flutter 的 API 直接当成跨平台标准。**SwiftUI/Flutter 可以作为各平台后端的实现参考。

这也意味着要尽早稳定跨平台契约：哪些组件、样式和行为保证一致，哪些只在特定平台可用。否则后续容易变成“看起来是同一套 API，实际每个平台表现不同”。

目前 src/main.rs 已经把 TSX 解析、IR 和 UIKit 代码生成集中在一起。长期来看，建议逐步把它们拆成独立的语法解析、平台无关 IR、布局计算、平台后端，再扩展目标平台。

## 响应式原型（阶段 1）

当前 UIKit 后端支持一个标量 `state(...)` 声明。状态引用会生成可更新的 UILabel；更新只触碰引用该状态的标签，不重建整棵视图树。

```tsx
function App() {
    const count = state(0);
    return <View>
        <Text>{count}</Text>
        <Button onClick={() => count++}>增加</Button>
        <Button onClick={() => count--}>减少</Button>
        <Button onClick={() => count = 10}>设为 10</Button>
    </View>;
}
```

输入绑定使用字符串状态：

```tsx
function App() {
    const note = state("");
    return <View>
        <Text>{note}</Text>
        <Input onInput={(value) => note = value} />
    </View>;
}
```

`onFocus` 可以将一个布尔状态从 `false` 设为 `true`：

```tsx
function App() {
    const focused = state(false);
    return <View>
        <Input onFocus={() => focused = true} />
        <Text>{focused}</Text>
    </View>;
}
```

首阶段的约束：每个 App 只支持一个状态声明；状态初值为整数、字符串或布尔值。`onClick` 支持递增、递减和赋标量常量；`onInput` 支持把输入值赋给该状态；`onFocus` 支持空回调或将 `state(false)` 设为 `true`。阶段 3 起不再保留旧的 `useState` 写法，统一使用 `state(...)`。

## 响应式原型（阶段 2）

在阶段 1 的单状态基础上，当前还支持自定义函数组件、静态标量 props、条件渲染，以及组件级 `onMount/onUnmount`。

```tsx
function Banner({ title, height }) {
    onMount(() => {});
    onUnmount(() => {});
    return <View style={{ height }}>
        <Text>{title}</Text>
    </View>;
}

function App() {
    const shown = state(false);
    return <View>
        {shown && <Banner title="Welcome" height={72} />}
        <Button onClick={() => shown = true}>显示</Button>
        <Button onClick={() => shown = false}>隐藏</Button>
    </View>;
}
```

组件使用普通函数声明，可通过 `function Card({ title })` 解构 props 或用 `function Card(props)` 访问 props；当前支持字符串、整数和布尔值静态 props。条件表达式支持布尔条件的 `&&` 与三元表达式。条件切换会真实添加或移除 UIKit 视图，并触发布局更新；包含条件后代的 Flex 容器使用 `UIStackView`，`ScrollView` 条件内容通过 Auto Layout 更新内容尺寸。

生命周期回调支持空回调或阶段 1 已支持的状态赋值操作；静态组件在界面创建时挂载，条件组件会在对应分支加入/移除时触发挂载/卸载。暂不支持递归组件、动态 props 驱动的样式更新或任意生命周期回调。

## 响应式原型（阶段 3）：多状态与结构化状态

状态不再限制为单个标量：一个 App 可以声明任意多个 `state(...)`，每个声明也可以是一层对象字面量。

```tsx
function App() {
    const count = state(0);
    const profile = state({ name: "guest", visits: 0 });
    return <View>
        <Text>{count}</Text>
        <Text>{profile.name}</Text>
        <Text>{profile.visits}</Text>
        {profile.active ? <Text>online</Text> : <Text>offline</Text>}
        <Button onClick={() => count++}>+1</Button>
        <Button onClick={() => profile.visits++}>visit</Button>
        <Button onClick={() => profile.name = "Ada"}>rename</Button>
        <Input onInput={(value) => profile.name = value} />
    </View>;
}
```

对象状态会被展开成**状态单元**（cell）：标量声明就是一个单元，对象声明是每个字段各一个单元（`profile.name`、`profile.visits`）。每个单元有自己的存储全局变量和标签列表，所以 `profile.visits++` 只刷新引用 `profile.visits` 的标签和条件分支，`count`、`profile.name` 完全不受影响。

- 赋值形式：`x++` / `x--`、`x = <标量字面量>`、`x = x + 1` / `x - 1`；
- `onInput={(value) => 文本单元 = value}` 把输入框文本写入字符串单元；
- `onFocus={() => 布尔单元 = true}` 把布尔单元置为真；
- 单元类型不匹配（例如把输入文本写进布尔单元）会在编译期报错；
- 仍不支持：嵌套对象、跨单元的计算表达式、任意函数体；数组和列表见下面的阶段 4。
- 演示见 `app/App.tsx`：一个布尔状态控制条件渲染，一个对象状态（`name` + `visits`）各自独立更新。

## 响应式原型（阶段 4）：数组状态、列表渲染与多语句 handler

```tsx
function App() {
    const items = state([
        { id: 1, title: "写周报", done: false },
        { id: 2, title: "评审 PR", done: false }
    ]);
    const draft = state("");
    const nextId = state(3);
    return <View>
        <Input onInput={(value) => draft = value} />
        <Button onClick={() => {
            items.push({ id: nextId, title: draft, done: false });
            nextId++;
            draft = "";
        }}>添加</Button>
        {items.map((item, index) => (
            <View>
                <Text>{item.title}</Text>
                <Button onClick={() => items.removeAt(index)}>删除</Button>
            </View>
        ))}
    </View>;
}
```

- `state([...])` 是对象数组状态：字段集由初始值的第一行决定（所以不能用空数组开头），字段只能是
  number / string / boolean；`{items.length}` 是列表长度单元，push / removeAt / 清空都会同步；
- 对象状态可以多层嵌套：`state({ meta: { level: 1 } })` 展开成 `meta.level` 单元；
- `{items.map((item, index) => <Row />)}` 编译成一个列表槽位，行内 `{item.title}` 绑定到当前行字段；
- 列表动作：`items.push({...})`（字段值可以写字面量，也可以写状态单元名）、行内
  `item.field = <字面量>`、`items.removeAt(index)`、整表赋值 `items = []`；
- 整对象覆盖 `profile = { name: "Ada", visits: 7 }` 会逐单元写入；
- 跨单元数值赋值：`a = b`、`a = b + 1`、`a = b - 1`；
- 组件可以接收行数据：`<Row item={item} />`（组件体里 `{item.title}`、`item.done = true` 都作用在当前行）、
  或只收单个字段 `<Row title={item.title} />`；再传 `index={index}` 就能在组件里 `items.removeAt(index)`；
- 行内条件渲染：`{item.done ? <Text>已完成</Text> : <Text>待办</Text>}`（按当前行布尔字段分支，重建时重算）；
- 组件体里声明的 `state(...)` 按实例分配单元：两个 `<Counter />` 各自独立；模块级 / App 里的 `state(...)` 仍是共享单元；
  组件内的列表状态也按实例分配（行存储与长度单元都在实例前缀下）；
- 同一个列表状态可以渲染多次（每次渲染各自持有容器，数据列与长度单元共享）；
- 表达式文本：`{a + b}` / `{a - b}` 编译成依赖这两个单元的标签，任一单元变化就重算；
- handler 里可以写多条语句，`() => { a = true; b++ }` 会编译成按源码顺序执行的动作序列，每条各自刷新
  （标量单元只刷新自己那组标签，列表动作整表刷新）；
- 运行模型是「数据列 + 运行期建行」：字段数据按列存放，任何列表动作改完数据后调一次重建函数，
  清空现有行、按当前行数逐行调用行模板构造器重新建视图；不做虚拟 DOM，行数不设编译期上限（数据容量 256 条）；
- 行模板上的 `marginTop` / `marginBottom` / `marginHorizontal` 等会作用在行与行之间（实现上是给每行包一层
  wrapper，把 margin 变成行相对 wrapper 的内缩），行高按「内容高度 + 上下 margin」占位；
- 列表放进 `ScrollView` 时，`ScrollView` 要给固定高度（`percent` 或点数）；`flexGrow` 目前拿不到视口高度，
  会退化成内容高度，于是不会滚动；
- 仍不支持：行内图片、嵌套列表（列表里再渲染列表，需要按行嵌套存储）、跨行计算、字符串拼接类插值
  （表达式文本目前只支持数值 `+` / `-`）；组件递归展开（编译期展开，递归会直接报错）；
- Android 后端也支持列表渲染：native 侧按字段存 `char*` 数据列，行在 `list_rebuild_*` 里按行数循环新建
  `LinearLayout`，行内按钮把行号编码进点击 id（`1_000_000 + action*1024 + row`）再解码；
  列表容器和 Activity 都以 **global reference** 持有（local reference 跨 native 调用会失效，这是之前的崩溃点）；
- Android 的差异：表达式文本只渲染初始值（不会随单元变化重算），行内条件在行构建时静态判断；
- 平台：iOS / macOS / web 已实现（web 的列表只输出初始行，不能动态增删/更新），Android 后端还没有列表渲染，
  能力表会直接报错（`--allow-degrade` 时退化成空容器）。

## 内置示例

仓库里带三个示例，三端共用同一份 TSX，编译时用名字选（默认 `dashboard`）：

| 名称 | 源码 | 演示的能力 |
| --- | --- | --- |
| `dashboard` | `app/App.tsx` | 条件渲染整块组件、对象状态、滚动渐显、绝对定位底栏 |
| `todo` | `app/Todo.tsx` | 清单页：4 项逐项勾选 / 撤销、输入框改写标题、番茄计数、归档卡片 |
| `list` | `app/List.tsx` | 数组状态 + `.map()` 行渲染、行内删除、push / 多语句 handler |

```bash
cargo run                              # dashboard，UIKit IR
cargo run -- todo                      # todo，UIKit IR
cargo run -- list                      # list，UIKit IR
cargo run -- macos todo                # todo，AppKit IR
cargo run -- android todo              # todo，Android IR
LUMEN_APP=todo ./run.sh shot           # todo，打包进 iOS 模拟器并截图
LUMEN_APP=list ./run.sh shot           # list，同上
LUMEN_APP=todo ./mac.sh                # todo，macOS 版
LUMEN_APP=todo android/build-apk.sh    # todo，打 APK（再跑 android/run-apk.sh 安装）
```

`app/tsconfig.json` 与 `app/lumen.d.ts` 覆盖三个示例，供编辑器补全。`state(...)` 的赋值写法本身
不符合 TS 的 `const` 语义，所以 `npx -p typescript tsc -p app` 会报这类已知误报（声明只描述子集，不参与编译）。

`todo` 刻意保留在数组能力之前的写法：清单固定 4 项，每项有自己的布尔单元，勾选 / 撤销是两个按钮。
需要「任意新增 / 删除待办」时用 `list` 示例：`items` 是数组状态，行由 `.map()` 渲染，
添加按钮的 handler 一次写多个状态（阶段 4）。

## 平台后端：iOS 与 macOS

IR、布局和状态语义是共享的，各后端只替换视图类名、样式落点、事件和启动方式。

```bash
cargo run                # 生成 UIKit IR（默认），配 run.sh 打包进模拟器
cargo run -- macos       # 生成 AppKit IR
./mac.sh                 # 一步：生成 + 链接 + 运行 macOS 版
```

| | iOS（UIKit） | macOS（AppKit） |
| --- | --- | --- |
| 启动 | `UIApplicationMain` + AppDelegate 启动回调 | `NSApplication` + 直接建 `NSWindow`（不依赖启动回调时机） |
| 容器 | `UIView` / `UIStackView` | `NSView` / `NSStackView` |
| 文本 | `UILabel` | `NSTextField`（去边框、只读、默认左对齐） |
| 按钮 | `UIButton`（titleColor / contentEdgeInsets） | `NSButton`（contentTintColor，无边框 + layer 背景） |
| 背景 / 圆角 | `setBackgroundColor:` + layer | `wantsLayer` + layer |
| 事件 | `addTarget:action:forControlEvents:` | `setTarget:` / `setAction:` |
| 安全区 | `safeAreaLayoutGuide`（状态栏 / home indicator） | 同名 API，窗口内容区通常没有额外 inset |

macOS 首版尚未支持：`onFocus` 事件，以及含条件子级的容器里文本的横向对齐（`NSStackView` 对齐映射还不准）。`ScrollView` 已经是 `NSScrollView` + `isFlipped` 文档视图，`onScroll` 靠 `NSViewBoundsDidChange` 通知回传位置。

`Input` 在 macOS 上用 `NSTextField` 渲染（占位符走 `setPlaceholderString:`），并且已经接上了
`onInput`：构建期把输入框设为 AppDelegate 的 delegate，回调用 `controlTextDidChange:` 按身份
找到对应输入框，再调它的 `stateActionN:`（取值用 `stringValue`）。`onFocus` 仍未接。

iOS/macOS 的一个已知限制：条件分支如果直接写 `<Text>`，槽位拿不到分支的宽高，分支会渲染成一条细线。
需要在外面包一层给定尺寸的 `View`（`app/Todo.tsx` 里的状态徽标就是这么写的）。Android 后端直接增删
分支视图，没有这个限制。

## 平台后端：Android

Android 后端和 iOS / macOS 走同一条路线：编译器把同一份 IR 落成 LLVM IR，只是从 ObjC 消息调用换成
JNI 调用，产物是 `liblumen.so`，由 `MainActivity` 在 `nativeCreate` 里交给 native 侧建控件。

```bash
cargo run -- android            # 生成 out.ll（target: aarch64-linux-android26）
android/build-apk.sh            # 生成 IR + 链 .so + 打包签名，出 android/build/lumen.apk
android/run-apk.sh              # 启动模拟器（未启动时）→ 安装 → 启动 → 截图
```

`build-apk.sh` 不依赖 Gradle：`out.ll` 用 NDK clang 链成 `liblumen.so`，`AndroidManifest` 与
`android/src` 走 aapt2 / javac / d8，最后 zipalign + apksigner。`FROM_HOST_C=1` 会改为编译
`android/jni/host.c`——那是接入完整 IR 之前手写的 JNI 参照原型。

需要 **JDK 11+**（android build-tools 的 d8/apksigner 要 Java 11 以上的 class 文件），推荐 17+。
脚本会自己从 `LUMEN_JAVA_HOME` / `/usr/libexec/java_home -v 17` / Android Studio 自带 JBR /
Homebrew 里挑一个可用的，即使 `JAVA_HOME` 指向 JDK 8 也不影响；确实没有时直接报错提示。

| | iOS（UIKit） | Android（View） |
| --- | --- | --- |
| 启动 | `UIApplicationMain` + AppDelegate | `Activity.onCreate` → `nativeCreate` |
| 容器 | `UIView` / `UIStackView` | `LinearLayout`（含绝对定位子级时外层套 `FrameLayout`） |
| 文本 | `UILabel` | `TextView`（单行 + 省略，对齐 UILabel） |
| 按钮 | `UIButton`（titleColor / contentEdgeInsets） | `Button`（`setAllCaps(false)`，标题中间省略） |
| 滚动 | `UIScrollView` + content 约束 | `ScrollView` + 内容 `LinearLayout` |
| 布局 | Auto Layout 约束 | `LayoutParams`：尺寸 / `weight` / gravity / margins |
| 事件 | target-action / delegate | Java 桥接类 → `nativeOnClick` / `onInput` / `onFocus` / `onScroll` |
| 安全区 | `safeAreaLayoutGuide` | 状态栏 inset 由宿主壳补到内容视图上 |

状态语义两端一致：每个状态单元一个全局变量 + 引用它的控件列表，写入时只刷新这一组；
条件分支各自有一个 `FrameLayout` 槽位，切换时增删分支视图并触发挂载 / 卸载回调。

为对齐 UIKit 的默认行为，Android 侧额外做了两件事，否则同样的写法会看起来不一样：

- 文字类节点在行方向不拉伸到容器高度，而是按内容撑开 + `gravity` 居中。`UILabel` 不会被自己的
  高度裁掉字形，`TextView` 会——顶部栏「44dp 高 + 22pt 字号」就是这么被切掉的；
- 容器统一 `setClipChildren(false)` + `setClipToPadding(false)`。`UIView` 默认不裁剪子视图，
  `ViewGroup` 默认裁到自身 padding 区域（`ScrollView` 保留裁剪，对应 `UIScrollView` 的行为）。

Android 首版尚未对齐的地方（有意记录，后续再补）：

- `Image` 只创建一个空 `ImageView` 占位，还没接图片资源（Android 侧要走 assets / drawable）；
- `backgroundExtendsTo: "top"` 与 `ignoresSafeArea` 在 Android 上不生效：targetSdk 35 强制 edge-to-edge，
  宿主壳统一把状态栏高度补成内边距，顶部栏整体落在状态栏下方；
- `justifyContent: "space-between" / "space-around"` 用 spacer + `weight` 近似，容器主轴尺寸不确定时会退化成 start；
- 交叉轴上的百分比尺寸退化成 `match_parent`，主轴上的百分比用 `weight` 近似；
- `borderRadius` / `borderWidth` 走 `GradientDrawable`，因此按钮会替换掉 Material 的默认背景和涟漪效果。



### 已移除：`fadeTarget` / `fadeStart` / `fadeDistance`

这三个内建样式键已删掉，写了会直接编译报错。滚动渐显现在由 `onScroll` + `opacity` 组合出来，
具体写法见根目录 README 的「滚动事件」与「样式绑定」两节。

## 平台能力表与验证

平台之间不可能完全等价，所以把「不等价」写成机器可校验的东西，而不是靠记忆和肉眼看截图。

### 能力表（编译期校验）

`src/capabilities.rs` 里每个后端登记自己**实际支持**的能力（组件、事件、样式键），
编译器在生成代码前把 IR 里用到的能力收集出来逐条比对：

| 级别 | 含义 | 行为 |
| --- | --- | --- |
| `Yes` | 原生支持 | 正常生成 |
| `Emulated` | 能近似实现 | 打 warning 说明近似方式（如 Android 的 `Image` 只占位） |
| `Unsupported` | 没实现 | **编译失败**并点名是哪个能力；加 `--allow-degrade` 才降级成 warning |

```bash
cargo run -- macos todo                    # macOS 的 onFocus 仍不支持 → 直接报错
cargo run -- macos todo --allow-degrade    # 降级成 warning 后编译（mac.sh 就是这么跑的）
```

新增组件 / 样式 / 事件时，必须先在 `KNOWN_KEYS` 里登记，否则校验阶段会以「未登记的能力」报错。
这条规则是为了避免「新加了个样式，某个平台默默不生效」。

### 生成源码型后端（`--target=web`）

后端不必都是「发消息建控件」：也可以把 IR 翻译成目标平台源码。当前 web 目标就是这种形态，
输出静态 `out.html`（CSS Flexbox，与 iOS/Android 共用同一套布局语义），可用于视觉对照：

```bash
cargo run -- web todo        # 写 out.html
open out.html
```

HarmonyOS 的 ArkTS、WidgetKit 的 SwiftUI 都属于这一类后端，这里先把接口和一条可用链路打通。

### 等价度检查（脚本化断言，不看截图肉眼比）

```bash
scripts/parity.sh todo       # web 结构 + Android 几何/颜色 + iOS 颜色采样
scripts/verify.sh todo       # 平台验证矩阵：构建 / 启动 / 断言逐项打结果
```

`scripts/parity.sh` 的检查项：

- **web**：HTML 里的结构/样式标记（`display: flex`、`::placeholder` 等）；
- **Android**：`uiautomator` 视图树里「徽标文字在其容器内垂直居中」（上下留白差 ≤ 2px），
  以及截图采样「占位符颜色 = rgb(120,131,155)」；
- **iOS**：模拟器截图里按输入框底色定位输入框，取最暗像素校验占位符颜色；
- **macOS**：仍需人工看窗口（本机辅助功能权限不允许脚本驱动点击）。

平台验证矩阵、哪些项只能人工、以及当前的「已知不等价」清单都在
[docs/platform-matrix.md](docs/platform-matrix.md)。
