//! 平台能力表 + 编译期校验。
//!
//! 每个后端声明自己实际支持的能力（组件、事件、样式键）。编译器在生成代码前
//! 把 IR 里真正用到的能力收集出来，逐条和后端表比对：
//!
//! - `Yes`：原生支持；
//! - `Emulated`：能近似实现，打一条 warning 说明近似方式；
//! - `Unsupported`：默认编译失败（提示加 `--allow-degrade` 可降级为 warning）；
//! - 能力键不在 [`KNOWN_KEYS`] 里：说明是新增样式的漏登记，直接报错。
//!
//! 这样“某个平台上某能力不生效”会在编译期暴露，而不是等截图对比才发现。

use std::collections::BTreeSet;

use super::*;

/// 一个能力在后端的支持级别。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Support {
    Yes,
    Emulated(&'static str),
    Unsupported(&'static str),
}

/// 一个后端的能力表：只登记与「默认支持」不同的例外。
pub(crate) struct Backend {
    pub name: &'static str,
    exceptions: &'static [(&'static str, Support)],
}

impl Backend {
    pub(crate) fn support(&self, key: &str) -> Support {
        self.exceptions
            .iter()
            .find(|(known, _)| *known == key)
            .map(|(_, support)| *support)
            .unwrap_or(Support::Yes)
    }
}

/// 全部合法的能力键。新增组件 / 样式 / 事件时必须在这里登记，
/// 否则校验阶段会以「未登记的能力」报错，避免遗漏到运行时。
pub(crate) const KNOWN_KEYS: &[&str] = &[
    // 组件
    "View",
    "VStack",
    "ScrollView",
    "Text",
    "StateText",
    "Image",
    "Input",
    "Button",
    "Conditional",
    "List",
    "ItemText",
    "NestedList",
    "ExprText",
    "ItemConditional",
    "RecursiveComponent",
    // 事件
    "onClick",
    "onInput",
    "onFocus",
    "onScroll",
    // 尺寸
    "width",
    "height",
    "flexGrow",
    // 内边距
    "padding",
    "paddingHorizontal",
    "paddingVertical",
    "paddingTop",
    "paddingRight",
    "paddingBottom",
    "paddingLeft",
    // 外边距
    "margin",
    "marginHorizontal",
    "marginVertical",
    "marginTop",
    "marginRight",
    "marginBottom",
    "marginLeft",
    // 定位
    "position",
    "top",
    "right",
    "bottom",
    "left",
    // 布局
    "spacing",
    "flexDirection",
    "justifyContent",
    "alignItems",
    // 文本
    "fontSize",
    "textAlign",
    "color",
    "placeholderColor",
    // 透明度（常量或绑到 0..100 的数值单元）
    "opacity",
    // 背景与边框
    "backgroundColor",
    "borderWidth",
    "borderColor",
    "borderRadius",
    // 安全区与滚动
    "id",
    "backgroundExtendsTo",
    "ignoresSafeArea",
];

pub(crate) const IOS: Backend = Backend {
    name: "ios",
    // UIKit 后端目前逐项都有对应实现
    exceptions: &[],
};

pub(crate) const MACOS: Backend = Backend {
    name: "macos",
    exceptions: &[
        // ScrollView / onScroll 都已经实现：滚动位置走 NSViewBoundsDidChange 通知
        // （见 codegen_mac_scroll），所以不登记例外。
        ("onFocus", Support::Unsupported("AppKit 版还没有焦点回调")),
        (
            "backgroundExtendsTo",
            Support::Emulated("背景不向上延伸，按普通背景处理"),
        ),
    ],
};

pub(crate) const ANDROID: Backend = Backend {
    name: "android",
    exceptions: &[
        (
            "Image",
            Support::Emulated("只渲染占位视图，还没有接入资源加载"),
        ),
        (
            "ExprText",
            Support::Emulated("表达式文本只渲染初始值，不随单元变化重算"),
        ),
        (
            "ignoresSafeArea",
            Support::Emulated("统一把状态栏高度补成内容内边距"),
        ),
        (
            "backgroundExtendsTo",
            Support::Emulated("背景不向上延伸，按普通背景处理"),
        ),
    ],
};

