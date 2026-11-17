mod android;
mod capabilities;
mod source_backend;

use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::sync::Arc;

use swc_common::{FileName, SourceMap};
use swc_ecma_ast::*;
use swc_ecma_parser::{Parser, Syntax, TsSyntax, lexer::Lexer};
use swc_ecma_visit::{Visit, VisitWith};

// ---------- 你的原有部分：AST -> IRNode（保持不动） ----------

/// Input 占位符的默认颜色。三端原生默认各不相同（UIKit 偏浅、Android 偏深），
/// 这里统一成一个中性灰，样式里可以用 placeholderColor 覆盖。
pub(crate) const DEFAULT_PLACEHOLDER_COLOR: &str = "rgb(120, 131, 155)";

/// UILabel / TextView 不写 fontSize 时的默认字号（三端都用它，保证观感一致）。
pub(crate) const DEFAULT_FONT_SIZE: f64 = 17.0;

/// 表达式文本的操作数：单元或整数字面量。
#[derive(Debug, Clone, PartialEq)]
pub enum ExprOperand {
    Cell(String),
    Number(i64),
}

impl ExprOperand {
    fn initial(&self, cells: &[(String, StateValue)]) -> i64 {
        match self {
            Self::Number(value) => *value,
            Self::Cell(cell) => cells
                .iter()
                .find(|(path, _)| path == cell)
                .and_then(|(_, value)| match value {
                    StateValue::Number(number) => Some(*number),
                    _ => None,
                })
                .unwrap_or(0),
        }
    }
}

#[derive(Debug, Clone)]
pub enum IRNode {
    View { style: InlineStyle, children: Vec<IRNode> },
    VStack { style: InlineStyle, children: Vec<IRNode> },
    ScrollView { style: InlineStyle, children: Vec<IRNode> },
    Conditional {
        cell: String,
        style: InlineStyle,
        when_true: Box<IRNode>,
        when_false: Option<Box<IRNode>>,
        initial: bool,
        true_mount: Vec<usize>,
        true_unmount: Vec<usize>,
        false_mount: Vec<usize>,
        false_unmount: Vec<usize>,
    },
    Text { content: String, style: InlineStyle },
    StateText {
        cell: String,
        initial: StateValue,
        style: InlineStyle,
    },
    Image {
        source: String,
        mode: ImageMode,
        style: InlineStyle,
    },
    Input {
        placeholder: String,
        style: InlineStyle,
        /// 状态动作下标（SetFromInput）。
        on_input: Option<usize>,
        /// 状态动作下标（SetTrue 或 Noop）。
        on_focus: Option<usize>,
        unsupported_event_handler: bool,
    },
    Button {
        title: String,
        style: InlineStyle,
        state_action: Option<usize>,
        unsupported_handler: bool,
    },
    /// 列表渲染：`{items.map((item) => <Row />)}` 编译成一个按行实例化的模板。
    /// 行数由列表状态的运行时长度决定，模板里的字段引用走 [`IRNode::ItemText`]。
    List {
        style: InlineStyle,
        list: String,
        item: Box<IRNode>,
        /// 字段类型样本（按声明顺序）。
        fields: Vec<(String, StateValue)>,
        /// 构建期写入行存储的初始数据（DFS 顺序）。
        rows: Vec<Vec<StateValue>>,
        /// 每行的父行下标（None = 根层）；有值说明是嵌套列表。
        parents: Vec<Option<usize>>,
    },
    /// 表达式文本：`{a + b}`，依赖的单元变化时重算。
    ExprText {
        left: ExprOperand,
        right: ExprOperand,
        subtract: bool,
        deps: Vec<String>,
        initial: String,
        style: InlineStyle,
    },
    /// 列表模板里按当前行布尔字段分支（`{item.done ? a : b}`）。
    ItemConditional {
        list: String,
        field: String,
        initial: bool,
        when_true: Box<IRNode>,
        when_false: Option<Box<IRNode>>,
        style: InlineStyle,
    },
    /// 列表模板里绑定到「当前行字段」的文本（`{item.title}`）。
    ItemText {
        list: String,
        field: String,
        initial: StateValue,
        style: InlineStyle,
    },
    /// 递归组件槽位：`<TreeRow node={node.children} />`。
    /// 编译期不展开，运行期对「父行 = 当前行」的每一行再实例化同一份组件模板，
    /// 深度由行表的父子关系决定。
    RecursiveComponent {
        component: String,
        list: String,
        style: InlineStyle,
    },
}

#[derive(Debug, Clone, Default)]
pub struct InlineStyle {
    width: Option<Dimension>,
    height: Option<Dimension>,
    padding: Option<f64>,
    padding_horizontal: Option<f64>,
    padding_vertical: Option<f64>,
    padding_top: Option<f64>,
    padding_right: Option<f64>,
    padding_bottom: Option<f64>,
    padding_left: Option<f64>,
    margin: Option<f64>,
    margin_horizontal: Option<f64>,
    margin_vertical: Option<f64>,
    margin_top: Option<f64>,
    margin_right: Option<f64>,
    margin_bottom: Option<f64>,
    margin_left: Option<f64>,
    position: Position,
    top: Option<Dimension>,
    right: Option<Dimension>,
    bottom: Option<Dimension>,
    left: Option<Dimension>,
    spacing: Option<f64>,
    flex_direction: Option<FlexDirection>,
    justify_content: Option<JustifyContent>,
    align_items: Option<AlignItems>,
    flex_grow: f64,
    font_size: Option<f64>,
    text_align: Option<TextAlignment>,
    background_color: Option<String>,
    color: Option<String>,
    /// Input 占位符颜色；不写时用 DEFAULT_PLACEHOLDER_COLOR，保证三端一致。
    placeholder_color: Option<String>,
    border_width: Option<f64>,
    border_color: Option<String>,
    border_radius: Option<f64>,
    view_id: Option<String>,
    background_extends_to_top: bool,
    ignores_safe_area_top: bool,
    scroll_fade_target: Option<String>,
    scroll_fade_start: Option<f64>,
    scroll_fade_distance: Option<f64>,
    /// `onScroll={(y) => offset = y}`：滚动位置写进哪个数值单元（存动作下标）。
    scroll_action: Option<usize>,
    /// `style={{ opacity: 单元 }}`：透明度绑定到哪个数值单元（0..100 整数）。
    opacity_cell: Option<String>,
    /// 常量透明度（0..1），没绑单元时用。
    opacity_static: Option<f64>,
    mount_actions: Vec<usize>,
    unmount_actions: Vec<usize>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FlexDirection {
    Row,
    Column,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum Position {
    #[default]
    Relative,
    Absolute,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum JustifyContent {
    Start,
    Center,
    End,
    SpaceBetween,
    SpaceAround,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AlignItems {
    Start,
    Center,
    End,
    Stretch,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TextAlignment {
    Left,
    Center,
    Right,
}

impl TextAlignment {
    fn ui_value(self) -> u64 {
        match self {
            Self::Left => 0,
            Self::Center => 1,
            Self::Right => 2,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageMode {
    ScaleToFill,
    AspectFit,
    AspectFill,
    WidthFix,
    HeightFix,
}

impl ImageMode {
    fn from_attribute(value: Option<&str>) -> Self {
        match value {
            Some("aspectFit") => Self::AspectFit,
            Some("aspectFill") => Self::AspectFill,
            Some("widthFix") => Self::WidthFix,
            Some("heightFix") => Self::HeightFix,
            _ => Self::ScaleToFill,
        }
    }

    fn content_mode(self) -> u64 {
        match self {
            Self::ScaleToFill | Self::WidthFix | Self::HeightFix => 0,
            Self::AspectFit => 1,
            Self::AspectFill => 2,
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum Dimension {
    Points(f64),
    Percent(f64),
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct ChildLayout {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
struct Margins {
    top: f64,
    right: f64,
    bottom: f64,
    left: f64,
}

fn node_margins(node: &IRNode) -> Margins {
    let style = node_style(node);
    let all = style.margin.unwrap_or(0.0);
    let horizontal = style.margin_horizontal.unwrap_or(all);
    let vertical = style.margin_vertical.unwrap_or(all);
    Margins {
        top: style.margin_top.unwrap_or(vertical),
        right: style.margin_right.unwrap_or(horizontal),
        bottom: style.margin_bottom.unwrap_or(vertical),
        left: style.margin_left.unwrap_or(horizontal),
    }
}

/// 视图盒子内侧的间距。padding 先铺满四边，paddingHorizontal/Vertical 覆盖两轴，
/// 单边 paddingTop/Right/Bottom/Left 再覆盖对应边。
#[derive(Debug, Clone, Copy, Default, PartialEq)]
struct Padding {
    top: f64,
    right: f64,
    bottom: f64,
    left: f64,
}

impl Padding {
    fn horizontal(self) -> f64 {
        self.left + self.right
    }

    fn vertical(self) -> f64 {
        self.top + self.bottom
    }

    fn is_zero(self) -> bool {
        self == Padding::default()
    }
}

fn node_padding(style: &InlineStyle) -> Padding {
    let all = style.padding.unwrap_or(0.0);
    let horizontal = style.padding_horizontal.unwrap_or(all);
    let vertical = style.padding_vertical.unwrap_or(all);
    Padding {
        top: style.padding_top.unwrap_or(vertical),
        right: style.padding_right.unwrap_or(horizontal),
        bottom: style.padding_bottom.unwrap_or(vertical),
        left: style.padding_left.unwrap_or(horizontal),
    }
}

impl Dimension {
    fn resolve(self, parent_size: f64) -> f64 {
        match self {
            Self::Points(value) => value,
            Self::Percent(fraction) => parent_size * fraction,
        }
    }
}

fn node_height(node: &IRNode, available_height: f64) -> f64 {
    // Leaf estimates guide static flex placement; intrinsic UIKit views are not constrained to them.
    match node {
        IRNode::View { style, children }
        | IRNode::VStack { style, children }
        | IRNode::ScrollView { style, children } => style
            .height
            .map(|d| d.resolve(available_height))
            .unwrap_or_else(|| {
                let direction = style.flex_direction.unwrap_or(FlexDirection::Column);
                let spacing = style.spacing.unwrap_or(if matches!(node, IRNode::VStack { .. }) { 8.0 } else { 0.0 });
                let flow_count = children
                    .iter()
                    .filter(|child| node_style(child).position != Position::Absolute)
                    .count();
                let children_height = match direction {
                    FlexDirection::Row => children
                        .iter()
                        .filter(|child| node_style(child).position != Position::Absolute)
                        .map(|child| {
                            let margins = node_margins(child);
                            node_height(child, available_height) + margins.top + margins.bottom
                        })
                        .fold(0.0, f64::max),
                    FlexDirection::Column => children
                        .iter()
                        .filter(|child| node_style(child).position != Position::Absolute)
                        .map(|child| {
                            let margins = node_margins(child);
                            node_height(child, available_height) + margins.top + margins.bottom
                        })
                        .sum::<f64>()
                        + spacing * flow_count.saturating_sub(1) as f64,
                };
                children_height + node_padding(style).vertical()
            }),
        IRNode::ItemConditional {
            initial,
            when_true,
            when_false,
            ..
        } => {
            let active = if *initial {
                when_true.as_ref()
            } else {
                when_false.as_deref().unwrap_or(when_true)
            };
            node_height(active, available_height)
        }
        IRNode::Conditional {
            when_true,
            when_false,
            ..
        } => node_height(when_true, available_height).max(
            when_false
                .as_deref()
                .map(|node| node_height(node, available_height))
                .unwrap_or(0.0),
        ),
        IRNode::Text { style, .. }
        | IRNode::StateText { style, .. }
        | IRNode::ItemText { style, .. }
        | IRNode::ExprText { style, .. } => style
            .height
            .map(|dimension| dimension.resolve(available_height))
            .unwrap_or_else(|| style.font_size.unwrap_or(17.0) * 1.2 + node_padding(style).vertical()),
        // 列表容器的高度随行数增长，先按初始行数估。
        IRNode::List {
            style,
            item,
            rows,
            ..
        } => style
            .height
            .map(|dimension| dimension.resolve(available_height))
            .unwrap_or_else(|| node_height(item, available_height) * rows.len() as f64),
        IRNode::Image { style, .. }
        | IRNode::Input { style, .. }
        | IRNode::Button { style, .. } => style
            .height
            .map(|dimension| dimension.resolve(available_height))
            .unwrap_or(50.0 + node_padding(style).vertical()),
        // 递归槽位本身不占高度：子行在运行期由外层行模板撑开。
        IRNode::RecursiveComponent { .. } => 0.0,
    }
}

fn node_width(node: &IRNode, available_width: f64) -> f64 {
    match node {
        IRNode::View { style, children }
        | IRNode::VStack { style, children }
        | IRNode::ScrollView { style, children } => style.width.map(|d| d.resolve(available_width)).unwrap_or_else(|| {
            let direction = style.flex_direction.unwrap_or(FlexDirection::Column);
            let spacing = style.spacing.unwrap_or(if matches!(node, IRNode::VStack { .. }) { 8.0 } else { 0.0 });
            let flow_count = children
                .iter()
                .filter(|child| node_style(child).position != Position::Absolute)
                .count();
            let children_width = match direction {
                FlexDirection::Row => children
                    .iter()
                    .filter(|child| node_style(child).position != Position::Absolute)
                    .map(|child| {
                        let margins = node_margins(child);
                        node_width(child, available_width) + margins.left + margins.right
                    })
                    .sum::<f64>()
                    + spacing * flow_count.saturating_sub(1) as f64,
                FlexDirection::Column => children
                    .iter()
                    .filter(|child| node_style(child).position != Position::Absolute)
                    .map(|child| {
                        let margins = node_margins(child);
                        node_width(child, available_width) + margins.left + margins.right
                    })
                    .fold(0.0, f64::max),
            };
            children_width + node_padding(style).horizontal()
        }),
        IRNode::ItemConditional {
            initial,
            when_true,
            when_false,
            ..
        } => {
            let active = if *initial {
                when_true.as_ref()
            } else {
                when_false.as_deref().unwrap_or(when_true)
            };
            node_width(active, available_width)
        }
        IRNode::Conditional {
            when_true,
            when_false,
            ..
        } => node_width(when_true, available_width).max(
            when_false
                .as_deref()
                .map(|node| node_width(node, available_width))
                .unwrap_or(0.0),
        ),
        IRNode::Text { content, style } => style.width.map(|d| d.resolve(available_width)).unwrap_or_else(|| {
            // 粗估文字宽度：ASCII 按 0.6 em，CJK 按 1 em（不区分会把中文标签估得太窄）
            let em: f64 = content
                .chars()
                .map(|ch| if ch.is_ascii() { 0.6 } else { 1.0 })
                .sum();
            em * style.font_size.unwrap_or(17.0) + node_padding(style).horizontal()
        }),
        IRNode::StateText { style, .. }
        | IRNode::ItemText { style, .. }
        | IRNode::ExprText { style, .. } => style
            .width
            .map(|d| d.resolve(available_width))
            .unwrap_or(50.0 + node_padding(style).horizontal()),
        // 列表按容器宽度铺满，行内布局再由模板自己决定。
        IRNode::List { style, .. } => style
            .width
            .map(|d| d.resolve(available_width))
            .unwrap_or(available_width),
        IRNode::Image { style, .. } => style
            .width
            .map(|d| d.resolve(available_width))
            .unwrap_or(100.0 + node_padding(style).horizontal()),
        IRNode::Input { style, .. } => style
            .width
            .map(|d| d.resolve(available_width))
            .unwrap_or(240.0 + node_padding(style).horizontal()),
        IRNode::Button { title, style, .. } => style
            .width
            .map(|d| d.resolve(available_width))
            .unwrap_or_else(|| {
                title.chars().count() as f64 * 9.0 + 32.0 + node_padding(style).horizontal()
            }),
        IRNode::RecursiveComponent { .. } => 0.0,
    }
}

fn node_flex_grow(node: &IRNode) -> f64 {
    node_style(node).flex_grow.max(0.0)
}

fn node_image_mode(node: &IRNode) -> Option<ImageMode> {
    match node {
        IRNode::Image { mode, .. } => Some(*mode),
        _ => None,
    }
}

fn node_uses_intrinsic_dimension(node: &IRNode, vertical: bool) -> bool {
    let style = node_style(node);
    let dimension = if vertical { style.height } else { style.width };
    if dimension.is_some() {
        return false;
    }

    match node {
        IRNode::Text { .. }
        | IRNode::StateText { .. }
        | IRNode::ItemText { .. }
        | IRNode::ExprText { .. }
        | IRNode::Input { .. }
        | IRNode::Button { .. } => true,
        IRNode::Image { mode, .. } => {
            !matches!(mode, ImageMode::WidthFix | ImageMode::HeightFix)
        }
        IRNode::View { .. }
        | IRNode::VStack { .. }
        | IRNode::ScrollView { .. }
        | IRNode::List { .. }
        | IRNode::RecursiveComponent { .. } => false,
        IRNode::Conditional {
            initial,
            when_true,
            when_false,
            ..
        } => {
            let active = if *initial {
                Some(when_true.as_ref())
            } else {
                when_false.as_deref()
            };
            active.is_some_and(|node| node_uses_intrinsic_dimension(node, vertical))
        }
        IRNode::ItemConditional {
            initial,
            when_true,
            when_false,
            ..
        } => {
            let active = if *initial {
                when_true.as_ref()
            } else {
                when_false.as_deref().unwrap_or(when_true)
            };
            node_uses_intrinsic_dimension(active, vertical)
        }
    }
}

fn node_style(node: &IRNode) -> &InlineStyle {
    match node {
        IRNode::View { style, .. }
        | IRNode::VStack { style, .. }
        | IRNode::ScrollView { style, .. }
        | IRNode::Conditional { style, .. }
        | IRNode::Text { style, .. }
        | IRNode::StateText { style, .. }
        | IRNode::Image { style, .. }
        | IRNode::Input { style, .. }
        | IRNode::Button { style, .. }
        | IRNode::List { style, .. }
        | IRNode::ItemText { style, .. }
        | IRNode::ItemConditional { style, .. }
        | IRNode::RecursiveComponent { style, .. }
        | IRNode::ExprText { style, .. } => style,
    }
}

fn node_has_input_binding(node: &IRNode) -> bool {
    match node {
        IRNode::View { children, .. }
        | IRNode::VStack { children, .. }
        | IRNode::ScrollView { children, .. } => {
            children.iter().any(node_has_input_binding)
        }
        IRNode::Input { on_input, .. } => on_input.is_some(),
        IRNode::Conditional { when_true, when_false, .. } => {
            node_has_input_binding(when_true)
                || when_false.as_deref().is_some_and(node_has_input_binding)
        }
        _ => false,
    }
}

fn node_has_conditional(node: &IRNode) -> bool {
    match node {
        IRNode::Conditional { .. } => true,
        IRNode::View { children, .. }
        | IRNode::VStack { children, .. }
        | IRNode::ScrollView { children, .. } => children.iter().any(node_has_conditional),
        _ => false,
    }
}

/// 直接子级里有没有列表槽位：列表高度随行数变化，布局要走约束路径。
fn has_direct_list(children: &[IRNode]) -> bool {
    children
        .iter()
        .any(|child| matches!(child, IRNode::List { .. }))
}

fn has_direct_conditional(children: &[IRNode]) -> bool {
    children
        .iter()
        .any(|child| matches!(child, IRNode::Conditional { .. }))
}

fn validate_ignores_safe_area_views(nodes: &[IRNode]) -> Result<(), String> {
    for node in nodes {
        let style = node_style(node);
        if style.ignores_safe_area_top {
            if style.position == Position::Absolute {
                return Err("ignoresSafeArea: \"top\" only applies to views in flex flow".into());
            }
            if style.height.is_none() {
                return Err("ignoresSafeArea: \"top\" requires an explicit height".into());
            }
        }
        match node {
            IRNode::View { children, .. }
            | IRNode::VStack { children, .. }
            | IRNode::ScrollView { children, .. } => validate_ignores_safe_area_views(children)?,
            IRNode::Conditional {
                when_true,
                when_false,
                ..
            } => {
                validate_ignores_safe_area_views(std::slice::from_ref(when_true))?;
                if let Some(branch) = when_false.as_deref() {
                    validate_ignores_safe_area_views(std::slice::from_ref(branch))?;
                }
            }
            _ => {}
        }
    }
    Ok(())
}

fn conditional_active_at_state(node: &IRNode, state: bool) -> bool {
    match node {
        IRNode::Conditional { when_false, .. } => state || when_false.is_some(),
        _ => true,
    }
}

fn conditional_arranged_index(children: &[IRNode], target: usize, state: bool) -> usize {
    children
        .iter()
        .take(target)
        .filter(|child| {
            node_style(child).position != Position::Absolute
                && conditional_active_at_state(child, state)
        })
        .count()
}

/// 收集带 onInput 的输入框对应的状态动作下标（macOS 的 delegate 回调要用）。
fn collect_input_actions(nodes: &[IRNode], actions: &mut Vec<usize>) {
    for node in nodes {
        match node {
            IRNode::Input {
                on_input: Some(action),
                ..
            } => actions.push(*action),
            IRNode::View { children, .. }
            | IRNode::VStack { children, .. }
            | IRNode::ScrollView { children, .. } => collect_input_actions(children, actions),
            IRNode::Conditional {
                when_true,
                when_false,
                ..
            } => {
                collect_input_actions(std::slice::from_ref(when_true), actions);
                if let Some(node) = when_false {
                    collect_input_actions(std::slice::from_ref(node), actions);
                }
            }
            _ => {}
        }
    }
}

fn collect_initial_mount_actions(nodes: &[IRNode], actions: &mut Vec<usize>) {
    for node in nodes {
        actions.extend(node_style(node).mount_actions.iter().copied());
        match node {
            IRNode::View { children, .. }
            | IRNode::VStack { children, .. }
            | IRNode::ScrollView { children, .. } => {
                collect_initial_mount_actions(children, actions);
            }
            IRNode::Conditional {
                initial,
                when_true,
                when_false,
                true_mount,
                false_mount,
                ..
            } => {
                if *initial {
                    actions.extend(true_mount.iter().copied());
                    collect_initial_mount_actions(std::slice::from_ref(when_true), actions);
                } else if let Some(node) = when_false {
                    actions.extend(false_mount.iter().copied());
                    collect_initial_mount_actions(std::slice::from_ref(node), actions);
                }
            }
            _ => {}
        }
    }
}


fn find_view_by_id<'a>(nodes: &'a [IRNode], id: &str) -> Option<&'a IRNode> {    for node in nodes {
        match node {
            IRNode::View { style, .. } | IRNode::VStack { style, .. }
                if style.view_id.as_deref() == Some(id) =>
            {
                return Some(node);
            }

            IRNode::View { children, .. }
            | IRNode::VStack { children, .. }
            | IRNode::ScrollView { children, .. } => {
                if let Some(found) = find_view_by_id(children, id) {
                    return Some(found);
                }
            }
            IRNode::Conditional { when_true, when_false, .. } => {
                if let Some(found) = find_view_by_id(std::slice::from_ref(when_true), id) {
                    return Some(found);
                }
                if let Some(found) = when_false
                    .as_deref()
                    .and_then(|node| find_view_by_id(std::slice::from_ref(node), id))
                {
                    return Some(found);
                }
            }
            _ => {}
        }
    }
    None
}

fn find_top_background_view(nodes: &[IRNode]) -> Option<&IRNode> {
    for node in nodes {
        if node_style(node).background_extends_to_top {
            return Some(node);
        }
        let children = match node {
            IRNode::View { children, .. }
            | IRNode::VStack { children, .. }
            | IRNode::ScrollView { children, .. } => children,
            IRNode::Conditional {
                when_true,
                when_false,
                ..
            } => {
                if let Some(target) = find_top_background_view(std::slice::from_ref(when_true)) {
                    return Some(target);
                }
                if let Some(target) = when_false
                    .as_deref()
                    .and_then(|branch| find_top_background_view(std::slice::from_ref(branch)))
                {
                    return Some(target);
                }
                continue;
            }
            _ => continue,
        };
        if let Some(target) = find_top_background_view(children) {
            return Some(target);
        }
    }
    None
}

fn count_top_background_views(nodes: &[IRNode]) -> usize {
    nodes
        .iter()
        .map(|node| {
            usize::from(node_style(node).background_extends_to_top)
                + match node {
                    IRNode::View { children, .. }
                    | IRNode::VStack { children, .. }
                    | IRNode::ScrollView { children, .. } => count_top_background_views(children),
                    IRNode::Conditional {
                        when_true,
                        when_false,
                        ..
                    } => {
                        count_top_background_views(std::slice::from_ref(when_true))
                            + when_false.as_deref().map_or(0, |branch| {
                                count_top_background_views(std::slice::from_ref(branch))
                            })
                    }
                    _ => 0,
                }
        })
        .sum()
}

fn find_scroll_action(nodes: &[IRNode]) -> Option<usize> {
    for node in nodes {
        match node {
            IRNode::ScrollView { style, children } => {
                if let Some(action) = style.scroll_action {
                    return Some(action);
                }
                if let Some(action) = find_scroll_action(children) {
                    return Some(action);
                }
            }
            IRNode::View { children, .. }
            | IRNode::VStack { children, .. } => {
                if let Some(action) = find_scroll_action(children) {
                    return Some(action);
                }
            }
            IRNode::Conditional {
                when_true,
                when_false,
                ..
            } => {
                let found = find_scroll_action(std::slice::from_ref(when_true)).or_else(|| {
                    when_false
                        .as_deref()
                        .and_then(|node| find_scroll_action(std::slice::from_ref(node)))
                });
                if found.is_some() {
                    return found;
                }
            }
            IRNode::List { item, .. } => {
                if let Some(action) = find_scroll_action(std::slice::from_ref(item)) {
                    return Some(action);
                }
            }
            _ => {}
        }
    }
    None
}


fn count_scroll_actions(nodes: &[IRNode]) -> usize {
    let mut count = 0;
    for node in nodes {
        match node {
            IRNode::ScrollView { style, children } => {
                count += usize::from(style.scroll_action.is_some());
                count += count_scroll_actions(children);
            }
            IRNode::View { children, .. } | IRNode::VStack { children, .. } => {
                count += count_scroll_actions(children);
            }
            IRNode::Conditional {
                when_true,
                when_false,
                ..
            } => {
                count += count_scroll_actions(std::slice::from_ref(when_true));
                if let Some(node) = when_false {
                    count += count_scroll_actions(std::slice::from_ref(node));
                }
            }
            IRNode::List { item, .. } => {
                count += count_scroll_actions(std::slice::from_ref(item));
            }
            _ => {}
        }
    }
    count
}


fn scroll_action_cell(program: &UiProgram) -> Option<(&str, Option<(i64, i64, i64)>)> {
    let action = find_scroll_action(&program.nodes)?;
    match program.state_actions.get(action)? {
        StateMutation::SetFromScroll(cell) => Some((cell.as_str(), None)),
        StateMutation::SetFromScrollClamped {
            cell,
            low,
            high,
            shift,
        } => Some((cell.as_str(), Some((*low, *high, *shift)))),
        _ => None,
    }
}


fn count_views_with_id(nodes: &[IRNode], id: &str) -> usize {
    nodes
        .iter()
        .map(|node| match node {
            IRNode::View { style, children } | IRNode::VStack { style, children } => {
                usize::from(style.view_id.as_deref() == Some(id))
                    + count_views_with_id(children, id)
            }
            IRNode::ScrollView { children, .. } => count_views_with_id(children, id),
            IRNode::Conditional { when_true, when_false, .. } => {
                count_views_with_id(std::slice::from_ref(when_true), id)
                    + when_false.as_deref().map_or(0, |node| {
                        count_views_with_id(std::slice::from_ref(node), id)
                    })
            }
            _ => 0,
        })
        .sum()
}

fn scroll_content_size(
    children: &[IRNode],
    layouts: &[ChildLayout],
    viewport_width: f64,
    viewport_height: f64,
) -> (f64, f64) {
    children
        .iter()
        .zip(layouts)
        .filter(|(child, _)| node_style(child).position != Position::Absolute)
        .fold((viewport_width, viewport_height), |(width, height), (child, layout)| {
            let margins = node_margins(child);
            (
                width.max(layout.x + layout.width + margins.right),
                height.max(layout.y + layout.height + margins.bottom),
            )
        })
}

fn layout_flex_children(
    children: &[IRNode],
    style: &InlineStyle,
    width: f64,
    height: f64,
    default_direction: FlexDirection,
    default_gap: f64,
) -> Vec<ChildLayout> {
    let direction = style.flex_direction.unwrap_or(default_direction);
    let padding = node_padding(style);
    let gap = style.spacing.unwrap_or(default_gap);
    let inner_width = (width - padding.horizontal()).max(0.0);
    let inner_height = (height - padding.vertical()).max(0.0);
    let main_size = match direction {
        FlexDirection::Row => inner_width,
        FlexDirection::Column => inner_height,
    };
    let cross_size = match direction {
        FlexDirection::Row => inner_height,
        FlexDirection::Column => inner_width,
    };

    let margins: Vec<Margins> = children.iter().map(node_margins).collect();
    let flow_indices: Vec<usize> = children
        .iter()
        .enumerate()
        .filter_map(|(index, child)| {
            (node_style(child).position != Position::Absolute).then_some(index)
        })
        .collect();
    let mut main_sizes: Vec<f64> = flow_indices
        .iter()
        .map(|&index| match direction {
            FlexDirection::Row => node_width(&children[index], inner_width),
            FlexDirection::Column => match &children[index] {
                IRNode::ScrollView { style, .. }
                    if style.height.is_none() && style.flex_grow > 0.0 =>
                {
                    0.0
                }
                child => node_height(child, inner_height),
            },
        })
        .collect();
    let main_margin = |margin: Margins| match direction {
        FlexDirection::Row => margin.left + margin.right,
        FlexDirection::Column => margin.top + margin.bottom,
    };
    let main_margins_total: f64 = flow_indices
        .iter()
        .map(|&index| main_margin(margins[index]))
        .sum();
    let base_total = main_sizes.iter().sum::<f64>()
        + main_margins_total
        + gap * flow_indices.len().saturating_sub(1) as f64;
    let grow_total: f64 = flow_indices
        .iter()
        .map(|&index| node_flex_grow(&children[index]))
        .sum();
    let free_space = (main_size - base_total).max(0.0);
    if grow_total > 0.0 {
        for (size, &index) in main_sizes.iter_mut().zip(&flow_indices) {
            *size += free_space * node_flex_grow(&children[index]) / grow_total;
        }
    }

    let used_main = main_sizes.iter().sum::<f64>()
        + main_margins_total
        + gap * flow_indices.len().saturating_sub(1) as f64;
    let remaining = (main_size - used_main).max(0.0);
    let (main_offset, effective_gap) = match style.justify_content.unwrap_or(JustifyContent::Start) {
        JustifyContent::Start => (0.0, gap),
        JustifyContent::Center => (remaining / 2.0, gap),
        JustifyContent::End => (remaining, gap),
        JustifyContent::SpaceBetween if flow_indices.len() > 1 => {
            (0.0, gap + remaining / (flow_indices.len() - 1) as f64)
        }
        JustifyContent::SpaceAround if !flow_indices.is_empty() => {
            let extra = remaining / flow_indices.len() as f64;
            (extra / 2.0, gap + extra)
        }
        _ => (0.0, gap),
    };

    let mut layouts = vec![
        ChildLayout {
            x: 0.0,
            y: 0.0,
            width: 0.0,
            height: 0.0,
        };
        children.len()
    ];
    let mut cursor = main_offset;
    for (&index, child_main_size) in flow_indices.iter().zip(main_sizes) {
        let child = &children[index];
        let child_style = node_style(child);
        let margin = margins[index];
        let (main_margin_start, main_margin_end, cross_margin_start, cross_margin_end) =
            match direction {
                FlexDirection::Row => (margin.left, margin.right, margin.top, margin.bottom),
                FlexDirection::Column => (margin.top, margin.bottom, margin.left, margin.right),
            };
        let (explicit_cross, intrinsic_cross) = match direction {
            FlexDirection::Row => (
                child_style.height.map(|dimension| dimension.resolve(inner_height)),
                node_height(child, inner_height),
            ),
            FlexDirection::Column => (
                child_style.width.map(|dimension| dimension.resolve(inner_width)),
                node_width(child, inner_width),
            ),
        };
        let available_cross = (cross_size - cross_margin_start - cross_margin_end).max(0.0);
        let child_cross_size = explicit_cross.unwrap_or_else(|| {
            match style.align_items.unwrap_or(AlignItems::Stretch) {
                AlignItems::Stretch => available_cross,
                _ => intrinsic_cross.min(available_cross),
            }
        });
        let cross_offset = match style.align_items.unwrap_or(AlignItems::Stretch) {
            AlignItems::Center => {
                cross_margin_start + (available_cross - child_cross_size).max(0.0) / 2.0
            }
            AlignItems::End => cross_size - cross_margin_end - child_cross_size,
            _ => cross_margin_start,
        };
        cursor += main_margin_start;
        let relative_main_offset = if child_style.position == Position::Relative {
            match direction {
                FlexDirection::Row => child_style
                    .left
                    .map(|value| value.resolve(inner_width))
                    .or_else(|| child_style.right.map(|value| -value.resolve(inner_width)))
                    .unwrap_or(0.0),
                FlexDirection::Column => child_style
                    .top
                    .map(|value| value.resolve(inner_height))
                    .or_else(|| child_style.bottom.map(|value| -value.resolve(inner_height)))
                    .unwrap_or(0.0),
            }
        } else {
            0.0
        };
        let relative_cross_offset = if child_style.position == Position::Relative {
            match direction {
                FlexDirection::Row => child_style
                    .top
                    .map(|value| value.resolve(inner_height))
                    .or_else(|| child_style.bottom.map(|value| -value.resolve(inner_height)))
                    .unwrap_or(0.0),
                FlexDirection::Column => child_style
                    .left
                    .map(|value| value.resolve(inner_width))
                    .or_else(|| child_style.right.map(|value| -value.resolve(inner_width)))
                    .unwrap_or(0.0),
            }
        } else {
            0.0
        };
        layouts[index] = match direction {
            FlexDirection::Row => ChildLayout {
                x: padding.left + cursor + relative_main_offset,
                y: padding.top + cross_offset + relative_cross_offset,
                width: child_main_size,
                height: child_cross_size,
            },
            FlexDirection::Column => ChildLayout {
                x: padding.left + cross_offset + relative_cross_offset,
                y: padding.top + cursor + relative_main_offset,
                width: child_cross_size,
                height: child_main_size,
            },
        };
        cursor += child_main_size + main_margin_end + effective_gap;
    }

    for (index, child) in children.iter().enumerate() {
        let child_style = node_style(child);
        if child_style.position != Position::Absolute {
            continue;
        }
        let margin = margins[index];
        let left = child_style.left.map(|value| value.resolve(inner_width));
        let right = child_style.right.map(|value| value.resolve(inner_width));
        let top = child_style.top.map(|value| value.resolve(inner_height));
        let bottom = child_style.bottom.map(|value| value.resolve(inner_height));
        let child_width = child_style
            .width
            .map(|dimension| dimension.resolve(inner_width))
            .unwrap_or_else(|| {
                if left.is_some() && right.is_some() {
                    (inner_width - left.unwrap() - right.unwrap() - margin.left - margin.right)
                        .max(0.0)
                } else {
                    node_width(child, inner_width)
                }
            });
        let child_height = child_style
            .height
            .map(|dimension| dimension.resolve(inner_height))
            .unwrap_or_else(|| {
                if top.is_some() && bottom.is_some() {
                    (inner_height - top.unwrap() - bottom.unwrap() - margin.top - margin.bottom)
                        .max(0.0)
                } else {
                    node_height(child, inner_height)
                }
            });
        let x = left
            .map(|value| padding.left + value + margin.left)
            .or_else(|| {
                right.map(|value| width - padding.right - value - margin.right - child_width)
            })
            .unwrap_or(padding.left + margin.left);
        let y = top
            .map(|value| padding.top + value + margin.top)
            .or_else(|| {
                bottom.map(|value| height - padding.bottom - value - margin.bottom - child_height)
            })
            .unwrap_or(padding.top + margin.top);
        layouts[index] = ChildLayout {
            x,
            y,
            width: child_width,
            height: child_height,
        };
    }
    layouts
}

/// 状态初值：标量、一层对象字面量（对象会被展开成多个状态单元），
/// 或对象数组（列表状态：每个元素字段集相同，第一行同时充当字段类型样本）。
#[derive(Debug, Clone, PartialEq)]
enum StateInit {
    Scalar(StateValue),
    Object(Vec<(String, StateValue)>),
    List(ListInit),
}

#[derive(Debug, Clone, PartialEq)]
struct ListInit {
    fields: Vec<String>,
    rows: Vec<Vec<StateValue>>,
    /// 每行的父行下标（None = 根层），有值说明是嵌套列表。
    parents: Vec<Option<usize>>,
}

impl ListInit {
    fn has_children(&self) -> bool {
        self.parents.iter().any(|parent| parent.is_some())
    }
}

impl ListInit {
    fn field_kind(&self, field: &str) -> Option<&StateValue> {
        let index = self.fields.iter().position(|name| name == field)?;
        self.rows.first().and_then(|row| row.get(index))
    }

    fn field_index(&self, field: &str) -> Option<usize> {
        self.fields.iter().position(|name| name == field)
    }
}

fn list_init<'a>(states: &'a [StateDecl], name: &str) -> Option<&'a ListInit> {
    states.iter().find(|decl| decl.name == name).and_then(|decl| match &decl.initial {
        StateInit::List(list) => Some(list),
        _ => None,
    })
}

#[derive(Debug, Clone, PartialEq)]
struct StateDecl {
    name: String,
    initial: StateInit,
}

impl StateDecl {
    /// 展开成可独立更新的状态单元：标量状态是它自己，对象状态是每个字段。
    fn cells(&self) -> Vec<(String, StateValue)> {
        match &self.initial {
            StateInit::Scalar(value) => vec![(self.name.clone(), value.clone())],
            StateInit::Object(fields) => fields
                .iter()
                .map(|(field, value)| (format!("{}.{}", self.name, field), value.clone()))
                .collect(),
            // 列表的行有独立存储，整表重建；但长度对外仍是一个普通单元，
            // 这样 `{items.length}` 可以像其它文本一样绑定、由列表动作维护。
            StateInit::List(list) => vec![(
                format!("{}.length", self.name),
                StateValue::Number(list.rows.len() as i64),
            )],
        }
    }
}

fn state_cells(states: &[StateDecl]) -> Vec<(String, StateValue)> {
    states.iter().flat_map(StateDecl::cells).collect()
}

#[derive(Debug, Clone, PartialEq)]
pub enum StateValue {
    Number(i64),
    String(String),
    Boolean(bool),
}

#[derive(Debug, Clone, PartialEq)]
enum StateMutation {
    Noop,
    Increment(String),
    Decrement(String),
    Set(String, StateValue),
    SetFromInput(String),
    SetTrue(String),
    /// 多语句 handler（`() => { a = true; b++ }`）按源码顺序执行。
    Sequence(Vec<StateMutation>),
    /// `items.push({ title: draft })`：追加一行，字段取字面量或状态单元当前值。
    PushItem {
        list: String,
        values: Vec<(String, ItemValue)>,
    },
    /// 列表模板内的 `item.done = true`：写当前行（由控件 tag 带出下标）的字段。
    SetItemField {
        list: String,
        field: String,
        value: StateValue,
    },
    /// 列表模板内的 `items.removeAt(index)`：删除当前行。
    RemoveCurrentItem { list: String },
    /// `items = []`：清空列表。
    ClearList { list: String },
    /// `profile = { ... }`：整对象覆盖，逐单元写入。
    SetCells { values: Vec<(String, StateValue)> },
    /// `a = b` / `a = b + 1` / `a = b - 1`：从另一个数值单元取值。
    SetFromCell {
        target: String,
        source: String,
        delta: i64,
    },
    /// `onScroll={(y) => offset = y}`：滚动位置由平台回调写入这个数值单元。
    SetFromScroll(String),
    /// `onScroll={(y) => progress = clamp(y, lo, hi)}`：写入前先夹到 [lo, hi]。
    /// `shift` 是滚动位置的偏移（`clamp(y - 80, 0, 120)`）。
    SetFromScrollClamped {
        cell: String,
        low: i64,
        high: i64,
        shift: i64,
    },
}

/// push 一行时单个字段的取值来源。
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum ItemValue {
    Literal(StateValue),
    /// 读某个状态单元的当前值（`{ title: draft }`）。
    Cell(String),
}

impl StateMutation {
    /// 该动作写入的状态单元路径。
    fn cell(&self) -> Option<&str> {
        match self {
            Self::Noop | Self::Sequence(_) => None,
            Self::Increment(cell)
            | Self::Decrement(cell)
            | Self::SetFromInput(cell)
            | Self::SetTrue(cell) => Some(cell),
            Self::Set(cell, _) => Some(cell),
            Self::SetFromCell { target, .. } => Some(target),
            Self::SetFromScroll(cell) => Some(cell),
            Self::SetFromScrollClamped { cell, .. } => Some(cell),
            // 列表动作单独走行存储，不写标量单元。
            Self::PushItem { .. }
            | Self::SetItemField { .. }
            | Self::RemoveCurrentItem { .. }
            | Self::ClearList { .. }
            | Self::SetCells { .. } => None,
        }
    }
}

impl StateValue {
    fn display_value(&self) -> String {
        match self {
            Self::Number(value) => value.to_string(),
            Self::String(value) => value.clone(),
            Self::Boolean(value) => value.to_string(),
        }
    }
}

#[derive(Debug)]
struct UiProgram {
    nodes: Vec<IRNode>,
    states: Vec<StateDecl>,
    state_actions: Vec<StateMutation>,
    errors: Vec<String>,
}


#[derive(Debug, Clone)]
enum ComponentProp {
    Literal(StateValue),
    StateReference(String),
    /// 列表行引用：`<Row item={item} />`（field 为 None）或 `<Row title={item.title} />`。
    RowField {
        list: String,
        field: Option<String>,
    },
    /// 行下标：`<Row item={item} index={index} />`，组件里可用于 `items.removeAt(index)`。
    RowIndex { list: String },
    /// 子树行集合：`<TreeRow node={node.children} />`，运行期按父行递归渲染。
    RowChildren { list: String },
}

#[derive(Debug, Clone)]
enum BooleanCondition {
    State {
        cell: String,
        initial: bool,
        negated: bool,
    },
    Static(bool),
}

#[derive(Debug, Clone)]
struct ComponentDefinition<'a> {
    root: &'a JSXElement,
    props_parameter: Option<String>,
    named_props: Vec<String>,
    on_mount: Option<StateMutation>,
    on_unmount: Option<StateMutation>,
    unsupported_lifecycle: bool,
    /// 组件体自己声明的状态：每次展开按实例分配单元。
    state_decls: Vec<StateDecl>,
}

struct JsxToIrVisitor<'a> {
    ir_out: Vec<IRNode>,
    cells: Vec<(String, StateValue)>,
    state_actions: Vec<StateMutation>,
    components: HashMap<String, ComponentDefinition<'a>>,
    props: HashMap<String, ComponentProp>,
    component_stack: Vec<String>,
    lists: HashMap<String, ListInit>,
    item_scope: Option<ItemScope>,
    /// 组件实例分配出来的单元（展开结束时并入 program.states）
    instance_states: Vec<StateDecl>,
    /// 别名 -> 实际单元名（组件体内的局部名字）
    scopes: Vec<HashMap<String, String>>,
    /// 每个组件已展开的实例数
    instance_counters: HashMap<String, usize>,
    errors: Vec<String>,
}

/// `.map((item, index) => ...)` 里当前行的绑定：列表名 + 元素参数名 + 下标参数名。
#[derive(Debug, Clone)]
struct ItemScope {
    list: String,
    param: String,
    index_param: Option<String>,
}

impl JsxToIrVisitor<'_> {
    fn cell_initial(&self, cell: &str) -> Option<&StateValue> {
        let resolved = self.resolve_name(cell);
        self.cells
            .iter()
            .find(|(path, _)| *path == resolved)
            .map(|(_, value)| value)
    }

    /// 组件实例作用域：把组件体里的局部名字解析成带实例前缀的单元名。
    fn resolve_name(&self, path: &str) -> String {
        let Some(scope) = self.scopes.last() else {
            return path.to_string();
        };
        let head = path.split('.').next().unwrap_or(path);
        match scope.get(head) {
            Some(prefix) => format!("{prefix}{path}"),
            None => path.to_string(),
        }
    }

    /// 解析 handler 时用的单元表：把实例单元冒名成组件体里的局部名字。
    fn scoped_cells(&self) -> Vec<(String, StateValue)> {
        let mut cells = self.cells.clone();
        let Some(scope) = self.scopes.last() else {
            return cells;
        };
        for (alias, prefix) in scope {
            for (path, value) in &self.cells {
                if let Some(rest) = path.strip_prefix(prefix.as_str()) {
                    if rest == alias {
                        cells.push((alias.clone(), value.clone()));
                    } else if let Some(field) = rest.strip_prefix(&format!("{alias}.")) {
                        cells.push((format!("{alias}.{field}"), value.clone()));
                    }
                }
            }
        }
        cells
    }

    /// 行内条件：`{item.done ? <A /> : <B />}` 或 `{item.done && <A />}`。
    fn item_conditional(&mut self, expression: &Expr) -> Option<IRNode> {
        let scope = self.item_scope.clone()?;
        let (field, when_true_expr, when_false_expr) = match expression {
            Expr::Cond(conditional) => {
                let (list, field) = item_field_path(conditional.test.as_ref(), &scope)?;
                if list != scope.list {
                    return None;
                }
                (field, conditional.cons.as_ref(), Some(conditional.alt.as_ref()))
            }
            Expr::Bin(binary) if binary.op == BinaryOp::LogicalAnd => {
                let (list, field) = item_field_path(binary.left.as_ref(), &scope)?;
                if list != scope.list {
                    return None;
                }
                (field, binary.right.as_ref(), None)
            }
            _ => return None,
        };
        let initial = match self
            .lists
            .get(&scope.list)
            .and_then(|schema| schema.field_kind(&field))
        {
            Some(StateValue::Boolean(value)) => *value,
            _ => {
                self.errors.push(format!(
                    "列表行条件需要使用布尔字段：{}.{field}",
                    scope.list
                ));
                return None;
            }
        };
        let when_true = self.branch_node(when_true_expr)?;
        let when_false = match when_false_expr {
            Some(expr) => Some(Box::new(self.branch_node(expr)?)),
            None => None,
        };
        let active = if initial {
            &when_true
        } else {
            when_false.as_deref().unwrap_or(&when_true)
        };
        let style = node_style(active).clone();
        Some(IRNode::ItemConditional {
            list: scope.list.clone(),
            field,
            initial,
            when_true: Box::new(when_true),
            when_false,
            style,
        })
    }

    /// `{a + b}` / `{a - b}`：两个数值操作数（单元或整数字面量）编译成表达式文本。
    fn expression_text(&self, expression: &Expr) -> Option<IRNode> {
        let Expr::Bin(binary) = expression else {
            return None;
        };
        let subtract = match binary.op {
            BinaryOp::Add => false,
            BinaryOp::Sub => true,
            _ => return None,
        };
        let operand = |expr: &Expr| -> Option<ExprOperand> {
            if let Some(cell) = expr_state_path(expr) {
                let resolved = self.resolve_name(&cell);
                return match self.cell_initial(&resolved)? {
                    StateValue::Number(_) => Some(ExprOperand::Cell(resolved)),
                    _ => None,
                };
            }
            match expr {
                Expr::Lit(Lit::Num(number)) if number.value.fract() == 0.0 => {
                    Some(ExprOperand::Number(number.value as i64))
                }
                _ => None,
            }
        };
        let left = operand(binary.left.as_ref())?;
        let right = operand(binary.right.as_ref())?;
        let deps: Vec<String> = [&left, &right]
            .into_iter()
            .filter_map(|operand| match operand {
                ExprOperand::Cell(cell) => Some(cell.clone()),
                ExprOperand::Number(_) => None,
            })
            .collect();
        let (left_value, right_value) = (
            left.initial(&self.cells),
            right.initial(&self.cells),
        );
        let initial = if subtract {
            left_value - right_value
        } else {
            left_value + right_value
        };
        Some(IRNode::ExprText {
            left,
            right,
            subtract,
            deps,
            initial: initial.to_string(),
            style: InlineStyle::default(),
        })
    }

    /// 解析 `.map(...)` 用的列表名：组件内的列表是实例名。
    fn scoped_lists(&self) -> HashMap<String, ListInit> {
        let mut lists = self.lists.clone();
        let Some(scope) = self.scopes.last() else {
            return lists;
        };
        for (alias, prefix) in scope {
            if let Some(schema) = self.lists.get(&format!("{prefix}{alias}")) {
                lists.insert(alias.clone(), schema.clone());
            }
        }
        lists
    }

    /// 识别 `{items.map(...)}`，返回解析后的列表名。
    fn list_map_call_resolved<'a>(&self, expression: &'a Expr) -> Option<(String, &'a ArrowExpr)> {
        let Expr::Call(call) = expression else {
            return None;
        };
        let Callee::Expr(callee) = &call.callee else {
            return None;
        };
        let Expr::Member(member) = callee.as_ref() else {
            return None;
        };
        let MemberProp::Ident(method) = &member.prop else {
            return None;
        };
        if method.sym.as_str() != "map" {
            return None;
        }
        let Expr::Ident(list_name) = member.obj.as_ref() else {
            return None;
        };
        let resolved = self.resolve_name(list_name.sym.as_str());
        if !self.lists.contains_key(&resolved) {
            return None;
        }
        let [argument] = call.args.as_slice() else {
            return None;
        };
        let Expr::Arrow(arrow) = argument.expr.as_ref() else {
            return None;
        };
        Some((resolved, arrow))
    }

    /// 把动作里的单元名/列表名换回实例的真实名字。
    fn resolve_mutation(&self, mutation: StateMutation) -> StateMutation {
        match mutation {
            StateMutation::Increment(cell) => StateMutation::Increment(self.resolve_name(&cell)),
            StateMutation::Decrement(cell) => StateMutation::Decrement(self.resolve_name(&cell)),
            StateMutation::SetTrue(cell) => StateMutation::SetTrue(self.resolve_name(&cell)),
            StateMutation::SetFromInput(cell) => {
                StateMutation::SetFromInput(self.resolve_name(&cell))
            }
            StateMutation::Set(cell, value) => StateMutation::Set(self.resolve_name(&cell), value),
            StateMutation::SetFromScroll(cell) => {
                StateMutation::SetFromScroll(self.resolve_name(&cell))
            }
            StateMutation::SetFromScrollClamped {
                cell,
                low,
                high,
                shift,
            } => StateMutation::SetFromScrollClamped {
                cell: self.resolve_name(&cell),
                low,
                high,
                shift,
            },
            StateMutation::SetFromCell {
                target,
                source,
                delta,
            } => StateMutation::SetFromCell {
                target: self.resolve_name(&target),
                source: self.resolve_name(&source),
                delta,
            },
            StateMutation::SetCells { values } => StateMutation::SetCells {
                values: values
                    .into_iter()
                    .map(|(cell, value)| (self.resolve_name(&cell), value))
                    .collect(),
            },
            StateMutation::PushItem { list, values } => StateMutation::PushItem {
                list: self.resolve_name(&list),
                values,
            },
            StateMutation::SetItemField {
                list,
                field,
                value,
            } => StateMutation::SetItemField {
                list: self.resolve_name(&list),
                field,
                value,
            },
            StateMutation::RemoveCurrentItem { list } => StateMutation::RemoveCurrentItem {
                list: self.resolve_name(&list),
            },
            StateMutation::ClearList { list } => StateMutation::ClearList {
                list: self.resolve_name(&list),
            },
            StateMutation::Sequence(steps) => StateMutation::Sequence(
                steps
                    .into_iter()
                    .map(|step| self.resolve_mutation(step))
                    .collect(),
            ),
            StateMutation::Noop => StateMutation::Noop,
        }
    }

    fn add_action(&mut self, mutation: StateMutation) -> usize {
        self.state_actions.push(mutation);
        self.state_actions.len() - 1
    }

    fn branch_node(&mut self, expression: &Expr) -> Option<IRNode> {
        let Expr::JSXElement(element) = expression else {
            self.errors.push("conditional branches must return one JSX element".into());
            return None;
        };
        let start = self.ir_out.len();
        self.visit_jsx_element(element);
        let mut nodes: Vec<_> = self.ir_out.drain(start..).collect();
        if nodes.len() != 1 {
            self.errors.push("conditional branches must contain one component or element".into());
            return None;
        }
        nodes.pop()
    }

    fn conditional_node(
        &mut self,
        cell: &str,
        initial: bool,
        when_true_expr: &Expr,
        when_false_expr: Option<&Expr>,
        swapped: bool,
    ) -> Option<IRNode> {
        let mut when_true = self.branch_node(when_true_expr)?;
        let mut when_false = when_false_expr.and_then(|expr| self.branch_node(expr));
        if when_false_expr.is_some() && when_false.is_none() {
            return None;
        }
        if swapped {
            std::mem::swap(&mut when_true, when_false.as_mut().expect("false branch exists"));
        }

        let (true_mount, true_unmount) = take_lifecycle_actions(&mut when_true);
        let (false_mount, false_unmount) = if let Some(node) = &mut when_false {
            take_lifecycle_actions(node)
        } else {
            (Vec::new(), Vec::new())
        };
        let active = if initial {
            &when_true
        } else {
            when_false.as_ref().unwrap_or(&when_true)
        };
        let style = node_style(active).clone();
        Some(IRNode::Conditional {
            cell: cell.to_string(),
            style,
            when_true: Box::new(when_true),
            when_false: when_false.map(Box::new),
            initial,
            true_mount,
            true_unmount,
            false_mount,
            false_unmount,
        })
    }

    fn conditional_from_expr(&mut self, expression: &Expr) -> Option<IRNode> {
        match expression {
            Expr::Bin(binary) if binary.op == BinaryOp::LogicalAnd => {
                match self.resolve_boolean_condition(binary.left.as_ref())? {
                    BooleanCondition::State {
                        cell,
                        initial,
                        negated: false,
                    } => self.conditional_node(
                        &cell,
                        initial,
                        binary.right.as_ref(),
                        None,
                        false,
                    ),
                    BooleanCondition::State { negated: true, .. } => {
                        self.errors.push("use a ternary expression for negated conditional rendering".into());
                        None
                    }
                    BooleanCondition::Static(true) => self.branch_node(binary.right.as_ref()),
                    BooleanCondition::Static(false) => None,
                }
            }
            Expr::Cond(conditional) => {
                match self.resolve_boolean_condition(conditional.test.as_ref())? {
                    BooleanCondition::State {
                        cell,
                        initial,
                        negated,
                    } => self.conditional_node(
                        &cell,
                        initial,
                        conditional.cons.as_ref(),
                        Some(conditional.alt.as_ref()),
                        negated,
                    ),
                    BooleanCondition::Static(true) => self.branch_node(conditional.cons.as_ref()),
                    BooleanCondition::Static(false) => self.branch_node(conditional.alt.as_ref()),
                }
            }
            _ => None,
        }
    }

    fn resolve_boolean_condition(&mut self, expression: &Expr) -> Option<BooleanCondition> {
        let (expression, negated) = match expression {
            Expr::Unary(unary) if unary.op == UnaryOp::Bang => (unary.arg.as_ref(), true),
            expression => (expression, false),
        };
        if let Some(cell) = expr_state_path(expression) {
            if self.cell_initial(&cell).is_some() {
                let initial = match self.cell_initial(&cell) {
                    Some(StateValue::Boolean(value)) => *value,
                    _ => {
                        self.errors
                            .push("conditional rendering requires state(false) or state(true)".into());
                        return None;
                    }
                };
                return Some(BooleanCondition::State {
                    cell: self.resolve_name(&cell),
                    initial,
                    negated,
                });
            }
        }
        let Some(prop) = component_prop_for_expr(expression, &self.props) else {
            self.errors.push(
                "conditional rendering requires a boolean state or boolean component prop".into(),
            );
            return None;
        };
        match prop {
            ComponentProp::Literal(StateValue::Boolean(value)) => {
                Some(BooleanCondition::Static(value ^ negated))
            }
            ComponentProp::RowField { .. } | ComponentProp::RowIndex { .. } => {
                self.errors
                    .push("列表行内的条件渲染还没支持；先用文本字段展示".into());
                None
            }
            ComponentProp::StateReference(cell) => {
                let initial = match self.cell_initial(&cell) {
                    Some(StateValue::Boolean(value)) => *value,
                    _ => {
                        self.errors
                            .push("conditional rendering requires state(false) or state(true)".into());
                        return None;
                    }
                };
                Some(BooleanCondition::State {
                    cell,
                    initial,
                    negated,
                })
            }
            _ => {
                self.errors.push("conditional expressions require a boolean value".into());
                None
            }
        }
    }

    fn component_props(
        &mut self,
        node: &JSXElement,
        definition: &ComponentDefinition<'_>,
    ) -> HashMap<String, ComponentProp> {
        let mut props = HashMap::new();
        for attribute in &node.opening.attrs {
            let JSXAttrOrSpread::JSXAttr(attribute) = attribute else { continue };
            let JSXAttrName::Ident(name) = &attribute.name else { continue };
            let key = name.sym.as_str();
            let is_declared = definition.props_parameter.is_some()
                || definition.named_props.iter().any(|prop| prop == key);
            if !is_declared {
                continue;
            }
            let item_scope = self.item_scope.clone();
            let Some(value) =
                component_prop_value(attribute, &self.cells, &self.props, item_scope.as_ref())
            else {
                self.errors.push(format!(
                    "component prop '{key}' only supports string, integer, boolean, or state-reference values"
                ));
                continue;
            };
            if let Some(parameter) = &definition.props_parameter {
                props.insert(format!("{parameter}.{key}"), value.clone());
            }
            if definition.named_props.iter().any(|prop| prop == key) {
                props.insert(key.to_string(), value);
            }
        }
        props
    }
}

fn has_recursive_slot(node: &IRNode) -> bool {
    match node {
        IRNode::RecursiveComponent { .. } => true,
        IRNode::View { children, .. }
        | IRNode::VStack { children, .. }
        | IRNode::ScrollView { children, .. } => children.iter().any(has_recursive_slot),
        IRNode::Conditional {
            when_true,
            when_false,
            ..
        } => {
            has_recursive_slot(when_true)
                || when_false.as_deref().is_some_and(has_recursive_slot)
        }
        IRNode::ItemConditional {
            when_true,
            when_false,
            ..
        } => {
            has_recursive_slot(when_true)
                || when_false.as_deref().is_some_and(has_recursive_slot)
        }
        _ => false,
    }
}

/// 行模板根容器里是否带递归槽位：带的话，行函数要连子树一起建。
fn row_template_recurses(item: &IRNode) -> bool {
    match item {
        IRNode::View { children, .. }
        | IRNode::VStack { children, .. }
        | IRNode::ScrollView { children, .. } => children
            .iter()
            .any(|child| matches!(child, IRNode::RecursiveComponent { .. })),
        _ => false,
    }
}

/// 去掉槽位后的行体：子树由行函数自己挂，槽位不再是一个普通的子视图。
fn strip_recursive_slots(item: &IRNode) -> IRNode {
    let mut body = item.clone();
    if let IRNode::View { children, .. }
    | IRNode::VStack { children, .. }
    | IRNode::ScrollView { children, .. } = &mut body
    {
        children.retain(|child| !matches!(child, IRNode::RecursiveComponent { .. }));
    }
    body
}

/// 递归槽位必须是组件根容器的直接子节点：运行期拿它当「子行挂载点」，
/// 而且根容器要纵向排列，否则子行没有稳定的堆叠方向。
fn validate_recursive_slots(root: &IRNode) -> Result<(), String> {
    let (style, children) = match root {
        IRNode::View { style, children }
        | IRNode::VStack { style, children }
        | IRNode::ScrollView { style, children } => (style, children),
        _ => {
            return if has_recursive_slot(root) {
                Err("recursive component root must be a View/VStack/ScrollView".into())
            } else {
                Ok(())
            };
        }
    };
    let direct = children
        .iter()
        .filter(|child| matches!(child, IRNode::RecursiveComponent { .. }))
        .count();
    if direct > 1 {
        return Err("recursive component can host only one `.children` slot".into());
    }
    if children
        .iter()
        .any(|child| !matches!(child, IRNode::RecursiveComponent { .. }) && has_recursive_slot(child))
    {
        return Err("the `.children` slot must be a direct child of the component root".into());
    }
    if direct == 1 && style.flex_direction == Some(FlexDirection::Row) {
        return Err("recursive component root must stack its `.children` slot vertically".into());
    }
    Ok(())
}

impl Visit for JsxToIrVisitor<'_> {
    fn visit_jsx_element(&mut self, node: &JSXElement) {
        let tag_name = match &node.opening.name {
            JSXElementName::Ident(ident) => ident.sym.as_str(),
            _ => return,
        };

        if let Some(definition) = self.components.get(tag_name).cloned() {
            let component_props = self.component_props(node, &definition);
            // 递归组件靠「子树行集合」进入下一层：`<TreeRow node={node.children} />`。
            // 这里不展开组件体，只落一个运行期槽位。
            if let Some(list) = component_props.values().find_map(|prop| match prop {
                ComponentProp::RowChildren { list } => Some(list.clone()),
                _ => None,
            }) {
                if self.component_stack.last().map(String::as_str) != Some(tag_name) {
                    self.errors.push(format!(
                        "recursive component {tag_name} must call itself; mutual recursion is unsupported"
                    ));
                    return;
                }
                if !definition.state_decls.is_empty() {
                    self.errors.push(format!(
                        "recursive component {tag_name} cannot declare state(...): every runtime row shares one template"
                    ));
                }
                let current = self.item_scope.as_ref().map(|scope| scope.list.clone());
                if current.as_deref() != Some(list.as_str()) {
                    self.errors.push(format!(
                        "recursive component {tag_name} must recurse over the tree list it is rendered from"
                    ));
                    return;
                }
                self.ir_out.push(IRNode::RecursiveComponent {
                    component: tag_name.to_string(),
                    list,
                    style: InlineStyle::default(),
                });
                return;
            }
            if self.component_stack.iter().any(|component| component == tag_name) {
                self.errors.push(format!(
                    "recursive component expansion is unsupported: {tag_name} (recurse with a `.children` prop instead)"
                ));
                return;
            }
            if definition.unsupported_lifecycle {
                self.errors.push(format!(
                    "component {tag_name} has an unsupported onMount/onUnmount callback; use an empty callback or a supported state mutation"
                ));
            }
            // 整个行元素作为 prop（<Row item={item} />）时，组件体里的 `item.title`
            // 仍然是「当前行字段」，所以要把行作用域带进这次展开。
            let row_item = component_props.iter().find_map(|(key, prop)| match prop {
                ComponentProp::RowField { list, field: None } => Some((
                    list.clone(),
                    key.rsplit('.').next().unwrap_or(key).to_string(),
                )),
                _ => None,
            });
            let row_index = component_props.iter().find_map(|(key, prop)| match prop {
                ComponentProp::RowIndex { list } => Some((
                    list.clone(),
                    key.rsplit('.').next().unwrap_or(key).to_string(),
                )),
                _ => None,
            });
            let row_scope = match (row_item, row_index) {
                (Some((list, param)), index) => Some(ItemScope {
                    list,
                    param,
                    index_param: index.map(|(_, name)| name),
                }),
                (None, Some((list, index_param))) => Some(ItemScope {
                    list,
                    param: String::new(),
                    index_param: Some(index_param),
                }),
                (None, None) => None,
            };
            // 组件体里的 state 按实例分配单元：每次展开一个独立前缀。
            let mut instance_scope = HashMap::new();
            if !definition.state_decls.is_empty() {
                let instance = *self.instance_counters.get(tag_name).unwrap_or(&0);
                self.instance_counters.insert(tag_name.to_string(), instance + 1);
                let prefix = format!("{tag_name}_{instance}.");
                for decl in &definition.state_decls {
                    let allocated = StateDecl {
                        name: format!("{prefix}{}", decl.name),
                        initial: decl.initial.clone(),
                    };
                    if let StateInit::List(list) = &decl.initial {
                        self.lists.insert(allocated.name.clone(), list.clone());
                    }
                    self.cells.extend(allocated.cells());
                    self.instance_states.push(allocated);
                    instance_scope.insert(decl.name.clone(), prefix.clone());
                }
            }
            let instance_scope_active = !instance_scope.is_empty();
            if instance_scope_active {
                self.scopes.push(instance_scope);
            }
            let previous_props = std::mem::replace(&mut self.props, component_props);
            let previous_scope = match row_scope {
                Some(scope) => std::mem::replace(&mut self.item_scope, Some(scope)),
                None => self.item_scope.clone(),
            };
            self.component_stack.push(tag_name.to_string());
            let start = self.ir_out.len();
            self.visit_jsx_element(definition.root);
            self.component_stack.pop();
            self.props = previous_props;
            self.item_scope = previous_scope;
            if instance_scope_active {
                self.scopes.pop();
            }
            let mut expanded: Vec<_> = self.ir_out.drain(start..).collect();
            if expanded.len() != 1 {
                self.errors.push(format!("component {tag_name} must return one JSX element"));
                return;
            }
            if let Err(error) = validate_recursive_slots(&expanded[0]) {
                self.errors.push(error);
            }
            let root = &mut expanded[0];
            if let Some(mutation) = &definition.on_mount {
                let action = self.add_action(mutation.clone());
                node_style_mut(root).mount_actions.push(action);
            }
            if let Some(mutation) = &definition.on_unmount {
                let action = self.add_action(mutation.clone());
                node_style_mut(root).unmount_actions.push(action);
            }
            self.ir_out.push(expanded.remove(0));
            return;
        }

        let style = jsx_style(node, &self.props);
        if style.scroll_fade_target.is_some()
            || style.scroll_fade_start.is_some()
            || style.scroll_fade_distance.is_some()
        {
            self.errors.push(
                "fadeTarget/fadeStart/fadeDistance 已移除：改用 <ScrollView onScroll={(y) => progress = clamp(y, 0, 100)}> + <View style={{ opacity: progress }}>"
                    .into(),
            );
        }
        let title = jsx_string_attr_with_props(node, "title", &self.props).unwrap_or_default();
        let source = jsx_string_attr_with_props(node, "source", &self.props)
            .or_else(|| jsx_string_attr_with_props(node, "src", &self.props))
            .unwrap_or_default();
        let placeholder = jsx_string_attr_with_props(node, "placeholder", &self.props).unwrap_or_default();
        let cells = self.scoped_cells();
        let lists = self.scoped_lists();
        let item_scope = self.item_scope.clone();
        let click_handler = jsx_attr(node, "onClick").and_then(|attr| attr.value.as_ref());
        let action = click_handler
            .and_then(|value| parse_state_mutation(value, &cells, item_scope.as_ref(), &lists))
            .map(|mutation| self.resolve_mutation(mutation));
        let unsupported_handler = jsx_attr(node, "onClick").is_some() && action.is_none();
        let input_handler = jsx_attr(node, "onInput").and_then(|attr| attr.value.as_ref());
        let on_input = input_handler
            .and_then(|value| parse_state_mutation(value, &cells, item_scope.as_ref(), &lists))
            .map(|mutation| self.resolve_mutation(mutation));
        let focus_handler = jsx_attr(node, "onFocus").and_then(|attr| attr.value.as_ref());
        let on_focus = focus_handler.and_then(|value| {
            if is_empty_event_handler(value) {
                Some(StateMutation::Noop)
            } else {
                parse_state_mutation(value, &cells, item_scope.as_ref(), &lists)
                    .map(|mutation| self.resolve_mutation(mutation))
            }
        });
        let unsupported_input = jsx_attr(node, "onInput").is_some() && on_input.is_none();
        let unsupported_focus = jsx_attr(node, "onFocus").is_some() && on_focus.is_none();
        let mut children = Vec::new();
        for child in &node.children {
            match child {
                JSXElementChild::JSXText(t) => {
                    let content = t.value.as_str().unwrap_or("").trim().to_string();
                    if !content.is_empty() {
                        children.push(IRNode::Text {
                            content,
                            style: InlineStyle::default(),
                        });
                    }
                }
                JSXElementChild::JSXElement(el) => {
                    let before = self.ir_out.len();
                    self.visit_jsx_element(el);
                    for ir in self.ir_out.drain(before..) {
                        children.push(ir);
                    }
                }
                JSXElementChild::JSXExprContainer(container) => {
                    if let JSXExpr::Expr(expr) = &container.expr {
                        // 列表渲染：`{items.map((item) => <Row />)}`。
                        if let Some((list_name, arrow)) = self.list_map_call_resolved(expr) {
                            if self.item_scope.is_some() {
                                self.errors.push("nested list rendering is unsupported".into());
                                continue;
                            }
                            let (Some(element), Some(scope)) =
                                (map_body_element(arrow), list_item_scope(&list_name, arrow))
                            else {
                                self.errors.push(format!(
                                    "list rendering {list_name}.map(...) needs an (item) or (item, index) parameter and one returned JSX element"
                                ));
                                continue;
                            };
                            let schema = self.lists[&list_name].clone();
                            let previous = self.item_scope.replace(scope);
                            let before = self.ir_out.len();
                            self.visit_jsx_element(element);
                            let mut nodes: Vec<_> = self.ir_out.drain(before..).collect();
                            self.item_scope = previous;
                            if nodes.len() != 1 {
                                self.errors.push("list items must render one JSX element".into());
                                continue;
                            }
                            children.push(IRNode::List {
                                style: InlineStyle::default(),
                                list: list_name.to_string(),
                                item: Box::new(nodes.pop().expect("checked above")),
                                fields: schema
                                    .fields
                                    .iter()
                                    .map(|field| {
                                        (
                                            field.clone(),
                                            schema.field_kind(field).cloned().unwrap_or(StateValue::String(
                                                String::new(),
                                            )),
                                        )
                                    })
                                    .collect(),
                                rows: schema.rows.clone(),
                                parents: schema.parents.clone(),
                            });
                            continue;
                        }
                        // 列表模板里的字段文本：`{item.title}`。
                        if let Some(scope) = &self.item_scope {
                            if let Some((list, field)) = item_field_path(expr, scope) {
                                if let Some(kind) = self
                                    .lists
                                    .get(&list)
                                    .and_then(|schema| schema.field_kind(&field).cloned())
                                {
                                    children.push(IRNode::ItemText {
                                        list,
                                        field,
                                        initial: kind,
                                        style: InlineStyle::default(),
                                    });
                                    continue;
                                }
                            }
                        }
                        if let Some(node) = self.item_conditional(expr) {
                            children.push(node);
                            continue;
                        }
                        if let Some(node) = self.expression_text(expr) {
                            children.push(node);
                            continue;
                        }
                        if let Some(prop) = component_prop_for_expr(expr, &self.props) {
                            match prop {
                                ComponentProp::Literal(value) => children.push(IRNode::Text {
                                    content: value.display_value(),
                                    style: InlineStyle::default(),
                                }),
                                ComponentProp::StateReference(cell) => {
                                    if let Some(initial) = self.cell_initial(&cell).cloned() {
                                        children.push(IRNode::StateText {
                                            cell,
                                            initial,
                                            style: InlineStyle::default(),
                                        });
                                    }
                                }
                                ComponentProp::RowField {
                                    list,
                                    field: Some(field),
                                } => {
                                    if let Some(kind) = self
                                        .lists
                                        .get(&list)
                                        .and_then(|schema| schema.field_kind(&field).cloned())
                                    {
                                        children.push(IRNode::ItemText {
                                            list,
                                            field,
                                            initial: kind,
                                            style: InlineStyle::default(),
                                        });
                                    }
                                }
                                ComponentProp::RowField { field: None, .. }
                                | ComponentProp::RowIndex { .. }
                                | ComponentProp::RowChildren { .. } => {}
                            }
                            continue;
                        }
                    }
                    if let JSXExpr::Expr(expr) = &container.expr {
                        if let Some(cell) = expr_state_path(expr.as_ref()) {
                            let cell = self.resolve_name(&cell);
                            if let Some(initial) = self.cell_initial(&cell).cloned() {
                                children.push(IRNode::StateText {
                                    cell,
                                    initial,
                                    style: InlineStyle::default(),
                                });
                                continue;
                            }
                        }
                        if matches!(
                            expr.as_ref(),
                            Expr::Bin(binary) if binary.op == BinaryOp::LogicalAnd
                        ) || matches!(expr.as_ref(), Expr::Cond(_))
                        {
                            if let Some(conditional) = self.conditional_from_expr(expr) {
                                children.push(conditional);
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        match tag_name {
            "View" => self.ir_out.push(IRNode::View { style, children }),
            "VStack" => self.ir_out.push(IRNode::VStack { style, children }),
            "ScrollView" => {
                let mut style = style;
                // `onScroll={(y) => offset = y}`：把滚动位置接到状态单元上，
                // 滚动驱动的效果（渐显、视差、吸顶）就能写成普通 TSX。
                if let Some(handler) = jsx_attr(node, "onScroll").and_then(|attr| attr.value.as_ref()) {
                    match parse_scroll_mutation(handler, &self.cells) {
                        Some(mutation) => {
                            let mutation = self.resolve_mutation(mutation);
                            self.state_actions.push(mutation);
                            style.scroll_action = Some(self.state_actions.len() - 1);
                        }
                        None => self
                            .errors
                            .push("onScroll 只支持 `(y) => 数值单元 = y` 这一种写法".into()),
                    }
                }
                self.ir_out.push(IRNode::ScrollView { style, children });
            }
            "Text" => {
                if let Some((left, right, subtract, deps, initial)) =
                    children.iter().find_map(|child| match child {
                        IRNode::ExprText {
                            left,
                            right,
                            subtract,
                            deps,
                            initial,
                            ..
                        } => Some((
                            left.clone(),
                            right.clone(),
                            *subtract,
                            deps.clone(),
                            initial.clone(),
                        )),
                        _ => None,
                    })
                {
                    self.ir_out.push(IRNode::ExprText {
                        left,
                        right,
                        subtract,
                        deps,
                        initial,
                        style,
                    });
                    return;
                }
                if let Some((list, field, initial)) = children.iter().find_map(|child| match child {
                    IRNode::ItemText {
                        list, field, initial, ..
                    } => Some((list.clone(), field.clone(), initial.clone())),
                    _ => None,
                }) {
                    self.ir_out.push(IRNode::ItemText {
                        list,
                        field,
                        initial,
                        style,
                    });
                    return;
                }
                if let Some((cell, initial)) = children.iter().find_map(|child| match child {
                    IRNode::StateText { cell, initial, .. } => Some((cell.clone(), initial.clone())),
                    _ => None,
                }) {
                    self.ir_out.push(IRNode::StateText { cell, initial, style });
                    return;
                }
                let txt = children
                    .into_iter()
                    .filter_map(|c| match c {
                        IRNode::Text { content, .. } => Some(content),
                        _ => None,
                    })
                    .collect::<Vec<_>>()
                    .join("");
                self.ir_out.push(IRNode::Text { content: txt, style });
            }
            "Button" => {
                let title = if title.is_empty() {
                    children
                        .iter()
                        .filter_map(|child| match child {
                            IRNode::Text { content, .. } => Some(content.as_str()),
                            _ => None,
                        })
                        .collect::<String>()
                } else {
                    title
                };
                // 标题样式常写在子 Text 上（`<Button><Text style={{ color, fontSize }}>`）：
                // 合并进按钮样式，否则各端会回落到各自的默认标题色，同一份代码看起来不一样。
                let mut style = style;
                if let Some(child) = children.iter().find_map(|child| match child {
                    IRNode::Text { style, .. } => Some(style),
                    _ => None,
                }) {
                    if style.color.is_none() {
                        style.color = child.color.clone();
                    }
                    if style.font_size.is_none() {
                        style.font_size = child.font_size;
                    }
                }
                self.ir_out.push(IRNode::Button {
                    title,
                    style,
                    state_action: action.map(|action| {
                        self.state_actions.push(action);
                        self.state_actions.len() - 1
                    }),
                    unsupported_handler,
                });
            }
            "Image" => self.ir_out.push(IRNode::Image {
                source,
                mode: ImageMode::from_attribute(
                    jsx_string_attr_with_props(node, "mode", &self.props).as_deref(),
                ),
                style,
            }),
            "Input" => {
                let input_action = on_input.map(|mutation| self.add_action(mutation));
                let focus_action = on_focus.map(|mutation| self.add_action(mutation));
                self.ir_out.push(IRNode::Input {
                    placeholder,
                    style,
                    on_input: input_action,
                    on_focus: focus_action,
                    unsupported_event_handler: unsupported_input || unsupported_focus,
                })
            }
            _ => self.errors.push(format!("unsupported JSX element: {tag_name}")),
        }
    }
}

fn parse_tsx_to_ast(src: &str) -> UiProgram {
    let cm = Arc::new(SourceMap::default());
    let fm = cm.new_source_file(FileName::Anon.into(), src.to_string());

    let ts_cfg = TsSyntax {
        tsx: true,
        decorators: true,
        ..Default::default()
    };
    let syntax = Syntax::Typescript(ts_cfg);

    let lexer = Lexer::new(syntax, Default::default(), (&*fm).into(), None);
    let mut parser = Parser::new_from(lexer);
    let module = parser.parse_module().expect("parse tsx failed");

    let states = collect_state_decls(&module);
    let state_count = states.len();
    let mut counter = StateCallCounter { calls: 0 };
    module.visit_with(&mut counter);
    let cells = state_cells(&states);
    let components = collect_component_definitions(&module, &cells);
    let lists: HashMap<String, ListInit> = states
        .iter()
        .filter_map(|decl| match &decl.initial {
            StateInit::List(list) => Some((decl.name.clone(), list.clone())),
            _ => None,
        })
        .collect();
    let mut visitor = JsxToIrVisitor {
        ir_out: vec![],
        cells,
        state_actions: Vec::new(),
        components,
        props: HashMap::new(),
        component_stack: Vec::new(),
        lists,
        item_scope: None,
        instance_states: Vec::new(),
        scopes: Vec::new(),
        instance_counters: HashMap::new(),
        errors: Vec::new(),
    };
    if let Some(root) = find_app_root(&module) {
        visitor.visit_jsx_element(root);
    } else {
        visitor.errors.push("missing App function returning JSX".into());
    }
    let mut errors = visitor.errors;
    // 组件体内的 state 会按实例分配，所以计数要把实例单元也算进来
    if counter.calls > state_count + visitor.instance_states.len() {
        errors.push(
            "state(...) 只支持标量、一层对象字段或扁平对象数组；部分初值没被接受（嵌套对象 / 数组字段 / 空数组）"
                .into(),
        );
    }
    let mut states = states;
    states.extend(visitor.instance_states.iter().cloned());
    UiProgram {
        nodes: visitor.ir_out,
        states,
        state_actions: visitor.state_actions,
        errors,
    }
}

fn jsx_attr<'a>(node: &'a JSXElement, name: &str) -> Option<&'a JSXAttr> {
    node.opening.attrs.iter().find_map(|attr| match attr {
        JSXAttrOrSpread::JSXAttr(attr) => match &attr.name {
            JSXAttrName::Ident(ident) if ident.sym.as_str() == name => Some(attr),
            _ => None,
        },
        JSXAttrOrSpread::SpreadElement(_) => None,
    })
}

fn find_app_root<'a>(module: &'a swc_ecma_ast::Module) -> Option<&'a JSXElement> {
    module.body.iter().find_map(|item| {
        let ModuleItem::Stmt(Stmt::Decl(Decl::Fn(function))) = item else { return None };
        if function.ident.sym.as_str() != "App" {
            return None;
        }
        let body = function.function.body.as_ref()?;
        body.stmts.iter().find_map(|statement| {
            let Stmt::Return(return_statement) = statement else { return None };
            let expression = return_statement.arg.as_deref()?;
            jsx_return_element(expression)
        })
    })
}

fn collect_component_definitions<'a>(
    module: &'a swc_ecma_ast::Module,
    cells: &[(String, StateValue)],
) -> HashMap<String, ComponentDefinition<'a>> {
    module.body.iter().filter_map(|item| {
        let ModuleItem::Stmt(Stmt::Decl(Decl::Fn(function))) = item else { return None };
        let name = function.ident.sym.to_string();
        if name == "App" {
            return None;
        }
        let body = function.function.body.as_ref()?;
        let root = body.stmts.iter().find_map(|statement| {
            let Stmt::Return(return_statement) = statement else { return None };
            let expression = return_statement.arg.as_deref()?;
            jsx_return_element(expression)
        })?;
        let (props_parameter, named_props) = function.function.params.first().map_or_else(
            || (None, Vec::new()),
            |parameter| match &parameter.pat {
                Pat::Ident(identifier) => (Some(identifier.id.sym.to_string()), Vec::new()),
                Pat::Object(object) => {
                    let properties = object.props.iter().filter_map(|property| match property {
                        ObjectPatProp::KeyValue(property) => match &property.key {
                            PropName::Ident(name) => Some(name.sym.to_string()),
                            PropName::Str(name) => Some(name.value.to_string_lossy().into_owned()),
                            _ => None,
                        },
                        ObjectPatProp::Assign(property) => Some(property.key.id.sym.to_string()),
                        ObjectPatProp::Rest(_) => None,
                    }).collect();
                    (None, properties)
                }
                _ => (None, Vec::new()),
            },
        );
        let mut on_mount = None;
        let mut on_unmount = None;
        let mut unsupported_lifecycle = false;
        for statement in &body.stmts {
            let Stmt::Expr(statement) = statement else { continue };
            let Expr::Call(call) = statement.expr.as_ref() else { continue };
            let Callee::Expr(callee) = &call.callee else { continue };
            let Expr::Ident(name) = callee.as_ref() else { continue };
            let target = match name.sym.as_str() {
                "onMount" => &mut on_mount,
                "onUnmount" => &mut on_unmount,
                _ => continue,
            };
            if call.args.len() != 1 {
                continue;
            }
            let Expr::Arrow(arrow) = call.args[0].expr.as_ref() else { continue };
            *target = parse_lifecycle_mutation(arrow, cells);
            unsupported_lifecycle |= target.is_none();
        }
        Some((name, ComponentDefinition {
            root,
            props_parameter,
            named_props,
            on_mount,
            on_unmount,
            unsupported_lifecycle,
            state_decls: collect_component_state_decls(&function.function),
        }))
    }).collect()
}

fn jsx_return_element(expression: &Expr) -> Option<&JSXElement> {
    match expression {
        Expr::JSXElement(element) => Some(element.as_ref()),
        Expr::Paren(paren) => jsx_return_element(&paren.expr),
        Expr::Seq(sequence) => sequence.exprs.last().and_then(|expr| jsx_return_element(expr)),
        _ => None,
    }
}

fn map_body_element(arrow: &ArrowExpr) -> Option<&JSXElement> {
    match arrow.body.as_ref() {
        ArrowFunctionBody::Expr(expression) => jsx_return_element(expression),
        ArrowFunctionBody::FunctionBody(body) => {
            let [Stmt::Return(statement)] = body.stmts.as_slice() else {
                return None;
            };
            jsx_return_element(statement.arg.as_deref()?)
        }
    }
}

fn list_item_scope(list: &str, arrow: &ArrowExpr) -> Option<ItemScope> {
    let (param, index_param) = match arrow.params.as_slice() {
        [Pat::Ident(item)] => (item.id.sym.to_string(), None),
        [Pat::Ident(item), Pat::Ident(index)] => {
            (item.id.sym.to_string(), Some(index.id.sym.to_string()))
        }
        _ => return None,
    };
    Some(ItemScope {
        list: list.to_string(),
        param,
        index_param,
    })
}

/// `item.title` → (列表名, 字段名)，只有行参数上的成员访问算行字段。
fn item_field_path(expression: &Expr, scope: &ItemScope) -> Option<(String, String)> {
    let Expr::Member(member) = expression else {
        return None;
    };
    let Expr::Ident(object) = member.obj.as_ref() else {
        return None;
    };
    if object.sym.as_str() != scope.param {
        return None;
    }
    let MemberProp::Ident(prop) = &member.prop else {
        return None;
    };
    Some((scope.list.clone(), prop.sym.to_string()))
}

fn component_prop_for_expr(
    expression: &Expr,
    props: &HashMap<String, ComponentProp>,
) -> Option<ComponentProp> {
    match expression {
        Expr::Ident(identifier) => props.get(identifier.sym.as_str()).cloned(),
        Expr::Member(member) => {
            let Expr::Ident(object) = member.obj.as_ref() else {
                return None;
            };
            let MemberProp::Ident(property) = &member.prop else {
                return None;
            };
            if let Some(prop) = props.get(&format!("{}.{}", object.sym, property.sym)) {
                return Some(prop.clone());
            }
            // `item.title`：item 绑定的是整个行元素时，成员访问取行字段
            match props.get(object.sym.as_str()) {
                Some(ComponentProp::RowField { list, field: None }) => {
                    // `.children` 不是标量字段，而是「这一行的子树行集合」，
                    // 递归组件靠它把同一种模板下发到下一层。
                    if property.sym.as_str() == "children" {
                        return Some(ComponentProp::RowChildren { list: list.clone() });
                    }
                    Some(ComponentProp::RowField {
                        list: list.clone(),
                        field: Some(property.sym.to_string()),
                    })
                }
                _ => None,
            }
        }
        _ => None,
    }
}

fn component_prop_value(
    attribute: &JSXAttr,
    cells: &[(String, StateValue)],
    props: &HashMap<String, ComponentProp>,
    item: Option<&ItemScope>,
) -> Option<ComponentProp> {
    let Some(value) = &attribute.value else {
        return Some(ComponentProp::Literal(StateValue::Boolean(true)));
    };
    match value {
        JSXAttrValue::Str(value) => Some(ComponentProp::Literal(StateValue::String(
            value.value.to_string_lossy().into_owned(),
        ))),
        JSXAttrValue::JSXExprContainer(container) => {
            let JSXExpr::Expr(expression) = &container.expr else { return None };
            if let Some(prop) = component_prop_for_expr(expression, props) {
                return Some(prop);
            }
            // 行数据传递：<Row item={item} /> 或 <Row title={item.title} />
            if let Some(scope) = item {
                if let Some((list, field)) = item_field_path(expression, scope) {
                    if field == "children" {
                        return Some(ComponentProp::RowChildren { list });
                    }
                    return Some(ComponentProp::RowField {
                        list,
                        field: Some(field),
                    });
                }
                if let Expr::Ident(ident) = expression.as_ref() {
                    if ident.sym.as_str() == scope.param {
                        return Some(ComponentProp::RowField {
                            list: scope.list.clone(),
                            field: None,
                        });
                    }
                    if scope.index_param.as_deref() == Some(ident.sym.as_str()) {
                        return Some(ComponentProp::RowIndex {
                            list: scope.list.clone(),
                        });
                    }
                }
            }
            if let Some(value) = state_value_from_expr(expression) {
                return Some(ComponentProp::Literal(value));
            }
            let cell = expr_state_path(expression.as_ref())?;
            cells
                .iter()
                .any(|(path, _)| *path == cell)
                .then_some(ComponentProp::StateReference(cell))
        }
        _ => None,
    }
}

fn jsx_string_attr_with_props(
    node: &JSXElement,
    name: &str,
    props: &HashMap<String, ComponentProp>,
) -> Option<String> {
    let attribute = jsx_attr(node, name)?;
    match component_prop_value(attribute, &[], props, None)? {
        ComponentProp::Literal(value) => Some(value.display_value()),
        ComponentProp::StateReference(_)
        | ComponentProp::RowField { .. }
        | ComponentProp::RowIndex { .. }
        | ComponentProp::RowChildren { .. } => None,
    }
}

fn jsx_string_attr(node: &JSXElement, name: &str) -> Option<String> {
    match jsx_attr(node, name)?.value.as_ref()? {
        JSXAttrValue::Str(value) => Some(value.value.to_string_lossy().into_owned()),
        JSXAttrValue::JSXExprContainer(container) => match &container.expr {
            JSXExpr::Expr(expr) => match expr.as_ref() {
                Expr::Lit(Lit::Str(value)) => Some(value.value.to_string_lossy().into_owned()),
                _ => None,
            },
            JSXExpr::JSXEmptyExpr(_) => None,
        },
        _ => None,
    }
}

fn jsx_style(node: &JSXElement, props: &HashMap<String, ComponentProp>) -> InlineStyle {
    let mut style = InlineStyle::default();
    let Some(JSXAttrValue::JSXExprContainer(container)) =
        jsx_attr(node, "style").and_then(|attr| attr.value.as_ref())
    else {
        return style;
    };
    let JSXExpr::Expr(expr) = &container.expr else {
        return style;
    };
    let Expr::Object(object) = expr.as_ref() else {
        return style;
    };

    for prop in &object.props {
        let PropOrSpread::Prop(prop) = prop else {
            continue;
        };
        let property = match prop.as_ref() {
            Prop::KeyValue(property) => property,
            Prop::Shorthand(identifier) => {
                assign_style_shorthand(&mut style, identifier.sym.as_str(), props);
                continue;
            }
            _ => continue,
        };
        let key = match &property.key {
            PropName::Ident(ident) => ident.sym.as_str(),
            PropName::Str(string) => string.value.as_str().unwrap_or(""),
            _ => continue,
        };
        match key {
            "width" => style.width = jsx_dimension_expr_with_props(&property.value, props),
            "height" => style.height = jsx_dimension_expr_with_props(&property.value, props),
            "padding" => style.padding = jsx_number_expr_with_props(&property.value, props),
            "paddingHorizontal" => {
                style.padding_horizontal = jsx_number_expr_with_props(&property.value, props)
            }
            "paddingVertical" => {
                style.padding_vertical = jsx_number_expr_with_props(&property.value, props)
            }
            "paddingTop" => style.padding_top = jsx_number_expr_with_props(&property.value, props),
            "paddingRight" => style.padding_right = jsx_number_expr_with_props(&property.value, props),
            "paddingBottom" => {
                style.padding_bottom = jsx_number_expr_with_props(&property.value, props)
            }
            "paddingLeft" => style.padding_left = jsx_number_expr_with_props(&property.value, props),
            "margin" => style.margin = jsx_number_expr_with_props(&property.value, props),
            "marginHorizontal" => style.margin_horizontal = jsx_number_expr_with_props(&property.value, props),
            "marginVertical" => style.margin_vertical = jsx_number_expr_with_props(&property.value, props),
            "marginTop" => style.margin_top = jsx_number_expr_with_props(&property.value, props),
            "marginRight" => style.margin_right = jsx_number_expr_with_props(&property.value, props),
            "marginBottom" => style.margin_bottom = jsx_number_expr_with_props(&property.value, props),
            "marginLeft" => style.margin_left = jsx_number_expr_with_props(&property.value, props),
            "position" => {
                style.position = match jsx_string_expr_with_props(&property.value, props).as_deref() {
                    Some("absolute") => Position::Absolute,
                    Some("relative") | None => Position::Relative,
                    _ => Position::Relative,
                }
            }
            "top" => style.top = jsx_dimension_expr_with_props(&property.value, props),
            "right" => style.right = jsx_dimension_expr_with_props(&property.value, props),
            "bottom" => style.bottom = jsx_dimension_expr_with_props(&property.value, props),
            "left" => style.left = jsx_dimension_expr_with_props(&property.value, props),
            "spacing" | "gap" => style.spacing = jsx_number_expr_with_props(&property.value, props),
            "flexDirection" => {
                style.flex_direction = jsx_string_expr_with_props(&property.value, props)
                    .and_then(|value| match value.as_str() {
                        "row" => Some(FlexDirection::Row),
                        "column" => Some(FlexDirection::Column),
                        _ => None,
                    })
            }
            "justifyContent" => {
                style.justify_content = jsx_string_expr_with_props(&property.value, props)
                    .and_then(|value| match value.as_str() {
                        "flex-start" => Some(JustifyContent::Start),
                        "center" => Some(JustifyContent::Center),
                        "flex-end" => Some(JustifyContent::End),
                        "space-between" => Some(JustifyContent::SpaceBetween),
                        "space-around" => Some(JustifyContent::SpaceAround),
                        _ => None,
                    })
            }
            "alignItems" => {
                style.align_items = jsx_string_expr_with_props(&property.value, props)
                    .and_then(|value| match value.as_str() {
                        "flex-start" => Some(AlignItems::Start),
                        "center" => Some(AlignItems::Center),
                        "flex-end" => Some(AlignItems::End),
                        "stretch" => Some(AlignItems::Stretch),
                        _ => None,
                    })
            }
            "flexGrow" => style.flex_grow = jsx_number_expr_with_props(&property.value, props).unwrap_or(0.0),
            "fontSize" => style.font_size = jsx_number_expr_with_props(&property.value, props),
            "placeholderColor" => {
                style.placeholder_color = jsx_string_expr_with_props(&property.value, props)
            }
            "textAlign" => {
                style.text_align = jsx_string_expr_with_props(&property.value, props).and_then(|value| match value.as_str() {
                    "left" => Some(TextAlignment::Left),
                    "center" => Some(TextAlignment::Center),
                    "right" => Some(TextAlignment::Right),
                    _ => None,
                })
            }
            // `opacity: 单元`（0..100 整数）绑到状态；`opacity: 50` 就是常量（50%）
            "opacity" => {
                if let Some(cell) = expr_state_path(&property.value) {
                    // 初始值由各端从状态单元里查（0..100 → 0..1）
                    style.opacity_static = Some(1.0);
                    style.opacity_cell = Some(cell);
                } else {
                    style.opacity_static = jsx_number_expr_with_props(&property.value, props)
                        .map(|value| (value / 100.0).clamp(0.0, 1.0));
                }
            }
            "backgroundColor" => style.background_color = jsx_string_expr_with_props(&property.value, props),
            "color" => style.color = jsx_string_expr_with_props(&property.value, props),
            "borderWidth" => style.border_width = jsx_nonnegative_number_expr_with_props(&property.value, props),
            "borderColor" => style.border_color = jsx_string_expr_with_props(&property.value, props),
            "borderRadius" => style.border_radius = jsx_nonnegative_number_expr_with_props(&property.value, props),
            "id" => style.view_id = jsx_string_expr_with_props(&property.value, props),
            "backgroundExtendsTo" => {
                style.background_extends_to_top =
                    jsx_string_expr_with_props(&property.value, props).as_deref() == Some("top")
            }
            "ignoresSafeArea" => {
                style.ignores_safe_area_top =
                    jsx_string_expr_with_props(&property.value, props).as_deref() == Some("top")
            }
            // 这三个键已移除：保留解析只为了给出迁移报错
            "fadeTarget" => style.scroll_fade_target = jsx_string_expr_with_props(&property.value, props),
            "fadeStart" => style.scroll_fade_start = jsx_nonnegative_number_expr_with_props(&property.value, props),
            "fadeDistance" => {
                style.scroll_fade_distance = jsx_nonnegative_number_expr_with_props(&property.value, props)
            }
            _ => {}
        }
    }
    style
}

fn assign_style_shorthand(
    style: &mut InlineStyle,
    key: &str,
    props: &HashMap<String, ComponentProp>,
) {
    let Some(ComponentProp::Literal(value)) = props.get(key) else {
        return;
    };
    let number = match value {
        StateValue::Number(value) => Some(*value as f64),
        _ => None,
    };
    let string = match value {
        StateValue::String(value) => Some(value.as_str()),
        _ => None,
    };
    match key {
        "width" | "height" => {
            let dimension = number.map(Dimension::Points).or_else(|| {
                let percentage = string?.trim().strip_suffix('%')?.trim().parse::<f64>().ok()?;
                (percentage.is_finite() && percentage >= 0.0)
                    .then_some(Dimension::Percent(percentage / 100.0))
            });
            if key == "width" {
                style.width = dimension;
            } else {
                style.height = dimension;
            }
        }
        "padding" => style.padding = number,
        "paddingHorizontal" => style.padding_horizontal = number,
        "paddingVertical" => style.padding_vertical = number,
        "paddingTop" => style.padding_top = number,
        "paddingRight" => style.padding_right = number,
        "paddingBottom" => style.padding_bottom = number,
        "paddingLeft" => style.padding_left = number,
        "margin" => style.margin = number,
        "marginHorizontal" => style.margin_horizontal = number,
        "marginVertical" => style.margin_vertical = number,
        "marginTop" => style.margin_top = number,
        "marginRight" => style.margin_right = number,
        "marginBottom" => style.margin_bottom = number,
        "marginLeft" => style.margin_left = number,
        "spacing" | "gap" => style.spacing = number,
        "flexGrow" => style.flex_grow = number.unwrap_or(0.0),
        "fontSize" => style.font_size = number,
        "borderWidth" => style.border_width = number.filter(|value| *value >= 0.0),
        "borderRadius" => style.border_radius = number.filter(|value| *value >= 0.0),
        "fadeStart" => style.scroll_fade_start = number.filter(|value| *value >= 0.0),
        "fadeDistance" => style.scroll_fade_distance = number.filter(|value| *value >= 0.0),
        "position" => {
            style.position = if string == Some("absolute") {
                Position::Absolute
            } else {
                Position::Relative
            };
        }
        "flexDirection" => {
            style.flex_direction = match string {
                Some("row") => Some(FlexDirection::Row),
                Some("column") => Some(FlexDirection::Column),
                _ => None,
            };
        }
        "justifyContent" => {
            style.justify_content = match string {
                Some("flex-start") => Some(JustifyContent::Start),
                Some("center") => Some(JustifyContent::Center),
                Some("flex-end") => Some(JustifyContent::End),
                Some("space-between") => Some(JustifyContent::SpaceBetween),
                Some("space-around") => Some(JustifyContent::SpaceAround),
                _ => None,
            };
        }
        "alignItems" => {
            style.align_items = match string {
                Some("flex-start") => Some(AlignItems::Start),
                Some("center") => Some(AlignItems::Center),
                Some("flex-end") => Some(AlignItems::End),
                Some("stretch") => Some(AlignItems::Stretch),
                _ => None,
            };
        }
        "textAlign" => {
            style.text_align = match string {
                Some("left") => Some(TextAlignment::Left),
                Some("center") => Some(TextAlignment::Center),
                Some("right") => Some(TextAlignment::Right),
                _ => None,
            };
        }
        "backgroundColor" => style.background_color = string.map(str::to_string),
        "color" => style.color = string.map(str::to_string),
        "borderColor" => style.border_color = string.map(str::to_string),
        "id" => style.view_id = string.map(str::to_string),
        "backgroundExtendsTo" => style.background_extends_to_top = string == Some("top"),
        "ignoresSafeArea" => style.ignores_safe_area_top = string == Some("top"),
        "fadeTarget" => style.scroll_fade_target = string.map(str::to_string),
        _ => {}
    }
}

fn jsx_dimension_expr(expr: &Expr) -> Option<Dimension> {
    match expr {
        Expr::Lit(Lit::Num(number)) => Some(Dimension::Points(number.value)),
        Expr::Lit(Lit::Str(string)) => {
            let value = string.value.to_string_lossy();
            let percentage = value.trim().strip_suffix('%')?.trim().parse::<f64>().ok()?;
            (percentage.is_finite() && percentage >= 0.0)
                .then_some(Dimension::Percent(percentage / 100.0))
        }
        _ => None,
    }
}

fn component_prop_state_value(
    expr: &Expr,
    props: &HashMap<String, ComponentProp>,
) -> Option<StateValue> {
    match component_prop_for_expr(expr, props)? {
        ComponentProp::Literal(value) => Some(value),
        ComponentProp::StateReference(_)
        | ComponentProp::RowField { .. }
        | ComponentProp::RowIndex { .. }
        | ComponentProp::RowChildren { .. } => None,
    }
}

fn jsx_dimension_expr_with_props(
    expr: &Expr,
    props: &HashMap<String, ComponentProp>,
) -> Option<Dimension> {
    jsx_dimension_expr(expr).or_else(|| match component_prop_state_value(expr, props)? {
        StateValue::Number(value) => Some(Dimension::Points(value as f64)),
        StateValue::String(value) => {
            let percentage = value.trim().strip_suffix('%')?.trim().parse::<f64>().ok()?;
            (percentage.is_finite() && percentage >= 0.0)
                .then_some(Dimension::Percent(percentage / 100.0))
        }
        _ => None,
    })
}

fn jsx_number_expr(expr: &Expr) -> Option<f64> {
    match expr {
        Expr::Lit(Lit::Num(number)) if number.value.is_finite() => Some(number.value),
        _ => None,
    }
}

fn jsx_number_expr_with_props(
    expr: &Expr,
    props: &HashMap<String, ComponentProp>,
) -> Option<f64> {
    jsx_number_expr(expr).or_else(|| match component_prop_state_value(expr, props)? {
        StateValue::Number(value) => Some(value as f64),
        _ => None,
    })
}

fn jsx_nonnegative_number_expr(expr: &Expr) -> Option<f64> {
    jsx_number_expr(expr).filter(|value| *value >= 0.0)
}

fn jsx_nonnegative_number_expr_with_props(
    expr: &Expr,
    props: &HashMap<String, ComponentProp>,
) -> Option<f64> {
    jsx_number_expr_with_props(expr, props).filter(|value| *value >= 0.0)
}

fn jsx_string_expr(expr: &Expr) -> Option<String> {
    match expr {
        Expr::Lit(Lit::Str(string)) => Some(string.value.to_string_lossy().into_owned()),
        _ => None,
    }
}

fn jsx_string_expr_with_props(
    expr: &Expr,
    props: &HashMap<String, ComponentProp>,
) -> Option<String> {
    jsx_string_expr(expr).or_else(|| {
        component_prop_state_value(expr, props).map(|value| value.display_value())
    })
}

fn node_style_mut(node: &mut IRNode) -> &mut InlineStyle {
    match node {
        IRNode::View { style, .. }
        | IRNode::VStack { style, .. }
        | IRNode::ScrollView { style, .. }
        | IRNode::Conditional { style, .. }
        | IRNode::Text { style, .. }
        | IRNode::StateText { style, .. }
        | IRNode::Image { style, .. }
        | IRNode::Input { style, .. }
        | IRNode::Button { style, .. }
        | IRNode::List { style, .. }
        | IRNode::ItemText { style, .. }
        | IRNode::ItemConditional { style, .. }
        | IRNode::RecursiveComponent { style, .. }
        | IRNode::ExprText { style, .. } => style,
    }
}

fn take_lifecycle_actions(node: &mut IRNode) -> (Vec<usize>, Vec<usize>) {
    let style = node_style_mut(node);
    let mut mount = std::mem::take(&mut style.mount_actions);
    let mut unmount = std::mem::take(&mut style.unmount_actions);
    match node {
        IRNode::View { children, .. }
        | IRNode::VStack { children, .. }
        | IRNode::ScrollView { children, .. } => {
            for child in children {
                let (child_mount, child_unmount) = take_lifecycle_actions(child);
                mount.extend(child_mount);
                unmount.extend(child_unmount);
            }
        }
        IRNode::Conditional { .. } => {}
        _ => {}
    }
    (mount, unmount)
}

fn parse_functional_color(value: &str) -> Result<Option<(f64, f64, f64, f64)>, String> {
    let value = value.trim();
    let (function, channel_count) = if value
        .get(..4)
        .is_some_and(|prefix| prefix.eq_ignore_ascii_case("rgb("))
    {
        ("rgb", 3)
    } else if value
        .get(..5)
        .is_some_and(|prefix| prefix.eq_ignore_ascii_case("rgba("))
    {
        ("rgba", 4)
    } else {
        return Ok(None);
    };
    let arguments = &value[function.len() + 1..];
    let Some(arguments) = arguments.strip_suffix(')') else {
        return Err(format!("invalid {function} color: {value}"));
    };
    let components: Vec<_> = arguments.split(',').map(str::trim).collect();
    if components.len() != channel_count {
        return Err(format!(
            "{function} color requires {channel_count} comma-separated components: {value}"
        ));
    }

    let parse_channel = |component: &str| -> Result<f64, String> {
        let is_percentage = component.ends_with('%');
        let number = component
            .trim_end_matches('%')
            .parse::<f64>()
            .map_err(|_| format!("invalid {function} color component: {component}"))?;
        let channel = if is_percentage { number / 100.0 } else { number / 255.0 };
        if !number.is_finite() || !(0.0..=1.0).contains(&channel) {
            return Err(format!("{function} color component is out of range: {component}"));
        }
        Ok(channel)
    };
    let parse_alpha = |component: &str| -> Result<f64, String> {
        let is_percentage = component.ends_with('%');
        let alpha = component
            .trim_end_matches('%')
            .parse::<f64>()
            .map_err(|_| format!("invalid rgba alpha component: {component}"))?;
        let alpha = if is_percentage { alpha / 100.0 } else { alpha };
        if !alpha.is_finite() || !(0.0..=1.0).contains(&alpha) {
            return Err(format!("rgba alpha is out of range: {component}"));
        }
        Ok(alpha)
    };

    let alpha = if channel_count == 4 {
        parse_alpha(components[3])?
    } else {
        1.0
    };
    Ok(Some((
        parse_channel(components[0])?,
        parse_channel(components[1])?,
        parse_channel(components[2])?,
        alpha,
    )))
}

fn arrow_from_handler(value: &JSXAttrValue) -> Option<&ArrowExpr> {
    let JSXAttrValue::JSXExprContainer(container) = value else {
        return None;
    };
    let JSXExpr::Expr(expr) = &container.expr else {
        return None;
    };
    let Expr::Arrow(arrow) = expr.as_ref() else {
        return None;
    };
    Some(arrow)
}

fn is_empty_event_handler(value: &JSXAttrValue) -> bool {
    arrow_from_handler(value).is_some_and(|arrow| {
        matches!(&*arrow.body, ArrowFunctionBody::FunctionBody(body) if body.stmts.is_empty())
    })
}

/// 把表达式解析成状态单元路径：`count` → "count"，`user.name` → "user.name"。
fn expr_state_path(expr: &Expr) -> Option<String> {
    match expr {
        Expr::Ident(ident) => Some(ident.sym.to_string()),
        Expr::Member(member) => {
            let base = expr_state_path(member.obj.as_ref())?;
            let MemberProp::Ident(prop) = &member.prop else {
                return None;
            };
            Some(format!("{base}.{}", prop.sym))
        }
        _ => None,
    }
}

/// 赋值左值对应的状态单元路径。
fn assign_target_path(target: &AssignTarget) -> Option<String> {
    match target {
        AssignTarget::Simple(SimpleAssignTarget::Ident(ident)) => Some(ident.id.sym.to_string()),
        AssignTarget::Simple(SimpleAssignTarget::Member(member)) => {
            let base = expr_state_path(member.obj.as_ref())?;
            let MemberProp::Ident(prop) = &member.prop else {
                return None;
            };
            Some(format!("{base}.{}", prop.sym))
        }
        _ => None,
    }
}

/// 收集所有 `const|let x = state(<初值>)` 声明；对象初值会展开成多个状态单元。
/// 声明既可以写在模块顶层，也可以写在函数（例如 App）体里。
/// 统计源码里 `state(...)` 的调用次数。收集到的声明数少于调用数，
/// 说明有初值形态没被支持（嵌套对象、数组字段、空数组），必须报错而不是静默忽略。
struct StateCallCounter {
    calls: usize,
}

impl Visit for StateCallCounter {
    fn visit_call_expr(&mut self, node: &CallExpr) {
        if let Callee::Expr(callee) = &node.callee {
            if let Expr::Ident(ident) = callee.as_ref() {
                if ident.sym.as_str() == "state" {
                    self.calls += 1;
                }
            }
        }
        node.visit_children_with(self);
    }
}

fn collect_state_decls(module: &swc_ecma_ast::Module) -> Vec<StateDecl> {
    let mut decls = Vec::new();
    for item in &module.body {
        let ModuleItem::Stmt(statement) = item else { continue };
        // 组件体内的 state 属于组件实例，展开时再按实例分配单元；
        // 这里只收模块级声明和 App 自己的声明（App 只有一个实例）。
        match statement {
            Stmt::Decl(Decl::Fn(function)) if function.ident.sym.as_str() != "App" => {}
            _ => collect_state_decls_in_stmt(statement, &mut decls),
        }
    }
    decls
}

/// 收集某个组件体里的 state 声明（展开时按实例加前缀）。
fn collect_component_state_decls(function: &Function) -> Vec<StateDecl> {
    let mut decls = Vec::new();
    if let Some(body) = &function.body {
        for statement in &body.stmts {
            collect_state_decls_in_stmt(statement, &mut decls);
        }
    }
    decls
}

fn collect_state_decls_in_stmt(statement: &Stmt, decls: &mut Vec<StateDecl>) {
    match statement {
        Stmt::Decl(Decl::Var(var)) => {
            for declarator in &var.decls {
                let Pat::Ident(binding) = &declarator.name else {
                    continue;
                };
                let Some(init) = &declarator.init else {
                    continue;
                };
                let Some(initial) = state_init_from_expr(init) else {
                    continue;
                };
                decls.push(StateDecl {
                    name: binding.id.sym.to_string(),
                    initial,
                });
            }
        }
        Stmt::Decl(Decl::Fn(function)) => {
            if let Some(body) = &function.function.body {
                for statement in &body.stmts {
                    collect_state_decls_in_stmt(statement, decls);
                }
            }
        }
        _ => {}
    }
}

/// 把对象字面量递归拍平成「点路径 + 标量」的单元列表：
/// `{ meta: { level: 1 } }` → `meta.level = 1`。数组/函数等其它形态一律拒绝。
/// 把列表字面量（可能带 `children` 嵌套）拍平成 DFS 顺序的节点表：
/// `children` 只体现在 parents 上，不进入标量字段。
fn flatten_list_rows(
    elements: &[Option<ExprOrSpread>],
    parent: Option<usize>,
    fields: &mut Option<Vec<String>>,
    rows: &mut Vec<Vec<StateValue>>,
    parents: &mut Vec<Option<usize>>,
) -> Option<()> {
    for element in elements {
        let element = element.as_ref()?;
        let Expr::Object(object) = element.expr.as_ref() else {
            return None;
        };
        let mut names = Vec::new();
        let mut values = Vec::new();
        let mut children: Option<&Expr> = None;
        for prop in &object.props {
            let PropOrSpread::Prop(prop) = prop else {
                return None;
            };
            let Prop::KeyValue(entry) = prop.as_ref() else {
                return None;
            };
            let name = match &entry.key {
                PropName::Ident(ident) => ident.sym.to_string(),
                PropName::Str(value) => value.value.to_string_lossy().into_owned(),
                _ => return None,
            };
            if name == "children" {
                children = Some(entry.value.as_ref());
                continue;
            }
            names.push(name);
            values.push(state_value_from_expr(entry.value.as_ref())?);
        }
        match fields {
            None => *fields = Some(names),
            Some(expected) if *expected == names => {}
            Some(_) => return None,
        }
        let index = rows.len();
        rows.push(values);
        parents.push(parent);
        if let Some(Expr::Array(nested)) = children {
            flatten_list_rows(&nested.elems, Some(index), fields, rows, parents)?;
        }
    }
    Some(())
}

/// 节点在树里的深度（根层 = 0）。
fn depth_of(parents: &[Option<usize>], index: usize) -> usize {
    let mut depth = 0;
    let mut cursor = parents.get(index).copied().flatten();
    while let Some(parent) = cursor {
        depth += 1;
        cursor = parents.get(parent).copied().flatten();
    }
    depth
}

fn flatten_object_fields(
    props: &[PropOrSpread],
    prefix: &str,
    out: &mut Vec<(String, StateValue)>,
) -> Option<()> {
    for prop in props {
        let PropOrSpread::Prop(prop) = prop else {
            return None;
        };
        let Prop::KeyValue(entry) = prop.as_ref() else {
            return None;
        };
        let name = match &entry.key {
            PropName::Ident(ident) => ident.sym.to_string(),
            PropName::Str(value) => value.value.to_string_lossy().into_owned(),
            _ => return None,
        };
        let path = if prefix.is_empty() {
            name
        } else {
            format!("{prefix}.{name}")
        };
        match entry.value.as_ref() {
            Expr::Object(inner) => flatten_object_fields(&inner.props, &path, out)?,
            value => out.push((path, state_value_from_expr(value)?)),
        }
    }
    Some(())
}

fn state_init_from_expr(expr: &Expr) -> Option<StateInit> {
    let Expr::Call(call) = expr else {
        return None;
    };
    let Callee::Expr(callee) = &call.callee else {
        return None;
    };
    let Expr::Ident(ident) = callee.as_ref() else {
        return None;
    };
    if ident.sym.as_str() != "state" || call.args.len() != 1 {
        return None;
    }
    match call.args[0].expr.as_ref() {
        Expr::Object(object) => {
            let mut fields = Vec::new();
            flatten_object_fields(&object.props, "", &mut fields)?;
            (!fields.is_empty()).then_some(StateInit::Object(fields))
        }
        Expr::Array(array) => {
            let mut fields: Option<Vec<String>> = None;
            let mut rows = Vec::new();
            let mut parents = Vec::new();
            flatten_list_rows(&array.elems, None, &mut fields, &mut rows, &mut parents)?;
            // 空数组无法推断行字段，要求至少给一行初始值当字段样本。
            let fields = fields?;
            (!rows.is_empty()).then_some(StateInit::List(ListInit {
                fields,
                rows,
                parents,
            }))
        }
        expr => state_value_from_expr(expr).map(StateInit::Scalar),
    }
}

fn parse_state_mutation(
    value: &JSXAttrValue,
    cells: &[(String, StateValue)],
    item: Option<&ItemScope>,
    lists: &HashMap<String, ListInit>,
) -> Option<StateMutation> {
    let arrow = arrow_from_handler(value)?;
    let parameter = match arrow.params.as_slice() {
        [Pat::Ident(ident)] => Some(ident.id.sym.to_string()),
        _ => None,
    };
    match arrow.body.as_ref() {
        ArrowFunctionBody::Expr(body) => {
            parse_state_mutation_expr(body, cells, parameter.as_deref(), item, lists)
        }
        ArrowFunctionBody::FunctionBody(body) => {
            // 多语句 handler：按源码顺序执行，每条语句都要是支持的状态动作。
            let mutations: Vec<StateMutation> = body
                .stmts
                .iter()
                .map(|statement| match statement {
                    Stmt::Expr(statement) => parse_state_mutation_expr(
                        &statement.expr,
                        cells,
                        parameter.as_deref(),
                        item,
                        lists,
                    ),
                    _ => None,
                })
                .collect::<Option<Vec<_>>>()?;
            match mutations.len() {
                0 => Some(StateMutation::Noop),
                1 => mutations.into_iter().next(),
                _ => Some(StateMutation::Sequence(mutations)),
            }
        }
    }
}

/// `onScroll={(y) => offset = y}`：把滚动位置写进一个数值单元。
/// 只接受「单个参数 = 某个数值单元」这一种形态，其余写法直接报错，避免静默不生效。
fn parse_scroll_mutation(
    value: &JSXAttrValue,
    cells: &[(String, StateValue)],
) -> Option<StateMutation> {
    let arrow = arrow_from_handler(value)?;
    let [Pat::Ident(parameter)] = arrow.params.as_slice() else {
        return None;
    };
    let ArrowFunctionBody::Expr(body) = arrow.body.as_ref() else {
        return None;
    };
    let Expr::Assign(assignment) = body.as_ref() else {
        return None;
    };
    if assignment.op != AssignOp::Assign {
        return None;
    }
    let target = assign_target_path(&assignment.left)?;
    // 右边只有两种写法：`y` 或 `clamp(y, lo, hi)`
    let (low_high, shift) = match assignment.right.as_ref() {
        Expr::Ident(source) => {
            if source.sym.as_str() != parameter.id.sym.as_str() {
                return None;
            }
            (None, 0)
        }
        Expr::Call(call) => {
            let Callee::Expr(callee) = &call.callee else {
                return None;
            };
            let Expr::Ident(name) = callee.as_ref() else {
                return None;
            };
            if name.sym.as_str() != "clamp" || call.args.len() != 3 {
                return None;
            }
            // 第一个参数是 `y`、`y - N` 或 `y + N`
            let shift = match call.args[0].expr.as_ref() {
                Expr::Ident(source) => {
                    if source.sym.as_str() != parameter.id.sym.as_str() {
                        return None;
                    }
                    0
                }
                Expr::Bin(binary) if binary.op == BinaryOp::Sub || binary.op == BinaryOp::Add => {
                    let Expr::Ident(source) = binary.left.as_ref() else {
                        return None;
                    };
                    if source.sym.as_str() != parameter.id.sym.as_str() {
                        return None;
                    }
                    let Expr::Lit(Lit::Num(number)) = binary.right.as_ref() else {
                        return None;
                    };
                    let value = number.value as i64;
                    if binary.op == BinaryOp::Sub {
                        -value
                    } else {
                        value
                    }
                }
                _ => return None,
            };
            let bound = |expr: &Expr| match expr {
                Expr::Lit(Lit::Num(number)) if number.value.fract() == 0.0 => {
                    Some(number.value as i64)
                }
                _ => None,
            };
            let low = bound(call.args[1].expr.as_ref())?;
            let high = bound(call.args[2].expr.as_ref())?;
            (Some((low, high)), shift)
        }
        _ => return None,
    };
    let numeric = cells
        .iter()
        .any(|(path, value)| *path == target && matches!(value, StateValue::Number(_)));
    if !numeric {
        return None;
    }
    Some(match low_high {
        Some((low, high)) => StateMutation::SetFromScrollClamped {
            cell: target,
            low,
            high,
            shift,
        },
        _ => StateMutation::SetFromScroll(target),
    })
}

fn parse_lifecycle_mutation(
    arrow: &ArrowExpr,
    cells: &[(String, StateValue)],
) -> Option<StateMutation> {
    let lists = HashMap::new();
    match arrow.body.as_ref() {
        ArrowFunctionBody::FunctionBody(body) if body.stmts.is_empty() => Some(StateMutation::Noop),
        ArrowFunctionBody::Expr(expression) => {
            parse_state_mutation_expr(expression, cells, None, None, &lists)
        }
        ArrowFunctionBody::FunctionBody(body) => {
            let [Stmt::Expr(statement)] = body.stmts.as_slice() else {
                return None;
            };
            parse_state_mutation_expr(&statement.expr, cells, None, None, &lists)
        }
    }
}

/// 列表动作：`items.push({...})` / `items.removeAt(index)`。
fn parse_list_call(
    call: &CallExpr,
    cells: &[(String, StateValue)],
    item: Option<&ItemScope>,
    lists: &HashMap<String, ListInit>,
) -> Option<StateMutation> {
    let Callee::Expr(callee) = &call.callee else {
        return None;
    };
    let Expr::Member(member) = callee.as_ref() else {
        return None;
    };
    let Expr::Ident(list_name) = member.obj.as_ref() else {
        return None;
    };
    let MemberProp::Ident(method) = &member.prop else {
        return None;
    };
    let schema = lists.get(list_name.sym.as_str())?;
    match method.sym.as_str() {
        "push" => {
            let [argument] = call.args.as_slice() else {
                return None;
            };
            let Expr::Object(object) = argument.expr.as_ref() else {
                return None;
            };
            let mut values = Vec::new();
            for prop in &object.props {
                let PropOrSpread::Prop(prop) = prop else {
                    return None;
                };
                let Prop::KeyValue(entry) = prop.as_ref() else {
                    return None;
                };
                let name = match &entry.key {
                    PropName::Ident(ident) => ident.sym.to_string(),
                    PropName::Str(value) => value.value.to_string_lossy().into_owned(),
                    _ => return None,
                };
                let index = schema.field_index(&name)?;
                let value = match expr_state_path(entry.value.as_ref()) {
                    Some(cell) if cells.iter().any(|(path, _)| *path == cell) => {
                        ItemValue::Cell(cell)
                    }
                    _ => ItemValue::Literal(state_value_from_expr(entry.value.as_ref())?),
                };
                // 字段类型必须和初值样本一致，否则写入时会错位。
                if let (ItemValue::Literal(literal), Some(sample)) =
                    (&value, schema.rows.first().and_then(|row| row.get(index)))
                {
                    if std::mem::discriminant(literal) != std::mem::discriminant(sample) {
                        return None;
                    }
                }
                values.push((name, value));
            }
            Some(StateMutation::PushItem {
                list: list_name.sym.to_string(),
                values,
            })
        }
        "removeAt" => {
            let [argument] = call.args.as_slice() else {
                return None;
            };
            let scope = item?;
            let is_row_index = match (&scope.index_param, argument.expr.as_ref()) {
                (Some(index_param), Expr::Ident(ident)) => ident.sym.as_str() == index_param,
                _ => false,
            };
            is_row_index.then(|| StateMutation::RemoveCurrentItem {
                list: list_name.sym.to_string(),
            })
        }
        _ => None,
    }
}

fn parse_state_mutation_expr(
    body: &Expr,
    cells: &[(String, StateValue)],
    input_parameter: Option<&str>,
    item: Option<&ItemScope>,
    lists: &HashMap<String, ListInit>,
) -> Option<StateMutation> {
    let cell_initial = |cell: &str| {
        cells
            .iter()
            .find(|(path, _)| path == cell)
            .map(|(_, value)| value.clone())
    };
    if let Expr::Call(call) = body {
        if let Some(mutation) = parse_list_call(call, cells, item, lists) {
            return Some(mutation);
        }
    }
    match body {
        Expr::Update(update) => {
            let cell = expr_state_path(update.arg.as_ref())?;
            cell_initial(&cell)?;
            match update.op {
                UpdateOp::PlusPlus => Some(StateMutation::Increment(cell)),
                UpdateOp::MinusMinus => Some(StateMutation::Decrement(cell)),
            }
        }
        Expr::Assign(assignment) if assignment.op == AssignOp::Assign => {
            // 列表模板里的行字段：`item.done = true`。
            if let Some(scope) = item {
                let path = assign_target_path(&assignment.left)?;
                if let Some(field) = path.strip_prefix(&format!("{}.", scope.param)) {
                    let schema = lists.get(&scope.list)?;
                    schema.field_index(field)?;
                    let value = state_value_from_expr(assignment.right.as_ref())?;
                    return Some(StateMutation::SetItemField {
                        list: scope.list.clone(),
                        field: field.to_string(),
                        value,
                    });
                }
            }
            let target = assign_target_path(&assignment.left)?;
            // 列表整体赋值：只支持清空
            if lists.contains_key(&target) {
                if let Expr::Array(array) = assignment.right.as_ref() {
                    if array.elems.is_empty() {
                        return Some(StateMutation::ClearList { list: target });
                    }
                }
                return None;
            }
            // 整对象覆盖：逐单元写入
            if let Expr::Object(object) = assignment.right.as_ref() {
                let mut values = Vec::new();
                flatten_object_fields(&object.props, &target, &mut values)?;
                let known = values
                    .iter()
                    .all(|(path, _)| cells.iter().any(|(cell, _)| cell == path));
                return known.then_some(StateMutation::SetCells { values });
            }
            let cell = target;
            let initial = cell_initial(&cell)?;
            if let Expr::Bin(binary) = assignment.right.as_ref() {
                let same_cell =
                    expr_state_path(binary.left.as_ref()).as_deref() == Some(cell.as_str());
                if same_cell
                    && matches!(binary.right.as_ref(), Expr::Lit(Lit::Num(number)) if number.value == 1.0)
                {
                    return match binary.op {
                        BinaryOp::Add => Some(StateMutation::Increment(cell)),
                        BinaryOp::Sub => Some(StateMutation::Decrement(cell)),
                        _ => None,
                    };
                }
                // 跨单元数值：a = b + 1 / a = b - 1
                if let Some(source) = expr_state_path(binary.left.as_ref()) {
                    let numeric = matches!(cell_initial(&source), Some(StateValue::Number(_)))
                        && matches!(initial, StateValue::Number(_));
                    if numeric {
                        let delta = match (binary.op, binary.right.as_ref()) {
                            (BinaryOp::Add, Expr::Lit(Lit::Num(number)))
                                if number.value.fract() == 0.0 =>
                            {
                                number.value as i64
                            }
                            (BinaryOp::Sub, Expr::Lit(Lit::Num(number)))
                                if number.value.fract() == 0.0 =>
                            {
                                -(number.value as i64)
                            }
                            _ => return None,
                        };
                        return Some(StateMutation::SetFromCell {
                            target: cell,
                            source,
                            delta,
                        });
                    }
                }
            }
            // a = b（数值单元之间直接复制）
            if let Some(source) = expr_state_path(assignment.right.as_ref()) {
                let numeric = matches!(cell_initial(&source), Some(StateValue::Number(_)))
                    && matches!(initial, StateValue::Number(_));
                if numeric {
                    return Some(StateMutation::SetFromCell {
                        target: cell,
                        source,
                        delta: 0,
                    });
                }
            }
            if let (Some(parameter), Expr::Ident(value)) =
                (input_parameter, assignment.right.as_ref())
            {
                if value.sym.as_str() == parameter && matches!(initial, StateValue::String(_)) {
                    return Some(StateMutation::SetFromInput(cell));
                }
            }
            if matches!(assignment.right.as_ref(), Expr::Lit(Lit::Bool(value)) if value.value)
                && matches!(initial, StateValue::Boolean(_))
            {
                return Some(StateMutation::SetTrue(cell));
            }
            state_value_from_expr(assignment.right.as_ref())
                .map(|value| StateMutation::Set(cell, value))
        }
        _ => None,
    }
}

fn state_value_from_expr(expr: &Expr) -> Option<StateValue> {
    match expr {
        Expr::Lit(Lit::Num(number))
            if number.value.is_finite() && number.value.fract() == 0.0 =>
        {
            Some(StateValue::Number(number.value as i64))
        }
        Expr::Lit(Lit::Str(string)) => Some(StateValue::String(
            string.value.to_string_lossy().into_owned(),
        )),
        Expr::Lit(Lit::Bool(value)) => Some(StateValue::Boolean(value.value)),
        _ => None,
    }
}

// ---------- 新增部分：IRNode -> iOS LLVM IR ----------

use inkwell::AddressSpace;
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::targets::TargetTriple;
use inkwell::{FloatPredicate, IntPredicate};
use inkwell::types::{BasicMetadataTypeEnum, BasicType, BasicTypeEnum};
use inkwell::values::{
    BasicMetadataValueEnum, BasicValueEnum, FunctionValue, GlobalValue, IntValue, PointerValue,
};

/// 目标平台。IR 与布局共享，差异集中在视图类名、样式落点、事件与启动方式。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Platform {
    Ios,
    MacOs,
    Android,
}

impl Platform {
    fn is_mac(self) -> bool {
        matches!(self, Self::MacOs)
    }

    fn is_android(self) -> bool {
        matches!(self, Self::Android)
    }
}

struct IosCodeGen<'ctx> {
    platform: Platform,
    context: &'ctx Context,
    module: Module<'ctx>,
    builder: Builder<'ctx>,
    // 缓存三个 Objective-C runtime 的外部函数声明
    objc_get_class: FunctionValue<'ctx>,
    objc_msg_send: FunctionValue<'ctx>,
    sel_register_name: FunctionValue<'ctx>,
    /// 每个状态单元各自持有引用它的标签，更新时只触碰这一组。
    state_label_globals: RefCell<HashMap<String, Vec<GlobalValue<'ctx>>>>,
    state_label_counter: Cell<usize>,
    /// 所有状态单元（路径 + 初值），顺序决定全局变量名。
    state_cells: RefCell<Vec<(String, StateValue)>>,
    conditional_instances: RefCell<Vec<ConditionalRuntime<'ctx>>>,
    conditional_counter: Cell<usize>,
    /// AppDelegate 类与 class_addMethod，macOS 上要等界面构建完再注册状态动作
    delegate_class: RefCell<Option<PointerValue<'ctx>>>,
    class_add_method_fn: RefCell<Option<FunctionValue<'ctx>>>,
    /// --allow-degrade：把「平台不支持」的能力降级成 warning 而不是编译失败
    allow_degrade: bool,
    /// 列表渲染节点的运行期引用：同一个列表状态可以渲染多次
    list_slots: RefCell<HashMap<String, Vec<ListSlot<'ctx>>>>,
    /// 生成列表模板时当前所处的行
    list_row_ctx: RefCell<Option<ListRowContext<'ctx>>>,
    list_counter: Cell<usize>,
    /// `{a + b}` 这类表达式文本：依赖单元变化时重算
    expr_labels: RefCell<Vec<ExprLabel<'ctx>>>,
    expr_counter: Cell<usize>,
    /// 绑定了透明度的视图（按单元分组）：`style={{ opacity: fade }}`
    opacity_globals: RefCell<HashMap<String, Vec<GlobalValue<'ctx>>>>,
    opacity_counter: Cell<usize>,
}

#[derive(Clone)]
struct ExprLabel<'ctx> {
    deps: Vec<String>,
    label: GlobalValue<'ctx>,
    left: ExprOperand,
    right: ExprOperand,
    subtract: bool,
}

/// 列表数据容量：行视图是运行期按行数建的，这里只限制能存多少条数据。
pub(crate) const LIST_CAPACITY: usize = 256;

#[derive(Clone)]
struct ListSlot<'ctx> {
    /// 列表渲染节点编号（全局变量名的一部分）
    id: String,
    /// 列表状态名
    list: String,
    /// 字段名 -> `[LIST_CAPACITY x id]` 的数据列（存 NSString*）
    data_globals: HashMap<String, GlobalValue<'ctx>>,
    container_global: GlobalValue<'ctx>,
    count_global: GlobalValue<'ctx>,
    height_global: GlobalValue<'ctx>,
    parent_global: GlobalValue<'ctx>,
    depth_global: GlobalValue<'ctx>,
    /// 按当前行数重建行视图
    rebuild: FunctionValue<'ctx>,
}

/// 生成列表模板时当前所处的行：字段文本从这一行的数据列读取。
#[derive(Clone)]
struct ListRowContext<'ctx> {
    id: String,
    index: RowIndex<'ctx>,
}

#[derive(Clone, Copy)]
enum RowIndex<'ctx> {
    Const(usize),
    Runtime(IntValue<'ctx>),
}

impl<'ctx> RowIndex<'ctx> {
    fn value(&self, context: &'ctx Context) -> IntValue<'ctx> {
        match self {
            Self::Const(index) => context.i64_type().const_int(*index as u64, false),
            Self::Runtime(value) => *value,
        }
    }
}

#[derive(Clone)]
struct ConditionalRuntime<'ctx> {
    cell: String,
    /// 条件槽位视图（分支都挂在它下面）
    slot: GlobalValue<'ctx>,
    true_view: GlobalValue<'ctx>,
    false_view: GlobalValue<'ctx>,
    active_state: GlobalValue<'ctx>,
    /// 槽位高度约束，隐藏时把常量设为 0
    height_constraint: GlobalValue<'ctx>,
    slot_height: f64,
    true_mount: Vec<usize>,
    true_unmount: Vec<usize>,
    false_mount: Vec<usize>,
    false_unmount: Vec<usize>,
}

impl<'ctx> IosCodeGen<'ctx> {
    fn new(context: &'ctx Context) -> Self {
        Self::new_for(context, Platform::Ios)
    }

    fn new_for(context: &'ctx Context, platform: Platform) -> Self {
        let module = context.create_module("ios_app");
        let builder = context.create_builder();

        // 设置目标三元组：arm64-apple-ios
        module.set_triple(&TargetTriple::create(if platform.is_mac() {
            "arm64-apple-macosx14.0.0"
        } else if platform.is_android() {
            "aarch64-linux-android26"
        } else {
            "arm64-apple-ios"
        }));

        let ptr_type = context.ptr_type(AddressSpace::default());
        let i8_ptr = context.ptr_type(AddressSpace::default());

        // 声明：id objc_getClass(const char *)
        let get_class_type = ptr_type.fn_type(&[i8_ptr.into()], false);
        let objc_get_class = module.add_function("objc_getClass", get_class_type, None);

        // 声明：SEL sel_registerName(const char *)
        let sel_type = ptr_type.fn_type(&[i8_ptr.into()], false);
        let sel_register_name = module.add_function("sel_registerName", sel_type, None);

        // 声明：id objc_msgSend(id, SEL, ...)  可变参数
        let msg_send_type = ptr_type.fn_type(&[ptr_type.into(), ptr_type.into()], true);
        let objc_msg_send = module.add_function("objc_msgSend", msg_send_type, None);

        Self {
            platform,
            context,
            module,
            builder,
            objc_get_class,
            objc_msg_send,
            sel_register_name,
            state_label_globals: RefCell::new(HashMap::new()),
            state_label_counter: Cell::new(0),
            state_cells: RefCell::new(Vec::new()),
            conditional_instances: RefCell::new(Vec::new()),
            conditional_counter: Cell::new(0),
            delegate_class: RefCell::new(None),
            class_add_method_fn: RefCell::new(None),
            allow_degrade: false,
            list_slots: RefCell::new(HashMap::new()),
            list_row_ctx: RefCell::new(None),
            list_counter: Cell::new(0),
            expr_labels: RefCell::new(Vec::new()),
            expr_counter: Cell::new(0),
            opacity_globals: RefCell::new(HashMap::new()),
            opacity_counter: Cell::new(0),
        }
    }

    fn is_mac(&self) -> bool {
        self.platform.is_mac()
    }

    fn view_class(&self) -> &'static str {
        if self.is_mac() { "NSView" } else { "UIView" }
    }

    fn stack_class(&self) -> &'static str {
        if self.is_mac() { "NSStackView" } else { "UIStackView" }
    }

    fn label_class(&self) -> &'static str {
        if self.is_mac() { "NSTextField" } else { "UILabel" }
    }

    fn button_class(&self) -> &'static str {
        if self.is_mac() { "NSButton" } else { "UIButton" }
    }

    fn scroll_class(&self) -> &'static str {
        // macOS 首版把 ScrollView 当普通容器渲染（尚不支持滚动）
        if self.is_mac() { "NSView" } else { "UIScrollView" }
    }

    fn input_class(&self) -> &'static str {
        if self.is_mac() { "NSTextField" } else { "UITextField" }
    }

    fn image_view_class(&self) -> &'static str {
        if self.is_mac() { "NSImageView" } else { "UIImageView" }
    }

    fn generate(&mut self, program: &UiProgram) -> Result<(), String> {
        if !program.errors.is_empty() {
            return Err(program.errors.join("\n"));
        }
        // 平台能力校验：IR 里用到的能力必须在该后端的能力表里
        let warnings = capabilities::check(
            capabilities::backend_for(self.platform),
            program,
            self.allow_degrade,
        )?;
        for warning in warnings {
            eprintln!("warning: {warning}");
        }
        if self.platform.is_android() {
            return self.generate_android(program);
        }
        *self.state_cells.borrow_mut() = state_cells(&program.states);
        if program.states.is_empty() && program.nodes.iter().any(node_has_input_binding) {
            return Err("Input onInput requires a state(...) declaration".into());
        }
        if count_scroll_actions(&program.nodes) > 1 {
            return Err("this compiler phase supports one ScrollView onScroll binding per App".into());
        }
        if let Some((cell, bounds)) = scroll_action_cell(&program) {
            let numeric = state_cells(&program.states)
                .iter()
                .any(|(path, value)| path == cell && matches!(value, StateValue::Number(_)));
            if !numeric {
                return Err(format!(
                    "onScroll target '{cell}' must be a number state(...) cell"
                ));
            }
            if let Some((low, high, _shift)) = bounds {
                if low >= high {
                    return Err("onScroll clamp(y, low, high) 需要 low < high".into());
                }
            }
        }
        if count_top_background_views(&program.nodes) > 1 {
            return Err("this compiler phase supports one backgroundExtendsTo: \"top\" view per App".into());
        }
        if let Some(target) = find_top_background_view(&program.nodes) {
            if node_style(target).background_color.is_none() {
                return Err("backgroundExtendsTo: \"top\" requires a backgroundColor".into());
            }
        }
        validate_ignores_safe_area_views(&program.nodes)?;

        // Create the entry point before emitting runtime calls for the view tree.
        let i32_type = self.context.i32_type();
        let ptr_type = self.context.ptr_type(AddressSpace::default());
        let main_fn_type = i32_type.fn_type(&[i32_type.into(), ptr_type.into()], false);
        let main_fn = self.module.add_function("main", main_fn_type, None);
        let entry = self.context.append_basic_block(main_fn, "entry");
        self.builder.position_at_end(entry);

        // The view is created at runtime, so keep it in a mutable global initialized to nil.
        let global_view = self.module.add_global(ptr_type, None, "g_root_view");
        global_view.set_initializer(&ptr_type.const_null());

        for index in 0..self.state_cells.borrow().len() {
            let state_global = self
                .module
                .add_global(ptr_type, None, &format!("g_state_cell_{index}"));
            state_global.set_initializer(&ptr_type.const_null());
        }
        if let Some(target) = find_top_background_view(&program.nodes) {
            self.module
                .add_global(ptr_type, None, "g_top_background_view")
                .set_initializer(&ptr_type.const_null());
        }
        if self.is_mac() {
            self.register_flipped_view_class()?;
            for name in ["g_window", "g_app_delegate"] {
                let global = self.module.add_global(ptr_type, None, name);
                global.set_initializer(&ptr_type.const_null());
            }
        }
        // macOS：先把界面构建逻辑生成成一个函数，标签表在生成过程中就绪，
        // 之后注册 AppDelegate 的状态动作才能带上标签刷新。
        let ui_build_fn = if self.is_mac() {
            Some(self.build_ui_function(program)?)
        } else {
            None
        };
        // Build the view hierarchy from the app delegate after the app has launched.
        self.build_app_delegate_class(program)?;

        if self.is_mac() {
            let app = self
                .send_msg_ptr(
                    self.get_class("NSApplication")?,
                    self.get_selector("sharedApplication")?,
                    &[],
                )?
                .ok_or("NSApplication sharedApplication returned void")?;
            self.send_msg_void(
                app,
                "setActivationPolicy:",
                &[self.context.i64_type().const_int(0, false).into()],
            )?;
            let delegate_class = self.get_class("AppDelegate")?;
            let delegate = self
                .send_msg_ptr(delegate_class, self.get_selector("alloc")?, &[])?
                .ok_or("AppDelegate alloc returned void")?;
            let delegate = self
                .send_msg_ptr(delegate, self.get_selector("init")?, &[])?
                .ok_or("AppDelegate init returned void")?;
            if let Some(global) = self.module.get_global("g_app_delegate") {
                self.builder
                    .build_store(global.as_pointer_value(), delegate)
                    .map_err(|e| e.to_string())?;
            }
            self.send_msg_void(app, "setDelegate:", &[delegate.into()])?;
            // 界面构建逻辑已预先生成到 lumen_build_ui 里，这里只是调用
            let ui_fn = ui_build_fn.ok_or("macOS UI build function was not generated")?;
            self.builder
                .build_call(ui_fn, &[delegate.into()], "")
                .map_err(|e| e.to_string())?;
            self.send_msg_void(app, "run", &[])?;
            self.builder
                .build_return(Some(&i32_type.const_int(0, false)))
                .map_err(|e| e.to_string())?;
            self.module.verify().map_err(|e| e.to_string())?;
            self.module.print_to_file("out.ll").map_err(|e| e.to_string())?;
            println!("LLVM IR 已写入 out.ll (target: arm64-apple-macos)");
            return Ok(());
        }

        // Get argc and argv for UIApplicationMain.
        let argc = main_fn.get_nth_param(0).unwrap().into_int_value();
        let argv = main_fn.get_nth_param(1).unwrap().into_pointer_value();

        // UIApplicationMain expects NSString class names, not C string pointers.
        let nil_ptr = ptr_type.const_null();
        let delegate_name = self.create_nsstring("AppDelegate")?;
        
        // Declare and call UIApplicationMain.
        let uiapp_main_fn_type = i32_type.fn_type(&[
            i32_type.into(),
            ptr_type.into(),
            ptr_type.into(),
            ptr_type.into(),
        ], false);
        let uiapp_main_fn = self.module.add_function("UIApplicationMain", uiapp_main_fn_type, None);
        
        self.builder
            .build_call(
                uiapp_main_fn,
                &[
                    argc.into(),
                    argv.into(),
                    nil_ptr.into(),
                    delegate_name.into(),
                ],
                "",
            )
            .map_err(|e| e.to_string())?;

        // UIApplicationMain 理论上不会返回，但我们需要一个 return 指令来满足 IR 验证
        self.builder
            .build_return(Some(&i32_type.const_int(0, false)))
            .map_err(|e| e.to_string())?;

        self.module.verify().map_err(|e| e.to_string())?;
        self.module.print_to_file("out.ll").map_err(|e| e.to_string())?;
        println!("LLVM IR 已写入 out.ll (target: arm64-apple-ios)");
        Ok(())
    }

    /// 从 JNIEnv 的函数表按索引取出函数指针（JNINativeInterface 字段顺序由 JNI 规范固定）。
    fn jni_table_fn(&self, env: PointerValue<'ctx>, index: u64) -> Result<PointerValue<'ctx>, String> {
        let ptr_type = self.context.ptr_type(AddressSpace::default());
        let table = self
            .builder
            .build_load(ptr_type, env, "jni_table")
            .map_err(|e| e.to_string())?
            .into_pointer_value();
        let slot = unsafe {
            self.builder.build_gep(
                ptr_type,
                table,
                &[self.context.i64_type().const_int(index, false)],
                "jni_slot",
            )
        }
        .map_err(|e| e.to_string())?;
        Ok(self
            .builder
            .build_load(ptr_type, slot, "jni_fn")
            .map_err(|e| e.to_string())?
            .into_pointer_value())
    }

    fn jni_call(
        &self,
        env: PointerValue<'ctx>,
        index: u64,
        fn_type: inkwell::types::FunctionType<'ctx>,
        args: &[BasicMetadataValueEnum<'ctx>],
    ) -> Result<Option<BasicValueEnum<'ctx>>, String> {
        let fn_ptr = self.jni_table_fn(env, index)?;
        let call = self
            .builder
            .build_indirect_call(fn_type, fn_ptr, args, "jni_result")
            .map_err(|e| e.to_string())?;
        Ok(match call.try_as_basic_value() {
            inkwell::values::ValueKind::Basic(value) => Some(value),
            inkwell::values::ValueKind::Instruction(_) => None,
        })
    }

    /// 安卓入口：生成 JNI 导出和整套原生控件构建逻辑，实现见 `android` 模块。
    fn generate_android(&self, program: &UiProgram) -> Result<(), String> {
        android::emit(self, program)?;
        self.module
            .print_to_file("out.ll")
            .map_err(|error| error.to_string())?;
        println!("LLVM IR 已写入 out.ll (target: aarch64-linux-android26)");
        Ok(())
    }

    /// 把界面构建逻辑生成成独立函数 `void lumen_build_ui(id delegate)`。
    fn build_ui_function(&self, program: &UiProgram) -> Result<FunctionValue<'ctx>, String> {
        let ptr_type = self.context.ptr_type(AddressSpace::default());
        let fn_type = self.context.void_type().fn_type(&[ptr_type.into()], false);
        let function = self.module.add_function("lumen_build_ui", fn_type, None);
        let saved = self.builder.get_insert_block().unwrap();
        let entry = self.context.append_basic_block(function, "entry");
        self.builder.position_at_end(entry);
        let receiver = function
            .get_nth_param(0)
            .ok_or("lumen_build_ui receiver parameter is missing")?
            .into_pointer_value();
        self.emit_ui_construction(program, receiver)?;
        self.builder
            .build_return(None)
            .map_err(|e| e.to_string())?;
        self.builder.position_at_end(saved);
        Ok(function)
    }

    /// 给 AppDelegate 注册 stateActionN: 方法。
    /// 必须在界面构建之后调用：回调里要按单元刷新标签，依赖构建期登记的标签表。
    fn register_state_action_methods(
        &self,
        program: &UiProgram,
        class_add_method: FunctionValue<'ctx>,
        app_delegate_class: PointerValue<'ctx>,
    ) -> Result<(), String> {
        for (index, action) in program.state_actions.iter().enumerate() {
            let method = self.build_state_callback_method(action, index)?;
            let selector = self.get_selector(&format!("stateAction{index}:"))?;
            let encoding = self
                .builder
                .build_global_string_ptr("v@:@", &format!("state_action_{index}_encoding"))
                .map_err(|error| error.to_string())?;
            self.builder
                .build_call(
                    class_add_method,
                    &[
                        app_delegate_class.into(),
                        selector.into(),
                        method.as_global_value().as_pointer_value().into(),
                        encoding.as_pointer_value().into(),
                    ],
                    "",
                )
                .map_err(|error| error.to_string())?;
        }
        Ok(())
    }

    /// macOS 的输入框回调：`- (void)controlTextDidChange:(NSNotification *)note`。
    /// 从通知里取出 NSTextField，和构建期登记的每个 onInput 输入框比身份，
    /// 命中就调对应的 stateAction<k>:（参数就是输入框自己，动作里用 stringValue 取值）。
    fn build_input_delegate_method(
        &self,
        program: &UiProgram,
    ) -> Result<Option<FunctionValue<'ctx>>, String> {
        let mut actions = Vec::new();
        collect_input_actions(&program.nodes, &mut actions);
        if actions.is_empty() {
            return Ok(None);
        }

        let ptr_type = self.context.ptr_type(AddressSpace::default());
        let fn_type = self
            .context
            .void_type()
            .fn_type(&[ptr_type.into(), ptr_type.into(), ptr_type.into()], false);
        let function = self
            .module
            .add_function("app_delegate_control_text_did_change", fn_type, None);
        let saved = self.builder.get_insert_block();
        let entry = self.context.append_basic_block(function, "entry");
        self.builder.position_at_end(entry);

        let this = function
            .get_nth_param(0)
            .ok_or("delegate receiver is missing")?
            .into_pointer_value();
        let notification = function
            .get_nth_param(2)
            .ok_or("notification parameter is missing")?
            .into_pointer_value();
        let field = self
            .send_msg_ptr(notification, self.get_selector("object")?, &[])?
            .ok_or("NSNotification object returned void")?;
        let field_bits = self
            .builder
            .build_ptr_to_int(field, self.context.i64_type(), "edited_field")
            .map_err(|e| e.to_string())?;

        let done = self.context.append_basic_block(function, "done");
        for action in actions {
            let name = format!("g_input_source_{action}");
            let global = self
                .module
                .get_global(&name)
                .ok_or_else(|| format!("{name} was not created"))?;
            let stored = self
                .builder
                .build_load(ptr_type, global.as_pointer_value(), "input_source")
                .map_err(|e| e.to_string())?
                .into_pointer_value();
            let stored_bits = self
                .builder
                .build_ptr_to_int(stored, self.context.i64_type(), "input_source_bits")
                .map_err(|e| e.to_string())?;
            let same = self
                .builder
                .build_int_compare(IntPredicate::EQ, stored_bits, field_bits, "same_input")
                .map_err(|e| e.to_string())?;
            let hit = self
                .context
                .append_basic_block(function, &format!("input_{action}"));
            let next = self
                .context
                .append_basic_block(function, &format!("input_{action}_next"));
            self.builder
                .build_conditional_branch(same, hit, next)
                .map_err(|e| e.to_string())?;
            self.builder.position_at_end(hit);
            self.send_msg_void(this, &format!("stateAction{action}:"), &[field.into()])?;
            self.builder
                .build_unconditional_branch(done)
                .map_err(|e| e.to_string())?;
            self.builder.position_at_end(next);
        }
        self.builder
            .build_unconditional_branch(done)
            .map_err(|e| e.to_string())?;
        self.builder.position_at_end(done);
        self.builder.build_return(None).map_err(|e| e.to_string())?;
        if let Some(saved) = saved {
            self.builder.position_at_end(saved);
        }
        Ok(Some(function))
    }

    /// 记下 onInput 输入框，供 controlTextDidChange: 里比对身份。
    fn store_input_source(
        &self,
        action: usize,
        input: PointerValue<'ctx>,
    ) -> Result<(), String> {
        let name = format!("g_input_source_{action}");
        let ptr_type = self.context.ptr_type(AddressSpace::default());
        let global = match self.module.get_global(&name) {
            Some(global) => global,
            None => {
                let global = self.module.add_global(ptr_type, None, &name);
                global.set_initializer(&ptr_type.const_null());
                global
            }
        };
        self.builder
            .build_store(global.as_pointer_value(), input)
            .map_err(|e| e.to_string())?;
        Ok(())
    }
    /// `receiver` 同时作为控件的事件目标（按钮的 target）。
    fn emit_ui_construction(
        &self,
        program: &UiProgram,
        receiver: PointerValue<'ctx>,
    ) -> Result<(), String> {
        // 列表行在单独的函数里构建，事件目标通过全局变量共享。
        let event_target_global = self.object_global("g_event_target");
        self.builder
            .build_store(event_target_global.as_pointer_value(), receiver)
            .map_err(|e| e.to_string())?;
        let cells = self.state_cells.borrow().clone();
        for (index, (_, initial)) in cells.iter().enumerate() {
            let initial_value = self.create_nsstring(&initial.display_value())?;
            let state_global = self
                .module
                .get_global(&format!("g_state_cell_{index}"))
                .ok_or("state value global was not created")?;
            self.builder
                .build_store(state_global.as_pointer_value(), initial_value)
                .map_err(|error| error.to_string())?;
        }
        drop(cells);

        let event_target = Some(receiver);
        let root_view = if let Some(node) = program.nodes.first() {
            self.codegen_node(node, event_target, 0.0, 0.0, 350.0, 700.0, None, true)?
        } else {
            self.create_objc_instance_with_frame(self.view_class(), self.screen_bounds()?)?
        };
        let root_view_global = self
            .module
            .get_global("g_root_view")
            .ok_or("root view global was not created")?;
        self.builder
            .build_store(root_view_global.as_pointer_value(), root_view)
            .map_err(|e| e.to_string())?;

        self.build_show_window_logic(program.nodes.first())?;

        let mut initial_mount_actions = Vec::new();
        collect_initial_mount_actions(&program.nodes, &mut initial_mount_actions);
        for action in initial_mount_actions {
            self.emit_lifecycle_action(receiver, Some(action))?;
        }
        Ok(())
    }

    /// 动态构建 AppDelegate 类，并实现 application:didFinishLaunchingWithOptions: 方法
    fn build_app_delegate_class(&self, program: &UiProgram) -> Result<(), String> {
        let ptr_type = self.context.ptr_type(AddressSpace::default());
        let i8_ptr = ptr_type;

        // 声明 Objective-C 运行时函数
        // Class objc_allocateClassPair(Class superclass, const char *name, size_t extraBytes)
        let allocate_class_type = ptr_type.fn_type(&[ptr_type.into(), i8_ptr.into(), self.context.i64_type().into()], false);
        let objc_allocate_class_pair = self
            .module
            .get_function("objc_allocateClassPair")
            .unwrap_or_else(|| self.module.add_function("objc_allocateClassPair", allocate_class_type, None));

        // BOOL class_addMethod(Class cls, SEL name, IMP imp, const char *types)
        let add_method_type = self.context.bool_type().fn_type(&[ptr_type.into(), ptr_type.into(), ptr_type.into(), i8_ptr.into()], false);
        let class_add_method = self
            .module
            .get_function("class_addMethod")
            .unwrap_or_else(|| self.module.add_function("class_addMethod", add_method_type, None));

        // void objc_registerClassPair(Class cls)
        let register_class_type = self.context.void_type().fn_type(&[ptr_type.into()], false);
        let objc_register_class_pair = self
            .module
            .get_function("objc_registerClassPair")
            .unwrap_or_else(|| self.module.add_function("objc_registerClassPair", register_class_type, None));

        // 获取 NSObject 类作为父类
        let nsobject = self.get_class("NSObject")?;

        // 创建 AppDelegate 类
        let class_name_ptr = self
            .builder
            .build_global_string_ptr("AppDelegate", "app_delegate_class_name")
            .map_err(|e| e.to_string())?;
        let class_call = self
            .builder
            .build_call(
                objc_allocate_class_pair,
                &[
                    nsobject.into(),
                    class_name_ptr.as_pointer_value().into(),
                    self.context.i64_type().const_int(0, false).into(),
                ],
                "app_delegate_class",
            )
            .map_err(|e| e.to_string())?;
        let app_delegate_class = match class_call.try_as_basic_value() {
            inkwell::values::ValueKind::Basic(value) => value.into_pointer_value(),
            inkwell::values::ValueKind::Instruction(_) => {
                return Err("objc_allocateClassPair returned void".into());
            }
        };

        // iOS: -application:didFinishLaunchingWithOptions: (返回 BOOL)
        // macOS: -applicationDidFinishLaunching: (返回 void)
        let bool_type = self.context.i8_type();
        let did_finish_launching_fn_type = if self.is_mac() {
            self.context.void_type().fn_type(
                &[
                    ptr_type.into(), // self
                    ptr_type.into(), // _cmd
                    ptr_type.into(), // notification
                ],
                false,
            )
        } else {
            bool_type.fn_type(
                &[
                    ptr_type.into(), // self
                    ptr_type.into(), // _cmd
                    ptr_type.into(), // application
                    ptr_type.into(), // launchOptions
                ],
                false,
            )
        };
        let did_finish_launching_fn = self.module.add_function("app_delegate_did_finish_launching", did_finish_launching_fn_type, None);
        let fn_entry = self.context.append_basic_block(did_finish_launching_fn, "entry");
        // 保存当前的插入点，稍后恢复
        let saved_block = self.builder.get_insert_block().unwrap();
        self.builder.position_at_end(fn_entry);

        let delegate = did_finish_launching_fn
            .get_nth_param(0)
            .ok_or("app delegate self parameter is missing")?
            .into_pointer_value();
        if !self.is_mac() {
            // macOS 的界面由 main 直接构建，这里保持空实现
            self.emit_ui_construction(program, delegate)?;
        }

        if self.is_mac() {
            self.builder.build_return(None).map_err(|e| e.to_string())?;
        } else {
            self.builder
                .build_return(Some(&bool_type.const_int(1, false)))
                .map_err(|e| e.to_string())?;
        }
        // 恢复插入点
        self.builder.position_at_end(saved_block);

        // 将方法添加到类中
        let (launch_selector, launch_encoding) = if self.is_mac() {
            ("applicationDidFinishLaunching:", "v@:@")
        } else {
            ("application:didFinishLaunchingWithOptions:", "c@:@@")
        };
        let sel_register = self.get_selector(launch_selector)?;
        let type_encoding = self
            .builder
            .build_global_string_ptr(launch_encoding, "method_type_encoding")
            .map_err(|e| e.to_string())?;
        
        self.builder
            .build_call(
                class_add_method,
                &[
                    app_delegate_class.into(),
                    sel_register.into(),
                    did_finish_launching_fn.as_global_value().as_pointer_value().into(),
                    type_encoding.as_pointer_value().into(),
                ],
                "",
            )
            .map_err(|e| e.to_string())?;

        *self.delegate_class.borrow_mut() = Some(app_delegate_class);
        *self.class_add_method_fn.borrow_mut() = Some(class_add_method);
        if !program.state_actions.is_empty() {
            self.register_state_action_methods(program, class_add_method, app_delegate_class)?;
        }
        // macOS：输入框的实时回写靠 delegate 的 controlTextDidChange:
        if self.is_mac() {
            if let Some(method) = self.build_input_delegate_method(program)? {
                let selector = self.get_selector("controlTextDidChange:")?;
                let encoding = self
                    .builder
                    .build_global_string_ptr("v@:@", "input_delegate_encoding")
                    .map_err(|error| error.to_string())?;
                self.builder
                    .build_call(
                        class_add_method,
                        &[
                            app_delegate_class.into(),
                            selector.into(),
                            method.as_global_value().as_pointer_value().into(),
                            encoding.as_pointer_value().into(),
                        ],
                        "",
                    )
                    .map_err(|error| error.to_string())?;
            }
        }

        if find_scroll_action(&program.nodes).is_some() {
            let _ = self.object_global("g_scroll_action_view");
            let callback = self.build_scroll_delegate(program)?;
            let selector = self.get_selector("scrollViewDidScroll:")?;
            let encoding = self
                .builder
                .build_global_string_ptr("v@:@", "scroll_fade_callback_encoding")
                .map_err(|error| error.to_string())?;
            self.builder
                .build_call(
                    class_add_method,
                    &[
                        app_delegate_class.into(),
                        selector.into(),
                        callback.as_global_value().as_pointer_value().into(),
                        encoding.as_pointer_value().into(),
                    ],
                    "",
                )
                .map_err(|error| error.to_string())?;
        }

        // 注册类
        self.builder
            .build_call(
                objc_register_class_pair,
                &[app_delegate_class.into()],
                "",
            )
            .map_err(|e| e.to_string())?;

        Ok(())
    }

    /// 列表渲染的槽位：按列表状态名索引，可能有多份渲染。
    fn list_slots_for(&self, list: &str) -> Result<Vec<ListSlot<'ctx>>, String> {
        self.list_slots
            .borrow()
            .get(list)
            .cloned()
            .filter(|slots| !slots.is_empty())
            .ok_or_else(|| format!("list state '{list}' is not rendered by this App"))
    }

    fn current_function(&self) -> Result<FunctionValue<'ctx>, String> {
        self.builder
            .get_insert_block()
            .and_then(|block| block.get_parent())
            .ok_or_else(|| "missing insertion block".to_string())
    }

    /// 对象指针全局变量（不存在时创建并置 nil）。
    fn object_global(&self, name: &str) -> GlobalValue<'ctx> {
        if let Some(global) = self.module.get_global(name) {
            return global;
        }
        let ptr_type = self.context.ptr_type(AddressSpace::default());
        let global = self.module.add_global(ptr_type, None, name);
        global.set_initializer(&ptr_type.const_null());
        global
    }

    fn i64_global(&self, name: &str) -> GlobalValue<'ctx> {
        if let Some(global) = self.module.get_global(name) {
            return global;
        }
        let i64_type = self.context.i64_type();
        let global = self.module.add_global(i64_type, None, name);
        global.set_initializer(&i64_type.const_zero());
        global
    }

    /// 列表某个字段的数据列：`[LIST_CAPACITY x id]`，元素是 NSString*。
    fn list_data_global(&self, id: &str, field: &str) -> GlobalValue<'ctx> {
        let name = format!("g_list_{id}_data_{field}");
        if let Some(global) = self.module.get_global(&name) {
            return global;
        }
        let ptr_type = self.context.ptr_type(AddressSpace::default());
        let array_type = ptr_type.array_type(LIST_CAPACITY as u32);
        let global = self.module.add_global(array_type, None, &name);
        global.set_initializer(&array_type.const_zero());
        global
    }

    fn list_number_column(&self, id: &str, column: &str) -> GlobalValue<'ctx> {
        let name = format!("g_list_{id}_{column}");
        if let Some(global) = self.module.get_global(&name) {
            return global;
        }
        let array = self.context.i64_type().array_type(LIST_CAPACITY as u32);
        let global = self.module.add_global(array, None, &name);
        global.set_initializer(&array.const_zero());
        global
    }

    fn store_list_number(
        &self,
        id: &str,
        column: &str,
        index: IntValue<'ctx>,
        value: IntValue<'ctx>,
    ) -> Result<(), String> {
        let global = self.list_number_column(id, column);
        let array = self.context.i64_type().array_type(LIST_CAPACITY as u32);
        let slot = unsafe {
            self.builder.build_gep(
                array,
                global.as_pointer_value(),
                &[self.context.i64_type().const_zero(), index],
                "list_num_slot",
            )
        }
        .map_err(|e| e.to_string())?;
        self.builder
            .build_store(slot, value)
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    fn load_list_number(
        &self,
        id: &str,
        column: &str,
        index: IntValue<'ctx>,
    ) -> Result<IntValue<'ctx>, String> {
        let global = self.list_number_column(id, column);
        let array = self.context.i64_type().array_type(LIST_CAPACITY as u32);
        let slot = unsafe {
            self.builder.build_gep(
                array,
                global.as_pointer_value(),
                &[self.context.i64_type().const_zero(), index],
                "list_num_slot",
            )
        }
        .map_err(|e| e.to_string())?;
        Ok(self
            .builder
            .build_load(self.context.i64_type(), slot, "list_num_value")
            .map_err(|e| e.to_string())?
            .into_int_value())
    }

    fn store_list_value(
        &self,
        id: &str,
        field: &str,
        index: IntValue<'ctx>,
        value: PointerValue<'ctx>,
    ) -> Result<(), String> {
        let global = self.list_data_global(id, field);
        let ptr_type = self.context.ptr_type(AddressSpace::default());
        let array_type = ptr_type.array_type(LIST_CAPACITY as u32);
        let slot = unsafe {
            self.builder.build_gep(
                array_type,
                global.as_pointer_value(),
                &[self.context.i64_type().const_zero(), index],
                "list_data_slot",
            )
        }
        .map_err(|e| e.to_string())?;
        self.builder
            .build_store(slot, value)
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    fn load_list_value(
        &self,
        id: &str,
        field: &str,
        index: IntValue<'ctx>,
    ) -> Result<PointerValue<'ctx>, String> {
        let global = self.list_data_global(id, field);
        let ptr_type = self.context.ptr_type(AddressSpace::default());
        let array_type = ptr_type.array_type(LIST_CAPACITY as u32);
        let slot = unsafe {
            self.builder.build_gep(
                array_type,
                global.as_pointer_value(),
                &[self.context.i64_type().const_zero(), index],
                "list_data_slot",
            )
        }
        .map_err(|e| e.to_string())?;
        Ok(self
            .builder
            .build_load(ptr_type, slot, "list_data_value")
            .map_err(|e| e.to_string())?
            .into_pointer_value())
    }

    /// 行内按钮的 `tag` 就是行号。
    fn sender_row(&self, sender: PointerValue<'ctx>) -> Result<IntValue<'ctx>, String> {
        self.send_msg_impl(
            sender,
            self.get_selector("tag")?,
            &[],
            BasicTypeEnum::IntType(self.context.i64_type()),
        )?
        .ok_or_else(|| "sender tag returned void".to_string())
        .map(|value| value.into_int_value())
    }

    /// 高度约束：先激活，再把约束对象存起来，刷新时改 constant。
    fn add_list_height_constraint(
        &self,
        view: PointerValue<'ctx>,
        height: f64,
        global: GlobalValue<'ctx>,
    ) -> Result<(), String> {
        let anchor = self
            .send_msg_ptr(view, self.get_selector("heightAnchor")?, &[])?
            .ok_or_else(|| "heightAnchor returned void".to_string())?;
        let constraint = self
            .send_msg_ptr(
                anchor,
                self.get_selector("constraintEqualToConstant:")?,
                &[self.context.f64_type().const_float(height).into()],
            )?
            .ok_or_else(|| "height constraint creation returned void".to_string())?;
        self.activate_constraint(constraint)?;
        self.builder
            .build_store(global.as_pointer_value(), constraint)
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    /// `items.push({...})`：写一行数据，然后把列表整体刷新一次。
    fn emit_push_item(
        &self,
        list: &str,
        values: &[(String, ItemValue)],
        receiver: PointerValue<'ctx>,
    ) -> Result<(), String> {
        let slots = self.list_slots_for(list)?;
        let slot = slots[0].clone();
        let function = self.current_function()?;
        let i64_type = self.context.i64_type();
        let ptr_type = self.context.ptr_type(AddressSpace::default());
        let count = self
            .builder
            .build_load(i64_type, slot.count_global.as_pointer_value(), "list_count")
            .map_err(|e| e.to_string())?
            .into_int_value();
        let has_room = self
            .builder
            .build_int_compare(
                IntPredicate::SLT,
                count,
                i64_type.const_int(LIST_CAPACITY as u64, false),
                "list_has_room",
            )
            .map_err(|e| e.to_string())?;
        let append = self.context.append_basic_block(function, "list_append");
        let done = self.context.append_basic_block(function, "list_append_done");
        self.builder
            .build_conditional_branch(has_room, append, done)
            .map_err(|e| e.to_string())?;
        self.builder.position_at_end(append);

        let mut columns: Vec<String> = slot.data_globals.keys().cloned().collect();
        columns.sort();
        for column in &columns {
            let value = values.iter().find(|(name, _)| name == column).map(|(_, value)| value);
            let text = match value {
                Some(ItemValue::Literal(value)) => self.create_nsstring(&value.display_value())?,
                Some(ItemValue::Cell(cell)) => {
                    let name = self
                        .cell_global_name(cell)
                        .ok_or_else(|| format!("state cell '{cell}' was not collected"))?;
                    let global = self
                        .module
                        .get_global(&name)
                        .ok_or("state value global was not created")?;
                    let value = self
                        .builder
                        .build_load(ptr_type, global.as_pointer_value(), "push_cell_value")
                        .map_err(|e| e.to_string())?
                        .into_pointer_value();
                    // 单元后续会被改写，行存储必须拿一份自己的副本
                    self.copy_nsstring(value)?
                }
                None => self.create_nsstring("")?,
            };
            self.store_list_value(&slot.id, column, count, text)?;
        }
        // 新行挂在根层：parent = -1，depth = 0
        self.store_list_number(
            &slot.id,
            "parent",
            count,
            i64_type.const_int(u64::MAX, true),
        )?;
        self.store_list_number(&slot.id, "depth", count, i64_type.const_zero())?;
        let next = self
            .builder
            .build_int_add(count, i64_type.const_int(1, false), "list_count_next")
            .map_err(|e| e.to_string())?;
        self.builder
            .build_store(slot.count_global.as_pointer_value(), next)
            .map_err(|e| e.to_string())?;
        self.sync_list_length(list, next, receiver)?;
        for slot in &slots {
            self.builder
                .build_call(slot.rebuild, &[], "")
                .map_err(|e| e.to_string())?;
        }
        self.builder
            .build_unconditional_branch(done)
            .map_err(|e| e.to_string())?;
        self.builder.position_at_end(done);
        Ok(())
    }

    /// `items = []`：行数归零并重建。
    fn emit_clear_list(
        &self,
        list: &str,
        receiver: PointerValue<'ctx>,
    ) -> Result<(), String> {
        let slots = self.list_slots_for(list)?;
        let slot = slots[0].clone();
        let zero = self.context.i64_type().const_zero();
        self.builder
            .build_store(slot.count_global.as_pointer_value(), zero)
            .map_err(|e| e.to_string())?;
        self.sync_list_length(list, zero, receiver)?;
        for slot in &slots {
            self.builder
                .build_call(slot.rebuild, &[], "")
                .map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    /// 列表长度对外是普通单元：`{items.length}` 靠它刷新。
    fn sync_list_length(
        &self,
        list: &str,
        count: IntValue<'ctx>,
        receiver: PointerValue<'ctx>,
    ) -> Result<(), String> {
        let cell = format!("{list}.length");
        if self.cell_global_name(&cell).is_none() {
            return Ok(());
        }
        self.store_cell_number(&cell, count, receiver)
    }

    fn store_cell_text(
        &self,
        cell: &str,
        text: PointerValue<'ctx>,
        receiver: PointerValue<'ctx>,
    ) -> Result<(), String> {
        let name = self
            .cell_global_name(cell)
            .ok_or_else(|| format!("state cell '{cell}' was not collected"))?;
        let global = self
            .module
            .get_global(&name)
            .ok_or("state value global was not created")?;
        self.builder
            .build_store(global.as_pointer_value(), text)
            .map_err(|e| e.to_string())?;
        self.update_cell_labels(cell, text)?;
        self.update_conditional_views(self.current_function()?, receiver, cell, text)?;
        Ok(())
    }

    /// 把整数写进单元：`%lld` 格式化后按文本存储，和标量单元保持一致。
    fn store_cell_number(
        &self,
        cell: &str,
        value: IntValue<'ctx>,
        receiver: PointerValue<'ctx>,
    ) -> Result<(), String> {
        let sanitized: String = cell
            .chars()
            .map(|ch| if ch.is_alphanumeric() { ch } else { '_' })
            .collect();
        let buffer_type = self.context.i8_type().array_type(32);
        let name = format!("g_cell_number_buffer_{sanitized}");
        let buffer = match self.module.get_global(&name) {
            Some(global) => global,
            None => {
                let global = self.module.add_global(buffer_type, None, &name);
                global.set_initializer(&buffer_type.const_zero());
                global
            }
        };
        let format = self
            .builder
            .build_global_string_ptr("%lld", "cell_number_format")
            .map_err(|e| e.to_string())?;
        let ptr_type = self.context.ptr_type(AddressSpace::default());
        let snprintf_type = self.context.i32_type().fn_type(
            &[
                ptr_type.into(),
                self.context.i64_type().into(),
                ptr_type.into(),
            ],
            true,
        );
        let snprintf = self
            .module
            .get_function("snprintf")
            .unwrap_or_else(|| self.module.add_function("snprintf", snprintf_type, None));
        self.builder
            .build_call(
                snprintf,
                &[
                    buffer.as_pointer_value().into(),
                    self.context.i64_type().const_int(32, false).into(),
                    format.as_pointer_value().into(),
                    value.into(),
                ],
                "cell_number_text",
            )
            .map_err(|e| e.to_string())?;
        let text = self.create_nsstring_from_cstr(buffer.as_pointer_value())?;
        self.store_cell_text(cell, text, receiver)
    }

    /// 行内 `item.field = <字面量>`：写点击行的那一格。
    fn emit_set_item_field(
        &self,
        list: &str,
        field: &str,
        value: &StateValue,
        sender: PointerValue<'ctx>,
    ) -> Result<(), String> {
        let slots = self.list_slots_for(list)?;
        let slot = slots[0].clone();
        let row = self.sender_row(sender)?;
        let text = self.create_nsstring(&value.display_value())?;
        self.store_list_value(&slot.id, field, row, text)?;
        for slot in &slots {
            self.builder
                .build_call(slot.rebuild, &[], "")
                .map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    /// 行内 `items.removeAt(index)`：后面的行整体前移，行数减一。
    fn emit_remove_current_item(
        &self,
        list: &str,
        sender: PointerValue<'ctx>,
        receiver: PointerValue<'ctx>,
    ) -> Result<(), String> {
        let slots = self.list_slots_for(list)?;
        let slot = slots[0].clone();
        let function = self.current_function()?;
        let i64_type = self.context.i64_type();
        let row = self.sender_row(sender)?;
        let count = self
            .builder
            .build_load(i64_type, slot.count_global.as_pointer_value(), "list_count")
            .map_err(|e| e.to_string())?
            .into_int_value();
        let has_rows = self
            .builder
            .build_int_compare(
                IntPredicate::SGT,
                count,
                i64_type.const_zero(),
                "list_has_rows",
            )
            .map_err(|e| e.to_string())?;
        let remove = self.context.append_basic_block(function, "list_remove");
        let done = self.context.append_basic_block(function, "list_remove_done");
        self.builder
            .build_conditional_branch(has_rows, remove, done)
            .map_err(|e| e.to_string())?;
        self.builder.position_at_end(remove);

        let mut columns: Vec<String> = slot.data_globals.keys().cloned().collect();
        columns.sort();

        // 子树范围：从 row+1 往后扫，深度回落到 <= 本行深度就结束
        let row_depth = self.load_list_number(&slot.id, "depth", row)?;
        let end_cursor = self
            .builder
            .build_alloca(i64_type, "subtree_end")
            .map_err(|e| e.to_string())?;
        let start = self
            .builder
            .build_int_add(row, i64_type.const_int(1, false), "subtree_start")
            .map_err(|e| e.to_string())?;
        self.builder
            .build_store(end_cursor, start)
            .map_err(|e| e.to_string())?;
        let scan_block = self.context.append_basic_block(function, "subtree_scan");
        let scan_body = self.context.append_basic_block(function, "subtree_scan_body");
        let scan_done = self.context.append_basic_block(function, "subtree_scan_done");
        self.builder
            .build_unconditional_branch(scan_block)
            .map_err(|e| e.to_string())?;
        self.builder.position_at_end(scan_block);
        let end = self
            .builder
            .build_load(i64_type, end_cursor, "subtree_end_value")
            .map_err(|e| e.to_string())?
            .into_int_value();
        let in_range = self
            .builder
            .build_int_compare(IntPredicate::SLT, end, count, "subtree_in_range")
            .map_err(|e| e.to_string())?;
        let end_block = self.context.append_basic_block(function, "subtree_end_check");
        self.builder
            .build_conditional_branch(in_range, end_block, scan_done)
            .map_err(|e| e.to_string())?;
        self.builder.position_at_end(end_block);
        let end_depth = self.load_list_number(&slot.id, "depth", end)?;
        let deeper = self
            .builder
            .build_int_compare(IntPredicate::SGT, end_depth, row_depth, "subtree_deeper")
            .map_err(|e| e.to_string())?;
        self.builder
            .build_conditional_branch(deeper, scan_body, scan_done)
            .map_err(|e| e.to_string())?;
        self.builder.position_at_end(scan_body);
        let advanced_end = self
            .builder
            .build_int_add(end, i64_type.const_int(1, false), "subtree_next")
            .map_err(|e| e.to_string())?;
        self.builder
            .build_store(end_cursor, advanced_end)
            .map_err(|e| e.to_string())?;
        self.builder
            .build_unconditional_branch(scan_block)
            .map_err(|e| e.to_string())?;
        self.builder.position_at_end(scan_done);
        let subtree_end = self
            .builder
            .build_load(i64_type, end_cursor, "subtree_end_final")
            .map_err(|e| e.to_string())?
            .into_int_value();
        let span = self
            .builder
            .build_int_sub(subtree_end, row, "subtree_span")
            .map_err(|e| e.to_string())?;

        // 把 [subtree_end, count) 整段前移 span 格
        let cursor = self
            .builder
            .build_alloca(i64_type, "shift_index")
            .map_err(|e| e.to_string())?;
        self.builder
            .build_store(cursor, row)
            .map_err(|e| e.to_string())?;
        let loop_block = self.context.append_basic_block(function, "list_shift_loop");
        let body_block = self.context.append_basic_block(function, "list_shift_body");
        let end_block = self.context.append_basic_block(function, "list_shift_done");
        self.builder
            .build_unconditional_branch(loop_block)
            .map_err(|e| e.to_string())?;

        self.builder.position_at_end(loop_block);
        let index = self
            .builder
            .build_load(i64_type, cursor, "shift_index_value")
            .map_err(|e| e.to_string())?
            .into_int_value();
        let source = self
            .builder
            .build_int_add(index, span, "shift_source")
            .map_err(|e| e.to_string())?;
        let more = self
            .builder
            .build_int_compare(IntPredicate::SLT, source, count, "shift_has_more")
            .map_err(|e| e.to_string())?;
        self.builder
            .build_conditional_branch(more, body_block, end_block)
            .map_err(|e| e.to_string())?;

        self.builder.position_at_end(body_block);
        let next_index = self
            .builder
            .build_int_add(index, i64_type.const_int(1, false), "shift_next")
            .map_err(|e| e.to_string())?;
        for column in &columns {
            let value = self.load_list_value(&slot.id, column, source)?;
            self.store_list_value(&slot.id, column, index, value)?;
        }
        let source_parent = self.load_list_number(&slot.id, "parent", source)?;
        self.store_list_number(&slot.id, "parent", index, source_parent)?;
        let source_depth = self.load_list_number(&slot.id, "depth", source)?;
        self.store_list_number(&slot.id, "depth", index, source_depth)?;
        self.builder
            .build_store(cursor, next_index)
            .map_err(|e| e.to_string())?;
        self.builder
            .build_unconditional_branch(loop_block)
            .map_err(|e| e.to_string())?;

        self.builder.position_at_end(end_block);
        let next_count = self
            .builder
            .build_int_sub(count, span, "list_count_next")
            .map_err(|e| e.to_string())?;
        self.builder
            .build_store(slot.count_global.as_pointer_value(), next_count)
            .map_err(|e| e.to_string())?;
        self.sync_list_length(list, next_count, receiver)?;
        for slot in &slots {
            self.builder
                .build_call(slot.rebuild, &[], "")
                .map_err(|e| e.to_string())?;
        }
        self.builder
            .build_unconditional_branch(done)
            .map_err(|e| e.to_string())?;
        self.builder.position_at_end(done);
        Ok(())
    }

    /// 行模板构造函数：`list_row_<id>(index)` 按运行期下标建出一行。
    fn build_list_row_fn(
        &self,
        id: &str,
        item: &IRNode,
        needs_target: bool,
        width: f64,
        row_height: f64,
    ) -> Result<FunctionValue<'ctx>, String> {
        let ptr_type = self.context.ptr_type(AddressSpace::default());
        let function = self.module.add_function(
            &format!("list_row_{id}"),
            ptr_type.fn_type(
                &[self.context.i64_type().into(), self.context.i64_type().into()],
                false,
            ),
            None,
        );
        let entry = self.context.append_basic_block(function, "entry");
        let saved = self.builder.get_insert_block();
        self.builder.position_at_end(entry);
        let index = function
            .get_nth_param(0)
            .ok_or("list row index parameter is missing")?
            .into_int_value();
        let depth = function
            .get_nth_param(1)
            .ok_or("list row depth parameter is missing")?
            .into_int_value();
        *self.list_row_ctx.borrow_mut() = Some(ListRowContext {
            id: id.to_string(),
            index: RowIndex::Runtime(index),
        });
        // 事件目标在本函数里重新取，不能引用构建期函数里的值。
        let event_target = if needs_target {
            Some(
                self.builder
                    .build_load(ptr_type, self.object_global("g_event_target").as_pointer_value(), "event_target")
                    .map_err(|e| e.to_string())?
                    .into_pointer_value(),
            )
        } else {
            None
        };
        let row_view =
            self.codegen_node(item, event_target, 0.0, 0.0, width, row_height, None, false)?;
        *self.list_row_ctx.borrow_mut() = None;
        self.add_dimension_constraint(row_view, "heightAnchor", row_height)?;
        // 行是 stack 的 arrangedSubview，排版不会看它自己的 margin；
        // 包一层 wrapper，把四边 margin 变成行相对 wrapper 的内缩。
        let margins = node_margins(item);
        let row_stride = row_height + margins.top + margins.bottom;
        let wrapper = self.create_objc_instance_with_frame(
            self.view_class(),
            self.rect_constant(0.0, 0.0, width, row_stride),
        )?;
        self.add_dimension_constraint(wrapper, "heightAnchor", row_stride)?;
        self.send_msg_void(wrapper, "addSubview:", &[row_view.into()])?;
        self.send_msg_void(
            row_view,
            "setTranslatesAutoresizingMaskIntoConstraints:",
            &[self.context.bool_type().const_int(0, false).into()],
        )?;
        self.add_anchor_constraint(row_view, "topAnchor", wrapper, "topAnchor", margins.top)?;
        self.add_anchor_constraint(
            row_view,
            "bottomAnchor",
            wrapper,
            "bottomAnchor",
            -margins.bottom,
        )?;
        let indent = self
            .builder
            .build_signed_int_to_float(depth, self.context.f64_type(), "row_depth")
            .map_err(|e| e.to_string())?;
        let indent = self
            .builder
            .build_float_mul(
                indent,
                self.context.f64_type().const_float(16.0),
                "row_indent",
            )
            .map_err(|e| e.to_string())?;
        let indent = self
            .builder
            .build_float_add(
                indent,
                self.context.f64_type().const_float(margins.left),
                "row_leading",
            )
            .map_err(|e| e.to_string())?;
        self.add_anchor_constraint_value(
            row_view,
            "leadingAnchor",
            wrapper,
            "leadingAnchor",
            indent,
        )?;
        self.add_anchor_constraint(
            row_view,
            "trailingAnchor",
            wrapper,
            "trailingAnchor",
            -margins.right,
        )?;
        self.builder
            .build_return(Some(&wrapper))
            .map_err(|e| e.to_string())?;
        if let Some(saved) = saved {
            self.builder.position_at_end(saved);
        }
        Ok(function)
    }

    /// 递归组件行：先按行体建视图，再把「父行 = 当前行」的子行递归挂到下面。
    /// 深度和宽度都来自运行期的行表，编译期只有一份模板。
    fn build_recursive_row_fn(
        &self,
        id: &str,
        body: &IRNode,
        needs_target: bool,
        width: f64,
        row_height: f64,
        row_stride: f64,
    ) -> Result<FunctionValue<'ctx>, String> {
        let ptr_type = self.context.ptr_type(AddressSpace::default());
        let i64_type = self.context.i64_type();
        let function = self.module.add_function(
            &format!("list_row_{id}"),
            ptr_type.fn_type(&[i64_type.into(), i64_type.into()], false),
            None,
        );
        let entry = self.context.append_basic_block(function, "entry");
        let saved = self.builder.get_insert_block();
        self.builder.position_at_end(entry);
        let index = function
            .get_nth_param(0)
            .ok_or("list row index parameter is missing")?
            .into_int_value();
        let depth = function
            .get_nth_param(1)
            .ok_or("list row depth parameter is missing")?
            .into_int_value();
        *self.list_row_ctx.borrow_mut() = Some(ListRowContext {
            id: id.to_string(),
            index: RowIndex::Runtime(index),
        });
        let event_target = if needs_target {
            Some(
                self.builder
                    .build_load(
                        ptr_type,
                        self.object_global("g_event_target").as_pointer_value(),
                        "event_target",
                    )
                    .map_err(|e| e.to_string())?
                    .into_pointer_value(),
            )
        } else {
            None
        };
        let body_view =
            self.codegen_node(body, event_target, 0.0, 0.0, width, row_height, None, false)?;
        *self.list_row_ctx.borrow_mut() = None;
        self.add_dimension_constraint(body_view, "heightAnchor", row_height)?;

        // 行体包装：margin 变内缩、按深度缩进，和普通行一致。
        let margins = node_margins(body);
        let body_slot = self.create_objc_instance_with_frame(
            self.view_class(),
            self.rect_constant(0.0, 0.0, width, row_stride),
        )?;
        self.add_dimension_constraint(body_slot, "heightAnchor", row_stride)?;
        self.send_msg_void(body_slot, "addSubview:", &[body_view.into()])?;
        self.send_msg_void(
            body_view,
            "setTranslatesAutoresizingMaskIntoConstraints:",
            &[self.context.bool_type().const_int(0, false).into()],
        )?;
        self.add_anchor_constraint(body_view, "topAnchor", body_slot, "topAnchor", margins.top)?;
        self.add_anchor_constraint(
            body_view,
            "bottomAnchor",
            body_slot,
            "bottomAnchor",
            -margins.bottom,
        )?;
        let indent = self
            .builder
            .build_signed_int_to_float(depth, self.context.f64_type(), "row_depth")
            .map_err(|e| e.to_string())?;
        let indent = self
            .builder
            .build_float_mul(
                indent,
                self.context.f64_type().const_float(16.0),
                "row_indent",
            )
            .map_err(|e| e.to_string())?;
        let indent = self
            .builder
            .build_float_add(
                indent,
                self.context.f64_type().const_float(margins.left),
                "row_leading",
            )
            .map_err(|e| e.to_string())?;
        self.add_anchor_constraint_value(
            body_view,
            "leadingAnchor",
            body_slot,
            "leadingAnchor",
            indent,
        )?;
        self.add_anchor_constraint(
            body_view,
            "trailingAnchor",
            body_slot,
            "trailingAnchor",
            -margins.right,
        )?;

        // 子树容器：行体在上，子行逐个挂在下面，整体高度由内容决定。
        let wrapper = self.create_objc_instance_with_frame(
            self.stack_class(),
            self.rect_constant(0.0, 0.0, width, row_stride),
        )?;
        self.configure_conditional_stack(
            wrapper,
            &InlineStyle::default(),
            FlexDirection::Column,
            0.0,
        )?;
        self.send_msg_void(
            body_slot,
            "setTranslatesAutoresizingMaskIntoConstraints:",
            &[self.context.bool_type().const_int(0, false).into()],
        )?;
        self.send_msg_void(wrapper, "addArrangedSubview:", &[body_slot.into()])?;

        let count = self
            .builder
            .build_load(
                i64_type,
                self.i64_global(&format!("g_list_{id}_count"))
                    .as_pointer_value(),
                "row_count",
            )
            .map_err(|e| e.to_string())?
            .into_int_value();
        let cursor = self
            .builder
            .build_alloca(i64_type, "child_cursor")
            .map_err(|e| e.to_string())?;
        self.builder
            .build_store(cursor, i64_type.const_zero())
            .map_err(|e| e.to_string())?;
        let loop_block = self.context.append_basic_block(function, "child_loop");
        let match_block = self.context.append_basic_block(function, "child_match");
        let build_block = self.context.append_basic_block(function, "child_build");
        let next_block = self.context.append_basic_block(function, "child_next");
        let done_block = self.context.append_basic_block(function, "child_done");
        self.builder
            .build_unconditional_branch(loop_block)
            .map_err(|e| e.to_string())?;

        self.builder.position_at_end(loop_block);
        let cursor_value = self
            .builder
            .build_load(i64_type, cursor, "child_index")
            .map_err(|e| e.to_string())?
            .into_int_value();
        let more = self
            .builder
            .build_int_compare(IntPredicate::SLT, cursor_value, count, "child_has_more")
            .map_err(|e| e.to_string())?;
        self.builder
            .build_conditional_branch(more, match_block, done_block)
            .map_err(|e| e.to_string())?;

        self.builder.position_at_end(match_block);
        let node_parent = self.load_list_number(id, "parent", cursor_value)?;
        let is_child = self
            .builder
            .build_int_compare(IntPredicate::EQ, node_parent, index, "child_is_direct")
            .map_err(|e| e.to_string())?;
        self.builder
            .build_conditional_branch(is_child, build_block, next_block)
            .map_err(|e| e.to_string())?;

        self.builder.position_at_end(build_block);
        let child_depth = self
            .builder
            .build_int_add(depth, i64_type.const_int(1, false), "child_depth")
            .map_err(|e| e.to_string())?;
        let child = match self
            .builder
            .build_call(function, &[cursor_value.into(), child_depth.into()], "child_row")
            .map_err(|e| e.to_string())?
            .try_as_basic_value()
        {
            inkwell::values::ValueKind::Basic(value) => value.into_pointer_value(),
            inkwell::values::ValueKind::Instruction(_) => {
                return Err("recursive row function returned void".into());
            }
        };
        self.send_msg_void(wrapper, "addArrangedSubview:", &[child.into()])?;
        self.builder
            .build_unconditional_branch(next_block)
            .map_err(|e| e.to_string())?;

        self.builder.position_at_end(next_block);
        let advanced = self
            .builder
            .build_int_add(cursor_value, i64_type.const_int(1, false), "child_next")
            .map_err(|e| e.to_string())?;
        self.builder
            .build_store(cursor, advanced)
            .map_err(|e| e.to_string())?;
        self.builder
            .build_unconditional_branch(loop_block)
            .map_err(|e| e.to_string())?;

        self.builder.position_at_end(done_block);
        self.builder
            .build_return(Some(&wrapper))
            .map_err(|e| e.to_string())?;
        if let Some(saved) = saved {
            self.builder.position_at_end(saved);
        }
        Ok(function)
    }

    /// 列表重建：摘掉现有行，再按当前行数逐行构建；行数不受编译期限制。
    fn build_list_rebuild(
        &self,
        id: &str,
        container_global: GlobalValue<'ctx>,
        count_global: GlobalValue<'ctx>,
        height_global: GlobalValue<'ctx>,
        row_fn: FunctionValue<'ctx>,
        row_height: f64,
        recursive: bool,
    ) -> Result<FunctionValue<'ctx>, String> {
        let ptr_type = self.context.ptr_type(AddressSpace::default());
        let function = self.module.add_function(
            &format!("list_rebuild_{id}"),
            self.context.void_type().fn_type(&[], false),
            None,
        );
        let entry = self.context.append_basic_block(function, "entry");
        let saved = self.builder.get_insert_block();
        self.builder.position_at_end(entry);
        let i64_type = self.context.i64_type();

        let container = self
            .builder
            .build_load(ptr_type, container_global.as_pointer_value(), "list_container")
            .map_err(|e| e.to_string())?
            .into_pointer_value();
        // 一次消息把所有旧行摘掉（含 stack 的 arrangement）
        let subviews = self
            .send_msg_ptr(container, self.get_selector("subviews")?, &[])?
            .ok_or("list subviews returned void")?;
        let remove = self.get_selector("removeFromSuperview")?;
        self.send_msg_void(subviews, "makeObjectsPerformSelector:", &[remove.into()])?;

        let count = self
            .builder
            .build_load(i64_type, count_global.as_pointer_value(), "list_count")
            .map_err(|e| e.to_string())?
            .into_int_value();
        let height = self
            .builder
            .build_signed_int_to_float(count, self.context.f64_type(), "list_height")
            .map_err(|e| e.to_string())?;
        let height = self
            .builder
            .build_float_mul(
                height,
                self.context.f64_type().const_float(row_height),
                "list_height_scaled",
            )
            .map_err(|e| e.to_string())?;
        let constraint = self
            .builder
            .build_load(
                ptr_type,
                height_global.as_pointer_value(),
                "list_height_constraint",
            )
            .map_err(|e| e.to_string())?
            .into_pointer_value();
        self.send_msg_void(constraint, "setConstant:", &[height.into()])?;

        // 按节点表递归建行：根层从 -1 开始，子行紧跟父行。
        // 递归组件的行函数自己会把子树挂进去，这里只从根层起手。
        let render = self.build_list_render(id, container_global, row_fn, recursive)?;
        self.builder
            .build_call(
                render,
                &[
                    i64_type.const_int(u64::MAX, true).into(),
                    i64_type.const_zero().into(),
                ],
                "",
            )
            .map_err(|e| e.to_string())?;
        self.builder.build_return(None).map_err(|e| e.to_string())?;
        if let Some(saved) = saved {
            self.builder.position_at_end(saved);
        }
        Ok(function)
    }

    /// 递归建行：`list_render(parent, depth)` 渲染 parent 的直接子行，
    /// 每建完一行递归渲染它自己的子行，顺序即 DFS。
    /// `recursive` 为真时行函数已经内嵌了子树，这里只渲染根层。
    fn build_list_render(
        &self,
        id: &str,
        container_global: GlobalValue<'ctx>,
        row_fn: FunctionValue<'ctx>,
        recursive: bool,
    ) -> Result<FunctionValue<'ctx>, String> {
        let ptr_type = self.context.ptr_type(AddressSpace::default());
        let i64_type = self.context.i64_type();
        let function = self.module.add_function(
            &format!("list_render_{id}"),
            self.context
                .void_type()
                .fn_type(&[i64_type.into(), i64_type.into()], false),
            None,
        );
        let entry = self.context.append_basic_block(function, "entry");
        let saved = self.builder.get_insert_block();
        self.builder.position_at_end(entry);
        let parent = function
            .get_nth_param(0)
            .ok_or("list render parent parameter is missing")?
            .into_int_value();
        let depth = function
            .get_nth_param(1)
            .ok_or("list render depth parameter is missing")?
            .into_int_value();
        let container = self
            .builder
            .build_load(ptr_type, container_global.as_pointer_value(), "list_container")
            .map_err(|e| e.to_string())?
            .into_pointer_value();
        let count = self
            .builder
            .build_load(
                i64_type,
                self.i64_global(&format!("g_list_{id}_count"))
                    .as_pointer_value(),
                "list_count",
            )
            .map_err(|e| e.to_string())?
            .into_int_value();
        let cursor = self
            .builder
            .build_alloca(i64_type, "render_cursor")
            .map_err(|e| e.to_string())?;
        self.builder
            .build_store(cursor, i64_type.const_zero())
            .map_err(|e| e.to_string())?;
        let loop_block = self.context.append_basic_block(function, "render_loop");
        let body = self.context.append_basic_block(function, "render_body");
        let child = self.context.append_basic_block(function, "render_child");
        let next = self.context.append_basic_block(function, "render_next");
        let done = self.context.append_basic_block(function, "render_done");
        self.builder
            .build_unconditional_branch(loop_block)
            .map_err(|e| e.to_string())?;

        self.builder.position_at_end(loop_block);
        let index = self
            .builder
            .build_load(i64_type, cursor, "render_index")
            .map_err(|e| e.to_string())?
            .into_int_value();
        let more = self
            .builder
            .build_int_compare(IntPredicate::SLT, index, count, "render_has_more")
            .map_err(|e| e.to_string())?;
        self.builder
            .build_conditional_branch(more, body, done)
            .map_err(|e| e.to_string())?;

        self.builder.position_at_end(body);
        let node_parent = self.load_list_number(id, "parent", index)?;
        let matched = self
            .builder
            .build_int_compare(IntPredicate::EQ, node_parent, parent, "render_is_child")
            .map_err(|e| e.to_string())?;
        self.builder
            .build_conditional_branch(matched, child, next)
            .map_err(|e| e.to_string())?;

        self.builder.position_at_end(child);
        let row = match self
            .builder
            .build_call(row_fn, &[index.into(), depth.into()], "list_row")
            .map_err(|e| e.to_string())?
            .try_as_basic_value()
        {
            inkwell::values::ValueKind::Basic(value) => value.into_pointer_value(),
            inkwell::values::ValueKind::Instruction(_) => {
                return Err("list row function returned void".into());
            }
        };
        self.send_msg_void(container, "addArrangedSubview:", &[row.into()])?;
        if !recursive {
            let child_depth = self
                .builder
                .build_int_add(depth, i64_type.const_int(1, false), "render_child_depth")
                .map_err(|e| e.to_string())?;
            self.builder
                .build_call(function, &[index.into(), child_depth.into()], "render_children")
                .map_err(|e| e.to_string())?;
        }
        self.builder
            .build_unconditional_branch(next)
            .map_err(|e| e.to_string())?;

        self.builder.position_at_end(next);
        let advanced = self
            .builder
            .build_int_add(index, i64_type.const_int(1, false), "render_index_next")
            .map_err(|e| e.to_string())?;
        self.builder
            .build_store(cursor, advanced)
            .map_err(|e| e.to_string())?;
        self.builder
            .build_unconditional_branch(loop_block)
            .map_err(|e| e.to_string())?;

        self.builder.position_at_end(done);
        self.builder.build_return(None).map_err(|e| e.to_string())?;
        if let Some(saved) = saved {
            self.builder.position_at_end(saved);
        }
        Ok(function)
    }

    /// 列表渲染节点：建容器与数据列，行由运行期重建函数按当前行数创建。
    fn codegen_list(
        &self,
        node: &IRNode,
        event_target: Option<PointerValue<'ctx>>,
        x: f64,
        y: f64,
        available_width: f64,
        available_height: f64,
        allocated: Option<ChildLayout>,
    ) -> Result<PointerValue<'ctx>, String> {
        let IRNode::List {
            style,
            list,
            item,
            fields,
            rows,
            parents,
        } = node
        else {
            return Err("codegen_list requires a List node".into());
        };
        let index = self.list_counter.get();
        self.list_counter.set(index + 1);
        let id = format!("{list}_{index}");
        let i64_type = self.context.i64_type();
        let ptr_type = self.context.ptr_type(AddressSpace::default());
        let width = allocated.map(|layout| layout.width).unwrap_or_else(|| {
            style
                .width
                .map(|dimension| dimension.resolve(available_width))
                .unwrap_or(available_width)
        });
        let row_height = node_height(item, available_height).max(1.0);
        let margins = node_margins(item);
        let row_stride = row_height + margins.top + margins.bottom;
        let height = row_stride * rows.len().max(1) as f64;
        let container = self.create_objc_instance_with_frame(
            self.stack_class(),
            self.rect_constant(x, y, width, height),
        )?;
        self.apply_inline_style(container, style, false)?;
        self.configure_conditional_stack(container, style, FlexDirection::Column, 0.0)?;

        let container_global = self.object_global(&format!("g_list_{id}_container"));
        self.builder
            .build_store(container_global.as_pointer_value(), container)
            .map_err(|e| e.to_string())?;
        let count_global = self.i64_global(&format!("g_list_{id}_count"));
        self.builder
            .build_store(
                count_global.as_pointer_value(),
                i64_type.const_int(rows.len() as u64, false),
            )
            .map_err(|e| e.to_string())?;
        let mut data_globals = HashMap::new();
        for (column, _) in fields {
            data_globals.insert(column.clone(), self.list_data_global(&id, column));
        }
        // 节点表：父下标 + 深度（根层 -1 / 0）
        let parent_global = self.list_number_column(&id, "parent");
        let depth_global = self.list_number_column(&id, "depth");
        for (index, parent) in parents.iter().enumerate() {
            let parent_value = match parent {
                Some(parent) => *parent as i64,
                None => -1,
            };
            self.store_list_number(
                &id,
                "parent",
                i64_type.const_int(index as u64, false),
                i64_type.const_int(parent_value as u64, true),
            )?;
            self.store_list_number(
                &id,
                "depth",
                i64_type.const_int(index as u64, false),
                i64_type.const_int(depth_of(parents, index) as u64, false),
            )?;
        }
        for (row, values) in rows.iter().enumerate() {
            let mut column_index = 0;
            for column in fields.iter().map(|(name, _)| name) {
                let text = self.create_nsstring(&values[column_index].display_value())?;
                self.store_list_value(&id, column, i64_type.const_int(row as u64, false), text)?;
                column_index += 1;
            }
        }

        // 递归组件：行模板根容器里带 `.children` 槽位时，行函数要连整棵子树一起建。
        let recursive = row_template_recurses(item);
        let pruned;
        let template: &IRNode = if recursive {
            pruned = strip_recursive_slots(item);
            &pruned
        } else {
            item
        };
        // 行构造函数在另一个函数里执行，事件目标在那里从全局重新取。
        let row_fn = if recursive {
            self.build_recursive_row_fn(
                &id,
                template,
                event_target.is_some(),
                width,
                row_height,
                row_stride,
            )?
        } else {
            self.build_list_row_fn(&id, template, event_target.is_some(), width, row_height)?
        };
        let height_global = self.object_global(&format!("g_list_{id}_height"));
        self.add_list_height_constraint(container, height, height_global)?;
        let rebuild = self.build_list_rebuild(
            &id,
            container_global,
            count_global,
            height_global,
            row_fn,
            row_stride,
            recursive,
        )?;
        self.list_slots
            .borrow_mut()
            .entry(list.clone())
            .or_default()
            .push(ListSlot {
                id,
                list: list.clone(),
                data_globals,
                container_global,
                count_global,
                height_global,
                parent_global,
                depth_global,
                rebuild,
            });
        self.builder
            .build_call(rebuild, &[], "")
            .map_err(|e| e.to_string())?;
        Ok(container)
    }

    /// 状态单元对应的存储全局变量名。
    fn cell_global_name(&self, cell: &str) -> Option<String> {
        self.state_cells
            .borrow()
            .iter()
            .position(|(path, _)| path == cell)
            .map(|index| format!("g_state_cell_{index}"))
    }

    /// 一个状态动作对应一个 Objective-C 方法：按源码顺序应用每个步骤，
    /// 标量单元只刷新引用它的标签和条件视图；列表动作整表刷新。
    fn build_state_callback_method(
        &self,
        mutation: &StateMutation,
        index: usize,
    ) -> Result<FunctionValue<'ctx>, String> {
        let ptr_type = self.context.ptr_type(AddressSpace::default());
        let callback_type = self
            .context
            .void_type()
            .fn_type(&[ptr_type.into(), ptr_type.into(), ptr_type.into()], false);
        let callback = self
            .module
            .add_function(&format!("app_delegate_state_action_{index}"), callback_type, None);
        let entry = self.context.append_basic_block(callback, "entry");
        let saved_block = self
            .builder
            .get_insert_block()
            .ok_or("missing insertion block while generating state callback")?;
        self.builder.position_at_end(entry);

        let receiver = callback
            .get_nth_param(0)
            .ok_or("state callback self parameter is missing")?
            .into_pointer_value();
        let sender = callback
            .get_nth_param(2)
            .ok_or("state callback sender parameter is missing")?
            .into_pointer_value();
        self.emit_state_mutation_body(mutation, index, receiver, sender)?;

        self.builder
            .build_return(None)
            .map_err(|e| e.to_string())?;
        self.builder.position_at_end(saved_block);
        Ok(callback)
    }

    fn emit_state_mutation_body(
        &self,
        mutation: &StateMutation,
        index: usize,
        receiver: PointerValue<'ctx>,
        sender: PointerValue<'ctx>,
    ) -> Result<(), String> {
        match mutation {
            StateMutation::Noop => Ok(()),
            StateMutation::Sequence(steps) => {
                for step in steps {
                    self.emit_state_mutation_body(step, index, receiver, sender)?;
                }
                Ok(())
            }
            StateMutation::PushItem { list, values } => self.emit_push_item(list, values, receiver),
            StateMutation::SetItemField { list, field, value } => {
                self.emit_set_item_field(list, field, value, sender)
            }
            StateMutation::RemoveCurrentItem { list } => {
                self.emit_remove_current_item(list, sender, receiver)
            }
            StateMutation::ClearList { list } => self.emit_clear_list(list, receiver),
            StateMutation::SetCells { values } => {
                for (cell, value) in values {
                    let text = self.create_nsstring(&value.display_value())?;
                    self.store_cell_text(cell, text, receiver)?;
                }
                Ok(())
            }
            scalar => self.emit_scalar_mutation(scalar, index, receiver, sender),
        }
    }

    fn emit_scalar_mutation(
        &self,
        mutation: &StateMutation,
        index: usize,
        receiver: PointerValue<'ctx>,
        sender: PointerValue<'ctx>,
    ) -> Result<(), String> {
        let ptr_type = self.context.ptr_type(AddressSpace::default());
        let Some(cell) = mutation.cell() else {
            return Ok(());
        };
        let state_global_name = self
            .cell_global_name(cell)
            .ok_or_else(|| format!("state cell '{cell}' was not collected"))?;
        let state_global = self
            .module
            .get_global(&state_global_name)
            .ok_or("state value global was not created")?;
        let state_ptr = state_global.as_pointer_value();

        let text = match mutation {
            StateMutation::Set(_, value) => self.create_nsstring(&value.display_value())?,
            StateMutation::SetTrue(_) => self.create_nsstring("true")?,
            // 滚动位置由 delegate 直接写单元，这个动作方法不会被控件调用
            StateMutation::SetFromScroll(_) | StateMutation::SetFromScrollClamped { .. } => {
                self.create_nsstring("0")?
            }
            StateMutation::SetFromInput(_) => {
                // UITextField 用 text，AppKit 的 NSTextField 用 stringValue
                let selector = if self.is_mac() { "stringValue" } else { "text" };
                let text = self
                    .send_msg_ptr(sender, self.get_selector(selector)?, &[])?
                    .ok_or("text field text returned void")?;
                // 控件返回的是 autoreleased 字符串，状态单元要自己持有一份
                self.copy_nsstring(text)?
            }
            StateMutation::Increment(_)
            | StateMutation::Decrement(_)
            | StateMutation::SetFromCell { .. } => {
                let (source_ptr, delta) = match mutation {
                    StateMutation::SetFromCell { source, delta, .. } => {
                        let name = self.cell_global_name(source).ok_or_else(|| {
                            format!("state cell '{source}' was not collected")
                        })?;
                        let global = self
                            .module
                            .get_global(&name)
                            .ok_or("state value global was not created")?;
                        (global.as_pointer_value(), *delta)
                    }
                    StateMutation::Increment(_) => (state_ptr, 1),
                    _ => (state_ptr, -1),
                };
                let current = self
                    .builder
                    .build_load(ptr_type, source_ptr, "state_current")
                    .map_err(|e| e.to_string())?
                    .into_pointer_value();
                let current = self
                    .send_msg_impl(
                        current,
                        self.get_selector("longLongValue")?,
                        &[],
                        BasicTypeEnum::IntType(self.context.i64_type()),
                    )?
                    .ok_or("NSString longLongValue returned void")?
                    .into_int_value();
                let next = self
                    .builder
                    .build_int_add(
                        current,
                        self.context.i64_type().const_int(delta as u64, true),
                        "state_next",
                    )
                    .map_err(|e| e.to_string())?;
                let buffer_type = self.context.i8_type().array_type(32);
                let buffer = self.module.add_global(
                    buffer_type,
                    None,
                    &format!("g_state_text_buffer_{index}"),
                );
                buffer.set_initializer(&buffer_type.const_zero());
                let format = self
                    .builder
                    .build_global_string_ptr("%lld", "state_format")
                    .map_err(|e| e.to_string())?;
                let snprintf_type = self.context.i32_type().fn_type(
                    &[
                        ptr_type.into(),
                        self.context.i64_type().into(),
                        ptr_type.into(),
                    ],
                    true,
                );
                let snprintf = self
                    .module
                    .get_function("snprintf")
                    .unwrap_or_else(|| self.module.add_function("snprintf", snprintf_type, None));
                self.builder
                    .build_call(
                        snprintf,
                        &[
                            buffer.as_pointer_value().into(),
                            self.context.i64_type().const_int(32, false).into(),
                            format.as_pointer_value().into(),
                            next.into(),
                        ],
                        "formatted_state",
                    )
                    .map_err(|e| e.to_string())?;
                self.create_nsstring_from_cstr(buffer.as_pointer_value())?
            }
            StateMutation::Noop
            | StateMutation::Sequence(_)
            | StateMutation::PushItem { .. }
            | StateMutation::SetItemField { .. }
            | StateMutation::RemoveCurrentItem { .. }
            | StateMutation::ClearList { .. }
            | StateMutation::SetCells { .. } => {
                return Err("lifecycle actions only support scalar state mutations".into());
            }
        };
        self.builder
            .build_store(state_ptr, text)
            .map_err(|e| e.to_string())?;
        self.update_cell_labels(cell, text)?;
        self.update_conditional_views(self.current_function()?, receiver, cell, text)?;
        Ok(())
    }


    fn build_scroll_delegate(
        &self,
        program: &UiProgram,
    ) -> Result<FunctionValue<'ctx>, String> {
        let ptr_type = self.context.ptr_type(AddressSpace::default());
        let callback_type = self.context.void_type().fn_type(
            &[ptr_type.into(), ptr_type.into(), ptr_type.into()],
            false,
        );
        let callback = self
            .module
            .add_function("app_delegate_scroll_view_did_scroll", callback_type, None);
        let entry = self.context.append_basic_block(callback, "entry");
        let saved_block = self
            .builder
            .get_insert_block()
            .ok_or("missing insertion block while generating scroll callback")?;
        self.builder.position_at_end(entry);
        let sender = callback
            .get_nth_param(2)
            .ok_or("scroll view callback parameter is missing")?
            .into_pointer_value();

        // onScroll={(y) => 单元 = y}：把滚动位置写进数值单元，再刷新引用它的文本
        if let Some((cell, bounds)) = scroll_action_cell(program) {
            let action_view_global = self
                .module
                .get_global("g_scroll_action_view")
                .ok_or("scroll action view global was not created")?;
            let action_view = self
                .builder
                .build_load(
                    ptr_type,
                    action_view_global.as_pointer_value(),
                    "action_scroll",
                )
                .map_err(|error| error.to_string())?
                .into_pointer_value();
            let is_target = self
                .builder
                .build_int_compare(
                    inkwell::IntPredicate::EQ,
                    sender,
                    action_view,
                    "is_action_scroll",
                )
                .map_err(|error| error.to_string())?;
            let write_block = self
                .context
                .append_basic_block(callback, "scroll_action_write");
            let after_block = self
                .context
                .append_basic_block(callback, "scroll_action_after");
            self.builder
                .build_conditional_branch(is_target, write_block, after_block)
                .map_err(|error| error.to_string())?;

            self.builder.position_at_end(write_block);
            let offset = self
                .send_msg_impl(
                    sender,
                    self.get_selector("contentOffset")?,
                    &[],
                    BasicTypeEnum::StructType(self.context.struct_type(
                        &[self.context.f64_type().into(), self.context.f64_type().into()],
                        false,
                    )),
                )?
                .ok_or("UIScrollView contentOffset returned void")?
                .into_struct_value();
            let offset_y = self
                .builder
                .build_extract_value(offset, 1, "action_offset_y")
                .map_err(|error| error.to_string())?
                .into_float_value();
            let inset = self
                .send_msg_impl(
                    sender,
                    self.get_selector("adjustedContentInset")?,
                    &[],
                    BasicTypeEnum::StructType(self.context.struct_type(
                        &[
                            self.context.f64_type().into(),
                            self.context.f64_type().into(),
                            self.context.f64_type().into(),
                            self.context.f64_type().into(),
                        ],
                        false,
                    )),
                )?
                .ok_or("UIScrollView adjustedContentInset returned void")?
                .into_struct_value();
            let inset_top = self
                .builder
                .build_extract_value(inset, 0, "action_inset_top")
                .map_err(|error| error.to_string())?
                .into_float_value();
            let normalized = self
                .builder
                .build_float_add(offset_y, inset_top, "action_scroll_offset")
                .map_err(|error| error.to_string())?;
            let integer = self
                .builder
                .build_float_to_signed_int(normalized, self.context.i64_type(), "scroll_int")
                .map_err(|error| error.to_string())?;
            // clamp(y, low, high)：写入前夹到区间内
            let integer = match bounds {
                Some((low, high, shift)) => {
                    let i64_type = self.context.i64_type();
                    // clamp(y - shift, low, high)
                    let integer = if shift == 0 {
                        integer
                    } else {
                        self.builder
                            .build_int_add(
                                integer,
                                i64_type.const_int((-shift) as u64, true),
                                "scroll_shifted",
                            )
                            .map_err(|error| error.to_string())?
                    };
                    let above_low = self
                        .builder
                        .build_int_compare(
                            inkwell::IntPredicate::SGT,
                            integer,
                            i64_type.const_int(low as u64, true),
                            "scroll_above_low",
                        )
                        .map_err(|error| error.to_string())?;
                    let lower = self
                        .builder
                        .build_select(
                            above_low,
                            integer,
                            i64_type.const_int(low as u64, true),
                            "scroll_lower",
                        )
                        .map_err(|error| error.to_string())?
                        .into_int_value();
                    let below_high = self
                        .builder
                        .build_int_compare(
                            inkwell::IntPredicate::SLT,
                            lower,
                            i64_type.const_int(high as u64, true),
                            "scroll_below_high",
                        )
                        .map_err(|error| error.to_string())?;
                    self.builder
                        .build_select(
                            below_high,
                            lower,
                            i64_type.const_int(high as u64, true),
                            "scroll_clamped",
                        )
                        .map_err(|error| error.to_string())?
                        .into_int_value()
                }
                None => integer,
            };
            self.store_cell_number(cell, integer, sender)?;
            self.builder
                .build_unconditional_branch(after_block)
                .map_err(|error| error.to_string())?;
            self.builder.position_at_end(after_block);
        }

        self.builder
            .build_return(None)
            .map_err(|error| error.to_string())?;
        self.builder.position_at_end(saved_block);
        Ok(callback)
    }

    fn load_operand(&self, operand: &ExprOperand) -> Result<IntValue<'ctx>, String> {
        match operand {
            ExprOperand::Number(value) => {
                Ok(self.context.i64_type().const_int(*value as u64, true))
            }
            ExprOperand::Cell(cell) => {
                let name = self
                    .cell_global_name(cell)
                    .ok_or_else(|| format!("state cell '{cell}' was not collected"))?;
                let global = self
                    .module
                    .get_global(&name)
                    .ok_or("state value global was not created")?;
                let ptr_type = self.context.ptr_type(AddressSpace::default());
                let text = self
                    .builder
                    .build_load(ptr_type, global.as_pointer_value(), "expr_cell")
                    .map_err(|e| e.to_string())?
                    .into_pointer_value();
                self.send_msg_impl(
                    text,
                    self.get_selector("longLongValue")?,
                    &[],
                    BasicTypeEnum::IntType(self.context.i64_type()),
                )?
                .ok_or_else(|| "NSString longLongValue returned void".to_string())
                .map(|value| value.into_int_value())
            }
        }
    }

    /// 依赖单元变化后重算表达式文本。
    fn update_expression_labels(&self, cell: &str) -> Result<(), String> {
        let labels = self.expr_labels.borrow().clone();
        let ptr_type = self.context.ptr_type(AddressSpace::default());
        for (index, expr) in labels.iter().enumerate() {
            if !expr.deps.iter().any(|dep| dep == cell) {
                continue;
            }
            let left = self.load_operand(&expr.left)?;
            let right = self.load_operand(&expr.right)?;
            let value = if expr.subtract {
                self.builder.build_int_sub(left, right, "expr_value")
            } else {
                self.builder.build_int_add(left, right, "expr_value")
            }
            .map_err(|e| e.to_string())?;
            let buffer_type = self.context.i8_type().array_type(32);
            let buffer_name = format!("g_expr_buffer_{index}");
            let buffer = match self.module.get_global(&buffer_name) {
                Some(global) => global,
                None => {
                    let global = self.module.add_global(buffer_type, None, &buffer_name);
                    global.set_initializer(&buffer_type.const_zero());
                    global
                }
            };
            let format = self
                .builder
                .build_global_string_ptr("%lld", "expr_format")
                .map_err(|e| e.to_string())?;
            let snprintf_type = self.context.i32_type().fn_type(
                &[
                    ptr_type.into(),
                    self.context.i64_type().into(),
                    ptr_type.into(),
                ],
                true,
            );
            let snprintf = self
                .module
                .get_function("snprintf")
                .unwrap_or_else(|| self.module.add_function("snprintf", snprintf_type, None));
            self.builder
                .build_call(
                    snprintf,
                    &[
                        buffer.as_pointer_value().into(),
                        self.context.i64_type().const_int(32, false).into(),
                        format.as_pointer_value().into(),
                        value.into(),
                    ],
                    "expr_text",
                )
                .map_err(|e| e.to_string())?;
            let text = self.create_nsstring_from_cstr(buffer.as_pointer_value())?;
            let label = self
                .builder
                .build_load(ptr_type, expr.label.as_pointer_value(), "expr_label")
                .map_err(|e| e.to_string())?
                .into_pointer_value();
            self.send_msg_void(label, "setText:", &[text.into()])?;
        }
        Ok(())
    }

    fn update_cell_labels(&self, cell: &str, value: PointerValue<'ctx>) -> Result<(), String> {
        let ptr_type = self.context.ptr_type(AddressSpace::default());
        let labels = self
            .state_label_globals
            .borrow()
            .get(cell)
            .cloned()
            .unwrap_or_default();
        for (index, label_global) in labels.iter().enumerate() {
            let label = self
                .builder
                .build_load(
                    ptr_type,
                    label_global.as_pointer_value(),
                    &format!("state_label_{index}"),
                )
                .map_err(|error| error.to_string())?
                .into_pointer_value();
            self.set_label_text(label, value)?;
        }
        // 引用这个单元的表达式文本也要跟着重算
        self.update_expression_labels(cell)?;
        // 绑定了透明度的视图：把单元里的整数读回来 /100，再 setAlpha
        let opacity_views = self
            .opacity_globals
            .borrow()
            .get(cell)
            .cloned()
            .unwrap_or_default();
        if !opacity_views.is_empty() {
            let number = self
                .send_msg_impl(
                    value,
                    self.get_selector("longLongValue")?,
                    &[],
                    BasicTypeEnum::IntType(self.context.i64_type()),
                )?
                .ok_or("NSString longLongValue returned void")?
                .into_int_value();
            let raw = self
                .builder
                .build_signed_int_to_float(number, self.context.f64_type(), "opacity_raw")
                .map_err(|e| e.to_string())?;
            let ratio = self
                .builder
                .build_float_div(
                    raw,
                    self.context.f64_type().const_float(100.0),
                    "opacity_ratio",
                )
                .map_err(|e| e.to_string())?;
            let zero = self.context.f64_type().const_float(0.0);
            let one = self.context.f64_type().const_float(1.0);
            let above_zero = self
                .builder
                .build_float_compare(FloatPredicate::OGT, ratio, zero, "opacity_positive")
                .map_err(|e| e.to_string())?;
            let lower = self
                .builder
                .build_select(above_zero, ratio, zero, "opacity_lower")
                .map_err(|e| e.to_string())?
                .into_float_value();
            let below_one = self
                .builder
                .build_float_compare(FloatPredicate::OLT, lower, one, "opacity_small")
                .map_err(|e| e.to_string())?;
            let alpha = self
                .builder
                .build_select(below_one, lower, one, "opacity_alpha")
                .map_err(|e| e.to_string())?
                .into_float_value();
            for (index, global) in opacity_views.iter().enumerate() {
                let view = self
                    .builder
                    .build_load(
                        ptr_type,
                        global.as_pointer_value(),
                        &format!("opacity_view_{index}"),
                    )
                    .map_err(|error| error.to_string())?
                    .into_pointer_value();
                self.send_view_alpha(view, alpha)?;
            }
        }
        Ok(())
    }

    fn emit_lifecycle_action(
        &self,
        receiver: PointerValue<'ctx>,
        action: Option<usize>,
    ) -> Result<(), String> {
        if let Some(action) = action {
            self.send_msg_void(
                receiver,
                &format!("stateAction{action}:"),
                &[receiver.into()],
            )?;
        }
        Ok(())
    }

    fn emit_lifecycle_actions(
        &self,
        receiver: PointerValue<'ctx>,
        actions: &[usize],
    ) -> Result<(), String> {
        for action in actions {
            self.emit_lifecycle_action(receiver, Some(*action))?;
        }
        Ok(())
    }

    fn emit_state_selected_lifecycle_action(
        &self,
        callback: FunctionValue<'ctx>,
        state: inkwell::values::IntValue<'ctx>,
        when_true: &[usize],
        when_false: &[usize],
        receiver: PointerValue<'ctx>,
        suffix: &str,
    ) -> Result<(), String> {
        if when_true.is_empty() && when_false.is_empty() {
            return Ok(());
        }
        let true_block = self.context.append_basic_block(
            callback,
            &format!("{suffix}_true"),
        );
        let false_block = self.context.append_basic_block(
            callback,
            &format!("{suffix}_false"),
        );
        let join_block = self.context.append_basic_block(
            callback,
            &format!("{suffix}_join"),
        );
        self.builder
            .build_conditional_branch(state, true_block, false_block)
            .map_err(|error| error.to_string())?;
        self.builder.position_at_end(true_block);
        self.emit_lifecycle_actions(receiver, when_true)?;
        self.builder
            .build_unconditional_branch(join_block)
            .map_err(|error| error.to_string())?;
        self.builder.position_at_end(false_block);
        self.emit_lifecycle_actions(receiver, when_false)?;
        self.builder
            .build_unconditional_branch(join_block)
            .map_err(|error| error.to_string())?;
        self.builder.position_at_end(join_block);
        Ok(())
    }

    fn update_conditional_views(
        &self,
        callback: FunctionValue<'ctx>,
        receiver: PointerValue<'ctx>,
        cell: &str,
        state_value: PointerValue<'ctx>,
    ) -> Result<(), String> {
        if !self
            .conditional_instances
            .borrow()
            .iter()
            .any(|runtime| runtime.cell == cell)
        {
            return Ok(());
        }
        let raw_state = self
            .send_msg_impl(
                state_value,
                self.get_selector("boolValue")?,
                &[],
                BasicTypeEnum::IntType(self.context.i8_type()),
            )?
            .ok_or("NSString boolValue returned void")?
            .into_int_value();
        let new_state = self
            .builder
            .build_int_compare(
                IntPredicate::NE,
                raw_state,
                self.context.i8_type().const_zero(),
                "conditional_new_state",
            )
            .map_err(|error| error.to_string())?;

        // 注意：这里必须用条件在 conditional_instances 里的**真实下标**，不能用过滤后的序号，
        // 因为 g_conditional_height_<n> 这些全局变量是按创建顺序编号的。
        for (index, runtime) in self
            .conditional_instances
            .borrow()
            .iter()
            .cloned()
            .enumerate()
        {
            if runtime.cell != cell {
                continue;
            }
            let active_raw = self
                .builder
                .build_load(
                    self.context.i8_type(),
                    runtime.active_state.as_pointer_value(),
                    &format!("conditional_old_state_{index}"),
                )
                .map_err(|error| error.to_string())?
                .into_int_value();
            let old_state = self
                .builder
                .build_int_compare(
                    IntPredicate::NE,
                    active_raw,
                    self.context.i8_type().const_zero(),
                    &format!("conditional_old_bool_{index}"),
                )
                .map_err(|error| error.to_string())?;
            let changed = self
                .builder
                .build_int_compare(
                    IntPredicate::NE,
                    old_state,
                    new_state,
                    &format!("conditional_changed_{index}"),
                )
                .map_err(|error| error.to_string())?;
            let update_block = self.context.append_basic_block(
                callback,
                &format!("conditional_update_{index}"),
            );
            let next_block = self.context.append_basic_block(
                callback,
                &format!("conditional_next_{index}"),
            );
            self.builder
                .build_conditional_branch(changed, update_block, next_block)
                .map_err(|error| error.to_string())?;
            self.builder.position_at_end(update_block);

            let ptr_type = self.context.ptr_type(AddressSpace::default());
            let load_view = |builder: &Builder<'ctx>, global: GlobalValue<'ctx>, name: &str| {
                builder
                    .build_load(ptr_type, global.as_pointer_value(), name)
                    .map(|value| value.into_pointer_value())
                    .map_err(|error| error.to_string())
            };
            let true_view = load_view(
                &self.builder,
                runtime.true_view,
                &format!("conditional_true_view_{index}"),
            )?;
            let false_view = load_view(
                &self.builder,
                runtime.false_view,
                &format!("conditional_false_view_{index}"),
            )?;
            let old_view = self
                .builder
                .build_select(old_state, true_view, false_view, "conditional_old_view")
                .map_err(|error| error.to_string())?
                .into_pointer_value();
            let new_view = self
                .builder
                .build_select(new_state, true_view, false_view, "conditional_new_view")
                .map_err(|error| error.to_string())?
                .into_pointer_value();
            let stack = load_view(
                &self.builder,
                runtime.slot,
                &format!("conditional_stack_{index}"),
            )?;
            // 卸载旧分支：视图和它的边约束一起离开层级
            self.send_msg_void(old_view, "removeFromSuperview", &[])?;

            let no_new_branch = self
                .builder
                .build_is_null(new_view, "conditional_no_new_branch")
                .map_err(|error| error.to_string())?;
            // 槽位高度：有分支显示时是编译期算出的高度，否则收缩为 0
            let height_global = self
                .module
                .get_global(&format!("g_conditional_height_{index}"))
                .ok_or("conditional height global was not created")?;
            let height_constraint = self
                .builder
                .build_load(ptr_type, height_global.as_pointer_value(), "conditional_height")
                .map_err(|error| error.to_string())?
                .into_pointer_value();
            let height_value = self
                .builder
                .build_select(
                    no_new_branch,
                    self.context.f64_type().const_float(0.0),
                    self.context.f64_type().const_float(runtime.slot_height),
                    "conditional_height_value",
                )
                .map_err(|error| error.to_string())?;
            self.send_msg_void(height_constraint, "setConstant:", &[height_value.into()])?;
            let insert_block = self.context.append_basic_block(
                callback,
                &format!("conditional_insert_{index}"),
            );
            let layout_block = self.context.append_basic_block(
                callback,
                &format!("conditional_layout_{index}"),
            );
            self.builder
                .build_conditional_branch(no_new_branch, layout_block, insert_block)
                .map_err(|error| error.to_string())?;
            self.builder.position_at_end(insert_block);
            // 挂载新分支并重建它的边约束（约束随旧视图一起被移除过）
            self.send_msg_void(stack, "addSubview:", &[new_view.into()])?;
            self.add_anchor_constraint(new_view, "leadingAnchor", stack, "leadingAnchor", 0.0)?;
            self.add_anchor_constraint(new_view, "topAnchor", stack, "topAnchor", 0.0)?;
            self.add_anchor_constraint(new_view, "trailingAnchor", stack, "trailingAnchor", 0.0)?;
            self.add_anchor_constraint(new_view, "bottomAnchor", stack, "bottomAnchor", 0.0)?;
            self.builder
                .build_unconditional_branch(layout_block)
                .map_err(|error| error.to_string())?;
            self.builder.position_at_end(layout_block);
            let layout_selector = if self.is_mac() { "layoutSubtreeIfNeeded" } else { "layoutIfNeeded" };
            self.send_msg_void(stack, layout_selector, &[])?;

            let stored_state = self
                .builder
                .build_int_z_extend(new_state, self.context.i8_type(), "conditional_store_state")
                .map_err(|error| error.to_string())?;
            self.builder
                .build_store(runtime.active_state.as_pointer_value(), stored_state)
                .map_err(|error| error.to_string())?;
            let old_true_unmount = runtime.true_unmount;
            let old_false_unmount = runtime.false_unmount;
            self.emit_state_selected_lifecycle_action(
                callback,
                old_state,
                &old_true_unmount,
                &old_false_unmount,
                receiver,
                &format!("conditional_unmount_{index}"),
            )?;
            self.emit_state_selected_lifecycle_action(
                callback,
                new_state,
                &runtime.true_mount,
                &runtime.false_mount,
                receiver,
                &format!("conditional_mount_{index}"),
            )?;

            self.builder
                .build_unconditional_branch(next_block)
                .map_err(|error| error.to_string())?;
            self.builder.position_at_end(next_block);
        }
        Ok(())
    }

    /// 生成显示窗口的 IR 逻辑
    /// macOS：NSWindow + contentView，根视图直接约束到内容视图四边。
    fn build_mac_window_logic(&self, root_node: Option<&IRNode>) -> Result<(), String> {
        let ptr_type = self.context.ptr_type(AddressSpace::default());
        let window_class = self.get_class("NSWindow")?;
        let alloc = self
            .send_msg_ptr(window_class, self.get_selector("alloc")?, &[])?
            .ok_or("NSWindow alloc returned void")?;
        let window = self
            .send_msg_ptr(
                alloc,
                self.get_selector("initWithContentRect:styleMask:backing:defer:")?,
                &[
                    self.rect_constant(0.0, 0.0, 420.0, 900.0).into(),
                    // titled | closable | miniaturizable | resizable
                    self.context.i64_type().const_int(15, false).into(),
                    // buffered
                    self.context.i64_type().const_int(2, false).into(),
                    self.context.bool_type().const_int(0, false).into(),
                ],
            )?
            .ok_or("NSWindow init returned void")?;
        if let Some(global) = self.module.get_global("g_window") {
            self.builder
                .build_store(global.as_pointer_value(), window)
                .map_err(|e| e.to_string())?;
        }
        let title = self.create_nsstring("Lumen")?;
        self.send_msg_void(window, "setTitle:", &[title.into()])?;

        let content_view = self
            .send_msg_ptr(window, self.get_selector("contentView")?, &[])?
            .ok_or("NSWindow contentView returned void")?;
        let root_view = if let Some(node) = root_node {
            let global_view = self
                .module
                .get_global("g_root_view")
                .ok_or("root view global was not created")?;
            let view = self
                .builder
                .build_load(ptr_type, global_view.as_pointer_value(), "mac_root_view")
                .map_err(|e| e.to_string())?
                .into_pointer_value();
            self.send_msg_void(content_view, "addSubview:", &[view.into()])?;
            self.send_msg_void(
                view,
                "setTranslatesAutoresizingMaskIntoConstraints:",
                &[self.context.bool_type().const_int(0, false).into()],
            )?;
            self.constrain_root_view(view, content_view, Some(node_style(node)))?;
            Some(view)
        } else {
            None
        };
        if let (Some(view), Some(node)) = (root_view, root_node) {
            if let Some(target) = find_top_background_view(std::slice::from_ref(node)) {
                self.install_top_background_extension(view, node_style(target))?;
            }
        }

        self.send_msg_void(window, "makeKeyAndOrderFront:", &[ptr_type.const_null().into()])?;
        let app = self
            .send_msg_ptr(
                self.get_class("NSApplication")?,
                self.get_selector("sharedApplication")?,
                &[],
            )?
            .ok_or("NSApplication sharedApplication returned void")?;
        self.send_msg_void(
            app,
            "activateIgnoringOtherApps:",
            &[self.context.bool_type().const_int(1, false).into()],
        )?;
        Ok(())
    }

    /// 运行时注册一个 isFlipped 返回 YES 的 NSView 子类，供滚动文档视图使用。
    fn register_flipped_view_class(&self) -> Result<(), String> {
        let ptr_type = self.context.ptr_type(AddressSpace::default());
        let allocate_type = ptr_type.fn_type(
            &[ptr_type.into(), ptr_type.into(), self.context.i64_type().into()],
            false,
        );
        let allocate = self
            .module
            .get_function("objc_allocateClassPair")
            .unwrap_or_else(|| self.module.add_function("objc_allocateClassPair", allocate_type, None));
        let add_method_type = self.context.bool_type().fn_type(
            &[ptr_type.into(), ptr_type.into(), ptr_type.into(), ptr_type.into()],
            false,
        );
        let add_method = self
            .module
            .get_function("class_addMethod")
            .unwrap_or_else(|| self.module.add_function("class_addMethod", add_method_type, None));
        let register_type = self.context.void_type().fn_type(&[ptr_type.into()], false);
        let register = self
            .module
            .get_function("objc_registerClassPair")
            .unwrap_or_else(|| self.module.add_function("objc_registerClassPair", register_type, None));

        let is_flipped_type = self
            .context
            .i8_type()
            .fn_type(&[ptr_type.into(), ptr_type.into()], false);
        let is_flipped = self
            .module
            .add_function("lumen_view_is_flipped", is_flipped_type, None);
        let saved = self.builder.get_insert_block().unwrap();
        let entry = self.context.append_basic_block(is_flipped, "entry");
        self.builder.position_at_end(entry);
        self.builder
            .build_return(Some(&self.context.i8_type().const_int(1, false)))
            .map_err(|e| e.to_string())?;
        self.builder.position_at_end(saved);

        let superclass = self.get_class("NSView")?;
        let name = self
            .builder
            .build_global_string_ptr("LumenFlippedView", "flipped_class_name")
            .map_err(|e| e.to_string())?;
        let class_call = self
            .builder
            .build_call(
                allocate,
                &[
                    superclass.into(),
                    name.as_pointer_value().into(),
                    self.context.i64_type().const_int(0, false).into(),
                ],
                "flipped_class",
            )
            .map_err(|e| e.to_string())?;
        let class = match class_call.try_as_basic_value() {
            inkwell::values::ValueKind::Basic(value) => value.into_pointer_value(),
            inkwell::values::ValueKind::Instruction(_) => {
                return Err("objc_allocateClassPair returned void".into());
            }
        };
        let selector = self.get_selector("isFlipped")?;
        let encoding = self
            .builder
            .build_global_string_ptr("c@:", "flipped_encoding")
            .map_err(|e| e.to_string())?;
        self.builder
            .build_call(
                add_method,
                &[
                    class.into(),
                    selector.into(),
                    is_flipped.as_global_value().as_pointer_value().into(),
                    encoding.as_pointer_value().into(),
                ],
                "",
            )
            .map_err(|e| e.to_string())?;
        self.builder
            .build_call(register, &[class.into()], "")
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    fn build_show_window_logic(&self, root_node: Option<&IRNode>) -> Result<(), String> {
        if self.is_mac() {
            return self.build_mac_window_logic(root_node);
        }
        let ptr_type = self.context.ptr_type(AddressSpace::default());

        // Use the physical screen bounds for both the window and its root view.
        
        let screen_bounds = self.screen_bounds()?;
        let window = self.create_objc_instance_with_frame("UIWindow", screen_bounds)?;
        let white = self.ui_color("white")?;
        self.send_msg_void(window, "setBackgroundColor:", &[white.into()])?;
        
        // Keep UIKit's controller-owned root view and pin the generated view inside it.
        let view_controller = self.create_objc_instance("UIViewController")?;
        let global_view = self.module.get_global("g_root_view").unwrap().as_pointer_value();
        let global_view_val = self.builder.build_load(ptr_type, global_view, "loaded_view").map_err(|e| e.to_string())?.into_pointer_value();
        let controller_view = self
            .send_msg_ptr(view_controller, self.get_selector("view")?, &[])?
            .ok_or("UIViewController view returned void")?;
        self.send_msg_void(
            controller_view,
            "setBackgroundColor:",
            &[white.into()],
        )?;
        self.send_msg_void(controller_view, "addSubview:", &[global_view_val.into()])?;
        self.send_msg_void(
            global_view_val,
            "setTranslatesAutoresizingMaskIntoConstraints:",
            &[self.context.bool_type().const_int(0, false).into()],
        )?;
        self.constrain_root_view(
            global_view_val,
            controller_view,
            root_node.map(node_style),
        )?;
        if let Some(target) =
            root_node.and_then(|root| find_top_background_view(std::slice::from_ref(root)))
        {
            self.install_top_background_extension(global_view_val, node_style(target))?;
        }
        // 设置 window 的 rootViewController
        self.send_msg_void(window, "setRootViewController:", &[view_controller.into()])?;

        // 调用 makeKeyAndVisible
        self.send_msg_void(window, "makeKeyAndVisible", &[])?;

        Ok(())
    }

    fn install_top_background_extension(
        &self,
        root_view: PointerValue<'ctx>,
        target_style: &InlineStyle,
    ) -> Result<(), String> {
        let target_global = self
            .module
            .get_global("g_top_background_view")
            .ok_or("top background target global was not created")?;
        let target = self
            .builder
            .build_load(
                self.context.ptr_type(AddressSpace::default()),
                target_global.as_pointer_value(),
                "top_background_target",
            )
            .map_err(|error| error.to_string())?
            .into_pointer_value();
        let background = self.create_objc_instance(self.view_class())?;
        let color = self.ui_color(
            target_style
                .background_color
                .as_deref()
                .ok_or("top background extension requires a background color")?,
        )?;
        self.send_msg_void(background, "setBackgroundColor:", &[color.into()])?;
        self.send_msg_void(
            background,
            "setUserInteractionEnabled:",
            &[self.context.bool_type().const_int(0, false).into()],
        )?;
        self.send_msg_void(
            background,
            "setTranslatesAutoresizingMaskIntoConstraints:",
            &[self.context.bool_type().const_int(0, false).into()],
        )?;
        self.send_msg_void(
            root_view,
            "insertSubview:atIndex:",
            &[
                background.into(),
                self.context.i64_type().const_zero().into(),
            ],
        )?;
        self.add_anchor_constraint(background, "topAnchor", root_view, "topAnchor", 0.0)?;
        self.add_anchor_constraint(
            background,
            "leadingAnchor",
            root_view,
            "leadingAnchor",
            0.0,
        )?;
        self.add_anchor_constraint(
            background,
            "trailingAnchor",
            root_view,
            "trailingAnchor",
            0.0,
        )?;
        self.add_anchor_constraint(background, "bottomAnchor", target, "topAnchor", 0.0)?;

        self.send_msg_void(
            background,
            "setAlpha:",
            &[self
                .context
                .f64_type()
                .const_float(1.0)
                .into()],
        )?;
        Ok(())
    }

    fn constrain_root_view(
        &self,
        view: PointerValue<'ctx>,
        parent: PointerValue<'ctx>,
        style: Option<&InlineStyle>,
    ) -> Result<(), String> {
        for (anchor_name, parent_anchor_name) in [
            ("leadingAnchor", "leadingAnchor"),
            ("topAnchor", "topAnchor"),
        ] {
            let view_anchor = self
                .send_msg_ptr(view, self.get_selector(anchor_name)?, &[])?
                .ok_or_else(|| format!("root view {anchor_name} returned void"))?;
            let parent_anchor = self
                .send_msg_ptr(parent, self.get_selector(parent_anchor_name)?, &[])?
                .ok_or_else(|| format!("controller view {parent_anchor_name} returned void"))?;
            let constraint = self
                .send_msg_ptr(
                    view_anchor,
                    self.get_selector("constraintEqualToAnchor:")?,
                    &[parent_anchor.into()],
                )?
                .ok_or_else(|| format!("root {anchor_name} constraint creation returned void"))?;
            self.activate_constraint(constraint)?;
        }

        for (anchor_name, parent_anchor_name, edge_anchor_name, dimension) in [
            (
                "widthAnchor",
                "widthAnchor",
                "trailingAnchor",
                style.and_then(|style| style.width),
            ),
            (
                "heightAnchor",
                "heightAnchor",
                "bottomAnchor",
                style.and_then(|style| style.height),
            ),
        ] {
            let fills_parent = match dimension {
                None => true,
                Some(Dimension::Percent(value)) if value == 1.0 => true,
                Some(_) => false,
            };
            if fills_parent {
                let view_anchor = self
                    .send_msg_ptr(view, self.get_selector(edge_anchor_name)?, &[])?
                    .ok_or_else(|| format!("root view {edge_anchor_name} returned void"))?;
                let parent_anchor = self
                    .send_msg_ptr(parent, self.get_selector(edge_anchor_name)?, &[])?
                    .ok_or_else(|| format!("controller view {edge_anchor_name} returned void"))?;
                let constraint = self
                    .send_msg_ptr(
                        view_anchor,
                        self.get_selector("constraintEqualToAnchor:")?,
                        &[parent_anchor.into()],
                    )?
                    .ok_or_else(|| format!("root {edge_anchor_name} constraint creation returned void"))?;
                self.activate_constraint(constraint)?;
                continue;
            }
            let view_anchor = self
                .send_msg_ptr(view, self.get_selector(anchor_name)?, &[])?
                .ok_or_else(|| format!("root view {anchor_name} returned void"))?;
            let constraint = match dimension {
                Some(Dimension::Points(value)) => self
                    .send_msg_ptr(
                        view_anchor,
                        self.get_selector("constraintEqualToConstant:")?,
                        &[self.context.f64_type().const_float(value).into()],
                    )?
                    .ok_or_else(|| format!("root {anchor_name} constant constraint returned void"))?,
                Some(Dimension::Percent(fraction)) => {
                    let parent_anchor = self
                        .send_msg_ptr(parent, self.get_selector(parent_anchor_name)?, &[])?
                        .ok_or_else(|| {
                            format!("controller view {parent_anchor_name} returned void")
                        })?;
                    self.send_msg_ptr(
                        view_anchor,
                        self.get_selector("constraintEqualToAnchor:multiplier:")?,
                        &[
                            parent_anchor.into(),
                            self.context.f64_type().const_float(fraction).into(),
                        ],
                    )?
                    .ok_or_else(|| {
                        format!("root {anchor_name} percentage constraint returned void")
                    })?
                }
                None => unreachable!("unspecified root dimensions are pinned to the parent edge"),
            };
            self.activate_constraint(constraint)?;
        }
        Ok(())
    }

    fn activate_constraint(&self, constraint: PointerValue<'ctx>) -> Result<(), String> {
        self.send_msg_void(
            constraint,
            "setActive:",
            &[self.context.bool_type().const_int(1, false).into()],
        )
    }

    fn constrain_child_centers(
        &self,
        parent: PointerValue<'ctx>,
        child: PointerValue<'ctx>,
        layout: ChildLayout,
        center_x: Option<f64>,
        center_y: Option<f64>,
    ) -> Result<(), String> {
        self.send_msg_void(
            child,
            "setTranslatesAutoresizingMaskIntoConstraints:",
            &[self.context.bool_type().const_int(0, false).into()],
        )?;

        for (anchor_name, size) in [
            ("widthAnchor", layout.width),
            ("heightAnchor", layout.height),
        ] {
            let anchor = self
                .send_msg_ptr(child, self.get_selector(anchor_name)?, &[])?
                .ok_or_else(|| format!("child {anchor_name} returned void"))?;
            let constraint = self
                .send_msg_ptr(
                    anchor,
                    self.get_selector("constraintEqualToConstant:")?,
                    &[self.context.f64_type().const_float(size).into()],
                )?
                .ok_or_else(|| format!("child {anchor_name} constraint returned void"))?;
            self.activate_constraint(constraint)?;
        }

        for (anchor_name, constant) in [
            ("centerXAnchor", center_x),
            ("centerYAnchor", center_y),
        ] {
            let Some(constant) = constant else {
                continue;
            };
            let child_anchor = self
                .send_msg_ptr(child, self.get_selector(anchor_name)?, &[])?
                .ok_or_else(|| format!("child {anchor_name} returned void"))?;
            let parent_anchor = self
                .send_msg_ptr(parent, self.get_selector(anchor_name)?, &[])?
                .ok_or_else(|| format!("parent {anchor_name} returned void"))?;
            let constraint = self
                .send_msg_ptr(
                    child_anchor,
                    self.get_selector("constraintEqualToAnchor:constant:")?,
                    &[
                        parent_anchor.into(),
                        self.context.f64_type().const_float(constant).into(),
                    ],
                )?
                .ok_or_else(|| format!("{anchor_name} constraint creation returned void"))?;
            self.activate_constraint(constraint)?;
        }
        Ok(())
    }

    fn add_anchor_constraint_value(
        &self,
        child: PointerValue<'ctx>,
        child_anchor_name: &str,
        parent: PointerValue<'ctx>,
        parent_anchor_name: &str,
        constant: inkwell::values::FloatValue<'ctx>,
    ) -> Result<(), String> {
        let child_anchor = self
            .send_msg_ptr(child, self.get_selector(child_anchor_name)?, &[])?
            .ok_or_else(|| format!("{child_anchor_name} returned void"))?;
        let parent_anchor = self
            .send_msg_ptr(parent, self.get_selector(parent_anchor_name)?, &[])?
            .ok_or_else(|| format!("{parent_anchor_name} returned void"))?;
        let constraint = self
            .send_msg_ptr(
                child_anchor,
                self.get_selector("constraintEqualToAnchor:constant:")?,
                &[parent_anchor.into(), constant.into()],
            )?
            .ok_or_else(|| format!("{child_anchor_name} constraint creation returned void"))?;
        self.activate_constraint(constraint)
    }

    fn add_anchor_constraint(
        &self,
        child: PointerValue<'ctx>,
        child_anchor_name: &str,
        parent: PointerValue<'ctx>,
        parent_anchor_name: &str,
        constant: f64,
    ) -> Result<(), String> {
        let child_anchor = self
            .send_msg_ptr(child, self.get_selector(child_anchor_name)?, &[])?
            .ok_or_else(|| format!("{child_anchor_name} returned void"))?;
        let parent_anchor = self
            .send_msg_ptr(parent, self.get_selector(parent_anchor_name)?, &[])?
            .ok_or_else(|| format!("{parent_anchor_name} returned void"))?;
        let constraint = self
            .send_msg_ptr(
                child_anchor,
                self.get_selector("constraintEqualToAnchor:constant:")?,
                &[
                    parent_anchor.into(),
                    self.context.f64_type().const_float(constant).into(),
                ],
            )?
            .ok_or_else(|| format!("{child_anchor_name} constraint creation returned void"))?;
        self.activate_constraint(constraint)
    }

    /// 父视图的安全区布局参考，编译期坐标以它为原点。
    fn safe_area_guide(&self, view: PointerValue<'ctx>) -> Result<PointerValue<'ctx>, String> {
        self.send_msg_ptr(view, self.get_selector("safeAreaLayoutGuide")?, &[])?
            .ok_or_else(|| "UIView safeAreaLayoutGuide returned void".into())
    }

    /// 竖向 flow 锚点。只有「布局盒就是安全区」的容器（根视图，或声明了
    /// `ignoresSafeArea: "top"` 的容器）才锚到父视图的安全区——它们盖住了状态栏，
    /// 内容必须让开。其余容器用自己的 bounds：底部这种贴着 home indicator 的容器
    /// 如果按安全区算，内容会被往上顶半个 bottom inset。
    fn vertical_flow_target(
        &self,
        parent: PointerValue<'ctx>,
        anchor_name: &'static str,
        safe_area_box: bool,
        child_style: &InlineStyle,
    ) -> Result<(PointerValue<'ctx>, &'static str), String> {
        let vertical = matches!(anchor_name, "topAnchor" | "bottomAnchor" | "centerYAnchor");
        let hugs_physical_top = anchor_name == "topAnchor" && child_style.ignores_safe_area_top;
        // macOS 的 safeAreaLayoutGuide 在视图入窗之前可能是 nil，窗口内容区本身也没有额外 inset
        if !vertical || !safe_area_box || hugs_physical_top || self.is_mac() {
            return Ok((parent, anchor_name));
        }
        Ok((self.safe_area_guide(parent)?, anchor_name))
    }

    fn add_dimension_constraint(
        &self,
        view: PointerValue<'ctx>,
        anchor_name: &str,
        size: f64,
    ) -> Result<(), String> {
        let anchor = self
            .send_msg_ptr(view, self.get_selector(anchor_name)?, &[])?
            .ok_or_else(|| format!("{anchor_name} returned void"))?;
        let constraint = self
            .send_msg_ptr(
                anchor,
                self.get_selector("constraintEqualToConstant:")?,
                &[self.context.f64_type().const_float(size).into()],
            )?
            .ok_or_else(|| format!("{anchor_name} constraint creation returned void"))?;
        self.activate_constraint(constraint)
    }

    fn add_percentage_dimension_constraint(
        &self,
        view: PointerValue<'ctx>,
        anchor_name: &str,
        parent: PointerValue<'ctx>,
        parent_anchor_name: &str,
        fraction: f64,
        padding: Padding,
    ) -> Result<(), String> {
        // 百分比按内容区算，先扣掉容器两侧的 padding
        let inset = if anchor_name == "widthAnchor" {
            padding.horizontal()
        } else {
            padding.vertical()
        };
        let view_anchor = self
            .send_msg_ptr(view, self.get_selector(anchor_name)?, &[])?
            .ok_or_else(|| format!("{anchor_name} returned void"))?;
        let parent_anchor = self
            .send_msg_ptr(parent, self.get_selector(parent_anchor_name)?, &[])?
            .ok_or_else(|| format!("{parent_anchor_name} returned void"))?;
        let constraint = self
            .send_msg_ptr(
                view_anchor,
                self.get_selector("constraintEqualToAnchor:multiplier:constant:")?,
                &[
                    parent_anchor.into(),
                    self.context.f64_type().const_float(fraction).into(),
                    self.context
                        .f64_type()
                        .const_float(-inset * fraction)
                        .into(),
                ],
            )?
            .ok_or_else(|| format!("{anchor_name} percentage constraint returned void"))?;
        self.activate_constraint(constraint)
    }

    fn add_flex_dimension_constraint(
        &self,
        child: PointerValue<'ctx>,
        dimension: Option<Dimension>,
        anchor_name: &str,
        parent: PointerValue<'ctx>,
        parent_anchor_name: &str,
        size: f64,
        padding: Padding,
    ) -> Result<(), String> {
        match dimension {
            Some(Dimension::Percent(fraction)) => self.add_percentage_dimension_constraint(
                child,
                anchor_name,
                parent,
                parent_anchor_name,
                fraction,
                padding,
            ),
            Some(Dimension::Points(_)) | None => {
                self.add_dimension_constraint(child, anchor_name, size)
            }
        }
    }

    fn constrain_scroll_child_width(
        &self,
        parent: PointerValue<'ctx>,
        child: PointerValue<'ctx>,
        child_node: &IRNode,
        layout: ChildLayout,
        padding: Padding,
    ) -> Result<(), String> {
        let Some(Dimension::Percent(fraction)) = node_style(child_node).width else {
            return Ok(());
        };
        self.send_msg_void(
            child,
            "setTranslatesAutoresizingMaskIntoConstraints:",
            &[self.context.bool_type().const_int(0, false).into()],
        )?;
        self.add_anchor_constraint(child, "leadingAnchor", parent, "leadingAnchor", layout.x)?;
        self.add_anchor_constraint(child, "topAnchor", parent, "topAnchor", layout.y)?;
        self.add_percentage_dimension_constraint(
            child,
            "widthAnchor",
            parent,
            "widthAnchor",
            fraction,
            padding,
        )?;
        if node_image_mode(child_node) == Some(ImageMode::WidthFix) {
            let image = self
                .send_msg_ptr(child, self.get_selector("image")?, &[])?
                .ok_or("UIImageView image returned void")?;
            self.add_image_aspect_constraint(child, image)?;
        } else if !node_uses_intrinsic_dimension(child_node, true) {
            self.add_dimension_constraint(child, "heightAnchor", layout.height)?;
        }
        Ok(())
    }

    fn add_image_aspect_constraint(
        &self,
        image_view: PointerValue<'ctx>,
        image: PointerValue<'ctx>,
    ) -> Result<(), String> {
        let f64_type = self.context.f64_type();
        let size_type = self.context.struct_type(&[f64_type.into(), f64_type.into()], false);
        let size = self
            .send_msg_impl(
                image,
                self.get_selector("size")?,
                &[],
                BasicTypeEnum::StructType(size_type),
            )?
            .ok_or("UIImage size returned void")?
            .into_struct_value();
        let image_width = self
            .builder
            .build_extract_value(size, 0, "image_width")
            .map_err(|error| error.to_string())?
            .into_float_value();
        let image_height = self
            .builder
            .build_extract_value(size, 1, "image_height")
            .map_err(|error| error.to_string())?
            .into_float_value();
        let aspect_ratio = self
            .builder
            .build_float_div(image_width, image_height, "image_aspect_ratio")
            .map_err(|error| error.to_string())?;
        let width_anchor = self
            .send_msg_ptr(image_view, self.get_selector("widthAnchor")?, &[])?
            .ok_or("UIImageView widthAnchor returned void")?;
        let height_anchor = self
            .send_msg_ptr(image_view, self.get_selector("heightAnchor")?, &[])?
            .ok_or("UIImageView heightAnchor returned void")?;
        let constraint = self
            .send_msg_ptr(
                width_anchor,
                self.get_selector("constraintEqualToAnchor:multiplier:")?,
                &[height_anchor.into(), aspect_ratio.into()],
            )?
            .ok_or("UIImageView aspect-ratio constraint returned void")?;
        self.activate_constraint(constraint)
    }

    fn constrain_positioned_child(
        &self,
        parent: PointerValue<'ctx>,
        child: PointerValue<'ctx>,
        child_node: &IRNode,
        layout: ChildLayout,
        padding: Padding,
        parent_width: f64,
        parent_height: f64,
        position: Position,
    ) -> Result<(), String> {
        let style = node_style(child_node);
        let margins = node_margins(child_node);
        let width = style
            .width
            .map(|value| value.resolve(parent_width))
            .unwrap_or(layout.width);
        let height = style
            .height
            .map(|value| value.resolve(parent_height))
            .unwrap_or(layout.height);
        let left = style.left.map(|value| value.resolve(parent_width));
        let right = style.right.map(|value| value.resolve(parent_width));
        let top = style.top.map(|value| value.resolve(parent_height));
        let bottom = style.bottom.map(|value| value.resolve(parent_height));

        self.send_msg_void(
            child,
            "setTranslatesAutoresizingMaskIntoConstraints:",
            &[self.context.bool_type().const_int(0, false).into()],
        )?;
        if (position != Position::Absolute || left.is_none() || right.is_none())
            && !node_uses_intrinsic_dimension(child_node, false)
        {
            self.add_dimension_constraint(child, "widthAnchor", width)?;
        }
        if (position != Position::Absolute || top.is_none() || bottom.is_none())
            && !node_uses_intrinsic_dimension(child_node, true)
        {
            self.add_dimension_constraint(child, "heightAnchor", height)?;
        }

        match if position == Position::Absolute {
            (left, right)
        } else {
            (None, None)
        } {
            (Some(left), Some(right)) => {
                self.add_anchor_constraint(
                    child,
                    "leadingAnchor",
                    parent,
                    "leadingAnchor",
                    padding.left + left + margins.left,
                )?;
                self.add_anchor_constraint(
                    child,
                    "trailingAnchor",
                    parent,
                    "trailingAnchor",
                    -padding.right - right - margins.right,
                )?;
            }
            (Some(left), None) => self.add_anchor_constraint(
                child,
                "leadingAnchor",
                parent,
                "leadingAnchor",
                padding.left + left + margins.left,
            )?,
            (None, Some(right)) => self.add_anchor_constraint(
                child,
                "trailingAnchor",
                parent,
                "trailingAnchor",
                -padding.right - right - margins.right,
            )?,
            (None, None) => self.add_anchor_constraint(
                child,
                "leadingAnchor",
                parent,
                "leadingAnchor",
                layout.x,
            )?,
        }
        match if position == Position::Absolute {
            (top, bottom)
        } else {
            (None, None)
        } {
            (Some(top), Some(bottom)) => {
                self.add_anchor_constraint(
                    child,
                    "topAnchor",
                    parent,
                    "topAnchor",
                    padding.top + top + margins.top,
                )?;
                self.add_anchor_constraint(
                    child,
                    "bottomAnchor",
                    parent,
                    "bottomAnchor",
                    -padding.bottom - bottom - margins.bottom,
                )?;
            }
            (Some(top), None) => self.add_anchor_constraint(
                child,
                "topAnchor",
                parent,
                "topAnchor",
                padding.top + top + margins.top,
            )?,
            (None, Some(bottom)) => self.add_anchor_constraint(
                child,
                "bottomAnchor",
                parent,
                "bottomAnchor",
                -padding.bottom - bottom - margins.bottom,
            )?,
            (None, None) => self.add_anchor_constraint(
                child,
                "topAnchor",
                parent,
                "topAnchor",
                layout.y,
            )?,
        }
        Ok(())
    }

    fn constrain_flex_children(
        &self,
        parent: PointerValue<'ctx>,
        children: &[PointerValue<'ctx>],
        child_nodes: &[IRNode],
        layouts: &[ChildLayout],
        style: &InlineStyle,
        default_direction: FlexDirection,
        default_gap: f64,
        width: f64,
        height: f64,
        safe_area_box: bool,
    ) -> Result<(), String> {
        let direction = style.flex_direction.unwrap_or(default_direction);
        let vertical_flow = direction == FlexDirection::Column;
        let padding = node_padding(style);
        // 主轴 / 交叉轴两端的盒内边距
        let (main_start_inset, main_end_inset, cross_start_inset, cross_end_inset) = match direction {
            FlexDirection::Row => (padding.left, padding.right, padding.top, padding.bottom),
            FlexDirection::Column => (padding.top, padding.bottom, padding.left, padding.right),
        };
        let gap = style.spacing.unwrap_or(default_gap);
        let (main_start, main_end, main_center, cross_start, cross_end, cross_center) =
            match direction {
                FlexDirection::Row => (
                    "leadingAnchor",
                    "trailingAnchor",
                    "centerXAnchor",
                    "topAnchor",
                    "bottomAnchor",
                    "centerYAnchor",
                ),
                FlexDirection::Column => (
                    "topAnchor",
                    "bottomAnchor",
                    "centerYAnchor",
                    "leadingAnchor",
                    "trailingAnchor",
                    "centerXAnchor",
                ),
            };
        let main_sizes: Vec<f64> = layouts
            .iter()
            .map(|layout| if direction == FlexDirection::Row { layout.width } else { layout.height })
            .collect();
        let flow_indices: Vec<usize> = child_nodes
            .iter()
            .enumerate()
            .filter_map(|(index, child)| {
                (node_style(child).position != Position::Absolute).then_some(index)
            })
            .collect();
        let main_margins_total: f64 = flow_indices
            .iter()
            .map(|&index| node_margins(&child_nodes[index]))
            .map(|margin| match direction {
                FlexDirection::Row => margin.left + margin.right,
                FlexDirection::Column => margin.top + margin.bottom,
            })
            .sum();
        let used_main = flow_indices.iter().map(|&index| main_sizes[index]).sum::<f64>()
            + main_margins_total
            + gap * flow_indices.len().saturating_sub(1) as f64;
        let main_extent = if direction == FlexDirection::Row {
            width - padding.horizontal()
        } else {
            height - padding.vertical()
        };
        let free_space = (main_extent - used_main).max(0.0);
        let justify = style.justify_content.unwrap_or(JustifyContent::Start);

        for (index, (child, child_node, layout)) in children
            .iter()
            .zip(child_nodes)
            .zip(layouts)
            .map(|((child, node), layout)| (child, node, layout))
            .enumerate()
        {
            if child.is_null() {
                continue;
            }
            if node_style(child_node).position == Position::Absolute
                || node_style(child_node).left.is_some()
                || node_style(child_node).right.is_some()
                || node_style(child_node).top.is_some()
                || node_style(child_node).bottom.is_some()
            {
                self.constrain_positioned_child(
                    parent,
                    *child,
                    child_node,
                    *layout,
                    padding,
                    width,
                    height,
                    node_style(child_node).position,
                )?;
                continue;
            }
            let flow_position = flow_indices
                .iter()
                .position(|&flow_index| flow_index == index)
                .expect("relative child belongs to flex flow");
            let margins = node_margins(child_node);
            let (flow_start_parent, flow_start_anchor_name) = if vertical_flow {
                self.vertical_flow_target(
                    parent,
                    main_start,
                    safe_area_box,
                    node_style(child_node),
                )?
            } else {
                (parent, main_start)
            };
            let (main_margin_start, main_margin_end, cross_margin_start, cross_margin_end) =
                match direction {
                    FlexDirection::Row => (margins.left, margins.right, margins.top, margins.bottom),
                    FlexDirection::Column => (margins.top, margins.bottom, margins.left, margins.right),
                };
            self.send_msg_void(
                *child,
                "setTranslatesAutoresizingMaskIntoConstraints:",
                &[self.context.bool_type().const_int(0, false).into()],
            )?;
            let image_mode = node_image_mode(child_node);
            let main_axis_auto = matches!(
                (direction, image_mode),
                (FlexDirection::Row, Some(ImageMode::HeightFix))
                    | (FlexDirection::Column, Some(ImageMode::WidthFix))
            );
            // 条件槽位和列表的高度都由运行期决定，父容器不再替它们加主轴尺寸约束。
            let self_sized = matches!(
                child_node,
                IRNode::Conditional { .. } | IRNode::List { .. }
            );
            if !main_axis_auto && !(self_sized && vertical_flow) {
                let (dimension, child_anchor, parent_anchor) = match direction {
                    FlexDirection::Row => (
                        node_style(child_node).width,
                        "widthAnchor",
                        "widthAnchor",
                    ),
                    FlexDirection::Column => (
                        node_style(child_node).height,
                        "heightAnchor",
                        "heightAnchor",
                    ),
                };
                let vertical = direction == FlexDirection::Column;
                if vertical && node_style(child_node).ignores_safe_area_top {
                    // 上边缘越过安全区，下边缘留在排版位置，高度由安全区高度撑开
                    let bottom_target = if safe_area_box && !self.is_mac() {
                        self.safe_area_guide(parent)?
                    } else {
                        parent
                    };
                    self.add_anchor_constraint(
                        *child,
                        main_end,
                        bottom_target,
                        "topAnchor",
                        layout.y + main_sizes[index],
                    )?;
                } else if vertical
                    && node_style(child_node).flex_grow > 0.0
                    && node_style(child_node).height.is_none()
                    && flow_position + 1 == flow_indices.len()
                {
                    // 容器高度只有运行时才知道，grow 的尾节点改撑满剩余空间
                    self.add_anchor_constraint(
                        *child,
                        main_end,
                        parent,
                        main_end,
                        -main_end_inset - main_margin_end,
                    )?;
                } else if !node_uses_intrinsic_dimension(child_node, vertical)
                    || (self_sized && !vertical)
                {
                    self.add_flex_dimension_constraint(
                        *child,
                        dimension,
                        child_anchor,
                        parent,
                        parent_anchor,
                        main_sizes[index],
                        padding,
                    )?;
                }
            }

            let explicit_cross = if direction == FlexDirection::Row {
                node_style(child_node).height
            } else {
                node_style(child_node).width
            };
            let cross_size = if direction == FlexDirection::Row { layout.height } else { layout.width };
            let align = style.align_items.unwrap_or(AlignItems::Stretch);
            let cross_axis_auto = matches!(
                (direction, image_mode),
                (FlexDirection::Row, Some(ImageMode::WidthFix))
                    | (FlexDirection::Column, Some(ImageMode::HeightFix))
            );
            let cross_is_stretched =
                align == AlignItems::Stretch && explicit_cross.is_none() && !cross_axis_auto;
            let (cross_start_target, cross_start_anchor) = self.vertical_flow_target(
                parent,
                cross_start,
                safe_area_box,
                node_style(child_node),
            )?;
            let (cross_end_target, cross_end_anchor) = self.vertical_flow_target(
                parent,
                cross_end,
                safe_area_box,
                node_style(child_node),
            )?;
            if cross_is_stretched {
                self.add_anchor_constraint(
                    *child,
                    cross_start,
                    cross_start_target,
                    cross_start_anchor,
                    cross_start_inset + cross_margin_start,
                )?;
                self.add_anchor_constraint(
                    *child,
                    cross_end,
                    cross_end_target,
                    cross_end_anchor,
                    -cross_end_inset - cross_margin_end,
                )?;
            } else {
                let vertical = direction == FlexDirection::Row;
                if !cross_axis_auto
                    && !node_uses_intrinsic_dimension(child_node, vertical)
                {
                    let (dimension, child_anchor, parent_anchor) = match direction {
                        FlexDirection::Row => (
                            node_style(child_node).height,
                            "heightAnchor",
                            "heightAnchor",
                        ),
                        FlexDirection::Column => (
                            node_style(child_node).width,
                            "widthAnchor",
                            "widthAnchor",
                        ),
                    };
                    self.add_flex_dimension_constraint(
                        *child,
                        dimension,
                        child_anchor,
                        parent,
                        parent_anchor,
                        cross_size,
                        padding,
                    )?;
                }
                match align {
                    AlignItems::Center => {
                        let (position, container_size, child_size) = match direction {
                            FlexDirection::Row => (layout.y, height, layout.height),
                            FlexDirection::Column => (layout.x, width, layout.width),
                        };
                        let (center_target, center_anchor) = self.vertical_flow_target(
                            parent,
                            cross_center,
                            safe_area_box,
                            node_style(child_node),
                        )?;
                        self.add_anchor_constraint(
                            *child,
                            cross_center,
                            center_target,
                            center_anchor,
                            position + child_size / 2.0 - container_size / 2.0,
                        )?;
                    }
                    AlignItems::End => self.add_anchor_constraint(
                        *child,
                        cross_end,
                        cross_end_target,
                        cross_end_anchor,
                        -cross_end_inset - cross_margin_end,
                    )?,
                    AlignItems::Start | AlignItems::Stretch => self.add_anchor_constraint(
                        *child,
                        cross_start,
                        cross_start_target,
                        cross_start_anchor,
                        cross_start_inset + cross_margin_start,
                    )?,
                }
            }
            if matches!(image_mode, Some(ImageMode::WidthFix | ImageMode::HeightFix)) {
                let image = match child_node {
                    IRNode::Image { source: _, mode: _, style: _ } => self
                        .send_msg_ptr(*child, self.get_selector("image")?, &[])?
                        .ok_or("UIImageView image returned void")?,
                    _ => unreachable!("only image nodes have image sizing modes"),
                };
                self.add_image_aspect_constraint(*child, image)?;
            }

            match justify {
                JustifyContent::Start => {
                    if flow_position == 0 {
                        self.add_anchor_constraint(
                            *child,
                            main_start,
                            flow_start_parent,
                            flow_start_anchor_name,
                            main_start_inset + main_margin_start,
                        )?;
                    } else {
                        let previous_index = flow_indices[flow_position - 1];
                        let previous_margins = node_margins(&child_nodes[previous_index]);
                        let previous_end_margin = match direction {
                            FlexDirection::Row => previous_margins.right,
                            FlexDirection::Column => previous_margins.bottom,
                        };
                        self.add_anchor_constraint(
                            *child,
                            main_start,
                            children[previous_index],
                            main_end,
                            previous_end_margin + gap + main_margin_start,
                        )?;
                    }
                }
                JustifyContent::End => {
                    if flow_position + 1 == flow_indices.len() {
                        self.add_anchor_constraint(
                            *child,
                            main_end,
                            parent,
                            main_end,
                            -main_end_inset - main_margin_end,
                        )?;
                    } else {
                        let next_index = flow_indices[flow_position + 1];
                        let next_margins = node_margins(&child_nodes[next_index]);
                        let next_start_margin = match direction {
                            FlexDirection::Row => next_margins.left,
                            FlexDirection::Column => next_margins.top,
                        };
                        self.add_anchor_constraint(
                            *child,
                            main_end,
                            children[next_index],
                            main_start,
                            -gap - next_start_margin - main_margin_end,
                        )?;
                    }
                }
                JustifyContent::Center => {
                    if flow_position == 0 {
                        let offset =
                            -used_main / 2.0 + main_margin_start + main_sizes[index] / 2.0;
                        self.add_anchor_constraint(*child, main_center, parent, main_center, offset)?;
                    } else {
                        let previous_index = flow_indices[flow_position - 1];
                        let previous_margins = node_margins(&child_nodes[previous_index]);
                        let previous_end_margin = match direction {
                            FlexDirection::Row => previous_margins.right,
                            FlexDirection::Column => previous_margins.bottom,
                        };
                        self.add_anchor_constraint(
                            *child,
                            main_start,
                            children[previous_index],
                            main_end,
                            previous_end_margin + gap + main_margin_start,
                        )?;
                    }
                }
                JustifyContent::SpaceBetween => {
                    if flow_position == 0 {
                        self.add_anchor_constraint(
                            *child,
                            main_start,
                            flow_start_parent,
                            flow_start_anchor_name,
                            main_start_inset + main_margin_start,
                        )?;
                    } else if flow_position + 1 == flow_indices.len() {
                        self.add_anchor_constraint(
                            *child,
                            main_end,
                            parent,
                            main_end,
                            -main_end_inset - main_margin_end,
                        )?;
                    } else {
                        let distributed_gap = gap + free_space / flow_indices.len().saturating_sub(1).max(1) as f64;
                        let previous_index = flow_indices[flow_position - 1];
                        let previous_margins = node_margins(&child_nodes[previous_index]);
                        let previous_end_margin = match direction {
                            FlexDirection::Row => previous_margins.right,
                            FlexDirection::Column => previous_margins.bottom,
                        };
                        self.add_anchor_constraint(
                            *child,
                            main_start,
                            children[previous_index],
                            main_end,
                            previous_end_margin + distributed_gap + main_margin_start,
                        )?;
                    }
                }
                JustifyContent::SpaceAround => {
                    let (position, container_size, child_size) = match direction {
                        FlexDirection::Row => (layout.x, width, layout.width),
                        FlexDirection::Column => (layout.y, height, layout.height),
                    };
                    let center_offset = position + child_size / 2.0 - container_size / 2.0;
                    self.add_anchor_constraint(*child, main_center, parent, main_center, center_offset)?;
                }
            }
        }
        Ok(())
    }

    fn configure_conditional_stack(
        &self,
        stack: PointerValue<'ctx>,
        style: &InlineStyle,
        direction: FlexDirection,
        default_gap: f64,
    ) -> Result<PointerValue<'ctx>, String> {
        if self.is_mac() {
            // NSStackView 用 orientation/alignment/spacing 表达；alignment 取 NSLayoutAttribute
            let vertical = direction == FlexDirection::Column;
            self.send_msg_void(
                stack,
                "setOrientation:",
                &[self.context.i64_type().const_int(u64::from(vertical), false).into()],
            )?;
            let alignment = match (vertical, style.align_items.unwrap_or(AlignItems::Stretch)) {
                // 竖向栈：leading / centerX / trailing / width
                (true, AlignItems::Start) => 5,
                (true, AlignItems::Center) => 9,
                (true, AlignItems::End) => 6,
                (true, AlignItems::Stretch) => 7,
                // 横向栈：top / centerY / bottom / height
                (false, AlignItems::Start) => 3,
                (false, AlignItems::Center) => 10,
                (false, AlignItems::End) => 4,
                (false, AlignItems::Stretch) => 8,
            };
            self.send_msg_void(
                stack,
                "setAlignment:",
                &[self.context.i64_type().const_int(alignment, false).into()],
            )?;
            self.send_msg_void(
                stack,
                "setSpacing:",
                &[self
                    .context
                    .f64_type()
                    .const_float(style.spacing.unwrap_or(default_gap))
                    .into()],
            )?;
            return Ok(stack);
        }
        self.send_msg_void(
            stack,
            "setAxis:",
            &[self.context.i64_type().const_int(
                u64::from(direction == FlexDirection::Column),
                false,
            ).into()],
        )?;
        self.send_msg_void(
            stack,
            "setSpacing:",
            &[self.context.f64_type().const_float(style.spacing.unwrap_or(default_gap)).into()],
        )?;
        let alignment = match (direction, style.align_items.unwrap_or(AlignItems::Stretch)) {
            (_, AlignItems::Stretch) => 0,
            (FlexDirection::Column, AlignItems::Start) => 1,
            (FlexDirection::Column, AlignItems::Center) => 3,
            (FlexDirection::Column, AlignItems::End) => 4,
            (FlexDirection::Row, AlignItems::Start) => 6,
            (FlexDirection::Row, AlignItems::Center) => 3,
            (FlexDirection::Row, AlignItems::End) => 7,
        };
        self.send_msg_void(
            stack,
            "setAlignment:",
            &[self.context.i64_type().const_int(alignment, false).into()],
        )?;
        let distribution = match style.justify_content.unwrap_or(JustifyContent::Start) {
            JustifyContent::SpaceBetween | JustifyContent::SpaceAround => 3,
            _ => 0,
        };
        self.send_msg_void(
            stack,
            "setDistribution:",
            &[self.context.i64_type().const_int(distribution, false).into()],
        )?;
        self.send_msg_void(
            stack,
            "setLayoutMarginsRelativeArrangement:",
            &[self.context.i8_type().const_int(1, false).into()],
        )?;
        let padding = node_padding(style);
        let insets = self.ui_edge_insets(
            padding.top,
            padding.left,
            padding.bottom,
            padding.right,
        );
        self.send_msg_void(stack, "setLayoutMargins:", &[insets.into()])?;
        Ok(stack)
    }

    fn codegen_dynamic_children(
        &self,
        parent: PointerValue<'ctx>,
        children: &[IRNode],
        style: &InlineStyle,
        width: f64,
        height: f64,
        default_direction: FlexDirection,
        default_gap: f64,
        event_target: Option<PointerValue<'ctx>>,
        safe_area_box: bool,
        size_to_content: bool,
    ) -> Result<(), String> {
        // 含条件子级的容器同样是普通视图：位置和尺寸全部由约束决定
        let padding = node_padding(style);
        let layouts = layout_flex_children(
            children,
            style,
            width,
            height,
            default_direction,
            default_gap,
        );
        let mut child_views: Vec<PointerValue<'ctx>> = Vec::with_capacity(children.len());
        for (child, layout) in children.iter().zip(&layouts) {
            let child_view = self.codegen_node(
                child,
                event_target,
                0.0,
                0.0,
                (width - padding.horizontal()).max(0.0),
                (height - padding.vertical()).max(0.0),
                Some(*layout),
                false,
            )?;
            if !child_view.is_null() {
                self.send_msg_void(parent, "addSubview:", &[child_view.into()])?;
            }
            child_views.push(child_view);
        }
        self.constrain_flex_children(
            parent,
            &child_views,
            children,
            &layouts,
            style,
            default_direction,
            default_gap,
            width,
            height,
            safe_area_box,
        )?;
        if size_to_content {
            // 滚动内容容器没有固定高度，用最后一个子视图的底边撑出内容尺寸
            let last_view = children
                .iter()
                .zip(&child_views)
                .filter(|(child, view)| {
                    node_style(child).position != Position::Absolute && !view.is_null()
                })
                .map(|(_, view)| *view)
                .next_back();
            if let Some(last_view) = last_view {
                self.add_anchor_constraint(
                    parent,
                    "bottomAnchor",
                    last_view,
                    "bottomAnchor",
                    padding.bottom,
                )?;
            }
        }
        Ok(())
    }

    /// macOS 的滚动容器：NSScrollView + 一个 isFlipped 的文档视图，
    /// 内容按约束排进去，高度由最后一个子视图决定。
    fn codegen_mac_scroll(
        &self,
        node: &IRNode,
        style: &InlineStyle,
        children: &[IRNode],
        event_target: Option<PointerValue<'ctx>>,
        x: f64,
        y: f64,
        available_width: f64,
        available_height: f64,
        allocated: Option<ChildLayout>,
    ) -> Result<PointerValue<'ctx>, String> {
        let width = allocated
            .map(|layout| layout.width)
            .unwrap_or_else(|| {
                style
                    .width
                    .map(|dimension| dimension.resolve(available_width))
                    .unwrap_or_else(|| node_width(node, available_width))
            });
        let height = allocated
            .map(|layout| layout.height)
            .unwrap_or_else(|| node_height(node, available_height));
        let scroll_view =
            self.create_objc_instance_with_frame("NSScrollView", self.rect_constant(x, y, width, height))?;
        self.apply_inline_style(scroll_view, style, false)?;
        self.send_msg_void(scroll_view, "setHasVerticalScroller:", &[self.context.i8_type().const_int(1, false).into()])?;
        self.send_msg_void(scroll_view, "setAutohidesScrollers:", &[self.context.i8_type().const_int(1, false).into()])?;
        // 卡片背景画在滚动视图自己身上，并让圆角裁剪内部 clip view
        self.send_msg_void(scroll_view, "setDrawsBackground:", &[self.context.i8_type().const_int(1, false).into()])?;
        if let Some(color_name) = &style.background_color {
            let color = self.ui_color(color_name)?;
            self.send_msg_void(scroll_view, "setBackgroundColor:", &[color.into()])?;
        }
        if style.border_radius.is_some() {
            self.wants_layer(scroll_view)?;
            let layer = self
                .send_msg_ptr(scroll_view, self.get_selector("layer")?, &[])?
                .ok_or("NSScrollView layer returned void")?;
            self.send_msg_void(layer, "setMasksToBounds:", &[self.context.i8_type().const_int(1, false).into()])?;
        }

        let document = self.create_objc_instance_with_frame(
            "LumenFlippedView",
            self.rect_constant(0.0, 0.0, width, height),
        )?;
        self.send_msg_void(
            document,
            "setTranslatesAutoresizingMaskIntoConstraints:",
            &[self.context.i8_type().const_int(0, false).into()],
        )?;
        self.send_msg_void(scroll_view, "setDocumentView:", &[document.into()])?;
        let clip_view = self
            .send_msg_ptr(scroll_view, self.get_selector("contentView")?, &[])?
            .ok_or("NSScrollView contentView returned void")?;
        self.add_anchor_constraint(document, "leadingAnchor", clip_view, "leadingAnchor", 0.0)?;
        self.add_anchor_constraint(document, "trailingAnchor", clip_view, "trailingAnchor", 0.0)?;
        self.add_anchor_constraint(document, "topAnchor", clip_view, "topAnchor", 0.0)?;

        let padding = node_padding(style);
        let layouts = layout_flex_children(
            children,
            style,
            width,
            height,
            FlexDirection::Column,
            0.0,
        );
        let mut child_views: Vec<PointerValue<'ctx>> = Vec::with_capacity(children.len());
        for (child, layout) in children.iter().zip(&layouts) {
            let child_view = self.codegen_node(
                child,
                event_target,
                0.0,
                0.0,
                (width - padding.horizontal()).max(0.0),
                (height - padding.vertical()).max(0.0),
                Some(*layout),
                false,
            )?;
            if !child_view.is_null() {
                self.send_msg_void(document, "addSubview:", &[child_view.into()])?;
            }
            child_views.push(child_view);
        }
        self.constrain_flex_children(
            document,
            &child_views,
            children,
            &layouts,
            style,
            FlexDirection::Column,
            0.0,
            width,
            height,
            false,
        )?;
        let last_view = children
            .iter()
            .zip(&child_views)
            .filter(|(child, view)| {
                node_style(child).position != Position::Absolute && !view.is_null()
            })
            .map(|(_, view)| *view)
            .next_back();
        if let Some(last_view) = last_view {
            self.add_anchor_constraint(
                document,
                "bottomAnchor",
                last_view,
                "bottomAnchor",
                padding.bottom,
            )?;
        }
        Ok(scroll_view)
    }

    fn codegen_dynamic_scroll_content(
        &self,
        scroll_view: PointerValue<'ctx>,
        children: &[IRNode],
        style: &InlineStyle,
        width: f64,
        height: f64,
        event_target: Option<PointerValue<'ctx>>,
    ) -> Result<(), String> {
        let frame = self.rect_constant(0.0, 0.0, width, height);
        let content_stack = self.create_objc_instance_with_frame(self.view_class(), frame)?;
        let content_style = style.clone();
        self.send_msg_void(
            content_stack,
            "setTranslatesAutoresizingMaskIntoConstraints:",
            &[self.context.i8_type().const_int(0, false).into()],
        )?;
        self.send_msg_void(scroll_view, "addSubview:", &[content_stack.into()])?;

        let content_guide = self
            .send_msg_ptr(scroll_view, self.get_selector("contentLayoutGuide")?, &[])?
            .ok_or("UIScrollView contentLayoutGuide returned void")?;
        let frame_guide = self
            .send_msg_ptr(scroll_view, self.get_selector("frameLayoutGuide")?, &[])?
            .ok_or("UIScrollView frameLayoutGuide returned void")?;
        self.add_anchor_constraint(
            content_stack,
            "leadingAnchor",
            content_guide,
            "leadingAnchor",
            0.0,
        )?;
        self.add_anchor_constraint(
            content_stack,
            "trailingAnchor",
            content_guide,
            "trailingAnchor",
            0.0,
        )?;
        self.add_anchor_constraint(
            content_stack,
            "topAnchor",
            content_guide,
            "topAnchor",
            0.0,
        )?;
        self.add_anchor_constraint(
            content_stack,
            "bottomAnchor",
            content_guide,
            "bottomAnchor",
            0.0,
        )?;
        self.add_anchor_constraint(
            content_stack,
            "widthAnchor",
            frame_guide,
            "widthAnchor",
            0.0,
        )?;
        self.codegen_dynamic_children(
            content_stack,
            children,
            &content_style,
            width,
            height,
            FlexDirection::Column,
            0.0,
            event_target,
            false,
            true,
        )
    }

    fn constrain_dynamic_child(
        &self,
        stack: PointerValue<'ctx>,
        child_view: PointerValue<'ctx>,
        child: &IRNode,
        layout: ChildLayout,
        padding: Padding,
        direction: FlexDirection,
    ) -> Result<(), String> {
        let _ = direction;
        let style = node_style(child);
        let container = matches!(
            child,
            IRNode::View { .. } | IRNode::VStack { .. } | IRNode::ScrollView { .. }
        );
        let dynamic_container = node_has_conditional(child);
        match style.width {
            Some(Dimension::Points(width)) => {
                self.add_dimension_constraint(child_view, "widthAnchor", width)?;
            }
            Some(Dimension::Percent(fraction)) => {
                self.add_percentage_dimension_constraint(
                    child_view,
                    "widthAnchor",
                    stack,
                    "widthAnchor",
                    fraction,
                    padding,
                )?;
            }
            None if container && !dynamic_container => {
                self.add_dimension_constraint(
                    child_view,
                    "widthAnchor",
                    node_width(child, layout.width),
                )?;
            }
            None => {}
        }
        match style.height {
            Some(Dimension::Points(height)) => {
                self.add_dimension_constraint(child_view, "heightAnchor", height)?;
            }
            Some(Dimension::Percent(fraction)) => {
                self.add_percentage_dimension_constraint(
                    child_view,
                    "heightAnchor",
                    stack,
                    "heightAnchor",
                    fraction,
                    padding,
                )?;
            }
            None if container
                && !dynamic_container
                && !(style.flex_grow > 0.0) =>
            {
                self.add_dimension_constraint(
                    child_view,
                    "heightAnchor",
                    node_height(child, layout.height),
                )?;
            }
            None => {}
        }
        Ok(())
    }

    /// 递归：把每个 IRNode 生成 UIKit 调用，返回创建出来的 UIView* 指针
    fn codegen_node(
        &self,
        node: &IRNode,
        event_target: Option<PointerValue<'ctx>>,
        x: f64,
        y: f64,
        available_width: f64,
        available_height: f64,
        allocated: Option<ChildLayout>,
        is_root: bool,
    ) -> Result<PointerValue<'ctx>, String> {
        let (x, y) = allocated
            .map(|layout| (x + layout.x, y + layout.y))
            .unwrap_or((x, y));
        let allocated_width = allocated.map(|layout| layout.width);
        let allocated_height = allocated.map(|layout| layout.height);
        match node {
            // 递归槽位由行函数接管子树挂载，走到这里说明它没被包在行模板根容器里。
            IRNode::RecursiveComponent { component, .. } => Err(format!(
                "recursive component slot '{component}' must be a direct child of the row template root"
            )),
            IRNode::Conditional {
                cell,
                style: _,
                initial,
                when_true,
                when_false,
                true_mount,
                true_unmount,
                false_mount,
                false_unmount,
            } => {
                if is_root {
                    return Err("a conditional JSX branch must be inside a View or VStack".into());
                }
                let true_view = self.codegen_node(
                    when_true,
                    event_target,
                    x,
                    y,
                    available_width,
                    available_height,
                    allocated,
                    false,
                )?;
                let false_view = if let Some(branch) = when_false {
                    self.codegen_node(
                        branch,
                        event_target,
                        x,
                        y,
                        available_width,
                        available_height,
                        allocated,
                        false,
                    )?
                } else {
                    self.context.ptr_type(AddressSpace::default()).const_null()
                };
                self.send_msg_void(
                    true_view,
                    "setTranslatesAutoresizingMaskIntoConstraints:",
                    &[self.context.i8_type().const_int(0, false).into()],
                )?;
                if when_false.is_some() {
                    self.send_msg_void(
                        false_view,
                        "setTranslatesAutoresizingMaskIntoConstraints:",
                        &[self.context.i8_type().const_int(0, false).into()],
                    )?;
                }
                // 槽位视图：条件分支都挂在它下面，父级把它当普通 flow 子视图约束
                let slot_frame = self.rect_constant(
                    x,
                    y,
                    allocated_width.unwrap_or(available_width),
                    allocated_height.unwrap_or(available_height),
                );
                let slot = self.create_objc_instance_with_frame(self.view_class(), slot_frame)?;
                // 槽位自身的 frame 只是初值，尺寸要靠约束定（行方向给宽度、列方向给高度），
                // 否则 autoresizing 与锚点约束会打架，分支里的内容会被压成几个像素。
                self.send_msg_void(
                    slot,
                    "setTranslatesAutoresizingMaskIntoConstraints:",
                    &[self.context.i8_type().const_int(0, false).into()],
                )?;
                self.apply_background_color(slot, self.ui_color("clear")?)?;
                // 只挂载当前可见的分支，另一条先游离；切换时真增删
                let initial_view = if *initial { true_view } else { false_view };
                if !initial_view.is_null() {
                    self.send_msg_void(slot, "addSubview:", &[initial_view.into()])?;
                    self.add_anchor_constraint(initial_view, "leadingAnchor", slot, "leadingAnchor", 0.0)?;
                    self.add_anchor_constraint(initial_view, "topAnchor", slot, "topAnchor", 0.0)?;
                    self.add_anchor_constraint(initial_view, "trailingAnchor", slot, "trailingAnchor", 0.0)?;
                    self.add_anchor_constraint(initial_view, "bottomAnchor", slot, "bottomAnchor", 0.0)?;
                }
                let slot_height = allocated_height
                    .unwrap_or_else(|| node_height(when_true, available_height));
                let visible = *initial;
                let height_constraint = self
                    .send_msg_ptr(slot, self.get_selector("heightAnchor")?, &[])?
                    .ok_or("slot heightAnchor returned void")?;
                let height_constraint = self
                    .send_msg_ptr(
                        height_constraint,
                        self.get_selector("constraintEqualToConstant:")?,
                        &[self
                            .context
                            .f64_type()
                            .const_float(if visible { slot_height } else { 0.0 })
                            .into()],
                    )?
                    .ok_or("slot height constraint returned void")?;
                self.activate_constraint(height_constraint)?;
                let index = self.conditional_counter.get();
                self.conditional_counter.set(index + 1);
                let ptr_type = self.context.ptr_type(AddressSpace::default());
                let true_global = self.module.add_global(
                    ptr_type,
                    None,
                    &format!("g_conditional_true_{index}"),
                );
                let false_global = self.module.add_global(
                    ptr_type,
                    None,
                    &format!("g_conditional_false_{index}"),
                );
                let stack_global = self.module.add_global(
                    ptr_type,
                    None,
                    &format!("g_conditional_stack_{index}"),
                );
                let height_global = self.module.add_global(
                    ptr_type,
                    None,
                    &format!("g_conditional_height_{index}"),
                );
                true_global.set_initializer(&ptr_type.const_null());
                false_global.set_initializer(&ptr_type.const_null());
                stack_global.set_initializer(&ptr_type.const_null());
                height_global.set_initializer(&ptr_type.const_null());
                self.builder
                    .build_store(stack_global.as_pointer_value(), slot)
                    .map_err(|error| error.to_string())?;
                self.builder
                    .build_store(height_global.as_pointer_value(), height_constraint)
                    .map_err(|error| error.to_string())?;
                self.builder
                    .build_store(true_global.as_pointer_value(), true_view)
                    .map_err(|error| error.to_string())?;
                self.builder
                    .build_store(false_global.as_pointer_value(), false_view)
                    .map_err(|error| error.to_string())?;
                let active_type = self.context.i8_type();
                let active_global = self.module.add_global(
                    active_type,
                    None,
                    &format!("g_conditional_state_{index}"),
                );
                active_global.set_initializer(&active_type.const_int(0, false));
                self.builder
                    .build_store(
                        active_global.as_pointer_value(),
                        active_type.const_int(u64::from(*initial), false),
                    )
                    .map_err(|error| error.to_string())?;
                self.conditional_instances.borrow_mut().push(ConditionalRuntime {
                    cell: cell.clone(),
                    slot: stack_global,
                    true_view: true_global,
                    false_view: false_global,
                    active_state: active_global,
                    height_constraint: height_global,
                    slot_height,
                    true_mount: true_mount.clone(),
                    true_unmount: true_unmount.clone(),
                    false_mount: false_mount.clone(),
                    false_unmount: false_unmount.clone(),
                });
                Ok(slot)
            }
            IRNode::View { style, children } => {
                let width = allocated.map(|layout| layout.width).unwrap_or_else(|| {
                    style.width.map(|dimension| dimension.resolve(available_width))
                        .unwrap_or(available_width)
                });
                let height = allocated.map(|layout| layout.height)
                    .unwrap_or_else(|| node_height(node, available_height));
                let view_frame = if is_root && style.width.is_none() && style.height.is_none() {
                    self.screen_bounds()?
                } else {
                    self.rect_constant(x, y, width, height)
                };
                let dynamic_layout = has_direct_conditional(children);
                let view = self.create_objc_instance_with_frame(self.view_class(), view_frame)?;
                self.apply_inline_style(view, style, false)?;
                // 只有根视图需要一个不透明底色（窗口背景是黑的）；其他容器没写 backgroundColor
                // 时保持透明，否则嵌在有背景色的卡片里会画出一块白。
                if is_root && style.background_color.is_none() {
                    let white = self.ui_color("white")?;
                    self.apply_background_color(view, white)?;
                }
                let padding = node_padding(style);
                let safe_area_box = is_root || style.ignores_safe_area_top;
                if dynamic_layout {
                    self.codegen_dynamic_children(
                        view,
                        children,
                        style,
                        width,
                        height,
                        FlexDirection::Column,
                        0.0,
                        event_target,
                        safe_area_box,
                        false,
                    )?;
                } else {
                    let layouts = layout_flex_children(
                        children,
                        style,
                        width,
                        height,
                        FlexDirection::Column,
                        0.0,
                    );
                    let mut child_views = Vec::with_capacity(children.len());
                    for (child, layout) in children.iter().zip(&layouts) {
                        let child_ptr = self.codegen_node(
                            child,
                            event_target,
                            0.0,
                            0.0,
                            (width - padding.horizontal()).max(0.0),
                            (height - padding.vertical()).max(0.0),
                            Some(*layout),
                            false,
                        )?;
                        self.send_msg_void(view, "addSubview:", &[child_ptr.into()])?;
                        child_views.push(child_ptr);
                    }
                    self.constrain_flex_children(
                        view,
                        &child_views,
                        children,
                        &layouts,
                        style,
                        FlexDirection::Column,
                        0.0,
                        width,
                        height,
                        is_root || style.ignores_safe_area_top,
                    )?;
                }
                Ok(view)
            }
            IRNode::VStack { style, children } => {
                let width = allocated.map(|layout| layout.width).unwrap_or_else(|| {
                    style.width.map(|dimension| dimension.resolve(available_width))
                        .unwrap_or_else(|| node_width(node, available_width))
                });
                let height = allocated.map(|layout| layout.height)
                    .unwrap_or_else(|| node_height(node, available_height));
                let frame = self.rect_constant(x, y, width, height);
                let dynamic_layout = has_direct_conditional(children);
                let stack = self.create_objc_instance_with_frame(self.view_class(), frame)?;
                self.apply_inline_style(stack, style, false)?;
                let padding = node_padding(style);
                let safe_area_box = is_root || style.ignores_safe_area_top;
                if dynamic_layout {
                    self.codegen_dynamic_children(
                        stack,
                        children,
                        style,
                        width,
                        height,
                        FlexDirection::Column,
                        8.0,
                        event_target,
                        safe_area_box,
                        false,
                    )?;
                } else {
                    let layouts = layout_flex_children(
                        children,
                        style,
                        width,
                        height,
                        FlexDirection::Column,
                        8.0,
                    );
                    let mut child_views = Vec::with_capacity(children.len());
                    for (child, layout) in children.iter().zip(&layouts) {
                        let child_view = self.codegen_node(
                            child,
                            event_target,
                            0.0,
                            0.0,
                            (width - padding.horizontal()).max(0.0),
                            (height - padding.vertical()).max(0.0),
                            Some(*layout),
                            false,
                        )?;
                        self.send_msg_void(stack, "addSubview:", &[child_view.into()])?;
                        child_views.push(child_view);
                    }
                    self.constrain_flex_children(
                        stack,
                        &child_views,
                        children,
                        &layouts,
                        style,
                        FlexDirection::Column,
                        8.0,
                        width,
                        height,
                        is_root || style.ignores_safe_area_top,
                    )?;
                }
                Ok(stack)
            }
            IRNode::ScrollView { style, children } => {
                if self.is_mac() {
                    return self.codegen_mac_scroll(
                        node,
                        style,
                        children,
                        event_target,
                        x,
                        y,
                        available_width,
                        available_height,
                        allocated,
                    );
                }
                let width = allocated.map(|layout| layout.width).unwrap_or_else(|| {
                    style.width.map(|dimension| dimension.resolve(available_width))
                        .unwrap_or_else(|| node_width(node, available_width))
                });
                let height = allocated.map(|layout| layout.height)
                    .unwrap_or_else(|| node_height(node, available_height));
                let frame = self.rect_constant(x, y, width, height);
                let scroll_view = self.create_objc_instance_with_frame(self.scroll_class(), frame)?;
                self.apply_inline_style(scroll_view, style, false)?;
                if style.scroll_action.is_some() {
                    let target =
                        event_target.ok_or("ScrollView onScroll requires an event target")?;
                    self.send_msg_void(scroll_view, "setDelegate:", &[target.into()])?;
                    let global = self.object_global("g_scroll_action_view");
                    self.builder
                        .build_store(global.as_pointer_value(), scroll_view)
                        .map_err(|error| error.to_string())?;
                }
                // 滚动容器也不再默认涂白：没写 backgroundColor 就和 UIKit 一样透明。
                if has_direct_conditional(children) || has_direct_list(children) {
                    self.codegen_dynamic_scroll_content(
                        scroll_view,
                        children,
                        style,
                        width,
                        height,
                        event_target,
                    )?;
                    return Ok(scroll_view);
                }
                let padding = node_padding(style);
                let child_layouts = layout_flex_children(
                    children,
                    style,
                    width,
                    height,
                    FlexDirection::Column,
                    0.0,
                );
                let (content_width, content_height) =
                    scroll_content_size(children, &child_layouts, width, height);
                for (child, layout) in children.iter().zip(&child_layouts) {
                    let child_view = self.codegen_node(
                        child,
                        event_target,
                        0.0,
                        0.0,
                        (width - padding.horizontal()).max(0.0),
                        (height - padding.vertical()).max(0.0),
                        Some(*layout),
                        false,
                    )?;
                    self.send_msg_void(scroll_view, "addSubview:", &[child_view.into()])?;
                    self.constrain_scroll_child_width(
                        scroll_view,
                        child_view,
                        child,
                        *layout,
                        padding,
                    )?;
                }
                let content_size = self.context.const_struct(
                    &[
                        self.context
                            .f64_type()
                            .const_float(content_width)
                            .into(),
                        self.context.f64_type().const_float(content_height).into(),
                    ],
                    false,
                );
                self.send_msg_void(scroll_view, "setContentSize:", &[content_size.into()])?;
                Ok(scroll_view)
            }
            IRNode::Text { content, style } => {
                let frame = self.rect_constant(
                    x,
                    y,
                    allocated_width.unwrap_or_else(|| {
                        style.width.map(|dimension| dimension.resolve(available_width))
                            .unwrap_or(available_width)
                    }),
                    allocated_height.unwrap_or_else(|| node_height(node, available_height)),
                );
                let label = self.create_objc_instance_with_frame(self.label_class(), frame)?;
                self.configure_label(label)?;
                let nsstring = self.create_nsstring(content)?;
                self.set_label_text(label, nsstring)?;
                let padding = node_padding(style);
                if padding.is_zero() {
                    self.apply_inline_style(label, style, true)?;
                    Ok(label)
                } else {
                    self.apply_text_style(label, style)?;
                    self.wrap_padded_leaf(label, style, frame, padding)
                }
            }
            IRNode::StateText { cell, initial, style } => {
                let frame = self.rect_constant(
                    x,
                    y,
                    allocated_width.unwrap_or_else(|| {
                        style.width.map(|dimension| dimension.resolve(available_width))
                            .unwrap_or(available_width)
                    }),
                    allocated_height.unwrap_or_else(|| node_height(node, available_height)),
                );
                let label = self.create_objc_instance_with_frame(self.label_class(), frame)?;
                self.configure_label(label)?;
                let nsstring = self.create_nsstring(&initial.display_value())?;
                self.set_label_text(label, nsstring)?;
                let padding = node_padding(style);
                if padding.is_zero() {
                    self.apply_inline_style(label, style, true)?;
                } else {
                    self.apply_text_style(label, style)?;
                }
                let label_index = self.state_label_counter.get();
                self.state_label_counter.set(label_index + 1);
                let state_label = self.module.add_global(
                    self.context.ptr_type(AddressSpace::default()),
                    None,
                    &format!("g_state_label_{label_index}"),
                );
                state_label.set_initializer(
                    &self.context.ptr_type(AddressSpace::default()).const_null(),
                );
                self.builder
                    .build_store(state_label.as_pointer_value(), label)
                    .map_err(|e| e.to_string())?;
                self.state_label_globals
                    .borrow_mut()
                    .entry(cell.clone())
                    .or_default()
                    .push(state_label);
                Ok(label)
            }
            IRNode::Image { source, mode, style } => {
                if source.is_empty() {
                    return Err("Image requires a non-empty src or source asset name".into());
                }
                let image_name = self.create_nsstring(source)?;
                let image_class = self.get_class(if self.is_mac() { "NSImage" } else { "UIImage" })?;
                let image_named = self.get_selector("imageNamed:")?;
                let image = self
                    .send_msg_ptr(image_class, image_named, &[image_name.into()])?
                    .ok_or("UIImage imageNamed: returned void")?;
                let image_view_class = self.get_class(self.image_view_class())?;
                let alloc = self.get_selector("alloc")?;
                let image_view = self
                    .send_msg_ptr(image_view_class, alloc, &[])?
                    .ok_or("UIImageView alloc returned void")?;
                let init_with_image = self.get_selector("initWithImage:")?;
                let image_view = self
                    .send_msg_ptr(image_view, init_with_image, &[image.into()])?
                    .ok_or("UIImageView initWithImage: returned void")?;
                let frame = self.rect_constant(
                    x,
                    y,
                    allocated_width.unwrap_or_else(|| {
                        style.width.map(|dimension| dimension.resolve(available_width))
                            .unwrap_or(available_width)
                    }),
                    allocated_height.unwrap_or_else(|| node_height(node, available_height)),
                );
                self.send_msg_void(image_view, "setFrame:", &[frame.into()])?;
                if !self.is_mac() {
                    self.send_msg_void(
                        image_view,
                        "setContentMode:",
                        &[self.context.i64_type().const_int(mode.content_mode(), false).into()],
                    )?;
                }
                if *mode == ImageMode::AspectFill && !self.is_mac() {
                    self.send_msg_void(
                        image_view,
                        "setClipsToBounds:",
                        &[self.context.bool_type().const_int(1, false).into()],
                    )?;
                }
                let padding = node_padding(style);
                if padding.is_zero() {
                    self.apply_inline_style(image_view, style, false)?;
                    Ok(image_view)
                } else {
                    self.wrap_padded_leaf(image_view, style, frame, padding)
                }
            }
            IRNode::Input {
                placeholder,
                style,
                on_input,
                on_focus,
                unsupported_event_handler,
                ..
            } => {
                if *unsupported_event_handler {
                    return Err(
                        "Input callbacks support onInput={(value) => textState = value} and onFocus={() => booleanState = true}"
                            .into(),
                    );
                }
                let frame = self.rect_constant(
                    x,
                    y,
                    allocated_width.unwrap_or_else(|| {
                        style.width.map(|dimension| dimension.resolve(available_width))
                            .unwrap_or(available_width)
                    }),
                    allocated_height.unwrap_or_else(|| node_height(node, available_height)),
                );
                let input = self.create_objc_instance_with_frame(self.input_class(), frame)?;
                let padding = node_padding(style);
                // 原生控件自带镀铬：iOS 的 borderStyle、macOS 的 bezel/焦点环会在我们画的
                // 背景与圆角上再叠一层，三端观感就不一致。只要样式里给了背景或内边距，
                // 就关掉原生镀铬，统一由 apply_box_style + padding 负责外观。
                let custom_chrome = style.background_color.is_some() || !padding.is_zero();
                if !placeholder.is_empty() {
                    self.set_placeholder(input, placeholder, style)?;
                }
                if self.is_mac() {
                    let bezel = if custom_chrome { 0 } else { 1 };
                    self.send_msg_void(
                        input,
                        "setBezeled:",
                        &[self.context.i8_type().const_int(bezel, false).into()],
                    )?;
                    if custom_chrome {
                        let no = self.context.i8_type().const_int(0, false);
                        self.send_msg_void(input, "setBordered:", &[no.into()])?;
                        self.send_msg_void(input, "setDrawsBackground:", &[no.into()])?;
                        // NSFocusRingTypeNone：不要蓝色焦点环，与 iOS/Android 一致
                        self.send_msg_void(
                            input,
                            "setFocusRingType:",
                            &[self.context.i64_type().const_int(1, false).into()],
                        )?;
                    }
                } else {
                    let border_style = if custom_chrome { 0 } else { 3 };
                    self.send_msg_void(
                        input,
                        "setBorderStyle:",
                        &[self.context.i64_type().const_int(border_style, false).into()],
                    )?;
                }
                if padding.is_zero() {
                    self.apply_inline_style(input, style, true)?;
                } else {
                    self.apply_text_style(input, style)?;
                }                // onInput / onFocus：iOS 走 UIControl 事件，AppKit 没有这套，改用
                // NSTextField delegate 的 controlTextDidChange: 回调（见 build_input_delegate_method）。
                if let Some(action_index) = on_input.filter(|_| self.is_mac()) {
                    let target = event_target.ok_or("Input onInput requires an event target")?;
                    self.send_msg_void(input, "setDelegate:", &[target.into()])?;
                    self.store_input_source(action_index, input)?;
                }
                if let Some(action_index) = on_input.filter(|_| !self.is_mac()) {
                    let target = event_target.ok_or("Input onInput requires an event target")?;
                    let action = self.get_selector(&format!("stateAction{action_index}:"))?;
                    self.send_msg_void(
                        input,
                        "addTarget:action:forControlEvents:",
                        &[
                            target.into(),
                            action.into(),
                            self.context.i64_type().const_int(1 << 17, false).into(),
                        ],
                    )?;
                }
                if let Some(action_index) = on_focus.filter(|_| !self.is_mac()) {
                    let target = event_target.ok_or("Input onFocus requires an event target")?;
                    let action = self.get_selector(&format!("stateAction{action_index}:"))?;
                    self.send_msg_void(
                        input,
                        "addTarget:action:forControlEvents:",
                        &[
                            target.into(),
                            action.into(),
                            self.context.i64_type().const_int(1 << 16, false).into(),
                        ],
                    )?;
                }
                if padding.is_zero() {
                    Ok(input)
                } else {
                    self.wrap_padded_leaf(input, style, frame, padding)
                }
            }
            IRNode::List { .. } => self.codegen_list(
                node,
                event_target,
                x,
                y,
                available_width,
                available_height,
                allocated,
            ),
            IRNode::ExprText {
                left,
                right,
                subtract,
                deps,
                initial,
                style,
            } => {
                let frame = self.rect_constant(
                    x,
                    y,
                    allocated_width.unwrap_or_else(|| node_width(node, available_width)),
                    allocated_height.unwrap_or_else(|| node_height(node, available_height)),
                );
                let label = self.create_objc_instance_with_frame(self.label_class(), frame)?;
                self.configure_label(label)?;
                let text = self.create_nsstring(initial)?;
                self.set_label_text(label, text)?;
                self.apply_inline_style(label, style, true)?;
                let index = self.expr_counter.get();
                self.expr_counter.set(index + 1);
                let global = self.object_global(&format!("g_expr_label_{index}"));
                self.builder
                    .build_store(global.as_pointer_value(), label)
                    .map_err(|e| e.to_string())?;
                self.expr_labels.borrow_mut().push(ExprLabel {
                    deps: deps.clone(),
                    label: global,
                    left: left.clone(),
                    right: right.clone(),
                    subtract: *subtract,
                });
                Ok(label)
            }
            IRNode::ItemConditional {
                field,
                when_true,
                when_false,
                ..
            } => {
                let ctx = self.list_row_ctx.borrow().clone().ok_or_else(|| {
                    "list item conditionals are only valid inside items.map(...)".to_string()
                })?;
                let function = self.current_function()?;
                let value = self.load_list_value(
                    &ctx.id,
                    field,
                    ctx.index.value(self.context),
                )?;
                let expected = self.create_nsstring("true")?;
                let is_true = self
                    .send_msg_impl(
                        value,
                        self.get_selector("isEqualToString:")?,
                        &[expected.into()],
                        BasicTypeEnum::IntType(self.context.bool_type()),
                    )?
                    .ok_or_else(|| "NSString isEqualToString: returned void".to_string())?
                    .into_int_value();
                let truthy = self
                    .builder
                    .build_int_compare(
                        IntPredicate::NE,
                        is_true,
                        self.context.bool_type().const_zero(),
                        "item_condition",
                    )
                    .map_err(|e| e.to_string())?;
                let true_block = self.context.append_basic_block(function, "item_true");
                let false_block = self.context.append_basic_block(function, "item_false");
                let done_block = self.context.append_basic_block(function, "item_join");
                self.builder
                    .build_conditional_branch(truthy, true_block, false_block)
                    .map_err(|e| e.to_string())?;

                self.builder.position_at_end(true_block);
                let true_view = self.codegen_node(
                    when_true,
                    event_target,
                    x,
                    y,
                    available_width,
                    available_height,
                    allocated,
                    false,
                )?;
                self.builder
                    .build_unconditional_branch(done_block)
                    .map_err(|e| e.to_string())?;

                self.builder.position_at_end(false_block);
                let false_view = match when_false {
                    Some(node) => self.codegen_node(
                        node,
                        event_target,
                        x,
                        y,
                        available_width,
                        available_height,
                        allocated,
                        false,
                    )?,
                    None => self.context.ptr_type(AddressSpace::default()).const_null(),
                };
                self.builder
                    .build_unconditional_branch(done_block)
                    .map_err(|e| e.to_string())?;
                self.builder.position_at_end(done_block);
                let phi = self
                    .builder
                    .build_phi(self.context.ptr_type(AddressSpace::default()), "item_branch")
                    .map_err(|e| e.to_string())?;
                phi.add_incoming(&[(&true_view, true_block)]);
                phi.add_incoming(&[(&false_view, false_block)]);
                Ok(phi.as_basic_value().into_pointer_value())
            }
            IRNode::ItemText { field, style, .. } => {
                let ctx = self.list_row_ctx.borrow().clone().ok_or_else(|| {
                    "list item fields can only be rendered inside items.map(...)".to_string()
                })?;
                let frame = self.rect_constant(
                    x,
                    y,
                    allocated_width.unwrap_or_else(|| node_width(node, available_width)),
                    allocated_height.unwrap_or_else(|| node_height(node, available_height)),
                );
                let label = self.create_objc_instance_with_frame(self.label_class(), frame)?;
                self.configure_label(label)?;
                // 行是运行期按行号建的，文本直接从该行的数据列读。
                let text = self.load_list_value(&ctx.id, field, ctx.index.value(self.context))?;
                self.set_label_text(label, text)?;
                self.apply_inline_style(label, style, true)?;
                Ok(label)
            }
            IRNode::Button {
                title,
                style,
                state_action,
                unsupported_handler,
            } => {
                if *unsupported_handler {
                    return Err(
                        "supported Button onClick form is () => state++ / state-- / state = <literal> on a state(...) cell"
                            .into(),
                    );
                }
                let button = if self.is_mac() {
                    self.create_objc_instance_with_frame(self.button_class(), self.rect_constant(x, y, 0.0, 0.0))?
                } else {
                    let class = self.get_class(self.button_class())?;
                    let button_type = self.get_selector("buttonWithType:")?;
                    self.send_msg_ptr(
                        class,
                        button_type,
                        &[self.context.i64_type().const_int(0, false).into()],
                    )?
                    .ok_or("UIButton buttonWithType: returned void")?
                };
                self.send_msg_void(
                    button,
                    "setFrame:",
                    &[self.rect_constant(
                        x,
                        y,
                        allocated_width.unwrap_or_else(|| {
                            style.width.map(|dimension| dimension.resolve(available_width))
                                .unwrap_or(available_width)
                        }),
                        allocated_height.unwrap_or_else(|| node_height(node, available_height)),
                    ).into()],
                )?;
                let title = self.create_nsstring(title)?;
                if self.is_mac() {
                    self.send_msg_void(
                        button,
                        "setBordered:",
                        &[self.context.i8_type().const_int(0, false).into()],
                    )?;
                    self.send_msg_void(button, "setTitle:", &[title.into()])?;
                } else {
                    self.send_msg_void(
                        button,
                        "setTitle:forState:",
                        &[title.into(), self.context.i64_type().const_int(0, false).into()],
                    )?;
                }
                self.apply_inline_style(button, style, false)?;
                if let Some(index) = self
                    .list_row_ctx
                    .borrow()
                    .as_ref()
                    .map(|ctx| ctx.index.value(self.context))
                {
                    // 行内按钮用 tag 记住自己是第几行，回调据此定位数据。
                    self.send_msg_void(button, "setTag:", &[index.into()])?;
                }
                if let Some(padding) = style.padding {
                    // 叶子的 padding 落在标题四周，与容器 padding 的语义一致
                    let insets = self.ui_edge_insets(padding, padding, padding, padding);
                    if !self.is_mac() {
                        self.send_msg_void(button, "setContentEdgeInsets:", &[insets.into()])?;
                    }
                }
                if let Some(color_name) = &style.color {
                    let color = self.ui_color(color_name)?;
                    if self.is_mac() {
                        self.send_msg_void(button, "setContentTintColor:", &[color.into()])?;
                    } else {
                        self.send_msg_void(
                            button,
                            "setTitleColor:forState:",
                            &[color.into(), self.context.i64_type().const_int(0, false).into()],
                        )?;
                    }
                }
                if let Some(font_size) = style.font_size {
                    let font_class = self.get_class(if self.is_mac() { "NSFont" } else { "UIFont" })?;
                    let system_font_sel = self.get_selector("systemFontOfSize:")?;
                    let font = self
                        .send_msg_ptr(
                            font_class,
                            system_font_sel,
                            &[self.context.f64_type().const_float(font_size).into()],
                        )?
                        .ok_or("systemFontOfSize: returned void")?;
                    if self.is_mac() {
                        self.send_msg_void(button, "setFont:", &[font.into()])?;
                    } else {
                        let title_label = self
                            .send_msg_ptr(button, self.get_selector("titleLabel")?, &[])?
                            .ok_or("UIButton titleLabel returned void")?;
                        self.send_msg_void(title_label, "setFont:", &[font.into()])?;
                    }
                }

                if let Some(action_index) = state_action {
                    let target = event_target.ok_or(
                        "Button onClick requires a state(...) declaration",
                    )?;
                    let action = self.get_selector(&format!("stateAction{action_index}:"))?;
                    if self.is_mac() {
                        self.send_msg_void(button, "setTarget:", &[target.into()])?;
                        self.send_msg_void(button, "setAction:", &[action.into()])?;
                    } else {
                        self.send_msg_void(
                            button,
                            "addTarget:action:forControlEvents:",
                            &[
                                target.into(),
                                action.into(),
                                self.context.i64_type().const_int(1 << 6, false).into(),
                            ],
                        )?;
                    }
                }
                Ok(button)
            }
        }
    }

    /// 占位符统一走 attributed 版本，颜色才能由样式（placeholderColor）控制：
    /// UIKit 用 `attributedPlaceholder`，AppKit 用 `placeholderAttributedString`。
    fn set_placeholder(
        &self,
        field: PointerValue<'ctx>,
        text: &str,
        style: &InlineStyle,
    ) -> Result<(), String> {
        let ptr_type = self.context.ptr_type(AddressSpace::default());
        let string = self.create_nsstring(text)?;
        let color_name = style
            .placeholder_color
            .as_deref()
            .unwrap_or(DEFAULT_PLACEHOLDER_COLOR);
        let color = self.ui_color(color_name)?;
        // NSForegroundColorAttributeName 是 AppKit/UIKit 导出的 NSString 常量，声明成外部符号
        let key = match self.module.get_global("NSForegroundColorAttributeName") {
            Some(global) => global,
            None => self
                .module
                .add_global(ptr_type, None, "NSForegroundColorAttributeName"),
        };
        let key = self
            .builder
            .build_load(ptr_type, key.as_pointer_value(), "placeholder_key")
            .map_err(|error| error.to_string())?
            .into_pointer_value();
        let dictionary_class = self.get_class("NSDictionary")?;
        let attributes = self
            .send_msg_ptr(
                dictionary_class,
                self.get_selector("dictionaryWithObject:forKey:")?,
                &[color.into(), key.into()],
            )?
            .ok_or("NSDictionary dictionaryWithObject:forKey: returned void")?;
        let attributed_class = self.get_class("NSAttributedString")?;
        let alloc = self
            .send_msg_ptr(attributed_class, self.get_selector("alloc")?, &[])?
            .ok_or("NSAttributedString alloc returned void")?;
        let attributed = self
            .send_msg_ptr(
                alloc,
                self.get_selector("initWithString:attributes:")?,
                &[string.into(), attributes.into()],
            )?
            .ok_or("NSAttributedString initWithString:attributes: returned void")?;
        let selector = if self.is_mac() {
            "setPlaceholderAttributedString:"
        } else {
            "setAttributedPlaceholder:"
        };
        self.send_msg_void(field, selector, &[attributed.into()])
    }

    fn apply_inline_style(
        &self,
        view: PointerValue<'ctx>,
        style: &InlineStyle,
        text_like: bool,
    ) -> Result<(), String> {
        self.apply_box_style(view, style)?;
        if text_like {
            self.apply_text_style(view, style)?;
        }
        // 透明度：常量直接设，绑了单元就登记，单元变化时由 update_cell_labels 更新
        if let Some(cell) = &style.opacity_cell {
            let initial = self
                .state_cells
                .borrow()
                .iter()
                .find(|(path, _)| path == cell)
                .and_then(|(_, value)| match value {
                    StateValue::Number(value) => Some(*value as f64 / 100.0),
                    _ => None,
                })
                .unwrap_or(1.0)
                .clamp(0.0, 1.0);
            self.send_view_alpha(view, self.context.f64_type().const_float(initial))?;
        } else if let Some(alpha) = style.opacity_static {
            self.send_view_alpha(view, self.context.f64_type().const_float(alpha))?;
        }
        if let Some(cell) = &style.opacity_cell {
            let index = self.opacity_counter.get();
            self.opacity_counter.set(index + 1);
            let global = self.module.add_global(
                self.context.ptr_type(AddressSpace::default()),
                None,
                &format!("g_opacity_view_{index}"),
            );
            global.set_initializer(&self.context.ptr_type(AddressSpace::default()).const_null());
            self.builder
                .build_store(global.as_pointer_value(), view)
                .map_err(|e| e.to_string())?;
            self.opacity_globals
                .borrow_mut()
                .entry(cell.clone())
                .or_default()
                .push(global);
        }
        Ok(())
    }

    /// 设置背景色：UIKit 直接用 setBackgroundColor:，AppKit 要走 layer。
    /// 设置视图透明度：UIKit 用 setAlpha:，AppKit 用 setAlphaValue:。
    fn send_view_alpha(
        &self,
        view: PointerValue<'ctx>,
        alpha: inkwell::values::FloatValue<'ctx>,
    ) -> Result<(), String> {
        let selector = if self.is_mac() { "setAlphaValue:" } else { "setAlpha:" };
        self.send_msg_void(view, selector, &[alpha.into()])
    }

    fn apply_background_color(
        &self,
        view: PointerValue<'ctx>,
        color: PointerValue<'ctx>,
    ) -> Result<(), String> {
        if !self.is_mac() {
            return self.send_msg_void(view, "setBackgroundColor:", &[color.into()]);
        }
        self.wants_layer(view)?;
        let layer = self
            .send_msg_ptr(view, self.get_selector("layer")?, &[])?
            .ok_or("NSView layer returned void")?;
        let cg_color = self
            .send_msg_ptr(color, self.get_selector("CGColor")?, &[])?
            .ok_or("NSColor CGColor returned void")?;
        self.send_msg_void(layer, "setBackgroundColor:", &[cg_color.into()])
    }

    /// AppKit 的视图要显式开启 layer 才能画背景、边框和圆角。
    fn wants_layer(&self, view: PointerValue<'ctx>) -> Result<(), String> {
        if !self.is_mac() {
            return Ok(());
        }
        self.send_msg_void(
            view,
            "setWantsLayer:",
            &[self.context.i8_type().const_int(1, false).into()],
        )
    }

    /// macOS 的 NSTextField 默认是输入框，需要先变成"标签"。
    fn configure_label(&self, label: PointerValue<'ctx>) -> Result<(), String> {
        if !self.is_mac() {
            return Ok(());
        }
        // 默认左对齐，和 iOS 的 UILabel 保持一致
        self.send_msg_void(
            label,
            "setAlignment:",
            &[self.context.i64_type().const_int(0, false).into()],
        )?;
        for selector in [
            "setBordered:",
            "setBezeled:",
            "setEditable:",
            "setSelectable:",
            "setDrawsBackground:",
        ] {
            self.send_msg_void(
                label,
                selector,
                &[self.context.i8_type().const_int(0, false).into()],
            )?;
        }
        Ok(())
    }

    fn set_label_text(
        &self,
        label: PointerValue<'ctx>,
        text: PointerValue<'ctx>,
    ) -> Result<(), String> {
        let selector = if self.is_mac() { "setStringValue:" } else { "setText:" };
        self.send_msg_void(label, selector, &[text.into()])
    }

    /// 盒子类样式：背景、边框、圆角。padding 会让它们落在包裹层上。
    fn apply_box_style(&self, view: PointerValue<'ctx>, style: &InlineStyle) -> Result<(), String> {
        if let Some(color_name) = &style.background_color {
            let color = self.ui_color(color_name)?;
            self.apply_background_color(view, color)?;
        }
        if style.border_width.is_some() || style.border_color.is_some() || style.border_radius.is_some() {
            self.wants_layer(view)?;
            let layer = self
                .send_msg_ptr(view, self.get_selector("layer")?, &[])?
                .ok_or("UIView layer returned void")?;
            if let Some(width) = style.border_width {
                self.send_msg_void(
                    layer,
                    "setBorderWidth:",
                    &[self.context.f64_type().const_float(width).into()],
                )?;
            }
            if let Some(color_value) = &style.border_color {
                let color = self.ui_color(color_value)?;
                let cg_color = self
                    .send_msg_ptr(color, self.get_selector("CGColor")?, &[])?
                    .ok_or("UIColor CGColor returned void")?;
                self.send_msg_void(layer, "setBorderColor:", &[cg_color.into()])?;
            }
            if let Some(radius) = style.border_radius {
                self.send_msg_void(
                    layer,
                    "setCornerRadius:",
                    &[self.context.f64_type().const_float(radius).into()],
                )?;
            }
        }
        Ok(())
    }

    /// 文本类样式：对齐、颜色、字号。
    fn apply_text_style(&self, view: PointerValue<'ctx>, style: &InlineStyle) -> Result<(), String> {
        if let Some(alignment) = style.text_align {
            let selector = if self.is_mac() { "setAlignment:" } else { "setTextAlignment:" };
            self.send_msg_void(
                view,
                selector,
                &[self.context.i64_type().const_int(alignment.ui_value(), false).into()],
            )?;
        }
        if let Some(color_name) = &style.color {
            let color = self.ui_color(color_name)?;
            self.send_msg_void(view, "setTextColor:", &[color.into()])?;
        }
        if let Some(font_size) = style.font_size {
            let font_class = self.get_class(if self.is_mac() { "NSFont" } else { "UIFont" })?;
            let system_font = self.get_selector("systemFontOfSize:")?;
            let font = self
                .send_msg_ptr(
                    font_class,
                    system_font,
                    &[self.context.f64_type().const_float(font_size).into()],
                )?
                .ok_or("systemFontOfSize: returned void")?;
            self.send_msg_void(view, "setFont:", &[font.into()])?;
        }
        Ok(())
    }

    /// 叶子的 padding：外面包一层容器承载背景/边框，内容按 padding 内缩。
    /// 对齐 SwiftUI 的 `.padding(...).background(...)`。
    fn wrap_padded_leaf(
        &self,
        leaf: PointerValue<'ctx>,
        style: &InlineStyle,
        frame: inkwell::values::StructValue<'ctx>,
        padding: Padding,
    ) -> Result<PointerValue<'ctx>, String> {
        let wrapper = self.create_objc_instance_with_frame(self.view_class(), frame)?;
        self.apply_box_style(wrapper, style)?;
        self.send_msg_void(
            leaf,
            "setTranslatesAutoresizingMaskIntoConstraints:",
            &[self.context.bool_type().const_int(0, false).into()],
        )?;
        self.send_msg_void(wrapper, "addSubview:", &[leaf.into()])?;
        self.add_anchor_constraint(leaf, "leadingAnchor", wrapper, "leadingAnchor", padding.left)?;
        self.add_anchor_constraint(leaf, "topAnchor", wrapper, "topAnchor", padding.top)?;
        self.add_anchor_constraint(
            leaf,
            "trailingAnchor",
            wrapper,
            "trailingAnchor",
            -padding.right,
        )?;
        self.add_anchor_constraint(
            leaf,
            "bottomAnchor",
            wrapper,
            "bottomAnchor",
            -padding.bottom,
        )?;
        Ok(wrapper)
    }

    fn ui_color(&self, value: &str) -> Result<PointerValue<'ctx>, String> {
        if let Some((red, green, blue, alpha)) = parse_functional_color(value)? {
            let color_class = self.get_class(if self.is_mac() { "NSColor" } else { "UIColor" })?;
            let selector = self.get_selector("colorWithRed:green:blue:alpha:")?;
            return self
                .send_msg_ptr(
                    color_class,
                    selector,
                    &[
                        self.context.f64_type().const_float(red).into(),
                        self.context.f64_type().const_float(green).into(),
                        self.context.f64_type().const_float(blue).into(),
                        self.context.f64_type().const_float(alpha).into(),
                    ],
                )?
                .ok_or_else(|| "UIColor colorWithRed:green:blue:alpha: returned void".into());
        }
        let selector_name = match value.trim().to_ascii_lowercase().as_str() {
            "black" => "blackColor",
            "white" => "whiteColor",
            "red" => "redColor",
            "blue" => "blueColor",
            "green" => "greenColor",
            "gray" | "grey" => "grayColor",
            "clear" | "transparent" => "clearColor",
            _ => return Err(format!("unsupported inline color: {value}")),
        };
        let color_class = self.get_class(if self.is_mac() { "NSColor" } else { "UIColor" })?;
        let selector = self.get_selector(selector_name)?;
        self.send_msg_ptr(color_class, selector, &[])?
            .ok_or_else(|| format!("UIColor {selector_name} returned void"))
    }

    /// 生成 [[Class alloc] init]，返回实例指针
    fn create_objc_instance(&self, class_name: &str) -> Result<PointerValue<'ctx>, String> {
        let class = self.get_class(class_name)?;
        let alloc_sel = self.get_selector("alloc")?;
        let instance = self
            .send_msg_ptr(class, alloc_sel, &[])?
            .ok_or("alloc returned void")?;
        let init_sel = self.get_selector("init")?;
        self.send_msg_ptr(instance, init_sel, &[])?
            .ok_or("init returned void".into())
    }

    fn create_objc_instance_with_frame(
        &self,
        class_name: &str,
        frame: inkwell::values::StructValue<'ctx>,
    ) -> Result<PointerValue<'ctx>, String> {
        let class = self.get_class(class_name)?;
        let alloc_sel = self.get_selector("alloc")?;
        let instance = self
            .send_msg_ptr(class, alloc_sel, &[])?
            .ok_or("alloc returned void")?;
        let init_frame_sel = self.get_selector("initWithFrame:")?;
        self.send_msg_ptr(instance, init_frame_sel, &[frame.into()])?
            .ok_or_else(|| format!("{class_name} initWithFrame: returned void"))
    }

    fn screen_bounds(&self) -> Result<inkwell::values::StructValue<'ctx>, String> {
        if self.is_mac() {
            return Ok(self.rect_constant(0.0, 0.0, 420.0, 900.0));
        }
        let screen_class = self.get_class("UIScreen")?;
        let main_screen_sel = self.get_selector("mainScreen")?;
        let screen = self
            .send_msg_ptr(screen_class, main_screen_sel, &[])?
            .ok_or("UIScreen mainScreen returned void")?;
        let bounds_sel = self.get_selector("bounds")?;

        let f64_type = self.context.f64_type();
        let point_type = self
            .context
            .struct_type(&[f64_type.into(), f64_type.into()], false);
        let size_type = self
            .context
            .struct_type(&[f64_type.into(), f64_type.into()], false);
        let rect_type = self
            .context
            .struct_type(&[point_type.into(), size_type.into()], false);
        let result = self
            .send_msg_impl(
                screen,
                bounds_sel,
                &[],
                BasicTypeEnum::StructType(rect_type),
            )?
            .ok_or("UIScreen bounds returned void")?;
        match result {
            BasicValueEnum::StructValue(bounds) => Ok(bounds),
            _ => Err("UIScreen bounds did not return CGRect".into()),
        }
    }

    fn rect_constant(
        &self,
        x: f64,
        y: f64,
        width: f64,
        height: f64,
    ) -> inkwell::values::StructValue<'ctx> {
        let f64_type = self.context.f64_type();
        let origin = self.context.const_struct(
            &[
                f64_type.const_float(x).into(),
                f64_type.const_float(y).into(),
            ],
            false,
        );
        let size = self.context.const_struct(
            &[
                f64_type.const_float(width).into(),
                f64_type.const_float(height).into(),
            ],
            false,
        );
        self.context.const_struct(&[origin.into(), size.into()], false)
    }

    fn ui_edge_insets(
        &self,
        top: f64,
        left: f64,
        bottom: f64,
        right: f64,
    ) -> inkwell::values::StructValue<'ctx> {
        let f64_type = self.context.f64_type();
        self.context.const_struct(
            &[
                f64_type.const_float(top).into(),
                f64_type.const_float(left).into(),
                f64_type.const_float(bottom).into(),
                f64_type.const_float(right).into(),
            ],
            false,
        )
    }

    /// 生成 [[NSString alloc] initWithUTF8String:...]
    fn create_nsstring(&self, text: &str) -> Result<PointerValue<'ctx>, String> {
        let cstr = self
            .builder
            .build_global_string_ptr(text, "text_content")
            .map_err(|e| e.to_string())?;
        self.create_nsstring_from_cstr(cstr.as_pointer_value())
    }

    fn create_nsstring_from_cstr(
        &self,
        cstr: PointerValue<'ctx>,
    ) -> Result<PointerValue<'ctx>, String> {
        let cls = self.get_class("NSString")?;
        let alloc_sel = self.get_selector("alloc")?;
        let raw = self
            .send_msg_ptr(cls, alloc_sel, &[])?
            .ok_or("NSString alloc returned void")?;
        let init_sel = self.get_selector("initWithUTF8String:")?;
        self.send_msg_ptr(raw, init_sel, &[cstr.into()])?
            .ok_or("initWithUTF8String returned void".into())
    }

    /// 复制一个字符串并持有它。UIKit 控件给出的 `text` 是 autoreleased 的，
    /// 直接存进全局变量会在 autorelease 池回收后变成野指针。
    fn copy_nsstring(&self, text: PointerValue<'ctx>) -> Result<PointerValue<'ctx>, String> {
        let cls = self.get_class("NSString")?;
        let raw = self
            .send_msg_ptr(cls, self.get_selector("alloc")?, &[])?
            .ok_or("NSString alloc returned void")?;
        self.send_msg_ptr(raw, self.get_selector("initWithString:")?, &[text.into()])?
            .ok_or("initWithString: returned void".into())
    }

    fn get_class(&self, name: &str) -> Result<PointerValue<'ctx>, String> {
        let name_ptr = self
            .builder
            .build_global_string_ptr(name, "class_name")
            .map_err(|e| e.to_string())?;
        let call = self
            .builder
            .build_call(
                self.objc_get_class,
                &[name_ptr.as_pointer_value().into()],
                "cls",
            )
            .map_err(|e| e.to_string())?;
        match call.try_as_basic_value() {
            inkwell::values::ValueKind::Basic(bv) => Ok(bv.into_pointer_value()),
            inkwell::values::ValueKind::Instruction(_) => {
                Err("Expected BasicValue, got Instruction".into())
            }
        }
    }

    fn get_selector(&self, name: &str) -> Result<PointerValue<'ctx>, String> {
        let name_ptr = self
            .builder
            .build_global_string_ptr(name, "sel_name")
            .map_err(|e| e.to_string())?;
        let call = self
            .builder
            .build_call(
                self.sel_register_name,
                &[name_ptr.as_pointer_value().into()],
                "sel",
            )
            .map_err(|e| e.to_string())?;
        match call.try_as_basic_value() {
            inkwell::values::ValueKind::Basic(bv) => Ok(bv.into_pointer_value()),
            inkwell::values::ValueKind::Instruction(_) => {
                Err("sel_registerName returned void".into())
            }
        }
    }

    fn metadata_type(
        value: &BasicMetadataValueEnum<'ctx>,
    ) -> Result<BasicMetadataTypeEnum<'ctx>, String> {
        match value {
            BasicMetadataValueEnum::IntValue(v) => Ok(v.get_type().into()),
            BasicMetadataValueEnum::FloatValue(v) => Ok(v.get_type().into()),
            BasicMetadataValueEnum::PointerValue(v) => Ok(v.get_type().into()),
            BasicMetadataValueEnum::StructValue(v) => Ok(v.get_type().into()),
            BasicMetadataValueEnum::ArrayValue(v) => Ok(v.get_type().into()),
            BasicMetadataValueEnum::VectorValue(v) => Ok(v.get_type().into()),
            BasicMetadataValueEnum::ScalableVectorValue(v) => Ok(v.get_type().into()),
            BasicMetadataValueEnum::MetadataValue(_) => {
                Err("Metadata values cannot be passed as function arguments".into())
            }
        }
    }

    /// 发送消息，期望返回 id（指针）
    fn send_msg_ptr(
        &self,
        receiver: PointerValue<'ctx>,
        selector: PointerValue<'ctx>,
        args: &[BasicMetadataValueEnum<'ctx>],
    ) -> Result<Option<PointerValue<'ctx>>, String> {
        let ptr_type = self.context.ptr_type(AddressSpace::default());
        let call = self.send_msg_impl(
            receiver,
            selector,
            args,
            BasicTypeEnum::PointerType(ptr_type),
        )?;
        Ok(call.map(|v| v.into_pointer_value()))
    }

    /// 发送消息，返回 void（如 setText:、addSubview:）
fn send_msg_void(
    &self,
    receiver: PointerValue<'ctx>,
    selector_name: &str,
    args: &[BasicMetadataValueEnum<'ctx>],
) -> Result<(), String> {
    let selector = self.get_selector(selector_name)?;
    let void_type = self.context.void_type();

    // 构造 void 返回类型的函数签名
    let mut param_types: Vec<BasicMetadataTypeEnum<'ctx>> = vec![
        self.context.ptr_type(AddressSpace::default()).into(),
        self.context.ptr_type(AddressSpace::default()).into(),
    ];
    for a in args {
        param_types.push(Self::metadata_type(a)?);
    }
    let fn_type = void_type.fn_type(&param_types, false);

    let msg_send_ptr = self.objc_msg_send.as_global_value().as_pointer_value();
    let mut call_args: Vec<BasicMetadataValueEnum<'ctx>> = vec![receiver.into(), selector.into()];
    call_args.extend(args.iter().cloned());

    self.builder
        .build_indirect_call(fn_type, msg_send_ptr, &call_args, "")
        .map_err(|e| e.to_string())?;

    Ok(())
}

    /// 根据实际参数类型调用 objc_msgSend。
    fn send_msg_impl(
        &self,
        receiver: PointerValue<'ctx>,
        selector: PointerValue<'ctx>,
        args: &[BasicMetadataValueEnum<'ctx>],
        return_type: BasicTypeEnum<'ctx>,
    ) -> Result<Option<BasicValueEnum<'ctx>>, String> {
        let ptr_type = self.context.ptr_type(AddressSpace::default());

        // 1. 构造目标函数类型
        let mut param_types: Vec<BasicMetadataTypeEnum<'ctx>> =
            vec![ptr_type.into(), ptr_type.into()];
        for a in args {
            param_types.push(Self::metadata_type(a)?);
        }
        let fn_type = return_type.fn_type(&param_types, false);

        // 2. 在 opaque pointer 模型下，调用时传入函数类型即可。
        let msg_send_ptr = self.objc_msg_send.as_global_value().as_pointer_value();

        // 3. 准备调用参数
        let mut call_args: Vec<BasicMetadataValueEnum<'ctx>> =
            vec![receiver.into(), selector.into()];
        call_args.extend(args.iter().cloned());

        // 4. 使用函数指针和签名构建间接调用。
           let call = self
            .builder
            .build_indirect_call(fn_type, msg_send_ptr, &call_args, "msg_result")
            .map_err(|e| e.to_string())?;

        // 5. 处理返回值（不依赖 ValueKind，直接通过 BasicValueEnum 匹配）
        match call.try_as_basic_value() {
            // 注意：这里用完全限定名匹配，如果依然报错，可以尝试直接匹配 Basic 和 Instruction 变体
            inkwell::values::ValueKind::Basic(bv) => Ok(Some(bv)),
            inkwell::values::ValueKind::Instruction(_) => Ok(None),
        }
    }
}

// ---------- main ----------

/// 内置示例：名字 → 源码。`cargo run -- android todo` 编第二个，默认编 dashboard。
const APPS: &[(&str, &str)] = &[
    ("dashboard", include_str!("../app/App.tsx")),
    ("todo", include_str!("../app/Todo.tsx")),
    ("list", include_str!("../app/List.tsx")),
    ("tree", include_str!("../app/Tree.tsx")),
    ("bench", include_str!("../app/Bench.tsx")),
];

const DEFAULT_APP: &str = "dashboard";

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();

    // 后端形态：原生后端（发消息建控件）或生成源码后端（--target=web）
    let source_target = args
        .iter()
        .any(|arg| matches!(arg.as_str(), "web" | "--target=web"));
    let allow_degrade = args.iter().any(|arg| arg == "--allow-degrade");

    // `cargo run -- macos` 生成 AppKit 版本，"ios"（默认）生成 UIKit 版本。
    let platform = if args
        .iter()
        .any(|arg| matches!(arg.as_str(), "macos" | "mac" | "--target=macos"))
    {
        Platform::MacOs
    } else if args
        .iter()
        .any(|arg| matches!(arg.as_str(), "android" | "--target=android"))
    {
        Platform::Android
    } else {
        Platform::Ios
    };

    // `--app=todo` 或直接写 `todo`；认不出来的 --app= 直接报错，避免默默编错页面。
    let mut app = DEFAULT_APP.to_string();
    for arg in &args {
        if let Some(name) = arg.strip_prefix("--app=") {
            if !APPS.iter().any(|(known, _)| *known == name) {
                eprintln!(
                    "unknown app '{name}', available: {}",
                    APPS.iter().map(|(name, _)| *name).collect::<Vec<_>>().join(", ")
                );
                std::process::exit(1);
            }
            app = name.to_string();
        } else if APPS.iter().any(|(known, _)| *known == arg) {
            app = arg.clone();
        }
    }
    let tsx_code = APPS
        .iter()
        .find(|(name, _)| *name == app)
        .map(|(_, source)| *source)
        .unwrap_or(APPS[0].1);
    println!("==> 编译示例: {app}");

    let program = parse_tsx_to_ast(tsx_code);
    println!("AST: {:#?}\n", program);

    // 生成源码型后端：把 IR 翻译成目标平台源码，而不是调用平台 API
    if source_target {
        if !program.errors.is_empty() {
            eprintln!("codegen failed: {}", program.errors.join("\n"));
            std::process::exit(1);
        }
        match capabilities::check(&capabilities::WEB, &program, allow_degrade) {
            Ok(warnings) => {
                for warning in warnings {
                    eprintln!("warning: {warning}");
                }
            }
            Err(error) => {
                eprintln!("capability check failed: {error}");
                std::process::exit(1);
            }
        }
        match source_backend::generate_html(&program) {
            Ok(html) => {
                if let Err(error) = std::fs::write("out.html", &html) {
                    eprintln!("failed to write out.html: {error}");
                    std::process::exit(1);
                }
                println!(
                    "HTML 已写入 out.html (target: web, {} 字节)",
                    html.len()
                );
            }
            Err(error) => {
                eprintln!("codegen failed: {error}");
                std::process::exit(1);
            }
        }
        return;
    }

    let context = Context::create();
    let mut codegen = IosCodeGen::new_for(&context, platform);
    codegen.allow_degrade = allow_degrade;
    if let Err(e) = codegen.generate(&program) {
        eprintln!("codegen failed: {e}");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 递归组件的最小例子：`TreeRow` 把 `node.children` 交还给自己。
    const RECURSIVE_TREE_SOURCE: &str = r#"function TreeRow({ node }) {
        return <View style={{ flexDirection: "column" }}>
            <Text>{node.title}</Text>
            <TreeRow node={node.children} />
        </View>;
    }
    function App() {
        const tree = state([
            { title: "前端", children: [{ title: "渲染层" }] },
            { title: "后端" }
        ]);
        return <View>{tree.map((item) => <TreeRow node={item} />)}</View>;
    }"#;

    #[test]
    fn parses_scalar_state_and_native_input_events() {
        let program = parse_tsx_to_ast(
            r#"function App() {
                const note = state("");
                return <View>
                    <Text>{note}</Text>
                    <Input onInput={(value) => note = value} onFocus={() => {}} />
                </View>;
            }"#,
        );
        assert_eq!(program.states.len(), 1);
        assert_eq!(program.states[0].name, "note");
        assert_eq!(
            program.states[0].initial,
            StateInit::Scalar(StateValue::String(String::new()))
        );

        let IRNode::View { children, .. } = &program.nodes[0] else {
            panic!("expected root View");
        };
        assert!(matches!(children[0], IRNode::StateText { .. }));
        assert!(matches!(
            children[1],
            IRNode::Input {
                on_input: Some(0),
                on_focus: Some(1),
                ..
            }
        ));
    }

    #[test]
    fn supports_multiple_independent_states() {
        let program = parse_tsx_to_ast(
            r#"function App() {
                const count = state(0);
                const note = state("");
                const shown = state(false);
                return <View>
                    <Text>{count}</Text>
                    <Text>{note}</Text>
                    {shown && <Text>visible</Text>}
                    <Button onClick={() => count++}>Inc</Button>
                    <Button onClick={() => note = "hi"}>Set</Button>
                    <Button onClick={() => shown = true}>Show</Button>
                </View>;
            }"#,
        );
        assert!(program.errors.is_empty(), "{:?}", program.errors);
        assert_eq!(
            program.states.iter().map(|decl| decl.name.as_str()).collect::<Vec<_>>(),
            vec!["count", "note", "shown"]
        );
        assert_eq!(
            program.state_actions,
            vec![
                StateMutation::Increment("count".into()),
                StateMutation::Set("note".into(), StateValue::String("hi".into())),
                StateMutation::SetTrue("shown".into()),
            ]
        );

        let IRNode::View { children, .. } = &program.nodes[0] else {
            panic!("expected root View");
        };
        assert!(matches!(&children[0], IRNode::StateText { cell, .. } if cell == "count"));
        assert!(matches!(&children[1], IRNode::StateText { cell, .. } if cell == "note"));
        assert!(matches!(&children[2], IRNode::Conditional { cell, .. } if cell == "shown"));
    }

    #[test]
    fn expands_object_state_into_independent_cells() {
        let program = parse_tsx_to_ast(
            r#"function App() {
                const profile = state({ name: "", visits: 0, active: false });
                return <View>
                    <Text>{profile.name}</Text>
                    <Text>{profile.visits}</Text>
                    {profile.active && <Text>online</Text>}
                    <Button onClick={() => profile.visits++}>Visit</Button>
                    <Button onClick={() => profile.name = "Ada"}>Rename</Button>
                    <Input onInput={(value) => profile.name = value} />
                </View>;
            }"#,
        );
        assert!(program.errors.is_empty(), "{:?}", program.errors);
        assert_eq!(
            program.states[0].cells(),
            vec![
                ("profile.name".to_string(), StateValue::String(String::new())),
                ("profile.visits".to_string(), StateValue::Number(0)),
                ("profile.active".to_string(), StateValue::Boolean(false)),
            ]
        );
        assert_eq!(
            program.state_actions,
            vec![
                StateMutation::Increment("profile.visits".into()),
                StateMutation::Set("profile.name".into(), StateValue::String("Ada".into())),
                StateMutation::SetFromInput("profile.name".into()),
            ]
        );

        let IRNode::View { children, .. } = &program.nodes[0] else {
            panic!("expected root View");
        };
        assert!(matches!(&children[0], IRNode::StateText { cell, .. } if cell == "profile.name"));
        assert!(matches!(&children[2], IRNode::Conditional { cell, .. } if cell == "profile.active"));
    }

    #[test]
    fn each_cell_gets_its_own_storage_and_label_list() {
        let program = parse_tsx_to_ast(
            r#"function App() {
                const a = state(1);
                const b = state(2);
                return <View>
                    <Text>{a}</Text>
                    <Text>{b}</Text>
                    <Button onClick={() => a++}>IncA</Button>
                    <Button onClick={() => b++}>IncB</Button>
                </View>;
            }"#,
        );
        assert!(program.errors.is_empty(), "{:?}", program.errors);
        let context = Context::create();
        let mut codegen = IosCodeGen::new(&context);
        codegen.generate(&program).unwrap();
        let generated_ir = codegen.module.print_to_string().to_string();
        assert!(generated_ir.contains("g_state_cell_0"));
        assert!(generated_ir.contains("g_state_cell_1"));
        // 两个动作各自只刷新自己单元的标签
        assert!(generated_ir.contains("app_delegate_state_action_0"));
        assert!(generated_ir.contains("app_delegate_state_action_1"));
        assert!(generated_ir.contains("stateAction0:"));
        assert!(generated_ir.contains("stateAction1:"));
    }

    #[test]
    fn parses_boolean_and_numeric_signal_initial_values() {
        let program = parse_tsx_to_ast(
            r#"function App() {
                const focused = state(false);
                const count = state(4);
                return <View><Text>{focused}</Text><Text>{count}</Text></View>;
            }"#,
        );
        assert!(program.errors.is_empty(), "{:?}", program.errors);
        assert_eq!(
            program.states[0].initial,
            StateInit::Scalar(StateValue::Boolean(false))
        );
        assert_eq!(
            program.states[1].initial,
            StateInit::Scalar(StateValue::Number(4))
        );
    }

    #[test]
    fn compiles_decrement_and_scalar_assignment_actions() {
        let program = parse_tsx_to_ast(
            r#"function App() {
                const count = state(4);
                return <View>
                    <Button onClick={() => count--}>Down</Button>
                    <Button onClick={() => count = 12}>Set</Button>
                </View>;
            }"#,
        );
        assert_eq!(
            program.state_actions,
            vec![
                StateMutation::Decrement("count".into()),
                StateMutation::Set("count".into(), StateValue::Number(12)),
            ]
        );
    }

    #[test]
    fn generates_uikit_targets_for_signal_and_input_events() {
        let program = parse_tsx_to_ast(
            r#"function App() {
                const note = state("");
                const active = state(false);
                return <View>
                    <Text>{note}</Text>
                    <Text>{active}</Text>
                    <Button onClick={() => active = true}>Activate</Button>
                    <Input onInput={(value) => note = value} onFocus={() => active = true} />
                </View>;
            }"#,
        );
        let context = Context::create();
        let mut codegen = IosCodeGen::new(&context);
        codegen.generate(&program).unwrap();
        let generated_ir = codegen.module.print_to_string().to_string();
        assert!(generated_ir.contains("stateAction0:"));
        assert!(generated_ir.contains("stateAction1:"));
        assert!(generated_ir.contains("g_state_cell_0"));
        assert!(generated_ir.contains("g_state_label_0"));
        assert!(generated_ir.contains("g_state_label_1"));
    }

    #[test]
    fn expands_custom_components_with_scalar_props_and_lifecycle_hooks() {
        let program = parse_tsx_to_ast(
            r#"function Banner({ title, height }) {
                onMount(() => {});
                onUnmount(() => {});
                return <View style={{ height }}>
                    <Text>{title}</Text>
                </View>;
            }
            function App() {
                const shown = state(false);
                return <View>
                    <Text>before</Text>
                    {shown && <Banner title="Welcome" height={72} />}
                    <Text>after</Text>
                    <Button onClick={() => shown = true}>Show</Button>
                </View>;
            }"#,
        );
        assert!(program.errors.is_empty(), "{:?}", program.errors);
        assert_eq!(
            program.state_actions,
            vec![
                StateMutation::Noop,
                StateMutation::Noop,
                StateMutation::SetTrue("shown".into()),
            ]
        );

        let IRNode::View { children, .. } = &program.nodes[0] else {
            panic!("expected root View");
        };
        let IRNode::Conditional {
            when_true,
            initial,
            true_mount,
            true_unmount,
            ..
        } = &children[1]
        else {
            panic!("expected dynamic conditional");
        };
        assert!(!initial);
        assert_eq!((true_mount.as_slice(), true_unmount.as_slice()), (&[0][..], &[1][..]));
        let IRNode::View {
            style,
            children: banner_children,
        } = when_true.as_ref()
        else {
            panic!("expected component root View");
        };
        assert!(
            matches!(style.height, Some(Dimension::Points(72.0))),
            "expected height prop to resolve, got {style:?}"
        );
        assert!(matches!(
            &banner_children[0],
            IRNode::Text { content, .. } if content == "Welcome"
        ));
    }

    #[test]
    fn conditionals_mount_and_unmount_branch_views() {
        let program = parse_tsx_to_ast(
            r#"function App() {
                const shown = state(false);
                return <View style={{ width: 320, height: 600, spacing: 12 }}>
                    <Text>before</Text>
                    {shown ? <Text>visible</Text> : <Text>alternate</Text>}
                    <Text>after</Text>
                    <Button onClick={() => shown = true}>Show</Button>
                </View>;
            }"#,
        );
        assert!(program.errors.is_empty(), "{:?}", program.errors);
        let context = Context::create();
        let mut codegen = IosCodeGen::new(&context);
        codegen.generate(&program).unwrap();
        let generated_ir = codegen.module.print_to_string().to_string();
        // 真增删分支：卸载旧视图、挂载新视图并重建边约束，同时收缩槽位高度
        assert!(generated_ir.contains("removeFromSuperview"));
        assert!(generated_ir.contains("setConstant:"));
        assert!(generated_ir.contains("layoutIfNeeded"));
        assert!(!generated_ir.contains("insertArrangedSubview:atIndex:"));
    }

    #[test]
    fn conditionals_in_scroll_views_use_dynamic_content_layout() {
        let program = parse_tsx_to_ast(
            r#"function App() {
                const shown = state(false);
                return <ScrollView style={{ height: 600, spacing: 12 }}>
                    <Text>before</Text>
                    {shown && <View style={{ width: "100%", height: 120 }}>
                        <Text>dynamic content</Text>
                    </View>}
                    <Text>after</Text>
                    <Button onClick={() => shown = true}>Show</Button>
                </ScrollView>;
            }"#,
        );
        assert!(program.errors.is_empty(), "{:?}", program.errors);

        let context = Context::create();
        let mut codegen = IosCodeGen::new(&context);
        codegen.generate(&program).unwrap();
        let generated_ir = codegen.module.print_to_string().to_string();
        assert!(generated_ir.contains("contentLayoutGuide"));
        assert!(generated_ir.contains("frameLayoutGuide"));
        assert!(generated_ir.contains("removeFromSuperview"));
        assert!(!generated_ir.contains("insertArrangedSubview:atIndex:"));
    }

    #[test]
    fn fade_props_are_gone_and_on_scroll_replaces_them() {
        let program = parse_tsx_to_ast(
            r#"function App() {
                return <View>
                    <ScrollView style={{ fadeTarget: "topBar", fadeDistance: 90 }}>
                        <Text>Scrollable content</Text>
                    </ScrollView>
                </View>;
            }"#,
        );
        assert!(
            program
                .errors
                .iter()
                .any(|error| error.contains("fadeTarget/fadeStart/fadeDistance 已移除")),
            "{:?}",
            program.errors
        );

        // 替代写法：onScroll + clamp + opacity，同样生成滚动 delegate 与 setAlpha:
        let program = parse_tsx_to_ast(
            r#"function App() {
                const progress = state(0);
                return <View>
                    <View style={{ id: "topBar", backgroundColor: "rgb(20, 30, 40)", opacity: progress }}>
                        <Text style={{ color: "white" }}>Title</Text>
                    </View>
                    <ScrollView onScroll={(y) => progress = clamp(y, 0, 100)} style={{ height: 200 }}>
                        <Text>Scrollable content</Text>
                    </ScrollView>
                </View>;
            }"#,
        );
        assert!(program.errors.is_empty(), "{:?}", program.errors);
        let context = Context::create();
        let mut codegen = IosCodeGen::new(&context);
        codegen.generate(&program).unwrap();
        let generated_ir = codegen.module.print_to_string().to_string();
        assert!(generated_ir.contains("scrollViewDidScroll:"));
        assert!(generated_ir.contains("scroll_action_write"));
        assert!(generated_ir.contains("setAlpha:"));
        assert!(generated_ir.contains("scroll_clamped"));
    }

    #[test]
    fn scroll_content_size_grows_beyond_the_viewport_for_long_content() {
        let children = vec![
            IRNode::View {
                style: InlineStyle {
                    height: Some(Dimension::Points(160.0)),
                    ..InlineStyle::default()
                },
                children: vec![],
            },
            IRNode::View {
                style: InlineStyle {
                    height: Some(Dimension::Points(180.0)),
                    ..InlineStyle::default()
                },
                children: vec![],
            },
            IRNode::View {
                style: InlineStyle {
                    height: Some(Dimension::Points(190.0)),
                    ..InlineStyle::default()
                },
                children: vec![],
            },
        ];
        let scroll_style = InlineStyle {
            spacing: Some(12.0),
            ..InlineStyle::default()
        };
        let layouts = layout_flex_children(
            &children,
            &scroll_style,
            320.0,
            260.0,
            FlexDirection::Column,
            0.0,
        );
        let (_, content_height) = scroll_content_size(&children, &layouts, 320.0, 260.0);
        assert!(content_height > 260.0);
        assert_eq!(content_height, layouts[2].y + layouts[2].height);
    }

    #[test]
    fn scroll_fade_props_report_a_migration_error() {
        let program = parse_tsx_to_ast(
            r#"function App() {
                return <View>
                    <View style={{ id: "topBar", backgroundColor: "blue" }} />
                    <ScrollView style={{ fadeTarget: "topBar", fadeDistance: 0 }} />
                </View>;
            }"#,
        );
        assert!(
            program
                .errors
                .iter()
                .any(|error| error.contains("已移除")),
            "{:?}",
            program.errors
        );
    }

    #[test]
    fn top_bar_can_hug_the_physical_top_edge() {
        let program = parse_tsx_to_ast(
            r#"function App() {
                return <View style={{ width: "100%", height: "100%" }}>
                    <View style={{ ignoresSafeArea: "top", width: "100%", height: 44, backgroundColor: "blue" }}>
                        <Text>Title</Text>
                    </View>
                    <ScrollView style={{ flexGrow: 1 }}>
                        <Text>content</Text>
                    </ScrollView>
                </View>;
            }"#,
        );
        assert!(program.errors.is_empty(), "{:?}", program.errors);
        let IRNode::View { children, .. } = &program.nodes[0] else {
            panic!("expected root View");
        };
        let IRNode::View { style, .. } = &children[0] else {
            panic!("expected top bar View");
        };
        assert!(style.ignores_safe_area_top);

        let context = Context::create();
        let mut codegen = IosCodeGen::new(&context);
        codegen.generate(&program).unwrap();
        let generated_ir = codegen.module.print_to_string().to_string();
        assert!(generated_ir.contains("safeAreaLayoutGuide"));
    }

    #[test]
    fn button_padding_becomes_content_edge_insets() {
        let program = parse_tsx_to_ast(
            r#"function App() {
                return <View style={{ width: 320, height: 600 }}>
                    <Button title="Show" style={{ width: 140, height: 48, padding: 10 }} />
                </View>;
            }"#,
        );
        assert!(program.errors.is_empty(), "{:?}", program.errors);
        let context = Context::create();
        let mut codegen = IosCodeGen::new(&context);
        codegen.generate(&program).unwrap();
        let generated_ir = codegen.module.print_to_string().to_string();
        assert!(generated_ir.contains("setContentEdgeInsets:"));
    }

    #[test]
    fn padding_shorthand_yields_to_per_edge_values() {
        let style = InlineStyle {
            padding: Some(10.0),
            padding_top: Some(2.0),
            padding_horizontal: Some(4.0),
            ..InlineStyle::default()
        };
        assert_eq!(
            node_padding(&style),
            Padding { top: 2.0, right: 4.0, bottom: 10.0, left: 4.0 }
        );
    }

    #[test]
    fn horizontal_padding_only_insets_the_horizontal_axis() {
        let children = vec![IRNode::View {
            style: InlineStyle {
                width: Some(Dimension::Percent(1.0)),
                height: Some(Dimension::Points(40.0)),
                ..InlineStyle::default()
            },
            children: vec![],
        }];
        let style = InlineStyle {
            padding_horizontal: Some(16.0),
            flex_direction: Some(FlexDirection::Column),
            ..InlineStyle::default()
        };
        let layouts = layout_flex_children(
            &children,
            &style,
            320.0,
            200.0,
            FlexDirection::Column,
            0.0,
        );
        assert_eq!(layouts[0].x, 16.0);
        assert_eq!(layouts[0].y, 0.0);
        assert_eq!(layouts[0].width, 288.0);
    }

    #[test]
    fn padded_text_wraps_the_label_so_the_background_covers_the_padding() {
        let generate = |padding: &str| {
            let program = parse_tsx_to_ast(&format!(
                r#"function App() {{
                    return <View style={{{{ width: 320, height: 600 }}}}>
                        <Text style={{{{ fontSize: 14{padding} }}}}>Hello</Text>
                    </View>;
                }}"#
            ));
            assert!(program.errors.is_empty(), "{:?}", program.errors);
            let context = Context::create();
            let mut codegen = IosCodeGen::new(&context);
            codegen.generate(&program).unwrap();
            codegen.module.print_to_string().to_string()
        };

        let plain = generate("");
        let padded = generate(r#", padding: 8, backgroundColor: "rgb(255, 226, 0)""#);
        let anchor_constraints = |ir: &str| ir.matches("constraintEqualToAnchor:constant:").count();
        assert!(
            anchor_constraints(&padded) > anchor_constraints(&plain),
            "a padded label should be pinned inside an extra container"
        );
    }

    #[test]
    fn rejects_ignores_safe_area_without_fixed_height() {
        let program = parse_tsx_to_ast(
            r#"function App() {
                return <View>
                    <View style={{ ignoresSafeArea: "top", backgroundColor: "blue" }}>
                        <Text>Title</Text>
                    </View>
                </View>;
            }"#,
        );
        let context = Context::create();
        let mut codegen = IosCodeGen::new(&context);
        assert!(codegen
            .generate(&program)
            .unwrap_err()
            .contains("requires an explicit height"));
    }

    #[test]
    fn text_intrinsic_estimates_keep_daily_goal_items_apart() {
        let children = vec![
            IRNode::Text {
                content: "Daily goal".into(),
                style: InlineStyle {
                    font_size: Some(16.0),
                    ..InlineStyle::default()
                },
            },
            IRNode::Text {
                content: "You are making great progress".into(),
                style: InlineStyle {
                    font_size: Some(14.0),
                    ..InlineStyle::default()
                },
            },
        ];
        let layouts = layout_flex_children(
            &children,
            &InlineStyle {
                height: Some(Dimension::Points(86.0)),
                padding: Some(16.0),
                flex_direction: Some(FlexDirection::Column),
                justify_content: Some(JustifyContent::SpaceBetween),
                ..InlineStyle::default()
            },
            300.0,
            86.0,
            FlexDirection::Column,
            0.0,
        );

        assert!((layouts[0].height - 19.2).abs() < 1e-6);
        assert!((layouts[1].height - 16.8).abs() < 1e-6);
        assert!(layouts[1].y >= layouts[0].y + layouts[0].height);
    }

    #[test]
    fn native_controls_use_intrinsic_dimensions_unless_explicitly_sized() {
        let text = IRNode::Text {
            content: "UIKit measured".into(),
            style: InlineStyle::default(),
        };
        assert!(node_uses_intrinsic_dimension(&text, false));
        assert!(node_uses_intrinsic_dimension(&text, true));

        let sized_text = IRNode::Text {
            content: "Fixed".into(),
            style: InlineStyle {
                height: Some(Dimension::Points(24.0)),
                ..InlineStyle::default()
            },
        };
        assert!(!node_uses_intrinsic_dimension(&sized_text, true));
        assert!(node_uses_intrinsic_dimension(&sized_text, false));

        let button = IRNode::Button {
            title: "Continue".into(),
            style: InlineStyle::default(),
            state_action: None,
            unsupported_handler: false,
        };
        assert!(node_uses_intrinsic_dimension(&button, false));
        assert!(node_uses_intrinsic_dimension(&button, true));
    }

    #[test]
    fn flex_growing_scroll_view_uses_remaining_viewport_height() {
        let children = vec![
            IRNode::View {
                style: InlineStyle {
                    height: Some(Dimension::Points(72.0)),
                    ..InlineStyle::default()
                },
                children: vec![],
            },
            IRNode::ScrollView {
                style: InlineStyle {
                    flex_grow: 1.0,
                    ..InlineStyle::default()
                },
                children: vec![IRNode::Text {
                    content: "long content".into(),
                    style: InlineStyle {
                        height: Some(Dimension::Points(600.0)),
                        ..InlineStyle::default()
                    },
                }],
            },
        ];
        let layouts = layout_flex_children(
            &children,
            &InlineStyle {
                flex_direction: Some(FlexDirection::Column),
                spacing: Some(14.0),
                ..InlineStyle::default()
            },
            350.0,
            700.0,
            FlexDirection::Column,
            0.0,
        );

        assert_eq!(layouts[0].height, 72.0);
        assert_eq!(layouts[1].y, 86.0);
        assert_eq!(layouts[1].height, 614.0);
    }

    #[test]
    fn flex_child_percentage_width_is_based_on_inner_parent_width() {
        let child = IRNode::View {
            style: InlineStyle {
                width: Some(Dimension::Percent(1.0)),
                height: Some(Dimension::Points(40.0)),
                ..InlineStyle::default()
            },
            children: vec![],
        };
        let layouts = layout_flex_children(
            &[child],
            &InlineStyle {
                flex_direction: Some(FlexDirection::Column),
                padding: Some(16.0),
                ..InlineStyle::default()
            },
            350.0,
            100.0,
            FlexDirection::Column,
            0.0,
        );

        assert_eq!(layouts[0].x, 16.0);
        assert_eq!(layouts[0].width, 318.0);
    }

    #[test]
    fn absolute_children_do_not_take_space_in_flex_flow() {
        let children = vec![
            IRNode::Text {
                content: "flow".into(),
                style: InlineStyle {
                    width: Some(Dimension::Points(40.0)),
                    height: Some(Dimension::Points(30.0)),
                    ..InlineStyle::default()
                },
            },
            IRNode::View {
                style: InlineStyle {
                    position: Position::Absolute,
                    left: Some(Dimension::Points(10.0)),
                    right: Some(Dimension::Points(10.0)),
                    bottom: Some(Dimension::Points(5.0)),
                    height: Some(Dimension::Points(20.0)),
                    ..InlineStyle::default()
                },
                children: vec![],
            },
            IRNode::Text {
                content: "flow2".into(),
                style: InlineStyle {
                    width: Some(Dimension::Points(40.0)),
                    height: Some(Dimension::Points(30.0)),
                    ..InlineStyle::default()
                },
            },
        ];
        let layouts = layout_flex_children(
            &children,
            &InlineStyle {
                flex_direction: Some(FlexDirection::Column),
                spacing: Some(5.0),
                ..InlineStyle::default()
            },
            200.0,
            120.0,
            FlexDirection::Column,
            0.0,
        );

        assert_eq!(layouts[0].y, 0.0);
        assert_eq!(layouts[2].y, 35.0);
        assert_eq!(layouts[1], ChildLayout {
            x: 10.0,
            y: 95.0,
            width: 180.0,
            height: 20.0,
        });
    }

    #[test]
    fn parses_relative_and_absolute_position_styles() {
        let program = parse_tsx_to_ast(
            r#"function App() { return <View><Text>A</Text><Text style={{ position: "absolute", top: 12, right: "10%" }}>B</Text></View> }"#,
        );
        let IRNode::View { children, .. } = &program.nodes[0] else {
            panic!("expected View node");
        };
        assert_eq!(node_style(&children[0]).position, Position::Relative);
        assert_eq!(node_style(&children[1]).position, Position::Absolute);
        assert!(matches!(node_style(&children[1]).top, Some(Dimension::Points(12.0))));
        assert!(matches!(node_style(&children[1]).right, Some(Dimension::Percent(0.1))));
    }

    #[test]
    fn parses_margin_shorthand_and_directional_overrides() {
        let program = parse_tsx_to_ast(
            r#"function App() { return <View><Text style={{ margin: 10, marginHorizontal: 20, marginTop: 5, marginRight: 7 }}>A</Text></View> }"#,
        );
        let IRNode::View { children, .. } = &program.nodes[0] else {
            panic!("expected View node");
        };
        assert_eq!(
            node_margins(&children[0]),
            Margins {
                top: 5.0,
                right: 7.0,
                bottom: 10.0,
                left: 20.0,
            }
        );
    }

    #[test]
    fn margins_participate_in_flex_item_positions() {
        let children = vec![
            IRNode::Text {
                content: "A".into(),
                style: InlineStyle {
                    width: Some(Dimension::Points(40.0)),
                    height: Some(Dimension::Points(20.0)),
                    margin_left: Some(10.0),
                    margin_right: Some(20.0),
                    margin_top: Some(6.0),
                    ..InlineStyle::default()
                },
            },
            IRNode::Text {
                content: "B".into(),
                style: InlineStyle {
                    width: Some(Dimension::Points(40.0)),
                    height: Some(Dimension::Points(20.0)),
                    margin_horizontal: Some(5.0),
                    margin_bottom: Some(10.0),
                    ..InlineStyle::default()
                },
            },
        ];
        let layouts = layout_flex_children(
            &children,
            &InlineStyle {
                flex_direction: Some(FlexDirection::Row),
                spacing: Some(8.0),
                align_items: Some(AlignItems::Start),
                ..InlineStyle::default()
            },
            200.0,
            100.0,
            FlexDirection::Column,
            0.0,
        );

        assert_eq!(layouts[0].x, 10.0);
        assert_eq!(layouts[1].x, 83.0);
        assert_eq!(layouts[0].y, 6.0);
        assert_eq!(layouts[1].y, 0.0);
    }

    #[test]
    fn parses_image_modes_with_scale_to_fill_as_default() {
        let program = parse_tsx_to_ast(
            r#"function App() { return <View><Image src="logo" /><Image src="logo" mode="aspectFit" /><Image src="logo" mode="aspectFill" /><Image src="logo" mode="widthFix" /><Image src="logo" mode="heightFix" /></View> }"#,
        );
        let IRNode::View { children, .. } = &program.nodes[0] else {
            panic!("expected View node");
        };
        let modes: Vec<_> = children
            .iter()
            .map(|child| match child {
                IRNode::Image { mode, .. } => *mode,
                _ => panic!("expected Image node"),
            })
            .collect();
        assert_eq!(
            modes,
            [
                ImageMode::ScaleToFill,
                ImageMode::AspectFit,
                ImageMode::AspectFill,
                ImageMode::WidthFix,
                ImageMode::HeightFix,
            ]
        );
    }

    #[test]
    fn parses_border_styles_and_rgba_colors() {
        let program = parse_tsx_to_ast(
            r#"function App() { return <View style={{ borderWidth: 2, borderColor: "rgba(255, 0, 128, 0.5)", borderRadius: 12, backgroundColor: "rgba(100%, 0%, 50%, 25%)" }} /> }"#,
        );
        let IRNode::View { style, .. } = &program.nodes[0] else {
            panic!("expected View node");
        };
        assert_eq!(style.border_width, Some(2.0));
        assert_eq!(style.border_color.as_deref(), Some("rgba(255, 0, 128, 0.5)"));
        assert_eq!(style.border_radius, Some(12.0));
        assert_eq!(
            parse_functional_color(style.border_color.as_deref().unwrap()).unwrap(),
            Some((1.0, 0.0, 128.0 / 255.0, 0.5))
        );
        assert_eq!(
            parse_functional_color(style.background_color.as_deref().unwrap()).unwrap(),
            Some((1.0, 0.0, 0.5, 0.25))
        );
    }

    #[test]
    fn parses_rgb_colors_with_numeric_and_percentage_channels() {
        assert_eq!(
            parse_functional_color("rgb(255, 128, 0)").unwrap(),
            Some((1.0, 128.0 / 255.0, 0.0, 1.0))
        );
        assert_eq!(
            parse_functional_color("rgb(100%, 50%, 0%)").unwrap(),
            Some((1.0, 0.5, 0.0, 1.0))
        );
    }

    #[test]
    fn rejects_invalid_rgba_components() {
        assert!(parse_functional_color("rgba(256, 0, 0, 1)").is_err());
        assert!(parse_functional_color("rgba(0, 0, 0, 1.5)").is_err());
        assert!(parse_functional_color("rgba(0, 0, 0)").is_err());
        assert!(parse_functional_color("rgb(0, 0, 0, 1)").is_err());
    }

    #[test]
    fn parses_percentage_width_and_height() {
        let program = parse_tsx_to_ast(
            r#"function App() { return <View style={{ width: "80%", height: "25%" }} /> }"#,
        );
        let IRNode::View { style, .. } = &program.nodes[0] else {
            panic!("expected View node");
        };
        assert!(matches!(style.width, Some(Dimension::Percent(value)) if value == 0.8));
        assert!(matches!(style.height, Some(Dimension::Percent(value)) if value == 0.25));
    }

    #[test]
    fn resolves_percentages_against_parent_size() {
        assert_eq!(Dimension::Percent(0.75).resolve(400.0), 300.0);
    }

    #[test]
    fn parses_flex_container_and_child_properties() {
        let program = parse_tsx_to_ast(
            r#"function App() { return <View style={{ flexDirection: "row", justifyContent: "space-between", alignItems: "center" }}><Text style={{ flexGrow: 1, textAlign: "center" }}>A</Text></View> }"#,
        );
        let IRNode::View { style, children } = &program.nodes[0] else {
            panic!("expected View node");
        };
        assert_eq!(style.flex_direction, Some(FlexDirection::Row));
        assert_eq!(style.justify_content, Some(JustifyContent::SpaceBetween));
        assert_eq!(style.align_items, Some(AlignItems::Center));
        assert_eq!(node_style(&children[0]).flex_grow, 1.0);
        assert_eq!(
            node_style(&children[0]).text_align,
            Some(TextAlignment::Center)
        );
    }

    #[test]
    fn flex_grow_distributes_free_space_on_main_axis() {
        let children = vec![
            IRNode::Text {
                content: "A".into(),
                style: InlineStyle {
                    width: Some(Dimension::Points(50.0)),
                    height: Some(Dimension::Points(20.0)),
                    flex_grow: 1.0,
                    ..InlineStyle::default()
                },
            },
            IRNode::Text {
                content: "B".into(),
                style: InlineStyle {
                    width: Some(Dimension::Points(50.0)),
                    height: Some(Dimension::Points(20.0)),
                    flex_grow: 1.0,
                    ..InlineStyle::default()
                },
            },
        ];
        let layouts = layout_flex_children(
            &children,
            &InlineStyle {
                flex_direction: Some(FlexDirection::Row),
                ..InlineStyle::default()
            },
            300.0,
            100.0,
            FlexDirection::Column,
            0.0,
        );
        assert_eq!(layouts[0], ChildLayout { x: 0.0, y: 0.0, width: 150.0, height: 20.0 });
        assert_eq!(layouts[1], ChildLayout { x: 150.0, y: 0.0, width: 150.0, height: 20.0 });
    }

    #[test]
    fn justify_content_distributes_two_children_on_the_main_axis() {
        let children = vec![
            IRNode::Text {
                content: "A".into(),
                style: InlineStyle {
                    width: Some(Dimension::Points(50.0)),
                    height: Some(Dimension::Points(20.0)),
                    ..InlineStyle::default()
                },
            },
            IRNode::Text {
                content: "B".into(),
                style: InlineStyle {
                    width: Some(Dimension::Points(50.0)),
                    height: Some(Dimension::Points(20.0)),
                    ..InlineStyle::default()
                },
            },
        ];
        let expected = [
            (JustifyContent::Start, [0.0, 50.0]),
            (JustifyContent::Center, [100.0, 150.0]),
            (JustifyContent::End, [200.0, 250.0]),
            (JustifyContent::SpaceBetween, [0.0, 250.0]),
            (JustifyContent::SpaceAround, [50.0, 200.0]),
        ];
        for (justify_content, expected_x) in expected {
            let layouts = layout_flex_children(
                &children,
                &InlineStyle {
                    flex_direction: Some(FlexDirection::Row),
                    justify_content: Some(justify_content),
                    ..InlineStyle::default()
                },
                300.0,
                100.0,
                FlexDirection::Column,
                0.0,
            );
            assert_eq!([layouts[0].x, layouts[1].x], expected_x);
        }
    }

    #[test]
    fn space_around_distributes_free_space_with_half_sized_outer_gaps() {
        let children = vec![
            IRNode::Text {
                content: "A".into(),
                style: InlineStyle {
                    width: Some(Dimension::Points(40.0)),
                    height: Some(Dimension::Points(20.0)),
                    ..InlineStyle::default()
                },
            },
            IRNode::Text {
                content: "B".into(),
                style: InlineStyle {
                    width: Some(Dimension::Points(60.0)),
                    height: Some(Dimension::Points(20.0)),
                    ..InlineStyle::default()
                },
            },
            IRNode::Text {
                content: "C".into(),
                style: InlineStyle {
                    width: Some(Dimension::Points(40.0)),
                    height: Some(Dimension::Points(20.0)),
                    ..InlineStyle::default()
                },
            },
        ];
        let layouts = layout_flex_children(
            &children,
            &InlineStyle {
                flex_direction: Some(FlexDirection::Row),
                justify_content: Some(JustifyContent::SpaceAround),
                padding: Some(10.0),
                spacing: Some(8.0),
                ..InlineStyle::default()
            },
            300.0,
            100.0,
            FlexDirection::Column,
            0.0,
        );

        let expected_x = [30.666_666_666_666_668, 120.0, 229.333_333_333_333_34];
        for (layout, expected) in layouts.iter().zip(expected_x) {
            assert!((layout.x - expected).abs() < 1e-6);
        }

        let expected_center_offsets = [-99.333_333_333_333_33, 0.0, 99.333_333_333_333_34];
        for (layout, expected) in layouts.iter().zip(expected_center_offsets) {
            let center_offset = layout.x + layout.width / 2.0 - 300.0 / 2.0;
            assert!((center_offset - expected).abs() < 1e-6);
        }
    }

    #[test]
    fn space_around_centers_a_single_child() {
        let child = IRNode::Text {
            content: "A".into(),
            style: InlineStyle {
                width: Some(Dimension::Points(80.0)),
                height: Some(Dimension::Points(20.0)),
                ..InlineStyle::default()
            },
        };
        let layouts = layout_flex_children(
            &[child],
            &InlineStyle {
                flex_direction: Some(FlexDirection::Row),
                justify_content: Some(JustifyContent::SpaceAround),
                ..InlineStyle::default()
            },
            300.0,
            100.0,
            FlexDirection::Column,
            0.0,
        );

        assert_eq!(layouts[0].x, 110.0);
    }

    #[test]
    fn android_backend_emits_jni_entries_and_state_units() {
        let program = parse_tsx_to_ast(
            r#"function App() {
                const shown = state(false);
                const profile = state({ name: "guest", visits: 0 });
                return <View style={{ flexDirection: "column", justifyContent: "space-between" }}>
                    <View style={{ id: "topBar", backgroundColor: "rgb(20, 30, 40)" }}>
                        <Text>Title</Text>
                    </View>
                    <ScrollView style={{ height: 320 }}>
                        {shown && <Text>shown</Text>}
                        <Text>{profile.name}</Text>
                        <Button title="Rename" onClick={() => profile.name = "Ada"} />
                        <Button title="Visit" onClick={() => profile.visits++} />
                        <View style={{ left: 16, right: 16, bottom: 10, height: 64 }}>
                            <Input onInput={(value) => profile.name = value} />
                        </View>
                    </ScrollView>
                </View>;
            }"#,
        );
        assert!(program.errors.is_empty(), "{:?}", program.errors);

        let context = Context::create();
        let codegen = IosCodeGen::new_for(&context, Platform::Android);
        android::emit(&codegen, &program).unwrap();
        let ir = codegen.module.print_to_string().to_string();

        for entry in [
            "Java_com_lumen_app_MainActivity_nativeCreate",
            "Java_com_lumen_app_MainActivity_nativeOnClick",
            "Java_com_lumen_app_MainActivity_nativeOnInput",
            "Java_com_lumen_app_MainActivity_nativeOnScroll",
        ] {
            assert!(ir.contains(entry), "missing {entry}");
        }
        // 每个状态单元一套存储 + 刷新函数，条件分支有自己的开关
        assert!(ir.contains("@g_cell_0 = "));
        assert!(ir.contains("@g_cell_1_text = "));
        assert!(ir.contains("@g_cell_2 = "));
        assert!(ir.contains("define void @lumen_refresh_1()"));
        assert!(ir.contains("define void @lumen_toggle_0()"));
        assert!(ir.contains("@g_cond_0_slot = "));
        // 渐显目标与滚动回调都接上了
        assert!(ir.contains("@g_fade_view = "));
        assert!(ir.contains("setOnScrollChangeListener"));
        // 类 / 方法 ID 走 JNI 查找表，不再有 ObjC 运行时调用
        assert!(!ir.contains("objc_msgSend"));
        assert!(!ir.contains("UIApplicationMain"));
    }

    #[test]
    fn conditional_updates_use_the_matching_slot_globals() {
        // 多个条件时必须按“真实序号”取 g_conditional_height_<n>；
        // 用过滤后的序号会让第二个条件的高度约束指向第一个条件（卡片挂上但高度为 0）。
        let program = parse_tsx_to_ast(
            r#"function App() {
                const a = state(false);
                const b = state(false);
                return <View>
                    {a && <Text>a</Text>}
                    {b && <Text>b</Text>}
                    <Button title="A" onClick={() => a = true} />
                    <Button title="B" onClick={() => b = true} />
                </View>;
            }"#,
        );
        assert!(program.errors.is_empty(), "{:?}", program.errors);

        let context = Context::create();
        let mut codegen = IosCodeGen::new(&context);
        codegen.generate(&program).unwrap();
        let ir = codegen.module.print_to_string().to_string();

        let body_of = |name: &str| -> String {
            let start = ir.find(&format!("define void @{name}")).expect("action exists");
            let rest = &ir[start..];
            let end = rest[1..].find("\ndefine ").map_or(rest.len(), |offset| offset + 1);
            rest[..end].to_string()
        };
        let second = body_of("app_delegate_state_action_1");
        assert!(
            second.contains("@g_conditional_height_1"),
            "第二个条件应该用 g_conditional_height_1，实际:\n{second}"
        );
        assert!(
            !second.contains("@g_conditional_height_0"),
            "不应该动到第一个条件的高度约束"
        );
    }

    #[test]
    fn macos_input_binding_uses_text_field_delegate() {
        let program = parse_tsx_to_ast(
            r#"function App() {
                const note = state("");
                return <View>
                    <Text>{note}</Text>
                    <Input onInput={(value) => note = value} />
                </View>;
            }"#,
        );
        assert!(program.errors.is_empty(), "{:?}", program.errors);

        let context = Context::create();
        let mut codegen = IosCodeGen::new_for(&context, Platform::MacOs);
        codegen.generate(&program).unwrap();
        let ir = codegen.module.print_to_string().to_string();

        // AppKit 没有 UIControl 事件：走 NSTextField 的 delegate 回调，取值用 stringValue
        assert!(ir.contains("setDelegate:"));
        assert!(ir.contains("controlTextDidChange:"));
        assert!(ir.contains("app_delegate_control_text_did_change"));
        assert!(ir.contains("stringValue"));
        assert!(ir.contains("@g_input_source_0 = "));
    }

    #[test]
    fn placeholder_color_is_style_driven_on_every_backend() {
        let program = parse_tsx_to_ast(
            r#"function App() {
                const note = state("");
                return <View>
                    <Input placeholder="default hint" onInput={(value) => note = value} />
                    <Input
                        placeholder="custom hint"
                        onInput={(value) => note = value}
                        style={{ placeholderColor: "rgb(1, 2, 3)" }}
                    />
                </View>;
            }"#,
        );
        assert!(program.errors.is_empty(), "{:?}", program.errors);

        // 样式里的 placeholderColor 进 IR；不写时留 None，由后端用统一的默认色
        let IRNode::View { children, .. } = &program.nodes[0] else {
            panic!("expected root View");
        };
        let colors: Vec<_> = children
            .iter()
            .filter_map(|child| match child {
                IRNode::Input { style, .. } => Some(style.placeholder_color.clone()),
                _ => None,
            })
            .collect();
        assert_eq!(colors, vec![None, Some("rgb(1, 2, 3)".to_string())]);

        let context = Context::create();
        let mut ios = IosCodeGen::new(&context);
        ios.generate(&program).unwrap();
        let ir = ios.module.print_to_string().to_string();
        assert!(ir.contains("setAttributedPlaceholder:"));

        let context = Context::create();
        let android_codegen = IosCodeGen::new_for(&context, Platform::Android);
        android::emit(&android_codegen, &program).unwrap();
        let android_ir = android_codegen.module.print_to_string().to_string();
        assert!(android_ir.contains("setHintTextColor"));
    }

    #[test]
    fn capability_check_rejects_unsupported_features_unless_degraded() {
        // 用 onScroll 举例：iOS / Android 支持，macOS 还没接滚动回调
        let program = parse_tsx_to_ast(
            r#"function App() {
                const offset = state(0);
                return <View>
                    <Text>{offset}</Text>
                    <ScrollView onScroll={(y) => offset = y} style={{ height: 200 }}>
                        <Text>content</Text>
                    </ScrollView>
                </View>;
            }"#,
        );
        assert!(program.errors.is_empty(), "{:?}", program.errors);

        // macOS 后端还没有滚动回调：默认应该编译失败并点名 onScroll
        let error = capabilities::check(&capabilities::MACOS, &program, false)
            .expect_err("macOS 应该拒绝 onScroll");
        assert!(error.contains("onScroll"), "{error}");

        // 加上 --allow-degrade 后降级成 warning
        let warnings =
            capabilities::check(&capabilities::MACOS, &program, true).expect("降级后应该能编译");
        assert!(
            warnings.iter().any(|warning| warning.contains("onScroll")),
            "{warnings:?}"
        );

        // iOS 支持，应当没有 warning
        let warnings = capabilities::check(&capabilities::IOS, &program, false).unwrap();
        assert!(warnings.is_empty(), "{warnings:?}");
    }

    #[test]
    fn web_backend_emits_styled_html_for_the_bundled_apps() {
        for (name, source) in APPS {
            let program = parse_tsx_to_ast(source);
            assert!(program.errors.is_empty(), "{name}: {:?}", program.errors);
            let html = source_backend::generate_html(&program).unwrap_or_else(|error| {
                panic!("{name}: {error}");
            });
            assert!(html.contains("<!DOCTYPE html>"), "{name}");
            assert!(html.contains("display: flex"), "{name}");
            // 占位符颜色统一走 ::placeholder 规则
            if html.contains("<input") {
                assert!(html.contains("::placeholder"), "{name} 缺少 placeholder 规则");
            }
            // 结构必须闭合
            assert_eq!(
                html.matches("<div").count() > 0,
                true,
                "{name} 应该有 div 容器"
            );
        }
    }

    #[test]
    fn bundled_apps_compile_for_every_backend() {
        for (name, source) in APPS {
            let program = parse_tsx_to_ast(source);
            assert!(program.errors.is_empty(), "{name}: {:?}", program.errors);
            for platform in [Platform::Ios, Platform::MacOs, Platform::Android] {
                let context = Context::create();
                let mut codegen = IosCodeGen::new_for(&context, platform);
                // 两个示例都用了 macOS 暂未支持的滚动渐显，按降级模式编译
                codegen.allow_degrade = true;
                codegen
                    .generate(&program)
                    .unwrap_or_else(|error| panic!("{name} / {platform:?}: {error}"));
            }
        }
    }

    #[test]
    fn todo_app_expands_object_state_into_independent_cells() {
        let source = APPS
            .iter()
            .find(|(name, _)| *name == "todo")
            .map(|(_, source)| *source)
            .expect("todo app is bundled");
        let program = parse_tsx_to_ast(source);
        assert!(program.errors.is_empty(), "{:?}", program.errors);

        // 4 个标题（字符串）+ 4 个勾选（布尔）+ 番茄计数 + 归档开关
        let cells = state_cells(&program.states);
        let paths: Vec<&str> = cells.iter().map(|(path, _)| path.as_str()).collect();
        assert_eq!(
            paths,
            [
                "titles.a",
                "titles.b",
                "titles.c",
                "titles.d",
                "done.a",
                "done.b",
                "done.c",
                "done.d",
                "pomodoro",
                "archived",
            ]
        );

        let context = Context::create();
        let codegen = IosCodeGen::new_for(&context, Platform::Android);
        android::emit(&codegen, &program).unwrap();
        let ir = codegen.module.print_to_string().to_string();
        // 四个勾选各自一个条件开关，列表里每项都能独立增删
        for index in 0..4 {
            assert!(ir.contains(&format!("@g_cond_{index}_slot = ")), "conditional {index}");
        }
        assert!(ir.contains("@g_cell_0_text = "), "titles.a 用字符串单元");
        assert!(ir.contains("@g_cell_8 = "), "pomodoro 用数值单元");
    }

    #[test]
    fn parses_multi_statement_handlers_as_sequences() {
        let program = parse_tsx_to_ast(
            r#"function App() {
                const shown = state(false);
                const count = state(0);
                return <View>
                    <Button onClick={() => { shown = true; count++ }}>Run</Button>
                </View>;
            }"#,
        );
        assert!(program.errors.is_empty(), "{:?}", program.errors);
        assert_eq!(
            program.state_actions,
            vec![StateMutation::Sequence(vec![
                StateMutation::SetTrue("shown".into()),
                StateMutation::Increment("count".into()),
            ])]
        );
    }

    #[test]
    fn renders_array_state_as_list_rows() {
        let program = parse_tsx_to_ast(
            r#"function App() {
                const items = state([
                    { id: 1, title: "写周报", done: false },
                    { id: 2, title: "评审 PR", done: false }
                ]);
                const draft = state("");
                return <View>
                    <Input onInput={(value) => draft = value} />
                    <Button onClick={() => { items.push({ id: 3, title: draft, done: false }); draft = "" }}>
                        添加
                    </Button>
                    {items.map((item, index) => (
                        <View>
                            <Text>{item.title}</Text>
                            <Button onClick={() => items.removeAt(index)}>删除</Button>
                        </View>
                    ))}
                </View>;
            }"#,
        );
        assert!(program.errors.is_empty(), "{:?}", program.errors);
        // 列表状态不展开成标量单元
        assert_eq!(
            state_cells(&program.states),
            vec![
                ("items.length".to_string(), StateValue::Number(2)),
                ("draft".to_string(), StateValue::String(String::new())),
            ]
        );
        assert!(matches!(program.states[0].initial, StateInit::List(_)));
        assert_eq!(
            program.state_actions,
            vec![
                StateMutation::SetFromInput("draft".into()),
                StateMutation::Sequence(vec![
                    StateMutation::PushItem {
                        list: "items".into(),
                        values: vec![
                            ("id".into(), ItemValue::Literal(StateValue::Number(3))),
                            ("title".into(), ItemValue::Cell("draft".into())),
                            ("done".into(), ItemValue::Literal(StateValue::Boolean(false))),
                        ],
                    },
                    StateMutation::Set("draft".into(), StateValue::String(String::new())),
                ]),
                StateMutation::RemoveCurrentItem {
                    list: "items".into()
                },
            ]
        );

        let IRNode::View { children, .. } = &program.nodes[0] else {
            panic!("expected root View");
        };
        let list = children
            .iter()
            .find(|child| matches!(child, IRNode::List { .. }))
            .expect("list node");
        let IRNode::List { item, rows, .. } = list else {
            unreachable!()
        };
        assert_eq!(rows.len(), 2);
        assert!(matches!(
            item.as_ref(),
            IRNode::View { children, .. }
                if children.iter().any(|child| matches!(child, IRNode::ItemText { field, .. } if field == "title"))
        ));
    }

    #[test]
    fn list_codegen_emits_row_storage_and_refresh() {
        let program = parse_tsx_to_ast(
            r#"function App() {
                const items = state([{ id: 1, title: "写周报", done: false }]);
                const draft = state("");
                return <View>
                    <Input onInput={(value) => draft = value} />
                    <Button onClick={() => items.push({ id: 2, title: draft, done: false })}>添加</Button>
                    {items.map((item, index) => (
                        <View>
                            <Text>{item.title}</Text>
                            <Button onClick={() => items.removeAt(index)}>删除</Button>
                        </View>
                    ))}
                </View>;
            }"#,
        );
        assert!(program.errors.is_empty(), "{:?}", program.errors);
        let context = Context::create();
        let mut codegen = IosCodeGen::new(&context);
        codegen.generate(&program).unwrap();
        let ir = codegen.module.print_to_string().to_string();
        // 数据列按字段存放，刷新函数按行数显示/隐藏
        assert!(ir.contains("@g_list_items_0_data_title = "), "title 列");
        assert!(ir.contains("@g_list_items_0_count = "), "行数");
        assert!(ir.contains("define void @list_rebuild_items_0()"), "重建函数");
        assert!(ir.contains("define ptr @list_row_items_0(i64"), "行构造函数");
        // 行内按钮用 tag 记住行号
        assert!(ir.contains("\"setTag:\"") || ir.contains("setTag:"), "行内按钮 tag");
    }

    #[test]
    fn unsupported_state_shapes_are_reported_instead_of_silently_dropped() {
        let array_field = parse_tsx_to_ast(
            r#"function App() {
                const profile = state({ name: "", tags: [] });
                return <View><Text>{profile.name}</Text></View>;
            }"#,
        );
        assert!(
            array_field
                .errors
                .iter()
                .any(|error| error.contains("state(")),
            "{:?}",
            array_field.errors
        );

        // 行字段 prop（<Row title={item.title} />）与整行 prop（<Row item={item} />）都支持
        let row_prop = parse_tsx_to_ast(
            r#"function Row({ title }) { return <Text>{title}</Text>; }
            function App() {
                const items = state([{ title: "a" }]);
                return <View>{items.map((item) => <Row title={item.title} />)}</View>;
            }"#,
        );
        assert!(row_prop.errors.is_empty(), "{:?}", row_prop.errors);

        let item_prop = parse_tsx_to_ast(
            r#"function Row({ item }) {
                return <View>
                    <Text>{item.title}</Text>
                    <Button onClick={() => item.done = true}>done</Button>
                </View>;
            }
            function App() {
                const items = state([{ title: "a", done: false }]);
                return <View>{items.map((item) => <Row item={item} />)}</View>;
            }"#,
        );
        assert!(item_prop.errors.is_empty(), "{:?}", item_prop.errors);
        assert!(
            item_prop.state_actions.iter().any(|action| matches!(
                action,
                StateMutation::SetItemField { list, field, .. }
                    if list == "items" && field == "done"
            )),
            "{:?}",
            item_prop.state_actions
        );
    }

    #[test]
    fn nested_object_state_expands_into_dotted_cells() {
        let program = parse_tsx_to_ast(
            r#"function App() {
                const profile = state({ name: "", meta: { level: 1, tag: "x" } });
                return <View>
                    <Text>{profile.meta.level}</Text>
                    <Button onClick={() => profile.meta.level++}>up</Button>
                    <Button onClick={() => profile.meta.level = profile.meta.level + 2}>jump</Button>
                </View>;
            }"#,
        );
        assert!(program.errors.is_empty(), "{:?}", program.errors);
        assert_eq!(
            state_cells(&program.states),
            vec![
                ("profile.name".to_string(), StateValue::String(String::new())),
                ("profile.meta.level".to_string(), StateValue::Number(1)),
                ("profile.meta.tag".to_string(), StateValue::String("x".into())),
            ]
        );
        assert_eq!(
            program.state_actions,
            vec![
                StateMutation::Increment("profile.meta.level".into()),
                StateMutation::SetFromCell {
                    target: "profile.meta.level".into(),
                    source: "profile.meta.level".into(),
                    delta: 2,
                },
            ]
        );
    }

    #[test]
    fn supports_whole_assignment_and_cross_cell_arithmetic() {
        let program = parse_tsx_to_ast(
            r#"function App() {
                const items = state([{ title: "a" }, { title: "b" }]);
                const profile = state({ name: "guest", visits: 0 });
                const a = state(1);
                const b = state(4);
                return <View>
                    <Text>{items.length}</Text>
                    <Button onClick={() => items = []}>clear</Button>
                    <Button onClick={() => profile = { name: "Ada", visits: 7 }}>reset</Button>
                    <Button onClick={() => a = b + 1}>sync</Button>
                    <Button onClick={() => a = b}>copy</Button>
                </View>;
            }"#,
        );
        assert!(program.errors.is_empty(), "{:?}", program.errors);
        assert_eq!(
            program.state_actions,
            vec![
                StateMutation::ClearList { list: "items".into() },
                StateMutation::SetCells {
                    values: vec![
                        ("profile.name".into(), StateValue::String("Ada".into())),
                        ("profile.visits".into(), StateValue::Number(7)),
                    ],
                },
                StateMutation::SetFromCell {
                    target: "a".into(),
                    source: "b".into(),
                    delta: 1,
                },
                StateMutation::SetFromCell {
                    target: "a".into(),
                    source: "b".into(),
                    delta: 0,
                },
            ]
        );
        // {items.length} 绑到列表长度单元上
        let IRNode::View { children, .. } = &program.nodes[0] else {
            panic!("expected root View");
        };
        assert!(matches!(&children[0], IRNode::StateText { cell, .. } if cell == "items.length"));
    }

    #[test]
    fn scroll_handler_writes_a_numeric_cell() {
        let program = parse_tsx_to_ast(
            r#"function App() {
                const offset = state(0);
                return <View>
                    <Text>{offset}</Text>
                    <ScrollView onScroll={(y) => offset = y} style={{ height: 220 }}>
                        <Text>content</Text>
                    </ScrollView>
                </View>;
            }"#,
        );
        assert!(program.errors.is_empty(), "{:?}", program.errors);
        assert!(
            program
                .state_actions
                .iter()
                .any(|mutation| matches!(mutation, StateMutation::SetFromScroll(cell) if cell == "offset")),
            "{:?}",
            program.state_actions
        );

        // iOS：delegate 里把滚动位置写进单元
        let context = Context::create();
        let mut ios = IosCodeGen::new(&context);
        ios.generate(&program).unwrap();
        let ir = ios.module.print_to_string().to_string();
        assert!(ir.contains("scroll_action_write"), "{ir}");
        assert!(ir.contains("scrollViewDidScroll:"), "{ir}");

        // Android：nativeOnScroll 里写单元并刷新
        let context = Context::create();
        let mut android = IosCodeGen::new_for(&context, Platform::Android);
        android.generate(&program).unwrap();
        let ir = android.module.print_to_string().to_string();
        assert!(ir.contains("nativeOnScroll"), "{ir}");
        assert!(ir.contains("setOnScrollChange"), "{ir}");
    }

    #[test]
    fn scroll_handler_rejects_unsupported_shapes() {
        // 目标不是数值单元
        let program = parse_tsx_to_ast(
            r#"function App() {
                const note = state("");
                return <View>
                    <ScrollView onScroll={(y) => note = y} style={{ height: 200 }}>
                        <Text>content</Text>
                    </ScrollView>
                </View>;
            }"#,
        );
        assert!(
            program
                .errors
                .iter()
                .any(|error| error.contains("onScroll")),
            "{:?}",
            program.errors
        );
    }

    #[test]
    fn nested_lists_render_on_every_backend() {
        let program = parse_tsx_to_ast(
            r#"function App() {
                const items = state([
                    { title: "前端", children: [{ title: "渲染层" }] },
                    { title: "后端" }
                ]);
                return <View>{items.map((item) => <Text>{item.title}</Text>)}</View>;
            }"#,
        );
        let list = program
            .states
            .iter()
            .find_map(|decl| match &decl.initial {
                StateInit::List(list) => Some(list),
                _ => None,
            })
            .expect("list state");
        assert_eq!(list.rows.len(), 3);
        assert_eq!(list.parents, vec![None, Some(0), None]);
        assert!(list.has_children());

        // iOS 能生成（递归建行 + 层级缩进）
        let context = Context::create();
        let mut ios = IosCodeGen::new(&context);
        ios.generate(&program).unwrap();
        let ir = ios.module.print_to_string().to_string();
        assert!(ir.contains("@list_render_items_0("), "递归建行函数");
        // 定义 + 至少一次自递归调用
        assert!(
            ir.matches("@list_render_items_0(").count() >= 2,
            "缺少子行递归调用"
        );

        // macOS 走同一条生成路径，同样有 parent / depth 列
        let context = Context::create();
        let mut macos = IosCodeGen::new_for(&context, Platform::MacOs);
        macos.generate(&program).unwrap();
        let ir = macos.module.print_to_string().to_string();
        assert!(ir.contains("@g_list_items_0_parent"), "{ir}");

        // Android：parent / depth 列 + 逐行按深度缩进
        let context = Context::create();
        let mut android = IosCodeGen::new_for(&context, Platform::Android);
        android.generate(&program).unwrap();
        let ir = android.module.print_to_string().to_string();
        assert!(ir.contains("@g_list_items_0_parent"), "{ir}");
        assert!(ir.contains("@g_list_items_0_depth"), "{ir}");
        assert!(ir.contains("row_indent"), "{ir}");

        // web：静态 HTML 按 DFS 顺序输出并用 padding-left 表达层级
        let html = source_backend::generate_html(&program).unwrap();
        assert!(html.contains("padding-left: 16px;"), "{html}");
    }

    #[test]
    fn recursive_component_compiles_into_a_runtime_slot() {
        let program = parse_tsx_to_ast(RECURSIVE_TREE_SOURCE);
        assert!(program.errors.is_empty(), "{:?}", program.errors);

        // 编译期只留槽位，不再展开组件体
        let IRNode::View { children, .. } = &program.nodes[0] else {
            panic!("expected root View");
        };
        let Some(IRNode::List { item, parents, .. }) = children
            .iter()
            .find(|child| matches!(child, IRNode::List { .. }))
        else {
            panic!("expected a List node");
        };
        assert!(parents.iter().any(Option::is_some), "树形行表");
        let IRNode::View { children: body, .. } = item.as_ref() else {
            panic!("expected the row template root View");
        };
        assert!(
            body.iter().any(|child| matches!(
                child,
                IRNode::RecursiveComponent { list, .. } if list == "tree"
            )),
            "{body:?}"
        );

        // iOS：行函数自调用，重建只从根层起手
        let context = Context::create();
        let mut ios = IosCodeGen::new(&context);
        ios.generate(&program).unwrap();
        let ir = ios.module.print_to_string().to_string();
        assert!(
            ir.matches("@list_row_tree_0(").count() >= 2,
            "行函数应该自调用：{ir}"
        );
        assert_eq!(
            ir.matches("@list_render_tree_0(").count(),
            2,
            "行函数已内嵌子树，render 只调用一次：{ir}"
        );
        assert!(ir.contains("@g_list_tree_0_parent"), "{ir}");

        // Android：同一套运行期递归，父列 + 行函数自调用
        let context = Context::create();
        let mut android = IosCodeGen::new_for(&context, Platform::Android);
        android.generate(&program).unwrap();
        let ir = android.module.print_to_string().to_string();
        assert!(ir.contains("@g_list_tree_0_parent"), "{ir}");
        assert!(
            ir.matches("@list_row_tree_0(").count() >= 2,
            "Android 行函数应该自调用：{ir}"
        );

        // web：子树真的嵌在父行里面
        let html = source_backend::generate_html(&program).unwrap();
        assert!(html.contains("padding-left: 16px;"), "{html}");
        assert!(!html.contains("display: contents"), "{html}");
    }

    #[test]
    fn recursive_component_rejects_unsupported_shapes() {
        // 不带 `.children` 的自递归仍然是编译期展开死循环，必须报错
        let program = parse_tsx_to_ast(
            r#"function Node({ title }) {
                return <View><Text>{title}</Text><Node title={title} /></View>;
            }
            function App() {
                const items = state([{ title: "a" }]);
                return <View>{items.map((item) => <Node title={item.title} />)}</View>;
            }"#,
        );
        assert!(
            program
                .errors
                .iter()
                .any(|error| error.contains("recursive component expansion is unsupported")),
            "{:?}",
            program.errors
        );

        // 槽位必须挂在行模板根容器上
        let program = parse_tsx_to_ast(
            r#"function Node({ node }) {
                return <View>
                    <View><Node node={node.children} /></View>
                </View>;
            }
            function App() {
                const tree = state([{ title: "a", children: [{ title: "b" }] }]);
                return <View>{tree.map((item) => <Node node={item} />)}</View>;
            }"#,
        );
        assert!(
            program
                .errors
                .iter()
                .any(|error| error.contains("direct child of the component root")),
            "{:?}",
            program.errors
        );

        // 运行期行共享一份模板，实例级 state 无法按行分配
        let program = parse_tsx_to_ast(
            r#"function Node({ node }) {
                const hits = state(0);
                return <View>
                    <Text>{hits}</Text>
                    <Text>{node.title}</Text>
                    <Node node={node.children} />
                </View>;
            }
            function App() {
                const tree = state([{ title: "a", children: [{ title: "b" }] }]);
                return <View>{tree.map((item) => <Node node={item} />)}</View>;
            }"#,
        );
        assert!(
            program
                .errors
                .iter()
                .any(|error| error.contains("cannot declare state(...)")),
            "{:?}",
            program.errors
        );
    }

    #[test]
    fn component_state_is_allocated_per_instance() {
        let program = parse_tsx_to_ast(
            r#"function Counter() {
                const count = state(0);
                return <View>
                    <Text>{count}</Text>
                    <Button onClick={() => count++}>+</Button>
                </View>;
            }
            function App() {
                return <View>
                    <Counter />
                    <Counter />
                </View>;
            }"#,
        );
        assert!(program.errors.is_empty(), "{:?}", program.errors);
        // 两个实例各有一份单元
        assert_eq!(
            state_cells(&program.states),
            vec![
                ("Counter_0.count".to_string(), StateValue::Number(0)),
                ("Counter_1.count".to_string(), StateValue::Number(0)),
            ]
        );
        let IRNode::View { children, .. } = &program.nodes[0] else {
            panic!("expected root View");
        };
        let bound_cells: Vec<&str> = children
            .iter()
            .filter_map(|child| match child {
                IRNode::View { children, .. } => children.iter().find_map(|node| match node {
                    IRNode::StateText { cell, .. } => Some(cell.as_str()),
                    _ => None,
                }),
                _ => None,
            })
            .collect();
        assert_eq!(bound_cells, vec!["Counter_0.count", "Counter_1.count"]);
        assert_eq!(
            program.state_actions,
            vec![
                StateMutation::Increment("Counter_0.count".into()),
                StateMutation::Increment("Counter_1.count".into()),
            ]
        );
    }

    #[test]
    fn module_level_state_is_shared_across_components() {
        let program = parse_tsx_to_ast(
            r#"const hits = state(0);
            function Badge() { return <Text>{hits}</Text>; }
            function App() {
                return <View>
                    <Badge />
                    <Badge />
                    <Button onClick={() => hits++}>+</Button>
                </View>;
            }"#,
        );
        assert!(program.errors.is_empty(), "{:?}", program.errors);
        assert_eq!(
            state_cells(&program.states),
            vec![("hits".to_string(), StateValue::Number(0))]
        );
        let IRNode::View { children, .. } = &program.nodes[0] else {
            panic!("expected root View");
        };
        let bound: Vec<&str> = children
            .iter()
            .filter_map(|child| match child {
                IRNode::StateText { cell, .. } => Some(cell.as_str()),
                _ => None,
            })
            .collect();
        assert_eq!(bound, vec!["hits", "hits"]);
    }

    #[test]
    fn android_renders_lists_with_native_row_storage() {
        let program = parse_tsx_to_ast(
            r#"function App() {
                const items = state([{ title: "a" }]);
                return <View>
                    {items.map((item) => <Text>{item.title}</Text>)}
                </View>;
            }"#,
        );
        assert!(program.errors.is_empty(), "{:?}", program.errors);
        let context = Context::create();
        let mut codegen = IosCodeGen::new_for(&context, Platform::Android);
        codegen.generate(&program).unwrap();
        let ir = codegen.module.print_to_string().to_string();
        // 行数据列 + 运行期重建 + 行号编码的点击 id
        assert!(ir.contains("@g_list_items_0_col_title = "), "title 数据列");
        assert!(ir.contains("define void @list_rebuild_items_0()"), "重建函数");
        assert!(ir.contains("removeAllViews"), "重建前清空旧行");
    }
}
