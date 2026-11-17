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

`<Button><Text style={{ color, fontSize }}>…</Text></Button>` 里标题样式写在子 `Text` 上：编译时会把它合并进按钮样式（按钮自己写了 `color` / `fontSize` 时以按钮为准）。不合并的话，三端会各自回落到平台默认标题色——`UIButton`（custom）默认白色、Material `Button` 默认近黑，同一份代码就长得不一样。

生命周期回调支持空回调或阶段 1 已支持的状态赋值操作；静态组件在界面创建时挂载，条件组件会在对应分支加入/移除时触发挂载/卸载。暂不支持动态 props 驱动的样式更新或任意生命周期回调；递归组件在阶段 5 通过运行期槽位支持（编译期展开仍是单层）。

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
- 行模板自己写的 `width` / `height` 会真正生效：Android 侧的行参数按模板样式生成 `LayoutParams`，
  不再一律 `WRAP_CONTENT`——否则 `height: 52` 的卡片会被压成内容高度，看起来像是「内边距没了」；
- 列表放进 `ScrollView` 时，`ScrollView` 要给固定高度（`percent` 或点数）；`flexGrow` 目前拿不到视口高度，
  会退化成内容高度，于是不会滚动；
- 仍不支持：行内图片、跨行计算、字符串拼接类插值（表达式文本目前只支持数值 `+` / `-`）；
  递归组件见下面的阶段 5（必须走 `.children` 槽位，否则仍按编译期展开报错）；
- Android 后端也支持列表渲染：native 侧按字段存 `char*` 数据列，行在 `list_rebuild_*` 里按行数循环新建
  `LinearLayout`，行内按钮把行号编码进点击 id（`1_000_000 + action*1024 + row`）再解码；
  列表容器和 Activity 都以 **global reference** 持有（local reference 跨 native 调用会失效，这是之前的崩溃点）；
- Android 的差异：表达式文本只渲染初始值（不会随单元变化重算），行内条件在行构建时静态判断；
- 平台：iOS / macOS / Android 都已实现（web 只输出初始行，不带交互）。树形行表（`children`）在
  iOS / Android 上按深度缩进平铺，web 用 `padding-left` 表达层级。

## 响应式原型（阶段 5）：递归组件（运行期槽位）

组件可以渲染自己，递归深度由**数据**决定，编译期只生成一份模板。

```tsx
function TreeRow({ node }) {
    return <View style={{ flexDirection: "column" }}>
        <Text>{node.title}</Text>
        <TreeRow node={node.children} />
    </View>;
}

function App() {
    const tree = state([
        { title: "前端", children: [{ title: "渲染层" }, { title: "状态层" }] },
        { title: "后端" }
    ]);
    return <View>{tree.map((item) => <TreeRow node={item} />)}</View>;
}
```

- 递归入口是 `.children`：`<TreeRow node={node.children} />` 编译成一个 `RecursiveComponent`
  **槽位**，不展开组件体；运行期对「父行 = 当前行」的每一行再调一次同一份行模板，深度由行表的
  `parent` 列决定；
- 槽位必须是组件根容器的直接子节点，根容器要纵向排列（子行沿列方向堆叠）；
- 递归组件不能声明 `state(...)`：所有运行期行共享同一份模板，实例级单元无法按行分配；
- 只支持**直接自递归**：A→B→A 这种互相递归仍然报错；不带 `.children` 的自递归也仍按
  编译期展开报错（否则就是无限展开）；
- 行高仍是「模板行高 + 上下 margin」：每个节点占一份，子树高度 = 节点数 × 行距，所以容器总高度
  不变，`{tree.length}` 依然是整棵树的行数；
- 各端落地：iOS/macOS 生成自调用的行函数 `list_row_<id>(index, depth)`，子行嵌在父行下面；
  Android 同一套模型（`list_row_<id>(container, index, depth)`，JNI 局部引用帧按行成对出入）；
  web 是静态页，按初始行表递归输出嵌套 div；
- 示例：`app/Tree.tsx`（`cargo run -- tree`）。

## 内置示例

仓库里带四个示例，三端共用同一份 TSX，编译时用名字选（默认 `dashboard`）：