/// 生成源码型后端（当前目标：静态 HTML，用于视觉对照）。
pub(crate) const WEB: Backend = Backend {
    name: "web",
    exceptions: &[
        ("onClick", Support::Emulated("静态 HTML 不带交互，仅用于视觉对照")),
        ("onInput", Support::Emulated("静态 HTML 不带交互，仅用于视觉对照")),
        ("onScroll", Support::Emulated("静态 HTML 不带滚动交互")),
        ("onFocus", Support::Emulated("静态 HTML 不带交互，仅用于视觉对照")),
        (
            "ignoresSafeArea",
            Support::Emulated("按普通布局渲染，不模拟安全区"),
        ),
        (
            "backgroundExtendsTo",
            Support::Emulated("背景不向上延伸"),
        ),
        (
            "List",
            Support::Emulated("静态 HTML 只输出初始行"),
        ),
        (
            "NestedList",
            Support::Emulated("按 DFS 顺序输出并用 padding-left 表达层级"),
        ),
    ],
};

/// wasm + DOM 桥后端：IR 编成 wasm，视图调用落到 `dom.*` import 上。
pub(crate) const WASM: Backend = Backend {
    name: "wasm",
    exceptions: &[
        (
            "onInput",
            Support::Emulated("输入框可以渲染，onInput 还没接 DOM input 事件"),
        ),
        (
            "onFocus",
            Support::Emulated("焦点回调还没接 DOM focus 事件"),
        ),
        (
            "Image",
            Support::Emulated("只渲染占位视图，还没有接入资源加载"),
        ),
        (
            "ignoresSafeArea",
            Support::Emulated("按普通布局渲染，不模拟安全区"),
        ),
        (
            "backgroundExtendsTo",
            Support::Emulated("背景不向上延伸"),
        ),
        (
            "NestedList",
            Support::Emulated("树形行按深度缩进，子树平铺"),
        ),
        (
            "RecursiveComponent",
            Support::Emulated("可编译，子树内嵌在行模板里"),
        ),
    ],
};

pub(crate) fn backend_for(platform: Platform) -> &'static Backend {
    match platform {
        Platform::Ios => &IOS,
        Platform::MacOs => &MACOS,
        Platform::Android => &ANDROID,
        Platform::Wasm => &WASM,
    }
}

/// 校验结果：errors 非空时返回 Err，否则给出 warning 列表。
pub(crate) fn check(
    backend: &Backend,
    program: &UiProgram,
    allow_degrade: bool,
) -> Result<Vec<String>, String> {
    let features = collect_features(program);
    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    for feature in &features {
        if !KNOWN_KEYS.contains(feature) {
            errors.push(format!("未知的能力键 '{feature}'（新增样式请先在能力表登记）"));
            continue;
        }
        match backend.support(feature) {
            Support::Yes => {}
            Support::Emulated(reason) => {
                warnings.push(format!("[{}] {feature}：{reason}", backend.name));
            }
            Support::Unsupported(reason) => {
                let message = format!("[{}] 不支持 {feature}：{reason}", backend.name);
                if allow_degrade {
                    warnings.push(format!("{message}（已按 --allow-degrade 忽略）"));
                } else {
                    errors.push(message);
                }
            }
        }
    }

    if errors.is_empty() {
        Ok(warnings)
    } else {
        Err(format!(
            "{}\n提示：加 --allow-degrade 可以把「不支持」降级为 warning",
            errors.join("\n")
        ))
    }
}

/// 收集程序里实际用到的能力键。
pub(crate) fn collect_features(program: &UiProgram) -> BTreeSet<&'static str> {
    let mut out = BTreeSet::new();
    for node in &program.nodes {
        collect_node(node, &mut out);
    }
    out
}

