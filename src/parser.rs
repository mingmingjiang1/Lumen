use swc_core::common::{FileName, SourceMap};
use swc_core::ecma::parser::{Parser, Syntax, TsConfig};
use swc_core::ecma::visit::Visit;
use swc_core::ecma::ast::*;
use std::sync::Arc;

// 自定义IR定义，极简，只描述UI节点
#[derive(Debug, Clone)]
pub enum IRNode {
    View { children: Vec<IRNode> },
    Text { content: String },
}

// JSX AST 访问器，把 swc ast => 我们的 IR
struct JsxToIrVisitor {
    ir_out: Vec<IRNode>,
}

impl Visit for JsxToIrVisitor {
    fn visit_jsx_element(&mut self, node: &JSXElement) {
        let opening = &node.opening.name;
        let tag_name = match opening {
            JSXElementName::Ident(ident) => ident.sym.as_str(),
            _ => "",
        };
        let children: Vec<IRNode> = node.children.iter().filter_map(|child| {
            match child {
                JSXChild::Text(t) => {
                    Some(IRNode::Text { content: t.value.to_string() })
                }
                JSXChild::Element(el) => {
                    let mut v = JsxToIrVisitor { ir_out: vec![] };
                    v.visit_jsx_element(el);
                    v.ir_out.into_iter().next()
                }
                _ => None
            }
        }).collect();

        match tag_name {
            "View" => {
                self.ir_out.push(IRNode::View { children });
            }
            "Text" => {
                // 简化：Text 直接拿子文本
                let txt = children.iter().filter_map(|c| match c {
                    IRNode::Text { content } => Some(content.clone()),
                    _ => None
                }).collect::<Vec<_>>().join("");
                self.ir_out.push(IRNode::Text { content: txt });
            }
            _ => {}
        }
    }
}

fn parse_tsx_to_ir(src: &str) -> Vec<IRNode> {
    let cm = Arc::new(SourceMap::default());
    let fm = cm.new_source_file(FileName::Anon, src.to_string());

    let mut parser = Parser::new(
        &fm,
        Syntax::Typescript(TsConfig {
            tsx: true, // ✅ 开启TSX，支持React标签
            ..Default::default()
        }),
        None,
    );
    let module = parser.parse_module().expect("parse tsx failed");

    let mut visitor = JsxToIrVisitor { ir_out: vec![] };
    visitor.visit_module(&module, &module);
    visitor.ir_out
}

fn main() {
    // 我们的React-like TSX代码
    let tsx_code = r#"
function App() {
    return <View>
        <Text>Hello iOS</Text>
    </View>
}
    "#;
    let ir = parse_tsx_to_ir(tsx_code);
    println!("Generated IR: {:#?}", ir);
}