| 名称 | 源码 | 演示的能力 |
| --- | --- | --- |
| `dashboard` | `app/App.tsx` | 条件渲染整块组件、对象状态、滚动渐显、绝对定位底栏 |
| `todo` | `app/Todo.tsx` | 清单页：4 项逐项勾选 / 撤销、输入框改写标题、番茄计数、归档卡片 |
| `list` | `app/List.tsx` | 数组状态 + `.map()` 行渲染、行内删除、push / 多语句 handler |
| `tree` | `app/Tree.tsx` | 递归组件 + 树形行表、运行期按 `children` 展开子树 |
| `bench` | `app/Bench.tsx` | 性能示例：200 行初始数据 + 追加 / 删除 / 清空 / 选中，压运行期建行成本 |

```bash
cargo run                              # dashboard，UIKit IR
cargo run -- todo                      # todo，UIKit IR
cargo run -- list                      # list，UIKit IR
cargo run -- tree                      # tree，UIKit IR
cargo run -- macos todo                # todo，AppKit IR
cargo run -- android todo              # todo，Android IR
LUMEN_APP=todo ./run.sh shot           # todo，打包进 iOS 模拟器并截图
LUMEN_APP=list ./run.sh shot           # list，同上
LUMEN_APP=tree ./run.sh shot           # tree，同上
LUMEN_APP=todo ./mac.sh                # todo，macOS 版
LUMEN_APP=todo android/build-apk.sh    # todo，打 APK（再跑 android/run-apk.sh 安装）
```

`app/tsconfig.json` 与 `app/lumen.d.ts` 覆盖这些示例，供编辑器补全。`state(...)` 的赋值写法本身
不符合 TS 的 `const` 语义，所以 `npx -p typescript tsc -p app` 会报这类已知误报（声明只描述子集，不参与编译）。

`todo` 刻意保留在数组能力之前的写法：清单固定 4 项，每项有自己的布尔单元，勾选 / 撤销是两个按钮。
需要「任意新增 / 删除待办」时用 `list` 示例：`items` 是数组状态，行由 `.map()` 渲染，
添加按钮的 handler 一次写多个状态（阶段 4）。

### 性能示例与测量

衡量 React（以及各类 UI 框架）的事实标准是 **js-framework-benchmark**（`krausest/js-framework-benchmark`）。
它固定一组操作：create 1k rows、replace all rows、partial update（每 10 行改一次）、select row、
swap rows、remove row、append 1k rows、clear rows，用 ChromeDriver 驱动、取多次运行的几何平均，
另外还看 JS 堆内存、启动时间（startup SLO）和压缩后产物体积。

`app/Bench.tsx` 是这套操作集在本语言子集里的裁剪版：初始 200 行（运行期数据容量上限是 256），
「追加 / 删除 / 清空 / 选中」分别对应 append / remove / clear / update。整表操作会触发一次
运行期重建（本项目的更新模型就是「改数据 → 重建行」，没有 diff 与 Virtual DOM），
所以它压的是建行成本，而不是 React 那种 diff + commit 成本。受语言子集限制暂时做不了的是
swap rows（要同时改两行的字段）和任意行的 partial update（只能改当前行）。

页面自己不计时（语言里没有 `performance.now` / 定时器），要用各平台的外部工具量：

| 平台 | 命令 | 看什么 |
| --- | --- | --- |
| Android | `adb shell am start -W -n com.lumen.app/.MainActivity` | `TotalTime`：冷启动（含建 200 行） |
| Android | `adb shell dumpsys gfxinfo com.lumen.app` | Total frames / Janky frames / 50·90·99th |
| iOS | `xcrun xctrace record --template 'Time Profiler'` + Instruments | 启动耗时、建行时的调用栈 |
| macOS | 同 iOS | 同上 |
| web | `cargo run -- web bench`，看 `out.html` 大小 | 产物体积（静态页只含初始行） |

