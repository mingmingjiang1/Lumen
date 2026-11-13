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

首阶段的约束：每个 App 只支持一个状态声明；状态初值为整数、字符串或布尔值。`onClick` 支持递增、递减和赋标量常量；`onInput` 支持把输入值赋给该状态；`onFocus` 支持空回调或将 `state(false)` 设为 `true`。旧的数字 `useState` 递增写法仍兼容。

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

生命周期回调支持空回调或阶段 1 已支持的状态赋值操作；静态组件在界面创建时挂载，条件组件会在对应分支加入/移除时触发挂载/卸载。当前每个 App 仍只支持一个状态声明；暂不支持递归组件、动态 props 驱动的样式更新或任意生命周期回调。

## 滚动时顶部栏渐显

给顶部栏设置 `id`，并在对应的 `ScrollView` 上设置 `fadeTarget`。顶部栏整体（背景和子元素）初始透明；滚动超过 `fadeStart` 后，在 `fadeDistance` 点内渐显至完全可见。

```tsx
<View style={{ id: "topBar", backgroundColor: "rgb(28, 39, 61)" }}>
    <Text>My Dashboard</Text>
</View>
<ScrollView style={{
    fadeTarget: "topBar",
    fadeStart: 0,
    fadeDistance: 120
}}>
    {/* scrollable content */}
</ScrollView>
```

每个界面首版支持一个渐变目标；目标必须是带背景色的 `View`/`VStack`，`fadeDistance` 必须大于零。
