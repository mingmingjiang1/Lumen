//! 生成源码型后端：不调用平台 API，而是把 IR 翻译成目标平台的源码。
//!
//! 当前实现的是 `--target=web`：输出一份静态 HTML + CSS Flexbox。
//! 它有两个用处：
//! 1. 证明后端形态可以不止「发消息建控件」——也可以「生成源码」（HarmonyOS 的
//!    ArkTS、WidgetKit 的 SwiftUI 都得走这条路）；
//! 2. 作为一个可截图、可自动断言的视觉基准，参与 `tools/parity` 的三端对照。
//!
//! 说明：静态 HTML 不带交互（onClick/onInput/onFocus 只会在能力表里记成 Emulated），
//! 它只负责把「布局 + 样式 + 初始状态」渲染出来。

use std::collections::BTreeSet;

use super::*;
use crate::css::*;

pub(crate) fn generate_html(program: &UiProgram) -> Result<String, String> {
    let mut placeholders: Vec<(String, String)> = Vec::new();
    let mut counter = 0usize;
    let mut body = String::new();
    for node in &program.nodes {
        emit_node(node, 3, &mut counter, &mut placeholders, &mut body, None)?;
    }

    let mut placeholder_rules = String::new();
    for (class, color) in &placeholders {
        placeholder_rules.push_str(&format!(
            "      .{class}::placeholder {{ color: {color}; opacity: 1; }}\n"
        ));
    }

    Ok(format!(
        "<!DOCTYPE html>\n<html lang=\"zh\">\n<head>\n  <meta charset=\"utf-8\">\n  <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n  <title>Lumen (web)</title>\n  <style>\n    html, body {{ margin: 0; height: 100%; background: #ffffff; }}\n    body {{ font-family: -apple-system, \"PingFang SC\", \"Microsoft YaHei\", Roboto, sans-serif; }}\n    * {{ box-sizing: border-box; }}\n{placeholder_rules}  </style>\n</head>\n<body>\n{body}</body>\n</html>\n"
    ))
}

/// 行上下文：列表模板渲染时的字段表 + 当前行数据。
type RowContext<'a> = Option<(&'a [(String, StateValue)], &'a [StateValue])>;

