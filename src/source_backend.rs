//! 生成源码型后端：不调用平台 API，而是把 IR 翻译成目标平台的源码。
//!
//! 当前实现的是 `--target=web`：输出 HTML + CSS Flexbox 和内嵌状态运行时。
//! 它有两个用处：
//! 1. 证明后端形态可以不止「发消息建控件」——也可以「生成源码」（HarmonyOS 的
//!    ArkTS、WidgetKit 的 SwiftUI 都得走这条路）；
//! 2. 作为一个可截图、可自动断言的视觉基准，参与 `tools/parity` 的三端对照。
//!
//! 状态单元、条件分支、表达式、透明度以及标量事件动作在浏览器中运行；
//! 列表仍只渲染初始行。

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
    let runtime = generate_runtime(program);

    let mut placeholder_rules = String::new();
    for (class, color) in &placeholders {
        placeholder_rules.push_str(&format!(
            "      .{class}::placeholder {{ color: {color}; opacity: 1; }}\n"
        ));
    }

    Ok(format!(
        "<!DOCTYPE html>\n<html lang=\"zh\">\n<head>\n  <meta charset=\"utf-8\">\n  <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n  <title>Lumen (web)</title>\n  <style>\n    html, body {{ margin: 0; height: 100%; background: #ffffff; }}\n    body {{ font-family: -apple-system, \"PingFang SC\", \"Microsoft YaHei\", Roboto, sans-serif; }}\n    * {{ box-sizing: border-box; }}\n{placeholder_rules}  </style>\n</head>\n<body>\n{body}<script>\n{runtime}</script>\n</body>\n</html>\n"
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
            let default_gap = if matches!(node, IRNode::VStack { .. }) {
                8.0
            } else {
                0.0
            };
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
            let scroll_event = style.scroll_action.map_or_else(String::new, |action| {
                format!(" data-lumen-scroll-action=\"{action}\"")
            });
            out.push_str(&format!(
                "{pad}<div{}{scroll_event} style=\"{scroll_css}\">\n",
                dynamic_style_attrs(style)
            ));
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
                "{pad}<div{} style=\"{css}\">{}</div>\n",
                dynamic_style_attrs(style),
                escape_html(content)
            ));
        }
        IRNode::StateText {
            cell,
            initial,
            style,
        } => {
            let css = format!(
                "{}{}{}white-space: nowrap;overflow: hidden;text-overflow: ellipsis;line-height: 1.2;",
                css_box(style, false),
                css_text(style),
                css_layout(style)
            );
            out.push_str(&format!(
                "{pad}<div data-lumen-state=\"{}\"{} style=\"{css}\">{}</div>\n",
                escape_html_attr(cell),
                dynamic_style_attrs(style),
                escape_html(&initial.display_value())
            ));
        }
        IRNode::ExprText {
            left,
            right,
            subtract,
            initial,
            style,
            ..
        } => {
            let css = format!(
                "{}{}{}white-space: nowrap;overflow: hidden;text-overflow: ellipsis;line-height: 1.2;",
                css_box(style, false),
                css_text(style),
                css_layout(style)
            );
            out.push_str(&format!(
                "{pad}<div data-lumen-expr=\"{}\"{} style=\"{css}\">{}</div>\n",
                escape_html_attr(&expression_json(left, right, *subtract)),
                dynamic_style_attrs(style),
                escape_html(initial)
            ));
        }
        IRNode::ItemText {
            field,
            initial,
            style,
            ..
        } => {
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
                "{pad}<div{} style=\"{css}\">{}</div>\n",
                dynamic_style_attrs(style),
                escape_html(&value.display_value())
            ));
        }
        IRNode::Button {
            title,
            style,
            state_action,
            ..
        } => {
            let css = format!(
                "appearance: none;border: none;margin: 0;font: inherit;cursor: pointer;{}display: flex;align-items: center;justify-content: center;{}",
                css_box(style, false),
                css_layout(style)
            );
            out.push_str(&format!(
                "{pad}<button{}{} style=\"{css}\"><span style=\"{}\">{}</span></button>\n",
                state_action.map_or_else(String::new, |action| {
                    format!(" data-lumen-action=\"{action}\"")
                }),
                dynamic_style_attrs(style),
                css_text(style),
                escape_html(title)
            ));
        }
        IRNode::Input {
            placeholder,
            style,
            on_input,
            on_focus,
            ..
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
                "{pad}<input class=\"{class}\" placeholder=\"{}\"{}{}{} style=\"{css}\">\n",
                escape_html_attr(placeholder),
                on_input.map_or_else(String::new, |action| {
                    format!(" data-lumen-input-action=\"{action}\"")
                }),
                on_focus.map_or_else(String::new, |action| {
                    format!(" data-lumen-focus-action=\"{action}\"")
                }),
                dynamic_style_attrs(style)
            ));
        }
        IRNode::Image { style, .. } => {
            let css = format!(
                "{}display: flex;align-items: center;justify-content: center;color: #9aa3b2;font-size: 12px;border: 1px dashed #c7ccd8;",
                css_box(style, false)
            );
            out.push_str(&format!(
                "{pad}<div{} style=\"{css}\">image</div>\n",
                dynamic_style_attrs(style)
            ));
        }
        IRNode::Conditional {
            cell,
            initial,
            when_true,
            when_false,
            ..
        } => {
            emit_conditional_branch(
                cell,
                true,
                *initial,
                when_true,
                indent,
                counter,
                placeholders,
                out,
                row,
            )?;
            if let Some(branch) = when_false {
                emit_conditional_branch(
                    cell,
                    false,
                    *initial,
                    branch,
                    indent,
                    counter,
                    placeholders,
                    out,
                    row,
                )?;
            }
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
            out.push_str(&format!(
                "{pad}<div{} style=\"{css}\">\n",
                dynamic_style_attrs(style)
            ));
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

fn emit_conditional_branch(
    cell: &str,
    when: bool,
    initial: bool,
    branch: &IRNode,
    indent: usize,
    counter: &mut usize,
    placeholders: &mut Vec<(String, String)>,
    out: &mut String,
    row: RowContext<'_>,
) -> Result<(), String> {
    let pad = " ".repeat(indent);
    let display = if initial == when { "contents" } else { "none" };
    out.push_str(&format!(
        "{pad}<div data-lumen-conditional=\"{}\" data-lumen-when=\"{when}\" style=\"display: {display};\">\n",
        escape_html_attr(cell)
    ));
    emit_node(branch, indent + 2, counter, placeholders, out, row)?;
    out.push_str(&format!("{pad}</div>\n"));
    Ok(())
}

fn dynamic_style_attrs(style: &InlineStyle) -> String {
    style
        .opacity_cell
        .as_ref()
        .map_or_else(String::new, |cell| {
            format!(" data-lumen-opacity=\"{}\"", escape_html_attr(cell))
        })
}

fn expression_json(left: &ExprOperand, right: &ExprOperand, subtract: bool) -> String {
    fn operand_json(operand: &ExprOperand) -> String {
        match operand {
            ExprOperand::Cell(cell) => {
                format!("{{\"cell\":{}}}", json_string(cell))
            }
            ExprOperand::Number(number) => format!("{{\"number\":{number}}}"),
        }
    }

    format!(
        "{{\"left\":{},\"right\":{},\"subtract\":{subtract}}}",
        operand_json(left),
        operand_json(right)
    )
}

fn json_string(value: &str) -> String {
    let mut result = String::with_capacity(value.len() + 2);
    result.push('"');
    for ch in value.chars() {
        match ch {
            '"' => result.push_str("\\\""),
            '\\' => result.push_str("\\\\"),
            '\n' => result.push_str("\\n"),
            '\r' => result.push_str("\\r"),
            '\t' => result.push_str("\\t"),
            '\u{08}' => result.push_str("\\b"),
            '\u{0c}' => result.push_str("\\f"),
            '<' => result.push_str("\\u003c"),
            '>' => result.push_str("\\u003e"),
            '&' => result.push_str("\\u0026"),
            '\u{2028}' => result.push_str("\\u2028"),
            '\u{2029}' => result.push_str("\\u2029"),
            ch if ch <= '\u{001f}' => result.push_str(&format!("\\u{:04x}", ch as u32)),
            ch => result.push(ch),
        }
    }
    result.push('"');
    result
}

fn state_value_json(value: &StateValue) -> String {
    match value {
        StateValue::Number(number) => number.to_string(),
        StateValue::String(text) => json_string(text),
        StateValue::Boolean(value) => value.to_string(),
    }
}

fn action_json(action: &StateMutation) -> String {
    match action {
        StateMutation::Noop => "{\"type\":\"noop\"}".into(),
        StateMutation::Increment(cell) | StateMutation::Decrement(cell) => format!(
            "{{\"type\":\"{}\",\"cell\":{}}}",
            if matches!(action, StateMutation::Increment(_)) {
                "increment"
            } else {
                "decrement"
            },
            json_string(cell)
        ),
        StateMutation::Set(cell, value) => format!(
            "{{\"type\":\"set\",\"cell\":{},\"value\":{}}}",
            json_string(cell),
            state_value_json(value)
        ),
        StateMutation::SetFromInput(cell) => format!(
            "{{\"type\":\"input\",\"cell\":{}}}",
            json_string(cell)
        ),
        StateMutation::SetTrue(cell) => format!(
            "{{\"type\":\"set\",\"cell\":{},\"value\":true}}",
            json_string(cell)
        ),
        StateMutation::Sequence(actions) => format!(
            "{{\"type\":\"sequence\",\"actions\":[{}]}}",
            actions.iter().map(action_json).collect::<Vec<_>>().join(",")
        ),
        StateMutation::SetCells { values } => format!(
            "{{\"type\":\"setCells\",\"values\":{{{}}}}}",
            values
                .iter()
                .map(|(cell, value)| format!(
                    "{}:{}",
                    json_string(cell),
                    state_value_json(value)
                ))
                .collect::<Vec<_>>()
                .join(",")
        ),
        StateMutation::SetFromCell {
            target,
            source,
            delta,
        } => format!(
            "{{\"type\":\"fromCell\",\"cell\":{},\"source\":{},\"delta\":{delta}}}",
            json_string(target),
            json_string(source)
        ),
        StateMutation::SetFromScroll(cell) => format!(
            "{{\"type\":\"scroll\",\"cell\":{}}}",
            json_string(cell)
        ),
        StateMutation::SetFromScrollClamped {
            cell,
            low,
            high,
            shift,
        } => format!(
            "{{\"type\":\"scrollClamped\",\"cell\":{},\"low\":{low},\"high\":{high},\"shift\":{shift}}}",
            json_string(cell)
        ),
        StateMutation::PushItem { .. }
        | StateMutation::SetItemField { .. }
        | StateMutation::RemoveCurrentItem { .. }
        | StateMutation::ClearList { .. } => {
            "{\"type\":\"unsupportedListMutation\"}".into()
        }
    }
}

fn generate_runtime(program: &UiProgram) -> String {
    let initial_state = state_cells(&program.states)
        .iter()
        .map(|(cell, value)| format!("{}:{}", json_string(cell), state_value_json(value)))
        .collect::<Vec<_>>()
        .join(",");
    let actions = program
        .state_actions
        .iter()
        .map(action_json)
        .collect::<Vec<_>>()
        .join(",");
    format!(
        r#"(() => {{
  const state = {{{initial_state}}};
  const actions = [{actions}];
  const root = document.body;
  const readOperand = (operand) => "cell" in operand ? Number(state[operand.cell]) || 0 : operand.number;
  const apply = (action, inputValue, scrollValue) => {{
    switch (action.type) {{
      case "noop": return;
      case "increment": state[action.cell] = (Number(state[action.cell]) || 0) + 1; return;
      case "decrement": state[action.cell] = (Number(state[action.cell]) || 0) - 1; return;
      case "set": state[action.cell] = action.value; return;
      case "input": state[action.cell] = inputValue; return;
      case "setCells": Object.assign(state, action.values); return;
      case "fromCell": state[action.cell] = (Number(state[action.source]) || 0) + action.delta; return;
      case "scroll": state[action.cell] = scrollValue; return;
      case "scrollClamped": state[action.cell] = Math.max(action.low, Math.min(action.high, scrollValue + action.shift)); return;
      case "sequence": for (const step of action.actions) apply(step, inputValue, scrollValue); return;
      case "unsupportedListMutation": console.error("Lumen web: dynamic list mutations are not supported yet"); return;
      default: console.error("Lumen web: unknown state action", action.type);
    }}
  }};
  const sync = () => {{
    root.querySelectorAll("[data-lumen-state]").forEach((element) => {{
      const value = state[element.dataset.lumenState];
      element.textContent = value == null ? "" : String(value);
    }});
    root.querySelectorAll("[data-lumen-conditional]").forEach((element) => {{
      const expected = element.dataset.lumenWhen === "true";
      element.style.display = Boolean(state[element.dataset.lumenConditional]) === expected ? "contents" : "none";
    }});
    root.querySelectorAll("[data-lumen-opacity]").forEach((element) => {{
      const value = Number(state[element.dataset.lumenOpacity]);
      element.style.opacity = String(Math.max(0, Math.min(100, Number.isFinite(value) ? value : 0)) / 100);
    }});
    root.querySelectorAll("[data-lumen-expr]").forEach((element) => {{
      const expression = JSON.parse(element.dataset.lumenExpr);
      const left = readOperand(expression.left);
      const right = readOperand(expression.right);
      element.textContent = String(expression.subtract ? left - right : left + right);
    }});
  }};
  const run = (index, inputValue, scrollValue) => {{
    const action = actions[index];
    if (!action) {{ console.error("Lumen web: missing state action", index); return; }}
    apply(action, inputValue, scrollValue);
    sync();
  }};
  root.addEventListener("click", (event) => {{
    const button = event.target.closest("[data-lumen-action]");
    if (button) run(Number(button.dataset.lumenAction));
  }});
  root.addEventListener("input", (event) => {{
    const input = event.target.closest("[data-lumen-input-action]");
    if (input) run(Number(input.dataset.lumenInputAction), input.value);
  }});
  root.addEventListener("focusin", (event) => {{
    const input = event.target.closest("[data-lumen-focus-action]");
    if (input) run(Number(input.dataset.lumenFocusAction));
  }});
  root.addEventListener("scroll", (event) => {{
    const target = event.target.closest("[data-lumen-scroll-action]");
    if (target) run(Number(target.dataset.lumenScrollAction), undefined, target.scrollTop);
  }}, true);
  sync();
}})();"#
    )
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

    out.push_str(&format!(
        "{pad_str}<div{} style=\"{css}\">\n",
        dynamic_style_attrs(style)
    ));
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

fn escape_html_attr(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for ch in text.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
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
