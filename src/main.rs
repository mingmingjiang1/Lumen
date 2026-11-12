use std::sync::Arc;

use swc_common::{FileName, SourceMap};
use swc_ecma_ast::*;
use swc_ecma_parser::{Parser, Syntax, TsSyntax, lexer::Lexer};
use swc_ecma_visit::{Visit, VisitWith};

// ---------- 你的原有部分：AST -> IRNode（保持不动） ----------

#[derive(Debug, Clone)]
pub enum IRNode {
    View { style: InlineStyle, children: Vec<IRNode> },
    VStack { style: InlineStyle, children: Vec<IRNode> },
    ScrollView { style: InlineStyle, children: Vec<IRNode> },
    Text { content: String, style: InlineStyle },
    StateText { initial: i64, style: InlineStyle },
    Image {
        source: String,
        mode: ImageMode,
        style: InlineStyle,
    },
    Input {
        placeholder: String,
        style: InlineStyle,
    },
    Button {
        title: String,
        style: InlineStyle,
        updates_state: bool,
        unsupported_handler: bool,
    },
}

#[derive(Debug, Clone, Default)]
pub struct InlineStyle {
    width: Option<Dimension>,
    height: Option<Dimension>,
    padding: Option<f64>,
    margin: Option<f64>,
    margin_horizontal: Option<f64>,
    margin_vertical: Option<f64>,
    margin_top: Option<f64>,
    margin_right: Option<f64>,
    margin_bottom: Option<f64>,
    margin_left: Option<f64>,
    spacing: Option<f64>,
    flex_direction: Option<FlexDirection>,
    justify_content: Option<JustifyContent>,
    align_items: Option<AlignItems>,
    flex_grow: f64,
    font_size: Option<f64>,
    text_align: Option<TextAlignment>,
    background_color: Option<String>,
    color: Option<String>,
    border_width: Option<f64>,
    border_color: Option<String>,
    border_radius: Option<f64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FlexDirection {
    Row,
    Column,
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

impl Dimension {
    fn resolve(self, parent_size: f64) -> f64 {
        match self {
            Self::Points(value) => value,
            Self::Percent(fraction) => parent_size * fraction,
        }
    }
}

fn node_height(node: &IRNode, available_height: f64) -> f64 {
    match node {
        IRNode::View { style, children }
        | IRNode::VStack { style, children }
        | IRNode::ScrollView { style, children } => style
            .height
            .map(|d| d.resolve(available_height))
            .unwrap_or_else(|| {
                let direction = style.flex_direction.unwrap_or(FlexDirection::Column);
                let spacing = style.spacing.unwrap_or(if matches!(node, IRNode::VStack { .. }) { 8.0 } else { 0.0 });
                let children_height = match direction {
                    FlexDirection::Row => children
                        .iter()
                        .map(|child| {
                            let margins = node_margins(child);
                            node_height(child, available_height) + margins.top + margins.bottom
                        })
                        .fold(0.0, f64::max),
                    FlexDirection::Column => children
                        .iter()
                        .map(|child| {
                            let margins = node_margins(child);
                            node_height(child, available_height) + margins.top + margins.bottom
                        })
                        .sum::<f64>()
                        + spacing * children.len().saturating_sub(1) as f64,
                };
                children_height + style.padding.unwrap_or(0.0) * 2.0
            }),
        IRNode::Text { style, .. }
        | IRNode::StateText { style, .. }
        | IRNode::Image { style, .. }
        | IRNode::Input { style, .. }
        | IRNode::Button { style, .. } => style
            .height
            .map(|dimension| dimension.resolve(available_height))
            .unwrap_or(50.0),
    }
}

fn node_width(node: &IRNode, available_width: f64) -> f64 {
    match node {
        IRNode::View { style, children }
        | IRNode::VStack { style, children }
        | IRNode::ScrollView { style, children } => style.width.map(|d| d.resolve(available_width)).unwrap_or_else(|| {
            let direction = style.flex_direction.unwrap_or(FlexDirection::Column);
            let spacing = style.spacing.unwrap_or(if matches!(node, IRNode::VStack { .. }) { 8.0 } else { 0.0 });
            let children_width = match direction {
                FlexDirection::Row => children
                    .iter()
                    .map(|child| {
                        let margins = node_margins(child);
                        node_width(child, available_width) + margins.left + margins.right
                    })
                    .sum::<f64>()
                    + spacing * children.len().saturating_sub(1) as f64,
                FlexDirection::Column => children
                    .iter()
                    .map(|child| {
                        let margins = node_margins(child);
                        node_width(child, available_width) + margins.left + margins.right
                    })
                    .fold(0.0, f64::max),
            };
            children_width + style.padding.unwrap_or(0.0) * 2.0
        }),
        IRNode::Text { content, style } => style.width.map(|d| d.resolve(available_width)).unwrap_or_else(|| {
            content.chars().count() as f64 * style.font_size.unwrap_or(17.0) * 0.6
        }),
        IRNode::StateText { style, .. } => style.width.map(|d| d.resolve(available_width)).unwrap_or(50.0),
        IRNode::Image { style, .. } => style.width.map(|d| d.resolve(available_width)).unwrap_or(100.0),
        IRNode::Input { style, .. } => style.width.map(|d| d.resolve(available_width)).unwrap_or(240.0),
        IRNode::Button { title, style, .. } => style
            .width
            .map(|d| d.resolve(available_width))
            .unwrap_or_else(|| title.chars().count() as f64 * 9.0 + 32.0),
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

fn node_style(node: &IRNode) -> &InlineStyle {
    match node {
        IRNode::View { style, .. }
        | IRNode::VStack { style, .. }
        | IRNode::ScrollView { style, .. }
        | IRNode::Text { style, .. }
        | IRNode::StateText { style, .. }
        | IRNode::Image { style, .. }
        | IRNode::Input { style, .. }
        | IRNode::Button { style, .. } => style,
    }
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
    let padding = style.padding.unwrap_or(0.0);
    let gap = style.spacing.unwrap_or(default_gap);
    let inner_width = (width - 2.0 * padding).max(0.0);
    let inner_height = (height - 2.0 * padding).max(0.0);
    let main_size = match direction {
        FlexDirection::Row => inner_width,
        FlexDirection::Column => inner_height,
    };
    let cross_size = match direction {
        FlexDirection::Row => inner_height,
        FlexDirection::Column => inner_width,
    };

    let margins: Vec<Margins> = children.iter().map(node_margins).collect();
    let mut main_sizes: Vec<f64> = children
        .iter()
        .map(|child| match direction {
            FlexDirection::Row => node_width(child, inner_width),
            FlexDirection::Column => node_height(child, inner_height),
        })
        .collect();
    let main_margins_total: f64 = margins
        .iter()
        .map(|margin| match direction {
            FlexDirection::Row => margin.left + margin.right,
            FlexDirection::Column => margin.top + margin.bottom,
        })
        .sum();
    let base_total = main_sizes.iter().sum::<f64>()
        + main_margins_total
        + gap * children.len().saturating_sub(1) as f64;
    let grow_total = children.iter().map(node_flex_grow).sum::<f64>();
    let free_space = (main_size - base_total).max(0.0);
    if grow_total > 0.0 {
        for (size, child) in main_sizes.iter_mut().zip(children) {
            *size += free_space * node_flex_grow(child) / grow_total;
        }
    }

    let used_main = main_sizes.iter().sum::<f64>()
        + main_margins_total
        + gap * children.len().saturating_sub(1) as f64;
    let remaining = (main_size - used_main).max(0.0);
    let (main_offset, effective_gap) = match style.justify_content.unwrap_or(JustifyContent::Start) {
        JustifyContent::Start => (0.0, gap),
        JustifyContent::Center => (remaining / 2.0, gap),
        JustifyContent::End => (remaining, gap),
        JustifyContent::SpaceBetween if children.len() > 1 => {
            (0.0, gap + remaining / (children.len() - 1) as f64)
        }
        JustifyContent::SpaceAround if !children.is_empty() => {
            let extra = remaining / children.len() as f64;
            (extra / 2.0, gap + extra)
        }
        _ => (0.0, gap),
    };

    let mut cursor = main_offset;
    children
        .iter()
        .zip(main_sizes)
        .zip(margins)
        .map(|((child, main_size), margin)| {
            let (main_margin_start, main_margin_end, cross_margin_start, cross_margin_end) =
                match direction {
                    FlexDirection::Row => (margin.left, margin.right, margin.top, margin.bottom),
                    FlexDirection::Column => (margin.top, margin.bottom, margin.left, margin.right),
                };
            let child_style = node_style(child);
            let (explicit_cross, intrinsic_cross) = match direction {
                FlexDirection::Row => (
                    child_style.height.map(|d| d.resolve(inner_height)),
                    node_height(child, inner_height),
                ),
                FlexDirection::Column => (
                    child_style.width.map(|d| d.resolve(inner_width)),
                    node_width(child, inner_width),
                ),
            };
            let available_cross =
                (cross_size - cross_margin_start - cross_margin_end).max(0.0);
            let cross_size_for_child = explicit_cross.unwrap_or_else(|| {
                match style.align_items.unwrap_or(AlignItems::Stretch) {
                    AlignItems::Stretch => available_cross,
                    _ => intrinsic_cross.min(available_cross),
                }
            });
            let cross_offset = match style.align_items.unwrap_or(AlignItems::Stretch) {
                AlignItems::Center => {
                    cross_margin_start
                        + (available_cross - cross_size_for_child).max(0.0) / 2.0
                }
                AlignItems::End => cross_size - cross_margin_end - cross_size_for_child,
                _ => cross_margin_start,
            };
            cursor += main_margin_start;
            let layout = match direction {
                FlexDirection::Row => ChildLayout {
                    x: padding + cursor,
                    y: padding + cross_offset,
                    width: main_size,
                    height: cross_size_for_child,
                },
                FlexDirection::Column => ChildLayout {
                    x: padding + cross_offset,
                    y: padding + cursor,
                    width: cross_size_for_child,
                    height: main_size,
                },
            };
            cursor += main_size + main_margin_end + effective_gap;
            layout
        })
        .collect()
}

#[derive(Debug, Clone)]
struct StateConfig {
    name: String,
    setter: String,
    initial: i64,
}

#[derive(Debug)]
struct UiProgram {
    nodes: Vec<IRNode>,
    state: Option<StateConfig>,
}

struct JsxToIrVisitor {
    ir_out: Vec<IRNode>,
    state: Option<StateConfig>,
}

impl Visit for JsxToIrVisitor {
    fn visit_jsx_element(&mut self, node: &JSXElement) {
        let tag_name = match &node.opening.name {
            JSXElementName::Ident(ident) => ident.sym.as_str(),
            _ => return,
        };

        let style = jsx_style(node);
        let title = jsx_string_attr(node, "title").unwrap_or_default();
        let source = jsx_string_attr(node, "source")
            .or_else(|| jsx_string_attr(node, "src"))
            .unwrap_or_default();
        let placeholder = jsx_string_attr(node, "placeholder").unwrap_or_default();
        let updates_state = self.state.as_ref().is_some_and(|state| {
            jsx_attr(node, "onClick")
                .and_then(|attr| attr.value.as_ref())
                .is_some_and(|value| {
                    is_increment_handler(value, &state.setter, &state.name)
                })
        });
        let unsupported_handler = jsx_attr(node, "onClick").is_some() && !updates_state;
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
                    if let (Some(state), JSXExpr::Expr(expr)) = (&self.state, &container.expr) {
                        if let Expr::Ident(ident) = expr.as_ref() {
                            if ident.sym.as_str() == state.name {
                                children.push(IRNode::StateText {
                                    initial: state.initial,
                                    style: InlineStyle::default(),
                                });
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
                if let Some(initial) = children.iter().find_map(|child| match child {
                    IRNode::StateText { initial, .. } => Some(*initial),
                    _ => None,
                }) {
                    self.ir_out.push(IRNode::StateText { initial, style });
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
                    updates_state,
                    unsupported_handler,
                });
            }
            "Image" => self.ir_out.push(IRNode::Image {
                source,
                mode: ImageMode::from_attribute(
                    jsx_string_attr(node, "mode").as_deref(),
                ),
                style,
            }),
            "Input" => self
                .ir_out
                .push(IRNode::Input { placeholder, style }),
            _ => {}
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

    let state = parse_state_config(src);
    let mut visitor = JsxToIrVisitor {
        ir_out: vec![],
        state: state.clone(),
    };
    module.visit_with(&mut visitor);
    UiProgram {
        nodes: visitor.ir_out,
        state,
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

fn jsx_style(node: &JSXElement) -> InlineStyle {
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
        let Prop::KeyValue(property) = prop.as_ref() else {
            continue;
        };
        let key = match &property.key {
            PropName::Ident(ident) => ident.sym.as_str(),
            PropName::Str(string) => string.value.as_str().unwrap_or(""),
            _ => continue,
        };
        match key {
            "width" => style.width = jsx_dimension_expr(&property.value),
            "height" => style.height = jsx_dimension_expr(&property.value),
            "padding" => style.padding = jsx_number_expr(&property.value),
            "margin" => style.margin = jsx_number_expr(&property.value),
            "marginHorizontal" => style.margin_horizontal = jsx_number_expr(&property.value),
            "marginVertical" => style.margin_vertical = jsx_number_expr(&property.value),
            "marginTop" => style.margin_top = jsx_number_expr(&property.value),
            "marginRight" => style.margin_right = jsx_number_expr(&property.value),
            "marginBottom" => style.margin_bottom = jsx_number_expr(&property.value),
            "marginLeft" => style.margin_left = jsx_number_expr(&property.value),
            "spacing" | "gap" => style.spacing = jsx_number_expr(&property.value),
            "flexDirection" => {
                style.flex_direction = jsx_string_expr(&property.value)
                    .and_then(|value| match value.as_str() {
                        "row" => Some(FlexDirection::Row),
                        "column" => Some(FlexDirection::Column),
                        _ => None,
                    })
            }
            "justifyContent" => {
                style.justify_content = jsx_string_expr(&property.value)
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
                style.align_items = jsx_string_expr(&property.value)
                    .and_then(|value| match value.as_str() {
                        "flex-start" => Some(AlignItems::Start),
                        "center" => Some(AlignItems::Center),
                        "flex-end" => Some(AlignItems::End),
                        "stretch" => Some(AlignItems::Stretch),
                        _ => None,
                    })
            }
            "flexGrow" => style.flex_grow = jsx_number_expr(&property.value).unwrap_or(0.0),
            "fontSize" => style.font_size = jsx_number_expr(&property.value),
            "textAlign" => {
                style.text_align = jsx_string_expr(&property.value).and_then(|value| match value.as_str() {
                    "left" => Some(TextAlignment::Left),
                    "center" => Some(TextAlignment::Center),
                    "right" => Some(TextAlignment::Right),
                    _ => None,
                })
            }
            "backgroundColor" => style.background_color = jsx_string_expr(&property.value),
            "color" => style.color = jsx_string_expr(&property.value),
            "borderWidth" => style.border_width = jsx_nonnegative_number_expr(&property.value),
            "borderColor" => style.border_color = jsx_string_expr(&property.value),
            "borderRadius" => style.border_radius = jsx_nonnegative_number_expr(&property.value),
            _ => {}
        }
    }
    style
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

fn jsx_number_expr(expr: &Expr) -> Option<f64> {
    match expr {
        Expr::Lit(Lit::Num(number)) if number.value.is_finite() => Some(number.value),
        _ => None,
    }
}

fn jsx_nonnegative_number_expr(expr: &Expr) -> Option<f64> {
    jsx_number_expr(expr).filter(|value| *value >= 0.0)
}

fn jsx_string_expr(expr: &Expr) -> Option<String> {
    match expr {
        Expr::Lit(Lit::Str(string)) => Some(string.value.to_string_lossy().into_owned()),
        _ => None,
    }
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

fn is_increment_handler(value: &JSXAttrValue, setter: &str, state_name: &str) -> bool {
    let JSXAttrValue::JSXExprContainer(container) = value else {
        return false;
    };
    let JSXExpr::Expr(expr) = &container.expr else {
        return false;
    };
    let Expr::Arrow(arrow) = expr.as_ref() else {
        return false;
    };
    let ArrowFunctionBody::Expr(body) = arrow.body.as_ref() else {
        return false;
    };
    let Expr::Call(call) = body.as_ref() else {
        return false;
    };
    let Callee::Expr(callee) = &call.callee else {
        return false;
    };
    let Expr::Ident(callee) = callee.as_ref() else {
        return false;
    };
    if callee.sym.as_str() != setter || call.args.len() != 1 {
        return false;
    }
    let Expr::Bin(binary) = call.args[0].expr.as_ref() else {
        return false;
    };
    binary.op == BinaryOp::Add
        && matches!(binary.left.as_ref(), Expr::Ident(ident) if ident.sym.as_str() == state_name)
        && matches!(binary.right.as_ref(), Expr::Lit(Lit::Num(number)) if number.value == 1.0)
}

fn parse_state_config(src: &str) -> Option<StateConfig> {
    let hook_start = src.find("useState(")?;
    let binding_end = src[..hook_start].rfind(']')?;
    let binding_start = src[..binding_end].rfind('[')?;
    let bindings: Vec<_> = src[binding_start + 1..binding_end]
        .split(',')
        .map(str::trim)
        .collect();
    let name = *bindings.first()?;
    let setter = *bindings.get(1)?;
    if name.is_empty() || setter.is_empty() {
        return None;
    }
    let value_start = hook_start + "useState(".len();
    let value_end = src[value_start..].find(')')? + value_start;
    let initial = src[value_start..value_end].trim().parse().ok()?;
    Some(StateConfig {
        name: name.to_string(),
        setter: setter.to_string(),
        initial,
    })
}

// ---------- 新增部分：IRNode -> iOS LLVM IR ----------

use inkwell::AddressSpace;
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::targets::TargetTriple;
use inkwell::types::{BasicMetadataTypeEnum, BasicType, BasicTypeEnum};
use inkwell::values::{
    BasicMetadataValueEnum, BasicValueEnum, FunctionValue, PointerValue,
};

struct IosCodeGen<'ctx> {
    context: &'ctx Context,
    module: Module<'ctx>,
    builder: Builder<'ctx>,
    // 缓存三个 Objective-C runtime 的外部函数声明
    objc_get_class: FunctionValue<'ctx>,
    objc_msg_send: FunctionValue<'ctx>,
    sel_register_name: FunctionValue<'ctx>,
}

impl<'ctx> IosCodeGen<'ctx> {
    fn new(context: &'ctx Context) -> Self {
        let module = context.create_module("ios_app");
        let builder = context.create_builder();

        // 设置目标三元组：arm64-apple-ios
        module.set_triple(&TargetTriple::create("arm64-apple-ios"));

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
            context,
            module,
            builder,
            objc_get_class,
            objc_msg_send,
            sel_register_name,
        }
    }

    fn generate(&mut self, program: &UiProgram) -> Result<(), String> {
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

        if let Some(state) = &program.state {
            let state_global = self.module.add_global(
                self.context.i64_type(),
                None,
                "g_state_value",
            );
            state_global.set_initializer(&self.context.i64_type().const_int(
                state.initial as u64,
                true,
            ));
            let state_label = self.module.add_global(ptr_type, None, "g_state_label");
            state_label.set_initializer(&ptr_type.const_null());
        }

        // Build the view hierarchy from the app delegate after UIKit has launched.
        self.build_app_delegate_class(program)?;

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

    /// 动态构建 AppDelegate 类，并实现 application:didFinishLaunchingWithOptions: 方法
    fn build_app_delegate_class(&self, program: &UiProgram) -> Result<(), String> {
        let ptr_type = self.context.ptr_type(AddressSpace::default());
        let i8_ptr = ptr_type;

        // 声明 Objective-C 运行时函数
        // Class objc_allocateClassPair(Class superclass, const char *name, size_t extraBytes)
        let allocate_class_type = ptr_type.fn_type(&[ptr_type.into(), i8_ptr.into(), self.context.i64_type().into()], false);
        let objc_allocate_class_pair = self.module.add_function("objc_allocateClassPair", allocate_class_type, None);

        // BOOL class_addMethod(Class cls, SEL name, IMP imp, const char *types)
        let add_method_type = self.context.bool_type().fn_type(&[ptr_type.into(), ptr_type.into(), ptr_type.into(), i8_ptr.into()], false);
        let class_add_method = self.module.add_function("class_addMethod", add_method_type, None);

        // void objc_registerClassPair(Class cls)
        let register_class_type = self.context.void_type().fn_type(&[ptr_type.into()], false);
        let objc_register_class_pair = self.module.add_function("objc_registerClassPair", register_class_type, None);

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

        // Implement application:didFinishLaunchingWithOptions:, which returns BOOL.
        let bool_type = self.context.i8_type();
        let did_finish_launching_fn_type = bool_type.fn_type(&[
            ptr_type.into(), // self
            ptr_type.into(), // _cmd
            ptr_type.into(), // application
            ptr_type.into(), // launchOptions
        ], false);
        let did_finish_launching_fn = self.module.add_function("app_delegate_did_finish_launching", did_finish_launching_fn_type, None);
        let fn_entry = self.context.append_basic_block(did_finish_launching_fn, "entry");
        // 保存当前的插入点，稍后恢复
        let saved_block = self.builder.get_insert_block().unwrap();
        self.builder.position_at_end(fn_entry);

        let event_target = if program.state.is_some() {
            Some(
                did_finish_launching_fn
                    .get_nth_param(0)
                    .ok_or("app delegate self parameter is missing")?
                    .into_pointer_value(),
            )
        } else {
            None
        };
        let root_view = if let Some(node) = program.nodes.first() {
            self.codegen_node(node, event_target, 0.0, 0.0, 350.0, 700.0, None, true)?
        } else {
            self.create_objc_instance_with_frame("UIView", self.screen_bounds()?)?
        };
        let root_view_global = self
            .module
            .get_global("g_root_view")
            .ok_or("root view global was not created")?;
        self.builder
            .build_store(root_view_global.as_pointer_value(), root_view)
            .map_err(|e| e.to_string())?;

        // Create the window and attach the generated root view.
        self.build_show_window_logic(program.nodes.first())?;

        self.builder
            .build_return(Some(&bool_type.const_int(1, false)))
            .map_err(|e| e.to_string())?;
        // 恢复插入点
        self.builder.position_at_end(saved_block);

        // 将方法添加到类中
        let sel_register = self.get_selector("application:didFinishLaunchingWithOptions:")?;
        // Objective-C type encoding: BOOL, self, _cmd, application, launchOptions.
        let type_encoding = self
            .builder
            .build_global_string_ptr("c@:@@", "method_type_encoding")
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

        if program.state.is_some() {
            let callback = self.build_state_callback_method()?;
            let callback_selector = self.get_selector("incrementState:")?;
            let callback_encoding = self
                .builder
                .build_global_string_ptr("v@:@", "state_callback_encoding")
                .map_err(|e| e.to_string())?;
            self.builder
                .build_call(
                    class_add_method,
                    &[
                        app_delegate_class.into(),
                        callback_selector.into(),
                        callback.as_global_value().as_pointer_value().into(),
                        callback_encoding.as_pointer_value().into(),
                    ],
                    "",
                )
                .map_err(|e| e.to_string())?;
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

    fn build_state_callback_method(&self) -> Result<FunctionValue<'ctx>, String> {
        let ptr_type = self.context.ptr_type(AddressSpace::default());
        let callback_type = self
            .context
            .void_type()
            .fn_type(&[ptr_type.into(), ptr_type.into(), ptr_type.into()], false);
        let callback = self
            .module
            .add_function("app_delegate_increment_state", callback_type, None);
        let entry = self.context.append_basic_block(callback, "entry");
        let saved_block = self
            .builder
            .get_insert_block()
            .ok_or("missing insertion block while generating state callback")?;
        self.builder.position_at_end(entry);

        let state_global = self
            .module
            .get_global("g_state_value")
            .ok_or("state value global was not created")?;
        let state_ptr = state_global.as_pointer_value();
        let current = self
            .builder
            .build_load(self.context.i64_type(), state_ptr, "state_current")
            .map_err(|e| e.to_string())?
            .into_int_value();
        let next = self
            .builder
            .build_int_add(
                current,
                self.context.i64_type().const_int(1, false),
                "state_next",
            )
            .map_err(|e| e.to_string())?;
        self.builder
            .build_store(state_ptr, next)
            .map_err(|e| e.to_string())?;

        let buffer_type = self.context.i8_type().array_type(32);
        let buffer = self.module.add_global(buffer_type, None, "g_state_text_buffer");
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
        let snprintf = self.module.add_function("snprintf", snprintf_type, None);
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

        let text = self.create_nsstring_from_cstr(buffer.as_pointer_value())?;
        let label_global = self
            .module
            .get_global("g_state_label")
            .ok_or("state label global was not created")?;
        let label = self
            .builder
            .build_load(ptr_type, label_global.as_pointer_value(), "state_label")
            .map_err(|e| e.to_string())?
            .into_pointer_value();
        self.send_msg_void(label, "setText:", &[text.into()])?;
        self.builder
            .build_return(None)
            .map_err(|e| e.to_string())?;
        self.builder.position_at_end(saved_block);

        Ok(callback)
    }

    /// 生成显示窗口的 IR 逻辑
    fn build_show_window_logic(&self, root_node: Option<&IRNode>) -> Result<(), String> {
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

        // 设置 window 的 rootViewController
        self.send_msg_void(window, "setRootViewController:", &[view_controller.into()])?;

        // 调用 makeKeyAndVisible
        self.send_msg_void(window, "makeKeyAndVisible", &[])?;

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
    ) -> Result<(), String> {
        let direction = style.flex_direction.unwrap_or(default_direction);
        let padding = style.padding.unwrap_or(0.0);
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
        let main_margins_total: f64 = child_nodes
            .iter()
            .map(node_margins)
            .map(|margin| match direction {
                FlexDirection::Row => margin.left + margin.right,
                FlexDirection::Column => margin.top + margin.bottom,
            })
            .sum();
        let used_main = main_sizes.iter().sum::<f64>()
            + main_margins_total
            + gap * children.len().saturating_sub(1) as f64;
        let main_extent = if direction == FlexDirection::Row {
            width - 2.0 * padding
        } else {
            height - 2.0 * padding
        };
        let free_space = (main_extent - used_main).max(0.0);
        let justify = style.justify_content.unwrap_or(JustifyContent::Start);

        for (index, ((child, child_node), layout)) in children
            .iter()
            .zip(child_nodes)
            .zip(layouts)
            .enumerate()
        {
            let margins = node_margins(child_node);
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
            if !main_axis_auto {
                self.add_dimension_constraint(
                    *child,
                    if direction == FlexDirection::Row {
                        "widthAnchor"
                    } else {
                        "heightAnchor"
                    },
                    main_sizes[index],
                )?;
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
            if cross_is_stretched {
                self.add_anchor_constraint(
                    *child,
                    cross_start,
                    parent,
                    cross_start,
                    padding + cross_margin_start,
                )?;
                self.add_anchor_constraint(
                    *child,
                    cross_end,
                    parent,
                    cross_end,
                    -padding - cross_margin_end,
                )?;
            } else {
                if !cross_axis_auto {
                    self.add_dimension_constraint(
                        *child,
                        if direction == FlexDirection::Row {
                            "heightAnchor"
                        } else {
                            "widthAnchor"
                        },
                        cross_size,
                    )?;
                }
                match align {
                    AlignItems::Center => {
                        let (position, container_size, child_size) = match direction {
                            FlexDirection::Row => (layout.y, height, layout.height),
                            FlexDirection::Column => (layout.x, width, layout.width),
                        };
                        self.add_anchor_constraint(
                            *child,
                            cross_center,
                            parent,
                            cross_center,
                            position + child_size / 2.0 - container_size / 2.0,
                        )?;
                    }
                    AlignItems::End => self.add_anchor_constraint(
                        *child,
                        cross_end,
                        parent,
                        cross_end,
                        -padding - cross_margin_end,
                    )?,
                    AlignItems::Start | AlignItems::Stretch => self.add_anchor_constraint(
                        *child,
                        cross_start,
                        parent,
                        cross_start,
                        padding + cross_margin_start,
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
                    if index == 0 {
                        self.add_anchor_constraint(
                            *child,
                            main_start,
                            parent,
                            main_start,
                            padding + main_margin_start,
                        )?;
                    } else {
                        let previous_margins = node_margins(&child_nodes[index - 1]);
                        let previous_end_margin = match direction {
                            FlexDirection::Row => previous_margins.right,
                            FlexDirection::Column => previous_margins.bottom,
                        };
                        self.add_anchor_constraint(
                            *child,
                            main_start,
                            children[index - 1],
                            main_end,
                            previous_end_margin + gap + main_margin_start,
                        )?;
                    }
                }
                JustifyContent::End => {
                    if index + 1 == children.len() {
                        self.add_anchor_constraint(
                            *child,
                            main_end,
                            parent,
                            main_end,
                            -padding - main_margin_end,
                        )?;
                    } else {
                        let next_margins = node_margins(&child_nodes[index + 1]);
                        let next_start_margin = match direction {
                            FlexDirection::Row => next_margins.left,
                            FlexDirection::Column => next_margins.top,
                        };
                        self.add_anchor_constraint(
                            *child,
                            main_end,
                            children[index + 1],
                            main_start,
                            -gap - next_start_margin - main_margin_end,
                        )?;
                    }
                }
                JustifyContent::Center => {
                    if index == 0 {
                        let offset =
                            -used_main / 2.0 + main_margin_start + main_sizes[index] / 2.0;
                        self.add_anchor_constraint(*child, main_center, parent, main_center, offset)?;
                    } else {
                        let previous_margins = node_margins(&child_nodes[index - 1]);
                        let previous_end_margin = match direction {
                            FlexDirection::Row => previous_margins.right,
                            FlexDirection::Column => previous_margins.bottom,
                        };
                        self.add_anchor_constraint(
                            *child,
                            main_start,
                            children[index - 1],
                            main_end,
                            previous_end_margin + gap + main_margin_start,
                        )?;
                    }
                }
                JustifyContent::SpaceBetween => {
                    if index == 0 {
                        self.add_anchor_constraint(
                            *child,
                            main_start,
                            parent,
                            main_start,
                            padding + main_margin_start,
                        )?;
                    } else if index + 1 == children.len() {
                        self.add_anchor_constraint(
                            *child,
                            main_end,
                            parent,
                            main_end,
                            -padding - main_margin_end,
                        )?;
                    } else {
                        let distributed_gap = gap + free_space / children.len().saturating_sub(1).max(1) as f64;
                        let previous_margins = node_margins(&child_nodes[index - 1]);
                        let previous_end_margin = match direction {
                            FlexDirection::Row => previous_margins.right,
                            FlexDirection::Column => previous_margins.bottom,
                        };
                        self.add_anchor_constraint(
                            *child,
                            main_start,
                            children[index - 1],
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
                let view = self.create_objc_instance_with_frame("UIView", view_frame)?;
                self.apply_inline_style(view, style, false)?;
                if style.background_color.is_none() {
                    let white = self.ui_color("white")?;
                    self.send_msg_void(view, "setBackgroundColor:", &[white.into()])?;
                }
                let padding = style.padding.unwrap_or(0.0);
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
                        (width - 2.0 * padding).max(0.0),
                        (height - 2.0 * padding).max(0.0),
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
                )?;
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
                let stack = self.create_objc_instance_with_frame("UIView", frame)?;
                self.apply_inline_style(stack, style, false)?;
                let padding = style.padding.unwrap_or(0.0);
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
                        (width - 2.0 * padding).max(0.0),
                        (height - 2.0 * padding).max(0.0),
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
                )?;
                Ok(stack)
            }
            IRNode::ScrollView { style, children } => {
                let width = allocated.map(|layout| layout.width).unwrap_or_else(|| {
                    style.width.map(|dimension| dimension.resolve(available_width))
                        .unwrap_or_else(|| node_width(node, available_width))
                });
                let height = allocated.map(|layout| layout.height)
                    .unwrap_or_else(|| node_height(node, available_height));
                let frame = self.rect_constant(x, y, width, height);
                let scroll_view = self.create_objc_instance_with_frame("UIScrollView", frame)?;
                self.apply_inline_style(scroll_view, style, false)?;
                if style.background_color.is_none() {
                    let white = self.ui_color("white")?;
                    self.send_msg_void(scroll_view, "setBackgroundColor:", &[white.into()])?;
                }
                let padding = style.padding.unwrap_or(0.0);
                let child_layouts = layout_flex_children(
                    children,
                    style,
                    width,
                    height,
                    FlexDirection::Column,
                    0.0,
                );
                let content_width = child_layouts
                    .iter()
                    .zip(children)
                    .map(|(layout, child)| {
                        layout.x + layout.width + node_margins(child).right
                    })
                    .fold(width, f64::max);
                let content_height = child_layouts
                    .iter()
                    .zip(children)
                    .map(|(layout, child)| {
                        layout.y + layout.height + node_margins(child).bottom
                    })
                    .fold(height, f64::max);
                for (child, layout) in children.iter().zip(&child_layouts) {
                    let child_view = self.codegen_node(
                        child,
                        event_target,
                        0.0,
                        0.0,
                        (width - 2.0 * padding).max(0.0),
                        (height - 2.0 * padding).max(0.0),
                        Some(*layout),
                        false,
                    )?;
                    self.send_msg_void(scroll_view, "addSubview:", &[child_view.into()])?;
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
                let label =
                    self.create_objc_instance_with_frame(
                        "UILabel",
                        self.rect_constant(
                            x,
                            y,
                            allocated_width.unwrap_or_else(|| {
                                style.width.map(|dimension| dimension.resolve(available_width))
                                    .unwrap_or(available_width)
                            }),
                            allocated_height.unwrap_or_else(|| node_height(node, available_height)),
                        ),
                    )?;
                let nsstring = self.create_nsstring(content)?;
                self.send_msg_void(label, "setText:", &[nsstring.into()])?;
                self.apply_inline_style(label, style, true)?;
                Ok(label)
            }
            IRNode::StateText { initial, style } => {
                let label =
                    self.create_objc_instance_with_frame(
                        "UILabel",
                        self.rect_constant(
                            x,
                            y,
                            allocated_width.unwrap_or_else(|| {
                                style.width.map(|dimension| dimension.resolve(available_width))
                                    .unwrap_or(available_width)
                            }),
                            allocated_height.unwrap_or_else(|| node_height(node, available_height)),
                        ),
                    )?;
                let nsstring = self.create_nsstring(&initial.to_string())?;
                self.send_msg_void(label, "setText:", &[nsstring.into()])?;
                self.apply_inline_style(label, style, true)?;
                let state_label = self
                    .module
                    .get_global("g_state_label")
                    .ok_or("state label global was not created")?;
                self.builder
                    .build_store(state_label.as_pointer_value(), label)
                    .map_err(|e| e.to_string())?;
                Ok(label)
            }
            IRNode::Image { source, mode, style } => {
                if source.is_empty() {
                    return Err("Image requires a non-empty src or source asset name".into());
                }
                let image_name = self.create_nsstring(source)?;
                let image_class = self.get_class("UIImage")?;
                let image_named = self.get_selector("imageNamed:")?;
                let image = self
                    .send_msg_ptr(image_class, image_named, &[image_name.into()])?
                    .ok_or("UIImage imageNamed: returned void")?;
                let image_view_class = self.get_class("UIImageView")?;
                let alloc = self.get_selector("alloc")?;
                let image_view = self
                    .send_msg_ptr(image_view_class, alloc, &[])?
                    .ok_or("UIImageView alloc returned void")?;
                let init_with_image = self.get_selector("initWithImage:")?;
                let image_view = self
                    .send_msg_ptr(image_view, init_with_image, &[image.into()])?
                    .ok_or("UIImageView initWithImage: returned void")?;
                self.send_msg_void(
                    image_view,
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
                self.send_msg_void(
                    image_view,
                    "setContentMode:",
                    &[self.context.i64_type().const_int(mode.content_mode(), false).into()],
                )?;
                if *mode == ImageMode::AspectFill {
                    self.send_msg_void(
                        image_view,
                        "setClipsToBounds:",
                        &[self.context.bool_type().const_int(1, false).into()],
                    )?;
                }
                self.apply_inline_style(image_view, style, false)?;
                Ok(image_view)
            }
            IRNode::Input { placeholder, style } => {
                let input = self.create_objc_instance_with_frame(
                    "UITextField",
                    self.rect_constant(
                        x,
                        y,
                        allocated_width.unwrap_or_else(|| {
                            style.width.map(|dimension| dimension.resolve(available_width))
                                .unwrap_or(available_width)
                        }),
                        allocated_height.unwrap_or_else(|| node_height(node, available_height)),
                    ),
                )?;
                if !placeholder.is_empty() {
                    let placeholder = self.create_nsstring(placeholder)?;
                    self.send_msg_void(input, "setPlaceholder:", &[placeholder.into()])?;
                }
                self.send_msg_void(
                    input,
                    "setBorderStyle:",
                    &[self.context.i64_type().const_int(3, false).into()],
                )?;
                self.apply_inline_style(input, style, true)?;
                Ok(input)
            }
            IRNode::Button {
                title,
                style,
                updates_state,
                unsupported_handler,
            } => {
                if *unsupported_handler {
                    return Err(
                        "supported Button onClick form is () => setState(state + 1) with a numeric useState"
                            .into(),
                    );
                }
                let class = self.get_class("UIButton")?;
                let button_type = self.get_selector("buttonWithType:")?;
                let button = self
                    .send_msg_ptr(
                        class,
                        button_type,
                        &[self.context.i64_type().const_int(0, false).into()],
                    )?
                    .ok_or("UIButton buttonWithType: returned void")?;
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
                self.send_msg_void(
                    button,
                    "setTitle:forState:",
                    &[title.into(), self.context.i64_type().const_int(0, false).into()],
                )?;
                self.apply_inline_style(button, style, false)?;
                if let Some(color_name) = &style.color {
                    let color = self.ui_color(color_name)?;
                    self.send_msg_void(
                        button,
                        "setTitleColor:forState:",
                        &[color.into(), self.context.i64_type().const_int(0, false).into()],
                    )?;
                }
                if let Some(font_size) = style.font_size {
                    let title_label_sel = self.get_selector("titleLabel")?;
                    let title_label = self
                        .send_msg_ptr(button, title_label_sel, &[])?
                        .ok_or("UIButton titleLabel returned void")?;
                    let font_class = self.get_class("UIFont")?;
                    let system_font_sel = self.get_selector("systemFontOfSize:")?;
                    let font = self
                        .send_msg_ptr(
                            font_class,
                            system_font_sel,
                            &[self.context.f64_type().const_float(font_size).into()],
                        )?
                        .ok_or("UIFont systemFontOfSize: returned void")?;
                    self.send_msg_void(title_label, "setFont:", &[font.into()])?;
                }

                if *updates_state {
                    let target = event_target.ok_or(
                        "Button onClick needs a useState hook in this initial implementation",
                    )?;
                    let action = self.get_selector("incrementState:")?;
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
                Ok(button)
            }
        }
    }

    fn apply_inline_style(
        &self,
        view: PointerValue<'ctx>,
        style: &InlineStyle,
        text_like: bool,
    ) -> Result<(), String> {
        if let Some(color_name) = &style.background_color {
            let color = self.ui_color(color_name)?;
            self.send_msg_void(view, "setBackgroundColor:", &[color.into()])?;
        }
        if style.border_width.is_some() || style.border_color.is_some() || style.border_radius.is_some() {
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
        if text_like {
            if let Some(alignment) = style.text_align {
                self.send_msg_void(
                    view,
                    "setTextAlignment:",
                    &[self.context.i64_type().const_int(alignment.ui_value(), false).into()],
                )?;
            }
            if let Some(color_name) = &style.color {
                let color = self.ui_color(color_name)?;
                self.send_msg_void(view, "setTextColor:", &[color.into()])?;
            }
            if let Some(font_size) = style.font_size {
                let font_class = self.get_class("UIFont")?;
                let system_font = self.get_selector("systemFontOfSize:")?;
                let font = self
                    .send_msg_ptr(
                        font_class,
                        system_font,
                        &[self.context.f64_type().const_float(font_size).into()],
                    )?
                    .ok_or("UIFont systemFontOfSize: returned void")?;
                self.send_msg_void(view, "setFont:", &[font.into()])?;
            }
        }
        Ok(())
    }

    fn ui_color(&self, value: &str) -> Result<PointerValue<'ctx>, String> {
        if let Some((red, green, blue, alpha)) = parse_functional_color(value)? {
            let color_class = self.get_class("UIColor")?;
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
        let color_class = self.get_class("UIColor")?;
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

fn main() {
    let tsx_code = r#"
function App() {
    const [count, setCount] = useState(0);
    return <View style={{
            width: "100%", height: "100%",
            backgroundColor: "blue",
            flexDirection: "column",
            justifyContent: "space-around",
            alignItems: "center",
            gap: 8
        }}>
            <View style={{ 
                width: 200, 
                height: 200,  
                borderWidth: 2,
                borderColor: "rgba(255, 0, 0, 0.5)",
                borderRadius: 12, 
                backgroundColor: "rgb(211, 242, 100)" 
            }} 
            />
            <ScrollView style={{ height: 160, backgroundColor: "red" }}>
                    <View style={{ padding: 20, flexDirection: "column", alignItems: "center" }}>
                        <Text style={{ fontSize: 24, color: "blue", textAlign: "center" }}>{count}</Text>
                        <Button
                            title="Increment"
                            onClick={() => setCount(count + 1)}
                            style={{ marginVertical: 12, width: 200, height: 48, backgroundColor: "blue", color: "red" }}
                        />
                        <Image src="logo" style={{ width: 200, height: 120 }} />
                        <Input placeholder="Enter text" style={{ width: 300, height: 48 }} />
                        <Text>Scrollable content</Text>
                        <Text>More content</Text>
                        <Text>Additional content</Text>
                    </View>
                </ScrollView>
        </View>
}
    "#;

    let program = parse_tsx_to_ast(tsx_code);
    println!("AST: {:#?}\n", program);

    let context = Context::create();
    let mut codegen = IosCodeGen::new(&context);
    if let Err(e) = codegen.generate(&program) {
        eprintln!("codegen failed: {e}");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