fn collect_node(node: &IRNode, out: &mut BTreeSet<&'static str>) {
    let key = match node {
        IRNode::View { .. } => "View",
        IRNode::VStack { .. } => "VStack",
        IRNode::ScrollView { .. } => "ScrollView",
        IRNode::Text { .. } => "Text",
        IRNode::StateText { .. } => "StateText",
        IRNode::Image { .. } => "Image",
        IRNode::Input { .. } => "Input",
        IRNode::Button { .. } => "Button",
        IRNode::Conditional { .. } => "Conditional",
        IRNode::List { .. } => "List",
        IRNode::ItemText { .. } => "ItemText",
        IRNode::ExprText { .. } => "ExprText",
        IRNode::ItemConditional { .. } => "ItemConditional",
        IRNode::RecursiveComponent { .. } => "RecursiveComponent",
    };
    out.insert(key);

    let style = node_style(node);
    collect_style(style, out);

    match node {
        IRNode::View { children, .. }
        | IRNode::VStack { children, .. }
        | IRNode::ScrollView { children, .. } => {
            for child in children {
                collect_node(child, out);
            }
        }
        IRNode::Conditional {
            when_true,
            when_false,
            ..
        } => {
            collect_node(when_true, out);
            if let Some(node) = when_false {
                collect_node(node, out);
            }
        }
        IRNode::List {
            item, parents, ..
        } => {
            if parents.iter().any(|parent| parent.is_some()) {
                out.insert("NestedList");
            }
            collect_node(item, out);
        }
        IRNode::ItemConditional {
            when_true,
            when_false,
            ..
        } => {
            collect_node(when_true, out);
            if let Some(node) = when_false {
                collect_node(node, out);
            }
        }
        IRNode::Input {
            on_input, on_focus, ..
        } => {
            if on_input.is_some() {
                out.insert("onInput");
            }
            if on_focus.is_some() {
                out.insert("onFocus");
            }
        }
        IRNode::ScrollView { style, .. } => {
            if style.scroll_action.is_some() {
                out.insert("onScroll");
            }
        }
        IRNode::Button { state_action, .. } => {
            if state_action.is_some() {
                out.insert("onClick");
            }
        }
        _ => {}
    }
}

fn collect_style(style: &InlineStyle, out: &mut BTreeSet<&'static str>) {
    if style.width.is_some() {
        out.insert("width");
    }
    if style.height.is_some() {
        out.insert("height");
    }
    if style.flex_grow > 0.0 {
        out.insert("flexGrow");
    }
    if style.opacity_static.is_some() || style.opacity_cell.is_some() {
        out.insert("opacity");
    }
    if style.scroll_action.is_some() {
        out.insert("onScroll");
    }
    for (value, key) in [
        (style.padding, "padding"),
        (style.padding_horizontal, "paddingHorizontal"),
        (style.padding_vertical, "paddingVertical"),
        (style.padding_top, "paddingTop"),
        (style.padding_right, "paddingRight"),
        (style.padding_bottom, "paddingBottom"),
        (style.padding_left, "paddingLeft"),
        (style.margin, "margin"),
        (style.margin_horizontal, "marginHorizontal"),
        (style.margin_vertical, "marginVertical"),
        (style.margin_top, "marginTop"),
        (style.margin_right, "marginRight"),
        (style.margin_bottom, "marginBottom"),
        (style.margin_left, "marginLeft"),
        (style.spacing, "spacing"),
        (style.font_size, "fontSize"),
        (style.border_width, "borderWidth"),
        (style.border_radius, "borderRadius"),
    ] {
        if value.is_some() {
            out.insert(key);
        }
    }
    if style.position == Position::Absolute {
        out.insert("position");
    }
    for (value, key) in [
        (style.top, "top"),
        (style.right, "right"),
        (style.bottom, "bottom"),
        (style.left, "left"),
    ] {
        if value.is_some() {
            out.insert(key);
        }
    }
    if style.flex_direction.is_some() {
        out.insert("flexDirection");
    }
    if style.justify_content.is_some() {
        out.insert("justifyContent");
    }
    if style.align_items.is_some() {
        out.insert("alignItems");
    }
    if style.text_align.is_some() {
        out.insert("textAlign");
    }
    for (value, key) in [
        (&style.color, "color"),
        (&style.placeholder_color, "placeholderColor"),
        (&style.background_color, "backgroundColor"),
        (&style.border_color, "borderColor"),
        (&style.view_id, "id"),
    ] {
        if value.is_some() {
            out.insert(key);
        }
    }
    if style.background_extends_to_top {
        out.insert("backgroundExtendsTo");
    }
    if style.ignores_safe_area_top {
        out.insert("ignoresSafeArea");
    }
}
