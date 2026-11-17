//! 样式 → CSS 的映射，静态 HTML 后端和 wasm/DOM 后端共用一份。
//!
//! 这里是「布局权威」的落点：DOM 目标不把编译期算好的 frame 当坐标用，而是把同一套
//! style 翻译成 Flexbox，让浏览器自己排版。静态后端（`source_backend.rs`）直接拼进
//! `<div style="...">`，wasm 后端通过 `dom.style` 交给桥，两边拼出来的字符串一致，
//! `tools/parity` 的截图对照才有可比性。

use super::*;

/// 盒模型（尺寸、内外边距、背景、边框、定位）
pub(crate) fn css_box(style: &InlineStyle, _container: bool) -> String {
    let mut css = String::new();
    css.push_str(&css_layout(style));
    if let Some(opacity) = style.opacity_static {
        css.push_str(&format!("opacity: {opacity:.2};"));
    }
    let padding = node_padding(style);
    if !padding.is_zero() {
        css.push_str(&format!(
            "padding: {}px {}px {}px {}px;",
            padding.top, padding.right, padding.bottom, padding.left
        ));
    }
    let (margin_top, margin_right, margin_bottom, margin_left) = style_margins(style);
    if (margin_top, margin_right, margin_bottom, margin_left) != (0.0, 0.0, 0.0, 0.0) {
        css.push_str(&format!(
            "margin: {margin_top}px {margin_right}px {margin_bottom}px {margin_left}px;"
        ));
    }
    if let Some(color) = &style.background_color {
        css.push_str(&format!("background-color: {};", raw_color(color)));
    }
    if let Some(radius) = style.border_radius {
        css.push_str(&format!("border-radius: {radius}px;"));
    }
    if let Some(width) = style.border_width {
        let color = style
            .border_color
            .as_deref()
            .map(raw_color)
            .unwrap_or_else(|| "#000000".to_string());
        css.push_str(&format!("border: {width}px solid {color};"));
    }
    if style.position == Position::Absolute
        || style.left.is_some()
        || style.right.is_some()
        || style.top.is_some()
        || style.bottom.is_some()
    {
        css.push_str("position: absolute;");
        for (value, name) in [
            (style.top, "top"),
            (style.right, "right"),
            (style.bottom, "bottom"),
            (style.left, "left"),
        ] {
            if let Some(value) = value {
                css.push_str(&format!("{name}: {};", css_dimension(value)));
            }
        }
    }
    css
}

/// 尺寸与 flex 增长
pub(crate) fn css_layout(style: &InlineStyle) -> String {
    let mut css = String::new();
    if let Some(value) = style.width {
        css.push_str(&format!("width: {};", css_dimension(value)));
    }
    if let Some(value) = style.height {
        css.push_str(&format!("height: {};", css_dimension(value)));
    }
    if style.flex_grow > 0.0 {
        css.push_str(&format!(
            "flex-grow: {};flex-shrink: 1;flex-basis: 0;min-width: 0;min-height: 0;",
            style.flex_grow
        ));
    }
    css
}

pub(crate) fn css_justify(style: &InlineStyle) -> String {
    let value = match style.justify_content.unwrap_or(JustifyContent::Start) {
        JustifyContent::Start => "flex-start",
        JustifyContent::Center => "center",
        JustifyContent::End => "flex-end",
        JustifyContent::SpaceBetween => "space-between",
        JustifyContent::SpaceAround => "space-around",
    };
    format!("justify-content: {value};")
}

pub(crate) fn css_align(style: &InlineStyle) -> String {
    let value = match style.align_items.unwrap_or(AlignItems::Stretch) {
        AlignItems::Start => "flex-start",
        AlignItems::Center => "center",
        AlignItems::End => "flex-end",
        AlignItems::Stretch => "stretch",
    };
    format!("align-items: {value};")
}

pub(crate) fn css_text(style: &InlineStyle) -> String {
    let mut css = String::new();
    css.push_str(&format!(
        "font-size: {}px;",
        style.font_size.unwrap_or(DEFAULT_FONT_SIZE)
    ));
    if let Some(color) = &style.color {
        css.push_str(&format!("color: {};", raw_color(color)));
    }
    if let Some(align) = style.text_align {
        let value = match align {
            TextAlignment::Left => "left",
            TextAlignment::Center => "center",
            TextAlignment::Right => "right",
        };
        css.push_str(&format!("text-align: {value};"));
    }
    css
}