fn emit_node(
    node: &IRNode,
    indent: usize,
    counter: &mut usize,
    placeholders: &mut Vec<(String, String)>,
    out: &mut String,
    row: RowContext<'_>,
) -> Result<(), String> {
    let pad = " ".repeat(indent);
    match node {
        IRNode::View { style, children } | IRNode::VStack { style, children } => {
            let default_gap = if matches!(node, IRNode::VStack { .. }) { 8.0 } else { 0.0 };
            emit_container(
                style,
                children,
                FlexDirection::Column,
                default_gap,
                pad,
                counter,
                placeholders,
                out,
                row,
            )?;
        }
        IRNode::ScrollView { style, children } => {
            // 滚动容器：外层负责尺寸/圆角/背景，内层负责 padding + gap
            let mut scroll_css = css_box(style, true);
            scroll_css.push_str("overflow-y: auto;display: flex;flex-direction: column;");
            out.push_str(&format!("{pad}<div style=\"{scroll_css}\">\n"));
            let inner_pad = node_padding(style);
            let gap = style.spacing.unwrap_or(0.0);
            let mut inner_css = String::from("display: flex;flex-direction: column;width: 100%;");
            inner_css.push_str(&format!(
                "padding: {}px {}px {}px {}px;",
                inner_pad.top, inner_pad.right, inner_pad.bottom, inner_pad.left
            ));
            if gap > 0.0 {
                inner_css.push_str(&format!("gap: {gap}px;"));
            }
            out.push_str(&format!("{pad}  <div style=\"{inner_css}\">\n"));
            for child in children {
                emit_node(child, indent + 4, counter, placeholders, out, row)?;
            }
            out.push_str(&format!("{pad}  </div>\n{pad}</div>\n"));
        }
        IRNode::Text { content, style } => {
            let css = format!(
                "{}{}{}white-space: nowrap;overflow: hidden;text-overflow: ellipsis;line-height: 1.2;",
                css_box(style, false),
                css_text(style),
                css_layout(style)
            );
            out.push_str(&format!(
                "{pad}<div style=\"{css}\">{}</div>\n",
                escape_html(content)
            ));
        }
        IRNode::StateText { initial, style, .. } => {
            let css = format!(
                "{}{}{}white-space: nowrap;overflow: hidden;text-overflow: ellipsis;line-height: 1.2;",
                css_box(style, false),
                css_text(style),
                css_layout(style)
            );
            out.push_str(&format!(
                "{pad}<div style=\"{css}\">{}</div>\n",
                escape_html(&initial.display_value())
            ));
        }
        IRNode::ExprText { initial, style, .. } => {
            let css = format!(
                "{}{}{}white-space: nowrap;overflow: hidden;text-overflow: ellipsis;line-height: 1.2;",
                css_box(style, false),
                css_text(style),
                css_layout(style)
            );
            out.push_str(&format!(
                "{pad}<div style=\"{css}\">{}</div>\n",
                escape_html(initial)
            ));
        }
        IRNode::ItemText { field, initial, style, .. } => {
            // 行上下文里按字段名取当前行的值，否则回退到字段样本。
            let value = row
                .and_then(|(fields, values)| {
                    let index = fields.iter().position(|(name, _)| name == field)?;
                    values.get(index)
                })
                .unwrap_or(initial);
            let css = format!(
                "{}{}{}white-space: nowrap;overflow: hidden;text-overflow: ellipsis;line-height: 1.2;",
                css_box(style, false),
                css_text(style),
                css_layout(style)
            );
            out.push_str(&format!(
                "{pad}<div style=\"{css}\">{}</div>\n",
                escape_html(&value.display_value())
            ));
        }
        IRNode::Button { title, style, .. } => {
            let css = format!(
                "appearance: none;border: none;margin: 0;font: inherit;cursor: pointer;{}display: flex;align-items: center;justify-content: center;{}",
                css_box(style, false),
                css_layout(style)
            );
            out.push_str(&format!(
                "{pad}<button style=\"{css}\"><span style=\"{}\">{}</span></button>\n",
                css_text(style),
                escape_html(title)
            ));
        }
        IRNode::Input {
            placeholder, style, ..
        } => {
            let class = format!("ph{counter}");
            *counter += 1;
            let color = style
                .placeholder_color
                .clone()
                .unwrap_or_else(|| raw_color(DEFAULT_PLACEHOLDER_COLOR));
            placeholders.push((class.clone(), color));
            let css = format!(
                "border: none;outline: none;font: inherit;{}line-height: 1.2;{}",
                css_box(style, false),
                css_text(style)
            );
            out.push_str(&format!(
                "{pad}<input class=\"{class}\" placeholder=\"{}\" style=\"{css}\">\n",
                escape_html(placeholder)
            ));
        }
        IRNode::Image { style, .. } => {
            let css = format!(
                "{}display: flex;align-items: center;justify-content: center;color: #9aa3b2;font-size: 12px;border: 1px dashed #c7ccd8;",
                css_box(style, false)
            );
            out.push_str(&format!("{pad}<div style=\"{css}\">image</div>\n"));
        }
        IRNode::Conditional {
            initial,
            when_true,
            when_false,
            ..
        } => {
            // 静态渲染只输出当前分支；display:contents 让包装层不参与 flex 布局
            out.push_str(&format!("{pad}<div style=\"display: contents;\">\n"));
            let branch = if *initial {
                Some(when_true.as_ref())
            } else {
                when_false.as_deref()
            };
            if let Some(branch) = branch {
                emit_node(branch, indent + 2, counter, placeholders, out, row)?;
            }
            out.push_str(&format!("{pad}</div>\n"));
        }
        IRNode::ItemConditional {
            initial,
            when_true,
            when_false,
            ..
        } => {
            let branch = if *initial {
                Some(when_true.as_ref())
            } else {
                when_false.as_deref()
            };
            if let Some(branch) = branch {
                emit_node(branch, indent, counter, placeholders, out, row)?;
            }
        }
        IRNode::List {
            style,
            item,
            fields,
            rows: initial_rows,
            parents,
            ..
        } => {
            // 静态 HTML 只输出初始行；嵌套列表按 DFS 顺序输出，并按深度缩进。
            let mut css = css_box(style, true);
            css.push_str("display: flex;flex-direction: column;");
            out.push_str(&format!("{pad}<div style=\"{css}\">\n"));
            if row_template_recurses(item) {
                // 递归组件按真实层级展开：行体在上，子行嵌在下面一层。
                let body = strip_recursive_slots(item);
                for (index, _) in initial_rows.iter().enumerate() {
                    if parents.get(index).copied().flatten().is_some() {
                        continue;
                    }
                    emit_recursive_row(
                        &body,
                        index,
                        fields,
                        initial_rows,
                        parents,
                        indent + 4,
                        counter,
                        placeholders,
                        out,
                    )?;
                }
            } else {
                for (index, values) in initial_rows.iter().enumerate() {
                    let depth = depth_of(parents, index);
                    let row_css = if depth == 0 {
                        "display: contents;".to_string()
                    } else {
                        format!("display: contents;padding-left: {}px;", depth * 16)
                    };
                    out.push_str(&format!("{pad}  <div style=\"{row_css}\">\n"));
                    emit_node(
                        item,
                        indent + 4,
                        counter,
                        placeholders,
                        out,
                        Some((fields, values)),
                    )?;
                    out.push_str(&format!("{pad}  </div>\n"));
                }
            }
            out.push_str(&format!("{pad}</div>\n"));
        }
        // 递归槽位由渲染根行的递归函数接管，普通路径上不产出任何东西。
        IRNode::RecursiveComponent { .. } => {}
    }
    Ok(())
}

