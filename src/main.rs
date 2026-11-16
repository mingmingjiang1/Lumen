mod android;

use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::sync::Arc;

use swc_common::{FileName, SourceMap};
use swc_ecma_ast::*;
use swc_ecma_parser::{Parser, Syntax, TsSyntax, lexer::Lexer};
use swc_ecma_visit::Visit;

// ---------- 你的原有部分：AST -> IRNode（保持不动） ----------

/// Input 占位符的默认颜色。三端原生默认各不相同（UIKit 偏浅、Android 偏深），
/// 这里统一成一个中性灰，样式里可以用 placeholderColor 覆盖。
pub(crate) const DEFAULT_PLACEHOLDER_COLOR: &str = "rgb(120, 131, 155)";

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
        IRNode::Text { style, .. } | IRNode::StateText { style, .. } => style
            .height
            .map(|dimension| dimension.resolve(available_height))
            .unwrap_or_else(|| style.font_size.unwrap_or(17.0) * 1.2 + node_padding(style).vertical()),
        IRNode::Image { style, .. }
        | IRNode::Input { style, .. }
        | IRNode::Button { style, .. } => style
            .height
            .map(|dimension| dimension.resolve(available_height))
            .unwrap_or(50.0 + node_padding(style).vertical()),
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
        IRNode::StateText { style, .. } => style
            .width
            .map(|d| d.resolve(available_width))
            .unwrap_or(50.0 + node_padding(style).horizontal()),
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
        | IRNode::Input { .. }
        | IRNode::Button { .. } => true,
        IRNode::Image { mode, .. } => {
            !matches!(mode, ImageMode::WidthFix | ImageMode::HeightFix)
        }
        IRNode::View { .. } | IRNode::VStack { .. } | IRNode::ScrollView { .. } => false,
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
        | IRNode::Button { style, .. } => style,
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


fn find_scroll_fade_config(nodes: &[IRNode]) -> Option<ScrollFadeConfig> {
    fn find_in_node(node: &IRNode) -> Option<ScrollFadeConfig> {
        match node {
            IRNode::ScrollView { style, .. } if style.scroll_fade_target.is_some() => {
                Some(ScrollFadeConfig {
                    target_id: style.scroll_fade_target.clone().unwrap(),
                    start: style.scroll_fade_start.unwrap_or(0.0),
                    distance: style.scroll_fade_distance.unwrap_or(120.0),
                })
            }
            IRNode::View { children, .. }
            | IRNode::VStack { children, .. }
            | IRNode::ScrollView { children, .. } => children.iter().find_map(find_in_node),
            IRNode::Conditional { when_true, when_false, .. } => find_in_node(when_true)
                .or_else(|| when_false.as_deref().and_then(find_in_node)),
            _ => None,
        }
    }

    nodes.iter().find_map(find_in_node)
}