/// 外边距：margin 先铺满四边，marginHorizontal/Vertical 覆盖两轴，单边再覆盖。
pub(crate) fn style_margins(style: &InlineStyle) -> (f64, f64, f64, f64) {
    let all = style.margin.unwrap_or(0.0);
    let horizontal = style.margin_horizontal.unwrap_or(all);
    let vertical = style.margin_vertical.unwrap_or(all);
    (
        style.margin_top.unwrap_or(vertical),
        style.margin_right.unwrap_or(horizontal),
        style.margin_bottom.unwrap_or(vertical),
        style.margin_left.unwrap_or(horizontal),
    )
}

pub(crate) fn css_dimension(value: Dimension) -> String {
    match value {
        Dimension::Points(points) => format!("{points}px"),
        Dimension::Percent(fraction) => format!("{}%", fraction * 100.0),
    }
}

/// 颜色：functional 写法原样输出，命名色转成十六进制（与 iOS/Android 的取值一致）
pub(crate) fn raw_color(value: &str) -> String {
    match value.trim().to_ascii_lowercase().as_str() {
        "black" => "#000000".into(),
        "white" => "#FFFFFF".into(),
        "red" => "#FF3B30".into(),
        "blue" => "#007AFF".into(),
        "green" => "#34C759".into(),
        "gray" | "grey" => "#8E8E93".into(),
        "clear" | "transparent" => "transparent".into(),
        _ => value.trim().to_string(),
    }
}

// ---------- wasm/DOM 后端的成串样式 ----------
//
// 静态后端是「拼 HTML」，这里是「给已经建好的元素补一段 CSS」，所以每个函数只返回
// 该节点类型的完整声明串；调用顺序决定层叠顺序（后写的覆盖先写的）。

/// 容器：盒模型 + flex 方向 + 间距 + 主轴/交叉轴对齐。
/// 有绝对定位子节点时补 `position: relative`，否则浮层会相对祖先定位。
pub(crate) fn container_css(
    style: &InlineStyle,
    direction: FlexDirection,
    gap: f64,
    has_absolute_children: bool,
) -> String {
    let mut css = css_box(style, true);
    css.push_str(match direction {
        FlexDirection::Row => "display: flex;flex-direction: row;",
        FlexDirection::Column => "display: flex;flex-direction: column;",
    });
    if gap > 0.0 {
        css.push_str(&format!("gap: {gap}px;"));
    }
    if has_absolute_children {
        css.push_str("position: relative;");
    }
    css.push_str(&css_justify(style));
    css.push_str(&css_align(style));
    css
}

/// 容器里是否有绝对定位的子节点：有的话容器要成为它们的定位基准。
pub(crate) fn has_absolute_child(children: &[IRNode]) -> bool {
    children.iter().any(|child| {
        let style = node_style(child);
        style.position == Position::Absolute
            || style.left.is_some()
            || style.right.is_some()
            || style.top.is_some()
            || style.bottom.is_some()
    })
}

pub(crate) fn text_css(style: &InlineStyle) -> String {
    let mut css = css_box(style, false);
    css.push_str(&css_text(style));
    css.push_str(&css_layout(style));
    css.push_str("white-space: nowrap;overflow: hidden;text-overflow: ellipsis;line-height: 1.2;");
    css
}

pub(crate) fn button_css(style: &InlineStyle) -> String {
    let mut css = String::from(
        "appearance: none;border: none;margin: 0;font: inherit;cursor: pointer;",
    );
    css.push_str(&css_box(style, false));
    css.push_str("display: flex;align-items: center;justify-content: center;");
    css.push_str(&css_text(style));
    css.push_str(&css_layout(style));
    css
}

pub(crate) fn input_css(style: &InlineStyle) -> String {
    let mut css = String::from("border: none;outline: none;font: inherit;");
    css.push_str(&css_box(style, false));
    css.push_str(&css_text(style));
    css.push_str("line-height: 1.2;");
    css
}

pub(crate) fn image_css(style: &InlineStyle) -> String {
    let mut css = css_box(style, false);
    css.push_str(
        "display: flex;align-items: center;justify-content: center;color: #9aa3b2;font-size: 12px;",
    );
    css
}

/// 滚动容器：外层管尺寸/圆角/背景，内层管 padding 与 gap（和静态后端一致）。
pub(crate) fn scroll_css(style: &InlineStyle) -> String {
    let mut css = css_box(style, true);
    css.push_str("overflow-y: auto;display: flex;flex-direction: column;");
    css
}

pub(crate) fn scroll_content_css(style: &InlineStyle, gap: f64) -> String {
    let mut css = String::from("display: flex;flex-direction: column;width: 100%;");
    let padding = node_padding(style);
    css.push_str(&format!(
        "padding: {}px {}px {}px {}px;",
        padding.top, padding.right, padding.bottom, padding.left
    ));
    if gap > 0.0 {
        css.push_str(&format!("gap: {gap}px;"));
    }
    css
}