/// 递归组件在静态 HTML 里的展开：一行 = 行体 + 自己的子树。
fn emit_recursive_row(
    body: &IRNode,
    index: usize,
    fields: &[(String, StateValue)],
    rows: &[Vec<StateValue>],
    parents: &[Option<usize>],
    indent: usize,
    counter: &mut usize,
    placeholders: &mut Vec<(String, String)>,
    out: &mut String,
) -> Result<(), String> {
    let Some(values) = rows.get(index) else {
        return Ok(());
    };
    let pad = " ".repeat(indent);
    let depth = depth_of(parents, index);
    let row_css = format!(
        "display: flex;flex-direction: column;padding-left: {}px;",
        if depth == 0 { 0 } else { 16 }
    );
    out.push_str(&format!("{pad}<div style=\"{row_css}\">\n"));
    emit_node(
        body,
        indent + 2,
        counter,
        placeholders,
        out,
        Some((fields, values)),
    )?;
    for (child, parent) in parents.iter().enumerate() {
        if *parent != Some(index) {
            continue;
        }
        emit_recursive_row(
            body,
            child,
            fields,
            rows,
            parents,
            indent + 2,
            counter,
            placeholders,
            out,
        )?;
    }
    out.push_str(&format!("{pad}</div>\n"));
    Ok(())
}

fn emit_container(
    style: &InlineStyle,
    children: &[IRNode],
    default_direction: FlexDirection,
    default_gap: f64,
    pad_str: String,
    counter: &mut usize,
    placeholders: &mut Vec<(String, String)>,
    out: &mut String,
    row: RowContext<'_>,
) -> Result<(), String> {
    let direction = style.flex_direction.unwrap_or(default_direction);
    let gap = style.spacing.unwrap_or(default_gap);
    let has_absolute = children.iter().any(|child| {
        let child_style = node_style(child);
        child_style.position == Position::Absolute
            || child_style.left.is_some()
            || child_style.right.is_some()
            || child_style.top.is_some()
            || child_style.bottom.is_some()
    });

    let mut css = css_box(style, true);
    css.push_str(match direction {
        FlexDirection::Row => "display: flex;flex-direction: row;",
        FlexDirection::Column => "display: flex;flex-direction: column;",
    });
    if gap > 0.0 {
        css.push_str(&format!("gap: {gap}px;"));
    }
    if has_absolute {
        css.push_str("position: relative;");
    }
    css.push_str(&css_justify(style));
    css.push_str(&css_align(style));

    out.push_str(&format!("{pad_str}<div style=\"{css}\">\n"));
    for child in children {
        emit_node(child, pad_str.len() + 2, counter, placeholders, out, row)?;
    }
    out.push_str(&format!("{pad_str}</div>\n"));
    Ok(())
}

fn escape_html(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for ch in text.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            _ => out.push(ch),
        }
    }
    out
}

/// 视觉对照脚本用：列出 HTML 里出现的颜色，方便断言「占位符颜色一致」。
pub(crate) fn placeholder_colors(program: &UiProgram) -> BTreeSet<String> {
    let mut colors = BTreeSet::new();
    collect_input_colors(&program.nodes, &mut colors);
    colors
}

fn collect_input_colors(nodes: &[IRNode], colors: &mut BTreeSet<String>) {
    for node in nodes {
        match node {
            IRNode::Input { style, .. } => {
                let raw = style
                    .placeholder_color
                    .clone()
                    .unwrap_or_else(|| DEFAULT_PLACEHOLDER_COLOR.to_string());
                colors.insert(raw_color(&raw));
            }
            IRNode::View { children, .. }
            | IRNode::VStack { children, .. }
            | IRNode::ScrollView { children, .. } => collect_input_colors(children, colors),
            IRNode::Conditional {
                when_true,
                when_false,
                ..
            } => {
                collect_input_colors(std::slice::from_ref(when_true), colors);
                if let Some(node) = when_false {
                    collect_input_colors(std::slice::from_ref(node), colors);
                }
            }
            _ => {}
        }
    }
}