fn find_view_by_id<'a>(nodes: &'a [IRNode], id: &str) -> Option<&'a IRNode> {
    for node in nodes {
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

fn count_scroll_fade_targets(nodes: &[IRNode]) -> usize {
    nodes
        .iter()
        .map(|node| match node {
            IRNode::ScrollView { style, children } => {
                usize::from(style.scroll_fade_target.is_some())
                    + count_scroll_fade_targets(children)
            }
            IRNode::View { children, .. } | IRNode::VStack { children, .. } => {
                count_scroll_fade_targets(children)
            }
            IRNode::Conditional { when_true, when_false, .. } => {
                count_scroll_fade_targets(std::slice::from_ref(when_true))
                    + when_false.as_deref().map_or(0, |node| {
                        count_scroll_fade_targets(std::slice::from_ref(node))
                    })
            }
            _ => 0,
        })
        .sum()
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

/// 状态初值：标量，或一层对象字面量（对象会被展开成多个状态单元）。
#[derive(Debug, Clone, PartialEq)]
enum StateInit {
    Scalar(StateValue),
    Object(Vec<(String, StateValue)>),
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
}

impl StateMutation {
    /// 该动作写入的状态单元路径。
    fn cell(&self) -> Option<&str> {
        match self {
            Self::Noop => None,
            Self::Increment(cell)
            | Self::Decrement(cell)
            | Self::SetFromInput(cell)
            | Self::SetTrue(cell) => Some(cell),
            Self::Set(cell, _) => Some(cell),
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
    scroll_fade: Option<ScrollFadeConfig>,
    errors: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
struct ScrollFadeConfig {
    target_id: String,
    start: f64,
    distance: f64,
}

#[derive(Debug, Clone)]
enum ComponentProp {
    Literal(StateValue),
    StateReference(String),
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
}

struct JsxToIrVisitor<'a> {
    ir_out: Vec<IRNode>,
    cells: Vec<(String, StateValue)>,
    state_actions: Vec<StateMutation>,
    components: HashMap<String, ComponentDefinition<'a>>,
    props: HashMap<String, ComponentProp>,
    component_stack: Vec<String>,
    errors: Vec<String>,
}

impl JsxToIrVisitor<'_> {
    fn cell_initial(&self, cell: &str) -> Option<&StateValue> {
        self.cells
            .iter()
            .find(|(path, _)| path == cell)
            .map(|(_, value)| value)
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
                    cell,
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
                Some(BooleanCondition::Static(*value ^ negated))
            }
            ComponentProp::StateReference(cell) => {
                let initial = match self.cell_initial(cell) {
                    Some(StateValue::Boolean(value)) => *value,
                    _ => {
                        self.errors
                            .push("conditional rendering requires state(false) or state(true)".into());
                        return None;
                    }
                };
                Some(BooleanCondition::State {
                    cell: cell.clone(),
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
            let Some(value) = component_prop_value(attribute, &self.cells, &self.props) else {
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

impl Visit for JsxToIrVisitor<'_> {
    fn visit_jsx_element(&mut self, node: &JSXElement) {
        let tag_name = match &node.opening.name {
            JSXElementName::Ident(ident) => ident.sym.as_str(),
            _ => return,
        };

        if let Some(definition) = self.components.get(tag_name).cloned() {
            if self.component_stack.iter().any(|component| component == tag_name) {
                self.errors.push(format!("recursive component expansion is unsupported: {tag_name}"));
                return;
            }
            let component_props = self.component_props(node, &definition);
            if definition.unsupported_lifecycle {
                self.errors.push(format!(
                    "component {tag_name} has an unsupported onMount/onUnmount callback; use an empty callback or a supported state mutation"
                ));
            }
            let previous_props = std::mem::replace(&mut self.props, component_props);
            self.component_stack.push(tag_name.to_string());
            let start = self.ir_out.len();
            self.visit_jsx_element(definition.root);
            self.component_stack.pop();
            self.props = previous_props;
            let mut expanded: Vec<_> = self.ir_out.drain(start..).collect();
            if expanded.len() != 1 {
                self.errors.push(format!("component {tag_name} must return one JSX element"));
                return;
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
        let title = jsx_string_attr_with_props(node, "title", &self.props).unwrap_or_default();
        let source = jsx_string_attr_with_props(node, "source", &self.props)
            .or_else(|| jsx_string_attr_with_props(node, "src", &self.props))
            .unwrap_or_default();
        let placeholder = jsx_string_attr_with_props(node, "placeholder", &self.props).unwrap_or_default();
        let cells = self.cells.clone();
        let click_handler = jsx_attr(node, "onClick").and_then(|attr| attr.value.as_ref());
        let action = click_handler.and_then(|value| parse_state_mutation(value, &cells));
        let unsupported_handler = jsx_attr(node, "onClick").is_some() && action.is_none();
        let input_handler = jsx_attr(node, "onInput").and_then(|attr| attr.value.as_ref());
        let on_input = input_handler.and_then(|value| parse_state_mutation(value, &cells));
        let focus_handler = jsx_attr(node, "onFocus").and_then(|attr| attr.value.as_ref());
        let on_focus = focus_handler.and_then(|value| {
            if is_empty_event_handler(value) {
                Some(StateMutation::Noop)
            } else {
                parse_state_mutation(value, &cells)
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
                        if let Some(prop) = component_prop_for_expr(expr, &self.props) {
                            match prop {
                                ComponentProp::Literal(value) => children.push(IRNode::Text {
                                    content: value.display_value(),
                                    style: InlineStyle::default(),
                                }),
                                ComponentProp::StateReference(cell) => {
                                    let cell = cell.clone();
                                    if let Some(initial) = self.cell_initial(&cell).cloned() {
                                        children.push(IRNode::StateText {
                                            cell,
                                            initial,
                                            style: InlineStyle::default(),
                                        });
                                    }
                                }
                            }
                            continue;
                        }
                    }
                    if let JSXExpr::Expr(expr) = &container.expr {
                        if let Some(cell) = expr_state_path(expr.as_ref()) {
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
            "ScrollView" => self.ir_out.push(IRNode::ScrollView { style, children }),
            "Text" => {
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
    let cells = state_cells(&states);
    let components = collect_component_definitions(&module, &cells);
    let mut visitor = JsxToIrVisitor {
        ir_out: vec![],
        cells,
        state_actions: Vec::new(),
        components,
        props: HashMap::new(),
        component_stack: Vec::new(),
        errors: Vec::new(),
    };
    if let Some(root) = find_app_root(&module) {
        visitor.visit_jsx_element(root);
    } else {
        visitor.errors.push("missing App function returning JSX".into());
    }
    UiProgram {
        scroll_fade: find_scroll_fade_config(&visitor.ir_out),
        nodes: visitor.ir_out,
        states,
        state_actions: visitor.state_actions,
        errors: visitor.errors,
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

fn component_prop_for_expr<'a>(
    expression: &Expr,
    props: &'a HashMap<String, ComponentProp>,
) -> Option<&'a ComponentProp> {
    match expression {
        Expr::Ident(identifier) => props.get(identifier.sym.as_str()),
        Expr::Member(member) => {
            let Expr::Ident(object) = member.obj.as_ref() else { return None };
            let MemberProp::Ident(property) = &member.prop else { return None };
            props.get(&format!("{}.{}", object.sym, property.sym))
        }
        _ => None,
    }
}

fn component_prop_value(
    attribute: &JSXAttr,
    cells: &[(String, StateValue)],
    props: &HashMap<String, ComponentProp>,
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
                return Some(prop.clone());
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
    match component_prop_value(attribute, &[], props)? {
        ComponentProp::Literal(value) => Some(value.display_value()),
        ComponentProp::StateReference(_) => None,
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

fn component_prop_state_value<'a>(
    expr: &Expr,
    props: &'a HashMap<String, ComponentProp>,
) -> Option<&'a StateValue> {
    match component_prop_for_expr(expr, props)? {
        ComponentProp::Literal(value) => Some(value),
        ComponentProp::StateReference(_) => None,
    }
}

fn jsx_dimension_expr_with_props(
    expr: &Expr,
    props: &HashMap<String, ComponentProp>,
) -> Option<Dimension> {
    jsx_dimension_expr(expr).or_else(|| match component_prop_state_value(expr, props)? {
        StateValue::Number(value) => Some(Dimension::Points(*value as f64)),
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
        StateValue::Number(value) => Some(*value as f64),
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
        component_prop_state_value(expr, props).map(StateValue::display_value)
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
        | IRNode::Button { style, .. } => style,
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
            let Expr::Ident(object) = member.obj.as_ref() else {
                return None;
            };
            let MemberProp::Ident(prop) = &member.prop else {
                return None;
            };
            Some(format!("{}.{}", object.sym, prop.sym))
        }
        _ => None,
    }
}

/// 赋值左值对应的状态单元路径。
fn assign_target_path(target: &AssignTarget) -> Option<String> {
    match target {
        AssignTarget::Simple(SimpleAssignTarget::Ident(ident)) => Some(ident.id.sym.to_string()),
        AssignTarget::Simple(SimpleAssignTarget::Member(member)) => {
            let Expr::Ident(object) = member.obj.as_ref() else {
                return None;
            };
            let MemberProp::Ident(prop) = &member.prop else {
                return None;
            };
            Some(format!("{}.{}", object.sym, prop.sym))
        }
        _ => None,
    }
}

/// 收集所有 `const|let x = state(<初值>)` 声明；对象初值会展开成多个状态单元。
/// 声明既可以写在模块顶层，也可以写在函数（例如 App）体里。
fn collect_state_decls(module: &swc_ecma_ast::Module) -> Vec<StateDecl> {
    let mut decls = Vec::new();
    for item in &module.body {
        if let ModuleItem::Stmt(statement) = item {
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
                fields.push((name, state_value_from_expr(entry.value.as_ref())?));
            }
            (!fields.is_empty()).then_some(StateInit::Object(fields))
        }
        expr => state_value_from_expr(expr).map(StateInit::Scalar),
    }
}

fn parse_state_mutation(
    value: &JSXAttrValue,
    cells: &[(String, StateValue)],
) -> Option<StateMutation> {
    let arrow = arrow_from_handler(value)?;
    let ArrowFunctionBody::Expr(body) = arrow.body.as_ref() else {
        return None;
    };
    let parameter = match arrow.params.as_slice() {
        [Pat::Ident(ident)] => Some(ident.id.sym.to_string()),
        _ => None,
    };
    parse_state_mutation_expr(body, cells, parameter.as_deref())
}

fn parse_lifecycle_mutation(
    arrow: &ArrowExpr,
    cells: &[(String, StateValue)],
) -> Option<StateMutation> {
    match arrow.body.as_ref() {
        ArrowFunctionBody::FunctionBody(body) if body.stmts.is_empty() => Some(StateMutation::Noop),
        ArrowFunctionBody::Expr(expression) => parse_state_mutation_expr(expression, cells, None),
        ArrowFunctionBody::FunctionBody(body) => {
            let [Stmt::Expr(statement)] = body.stmts.as_slice() else {
                return None;
            };
            parse_state_mutation_expr(&statement.expr, cells, None)
        }
    }
}

fn parse_state_mutation_expr(
    body: &Expr,
    cells: &[(String, StateValue)],
    input_parameter: Option<&str>,
) -> Option<StateMutation> {
    let cell_initial = |cell: &str| {
        cells
            .iter()
            .find(|(path, _)| path == cell)
            .map(|(_, value)| value.clone())
    };
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
            let cell = assign_target_path(&assignment.left)?;
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
    BasicMetadataValueEnum, BasicValueEnum, FunctionValue, GlobalValue, PointerValue,
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
    scroll_fade_target_id: RefCell<Option<String>>,
    conditional_instances: RefCell<Vec<ConditionalRuntime<'ctx>>>,
    conditional_counter: Cell<usize>,
    /// AppDelegate 类与 class_addMethod，macOS 上要等界面构建完再注册状态动作
    delegate_class: RefCell<Option<PointerValue<'ctx>>>,
    class_add_method_fn: RefCell<Option<FunctionValue<'ctx>>>,
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
            scroll_fade_target_id: RefCell::new(None),
            conditional_instances: RefCell::new(Vec::new()),
            conditional_counter: Cell::new(0),
            delegate_class: RefCell::new(None),
            class_add_method_fn: RefCell::new(None),
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
        if self.platform.is_android() {
            return self.generate_android(program);
        }
        *self.state_cells.borrow_mut() = state_cells(&program.states);
        if program.states.is_empty() && program.nodes.iter().any(node_has_input_binding) {
            return Err("Input onInput requires a state(...) declaration".into());
        }
        if let Some(config) = &program.scroll_fade {
            *self.scroll_fade_target_id.borrow_mut() = Some(config.target_id.clone());
            if count_scroll_fade_targets(&program.nodes) > 1 {
                return Err("this compiler phase supports one ScrollView fadeTarget per App".into());
            }
            if !config.distance.is_finite() || config.distance <= 0.0 {
                return Err("ScrollView fadeDistance must be greater than zero".into());
            }
            if count_views_with_id(&program.nodes, &config.target_id) != 1 {
                return Err(format!(
                    "ScrollView fadeTarget '{}' must match exactly one View id",
                    config.target_id
                ));
            }
            let Some(target) = find_view_by_id(&program.nodes, &config.target_id) else {
                return Err(format!(
                    "ScrollView fadeTarget '{}' does not match a View id",
                    config.target_id
                ));
            };
            if node_style(target).background_color.is_none() {
                return Err(format!(
                    "View id '{}' needs a backgroundColor for scroll fading",
                    config.target_id
                ));
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
        if program.scroll_fade.is_some() {
            for name in ["g_scroll_fade_view", "g_scroll_fade_scroll"] {
                let global = self.module.add_global(ptr_type, None, name);
                global.set_initializer(&ptr_type.const_null());
            }
        }
        if let Some(target) = find_top_background_view(&program.nodes) {
            self.module
                .add_global(ptr_type, None, "g_top_background_view")
                .set_initializer(&ptr_type.const_null());
            if program.scroll_fade.as_ref().is_some_and(|fade| {
                node_style(target).view_id.as_deref() == Some(fade.target_id.as_str())
            }) {
                self.module
                    .add_global(ptr_type, None, "g_scroll_fade_background")
                    .set_initializer(&ptr_type.const_null());
            }
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

        if let Some(config) = program.scroll_fade.as_ref().filter(|_| !self.is_mac()) {
            let callback = self.build_scroll_fade_callback(config)?;
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

    /// 状态单元对应的存储全局变量名。
    fn cell_global_name(&self, cell: &str) -> Option<String> {
        self.state_cells
            .borrow()
            .iter()
            .position(|(path, _)| path == cell)
            .map(|index| format!("g_state_cell_{index}"))
    }

    /// 一个状态动作对应一个 Objective-C 方法：读当前值 → 计算 → 写回 →
    /// 只刷新引用该单元的标签和条件视图。
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

        let Some(cell) = mutation.cell() else {
            self.builder
                .build_return(None)
                .map_err(|e| e.to_string())?;
            self.builder.position_at_end(saved_block);
            return Ok(callback);
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
            StateMutation::SetFromInput(_) => {
                let sender = callback
                    .get_nth_param(2)
                    .ok_or("input sender parameter is missing")?
                    .into_pointer_value();
                // UITextField 用 text，AppKit 的 NSTextField 用 stringValue
                let selector = if self.is_mac() { "stringValue" } else { "text" };
                self.send_msg_ptr(sender, self.get_selector(selector)?, &[])?
                    .ok_or("text field text returned void")?
            }
            StateMutation::Increment(_) | StateMutation::Decrement(_) => {
                let current = self
                    .builder
                    .build_load(ptr_type, state_ptr, "state_current")
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
                let delta = if matches!(mutation, StateMutation::Increment(_)) {
                    1
                } else {
                    -1
                };
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
            StateMutation::Noop => return Err("lifecycle no-op actions have no state body".into()),
        };
        self.builder
            .build_store(state_ptr, text)
            .map_err(|e| e.to_string())?;
        self.update_cell_labels(cell, text)?;
        let receiver = callback
            .get_nth_param(0)
            .ok_or("state callback self parameter is missing")?
            .into_pointer_value();
        self.update_conditional_views(callback, receiver, cell, text)?;
        self.builder
            .build_return(None)
            .map_err(|e| e.to_string())?;
        self.builder.position_at_end(saved_block);

        Ok(callback)
    }

    fn build_scroll_fade_callback(
        &self,
        config: &ScrollFadeConfig,
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
            .ok_or("missing insertion block while generating scroll fade callback")?;
        self.builder.position_at_end(entry);

        let scroll = self
            .module
            .get_global("g_scroll_fade_scroll")
            .ok_or("scroll fade view global was not created")?;
        let scroll_value = self
            .builder
            .build_load(ptr_type, scroll.as_pointer_value(), "fade_scroll")
            .map_err(|error| error.to_string())?
            .into_pointer_value();
        let sender = callback
            .get_nth_param(2)
            .ok_or("scroll view callback parameter is missing")?
            .into_pointer_value();
        let is_target = self
            .builder
            .build_int_compare(
                inkwell::IntPredicate::EQ,
                sender,
                scroll_value,
                "is_fade_scroll",
            )
            .map_err(|error| error.to_string())?;
        let update_block = self.context.append_basic_block(callback, "update_fade");
        let return_block = self.context.append_basic_block(callback, "return");
        self.builder
            .build_conditional_branch(is_target, update_block, return_block)
            .map_err(|error| error.to_string())?;

        self.builder.position_at_end(update_block);
        let point_type = self.context.struct_type(
            &[self.context.f64_type().into(), self.context.f64_type().into()],
            false,
        );
        let offset = self
            .send_msg_impl(
                sender,
                self.get_selector("contentOffset")?,
                &[],
                BasicTypeEnum::StructType(point_type),
            )?
            .ok_or("UIScrollView contentOffset returned void")?
            .into_struct_value();
        let offset_y = self
            .builder
            .build_extract_value(offset, 1, "content_offset_y")
            .map_err(|error| error.to_string())?
            .into_float_value();

        let inset_type = self.context.struct_type(
            &[
                self.context.f64_type().into(),
                self.context.f64_type().into(),
                self.context.f64_type().into(),
                self.context.f64_type().into(),
            ],
            false,
        );
        let inset = self
            .send_msg_impl(
                sender,
                self.get_selector("adjustedContentInset")?,
                &[],
                BasicTypeEnum::StructType(inset_type),
            )?
            .ok_or("UIScrollView adjustedContentInset returned void")?
            .into_struct_value();
        let inset_top = self
            .builder
            .build_extract_value(inset, 0, "adjusted_inset_top")
            .map_err(|error| error.to_string())?
            .into_float_value();
        let normalized_offset = self
            .builder
            .build_float_add(offset_y, inset_top, "normalized_offset")
            .map_err(|error| error.to_string())?;
        let start = self.context.f64_type().const_float(config.start);
        let after_start = self
            .builder
            .build_float_sub(normalized_offset, start, "offset_after_fade_start")
            .map_err(|error| error.to_string())?;
        let positive_offset = self
            .builder
            .build_float_compare(
                FloatPredicate::OGT,
                after_start,
                self.context.f64_type().const_float(0.0),
                "past_fade_start",
            )
            .map_err(|error| error.to_string())?;
        let positive_value = self
            .builder
            .build_select(
                positive_offset,
                after_start,
                self.context.f64_type().const_float(0.0),
                "clamped_fade_offset",
            )
            .map_err(|error| error.to_string())?
            .into_float_value();
        let fraction = self
            .builder
            .build_float_div(
                positive_value,
                self.context.f64_type().const_float(config.distance),
                "fade_fraction",
            )
            .map_err(|error| error.to_string())?;
        let before_end = self
            .builder
            .build_float_compare(
                FloatPredicate::OLT,
                fraction,
                self.context.f64_type().const_float(1.0),
                "before_fade_end",
            )
            .map_err(|error| error.to_string())?;
        let alpha = self
            .builder
            .build_select(
                before_end,
                fraction,
                self.context.f64_type().const_float(1.0),
                "clamped_fade_alpha",
            )
            .map_err(|error| error.to_string())?
            .into_float_value();

        let fade_view_global = self
            .module
            .get_global("g_scroll_fade_view")
            .ok_or("scroll fade target global was not created")?;
        let fade_view = self
            .builder
            .build_load(ptr_type, fade_view_global.as_pointer_value(), "fade_target")
            .map_err(|error| error.to_string())?
            .into_pointer_value();
        self.send_msg_void(fade_view, "setAlpha:", &[alpha.into()])?;
        if let Some(background_global) = self.module.get_global("g_scroll_fade_background") {
            let background = self
                .builder
                .build_load(
                    ptr_type,
                    background_global.as_pointer_value(),
                    "fade_background",
                )
                .map_err(|error| error.to_string())?
                .into_pointer_value();
            self.send_msg_void(background, "setAlpha:", &[alpha.into()])?;
        }
        self.builder
            .build_unconditional_branch(return_block)
            .map_err(|error| error.to_string())?;
        self.builder.position_at_end(return_block);
        self.builder
            .build_return(None)
            .map_err(|error| error.to_string())?;
        self.builder.position_at_end(saved_block);
        Ok(callback)
    }

    fn update_cell_labels(&self, cell: &str, value: PointerValue<'ctx>) -> Result<(), String> {
        let ptr_type = self.context.ptr_type(AddressSpace::default());
        let globals = self.state_label_globals.borrow();
        let Some(labels) = globals.get(cell) else {
            return Ok(());
        };
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

        let shares_fade_target = target_style.view_id.as_ref().is_some_and(|id| {
            self.scroll_fade_target_id.borrow().as_ref() == Some(id)
        });
        self.send_msg_void(
            background,
            "setAlpha:",
            &[self
                .context
                .f64_type()
                .const_float(if shares_fade_target { 0.0 } else { 1.0 })
                .into()],
        )?;
        if let Some(background_global) = self.module.get_global("g_scroll_fade_background") {
            self.builder
                .build_store(background_global.as_pointer_value(), background)
                .map_err(|error| error.to_string())?;
        }
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
            let self_sized = matches!(child_node, IRNode::Conditional { .. });
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
                self.configure_scroll_fade_target(view, style)?;
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
                self.configure_scroll_fade_target(stack, style)?;
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
                if style.scroll_fade_target.is_some() {
                    let target = event_target.ok_or("ScrollView fadeTarget requires an event target")?;
                    self.send_msg_void(scroll_view, "setDelegate:", &[target.into()])?;
                    let scroll_global = self
                        .module
                        .get_global("g_scroll_fade_scroll")
                        .ok_or("scroll fade scroll global was not created")?;
                    self.builder
                        .build_store(scroll_global.as_pointer_value(), scroll_view)
                        .map_err(|error| error.to_string())?;
                }
                // 滚动容器也不再默认涂白：没写 backgroundColor 就和 UIKit 一样透明。
                if has_direct_conditional(children) {
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
        Ok(())
    }

    /// 设置背景色：UIKit 直接用 setBackgroundColor:，AppKit 要走 layer。
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

    fn configure_scroll_fade_target(
        &self,
        view: PointerValue<'ctx>,
        style: &InlineStyle,
    ) -> Result<(), String> {
        if self.is_mac() {
            return Ok(());
        }
        if style.background_extends_to_top {
            let target_global = self
                .module
                .get_global("g_top_background_view")
                .ok_or("top background target global was not created")?;
            self.builder
                .build_store(target_global.as_pointer_value(), view)
                .map_err(|error| error.to_string())?;
        }
        let is_target = style.view_id.as_ref().is_some_and(|id| {
            self.scroll_fade_target_id.borrow().as_ref() == Some(id)
        });
        if !is_target {
            return Ok(());
        }
        if style.background_color.is_none() {
            return Err("scroll fade target must define backgroundColor".into());
        }
        let fade_view_global = self
            .module
            .get_global("g_scroll_fade_view")
            .ok_or("scroll fade target global was not created")?;
        self.builder
            .build_store(fade_view_global.as_pointer_value(), view)
            .map_err(|error| error.to_string())?;
        self.send_msg_void(
            view,
            "setAlpha:",
            &[self.context.f64_type().const_float(0.0).into()],
        )?;
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
];

const DEFAULT_APP: &str = "dashboard";

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();

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

    let context = Context::create();
    let mut codegen = IosCodeGen::new_for(&context, platform);
    if let Err(e) = codegen.generate(&program) {
        eprintln!("codegen failed: {e}");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn generates_scroll_delegate_for_transparent_header_fade() {
        let program = parse_tsx_to_ast(
            r#"function App() {
                return <View>
                    <View style={{
                        id: "topBar",
                        backgroundColor: "rgb(20, 30, 40)",
                        backgroundExtendsTo: "top"
                    }}>
                        <Text>Title</Text>
                    </View>
                    <ScrollView style={{ fadeTarget: "topBar", fadeStart: 10, fadeDistance: 90 }}>
                        <Text>Scrollable content</Text>
                    </ScrollView>
                </View>;
            }"#,
        );
        assert_eq!(
            program.scroll_fade,
            Some(ScrollFadeConfig {
                target_id: "topBar".into(),
                start: 10.0,
                distance: 90.0,
            })
        );

        let context = Context::create();
        let mut codegen = IosCodeGen::new(&context);
        codegen.generate(&program).unwrap();
        let generated_ir = codegen.module.print_to_string().to_string();
        assert!(generated_ir.contains("scrollViewDidScroll:"));
        assert!(generated_ir.contains("setAlpha:"));
        assert!(generated_ir.contains("adjustedContentInset"));
        assert!(generated_ir.contains("g_top_background_view"));
        assert!(generated_ir.contains("insertSubview:atIndex:"));
        assert!(generated_ir.contains("g_scroll_fade_background"));
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
    fn rejects_zero_scroll_fade_distance() {
        let program = parse_tsx_to_ast(
            r#"function App() {
                return <View>
                    <View style={{ id: "topBar", backgroundColor: "blue" }} />
                    <ScrollView style={{ fadeTarget: "topBar", fadeDistance: 0 }} />
                </View>;
            }"#,
        );
        let context = Context::create();
        let mut codegen = IosCodeGen::new(&context);
        assert!(codegen
            .generate(&program)
            .unwrap_err()
            .contains("fadeDistance must be greater than zero"));
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
                    <ScrollView style={{ fadeTarget: "topBar", fadeDistance: 120 }}>
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
    fn bundled_apps_compile_for_every_backend() {
        for (name, source) in APPS {
            let program = parse_tsx_to_ast(source);
            assert!(program.errors.is_empty(), "{name}: {:?}", program.errors);
            for platform in [Platform::Ios, Platform::MacOs, Platform::Android] {
                let context = Context::create();
                let mut codegen = IosCodeGen::new_for(&context, platform);
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
}