本机 Android 模拟器（软件渲染、带窗口，绝对数值偏悲观，只适合横向对比）实测：
冷启动 6.1s / 8.3s / 10.3s（连续三次，随模拟器负载波动，**主要由进程启动 + JNI/ART 初始化决定，
和行数关系不大**：2 行的 `list` 也要 5.7–6.8s）；一次启动只渲染 5 帧、janky 5 帧、90th 1800ms。
交互（删一行 → 整表重建 199 行）在稳态下是 350–1000ms（同一操作连测 5 次的 50th percentile，方差很大）。

「行视图复用」试过但**没有收益**：改成「按位置复用 + 只 rebind 文字/点击 id」后，同一组测量是
300 / 400 / 650 / 1200ms，和全清重建打平。原因是每行建视图约 8 次 JNI、rebind 约 12 次
（`findViewById` + `setText` + 新建 `ClickBridge`），瓶颈是**每行的 JNI 往返次数**，
不是重建视图本身。开关留在 `src/android.rs` 的 `ROW_REUSE`（默认关）。

真正有效的是把**整行下沉到 Java 侧**（`android/src/com/lumen/app/LumenRow.java`）：native 只为每个列表
解析一次行模板，之后每行只做「拼一行字段文本 + 一次静态调用」，行视图按 parent 缓存在 Java 里，
重建时就地 rebind。同机 `bench`（200 行）每个操作连测 5 次的 50th percentile：

| 操作 | 全清重建（native 建行） | Java 侧建行 + 复用 |
| --- | --- | --- |
| 删一行（重建 199 行） | 350 / 400 / 600 / 950 / 1000ms | 200 / 200 / 200 / 350 / 350ms |
| 追加一行 | — | 101 / 150 / 500ms |
| 选中一行 | — | 150 / 150 / 200ms |
| 冷启动（含首屏建 200 行） | 4.0–10.3s | 6.4 / 6.6 / 8.4s（不变：首屏没有可复用的行） |

快路径只覆盖「横向容器 + 一排叶子控件」这种最常见的行（`row_spec` 判定，`list` / `bench` 的行都在内），
其余形态（递归组件、嵌套容器、图片/输入框）自动回退到 native 建行。

已知差异：这个长列表在 Android 上会画到 `ScrollView` 视口之外（内容超长时未裁剪），见文末「尚未对齐」。

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
| 滚动 | `UIScrollView` + `UIScrollViewDelegate` 的 `scrollViewDidScroll:` | `NSScrollView` + `isFlipped` 文档视图，位置靠 `NSViewBoundsDidChange` 通知回传 |
| 安全区 | `safeAreaLayoutGuide`（状态栏 / home indicator） | 同名 API，窗口内容区通常没有额外 inset |

macOS 首版尚未支持：`onFocus` 事件，以及含条件子级的容器里文本的横向对齐（`NSStackView` 对齐映射还不准）。

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

为对齐 UIKit 的默认行为，Android 侧额外做了三件事，否则同样的写法会看起来不一样：

- 文字类节点在行方向不拉伸到容器高度，而是按内容撑开 + `gravity` 居中。`UILabel` 不会被自己的
  高度裁掉字形，`TextView` 会——顶部栏「44dp 高 + 22pt 字号」就是这么被切掉的；
- 容器统一 `setClipChildren(false)` + `setClipToPadding(false)`。`UIView` 默认不裁剪子视图，
  `ViewGroup` 默认裁到自身 padding 区域（`ScrollView` 保留裁剪，对应 `UIScrollView` 的行为）；
- `Button` 在没有显式 padding 时把内边距清零。`UIButton` 的标题从 `contentEdgeInsets`（默认 0）起排，
  Material 的 `Button` 自带左右各 16dp 内边距：52×28 这种窄按钮扣完只剩几 dp 内容区，再叠加
  「单行 + 中间省略」，文案会被压成一条缝（`完成` 只剩一点笔画）。

Android 首版尚未对齐的地方（有意记录，后续再补）：

- 内容比视口长的 `ScrollView` 没有被裁到视口内：行会继续画到视口下方、滚动后还会盖住上方的兄弟节点
  （`app/Bench.tsx` 的 200 行最明显）。等长列表的滚动裁剪补上之后，这个示意的启动/掉帧数据才更有意义；
- 主轴上的百分比尺寸会变成 `weight`，而 `weight` 常量只在能被 `float` 精确表示时才能链接：
  `width: "48%"` 会让 `out.ll` 里出现 `float 4.800000e-01`，`clang` 直接报
  `floating point constant invalid for type`（`50%` / `25%` 这类可以）。`app/Bench.tsx` 因此只用 50%；

- `Image` 只创建一个空 `ImageView` 占位，还没接图片资源（Android 侧要走 assets / drawable）；
- `backgroundExtendsTo: "top"` 与 `ignoresSafeArea` 在 Android 上不生效：targetSdk 35 强制 edge-to-edge，
  宿主壳统一把状态栏高度补成内边距，顶部栏整体落在状态栏下方；
- `justifyContent: "space-between" / "space-around"` 用 spacer + `weight` 近似，容器主轴尺寸不确定时会退化成 start；
- 交叉轴上的百分比尺寸退化成 `match_parent`，主轴上的百分比用 `weight` 近似；
- `borderRadius` / `borderWidth` 走 `GradientDrawable`，因此按钮会替换掉 Material 的默认背景和涟漪效果。



### 已移除：`fadeTarget` / `fadeStart` / `fadeDistance`

这三个内建样式键已经删掉，写了会直接编译报错（报错信息给出迁移写法）。滚动渐显现在是
`onScroll` 写滚动位置 + `opacity` 绑这个数值单元，两件普通能力组合出来，iOS / macOS / Android 一致。

### 滚动事件（`onScroll`）

滚动位置是一条普通能力：`onScroll={(y) => ...}` 把当前滚动距离写进数值单元：

```tsx
function App() {
    const offset = state(0);
    return <View>
        <Text>{offset}</Text>
        <ScrollView onScroll={(y) => offset = y} style={{ height: 460 }}>
            {/* 内容 */}
        </ScrollView>
    </View>;
}
```

- 目标必须是一个数值单元（`state(0)`），其它形态直接编译报错，不会静默不生效；
- 写入前可以夹取：`onScroll={(y) => offset = clamp(y, 0, 100)}`（`clamp` 只接受整数字面量边界，
  且要求 `low < high`）；
- 滚动位置：iOS 走生成的 `scrollViewDidScroll:`（取 `contentOffset.y + adjustedContentInset.top`），
  macOS 走 `NSViewBoundsDidChange` 通知（clip view 的 `bounds.origin.y`，文档视图是 `isFlipped` 的，
  向下滚动时就是正数，不需要取反），Android 走 `nativeOnScroll(scrollY)`；写入后只刷新引用这个单元的文本，不重建视图树；
- 每个 App 目前支持一个滚动绑定；web 静态页登记为「不带滚动交互」；
- `app/Bench.tsx` 拿它显示当前滚动位置，滚动时数字会实时变化。

有了这个能力，「滚动渐显」就能逐步从 `fadeTarget` 改写成 TSX：`onScroll` 写位置 → 透明度绑样式。

### 样式绑定：`opacity`

```tsx
const offset = state(0);
<ScrollView onScroll={(y) => offset = y} style={{ height: 460 }}>
    <Text style={{ opacity: offset }}>滚动 100px 后完全显现</Text>
</ScrollView>
```

- 值可以是数值单元（0..100，换算成 `alpha = v / 100` 并夹到 0..1），也可以是常量（`opacity: 50` = 50%）；
- iOS/macOS 在 `update_cell_labels` 里顺带 `setAlpha:`（AppKit 是 `setAlphaValue:`），Android 在各单元的
  `lumen_refresh_N` 里顺带 `setAlpha`，所以和文本刷新是同一条路径——滚动时只更新这一个视图，不重建视图树；
- 写死的透明度（`opacity: 0`）也走同一套，三端一致；
- web 是静态页，只渲染**初始值**（没有运行时更新）；
- 顺手修掉一个 Android 侧的真坑：绑定的视图句柄必须存 **global ref**，存 local ref 的话
  native 调用返回后句柄失效，滚动时会直接 SIGSEGV（`lumen_refresh_N` 里踩过）。

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
