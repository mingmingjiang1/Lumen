use crate::hir::{BinOp, HirComponent, HirExpr, TAG_NUM, TAG_OBJ, TAG_STR, TAG_UNDEF};

/// 全局状态栈下标，useState 按顺序分配 slot
static mut STATE_SLOT: u32 = 0;

pub fn generate_wat(comp: &HirComponent) -> String {
    unsafe { STATE_SLOT = 0; }
    let body = gen_expr(&comp.body);
    format!(
r#"(module
  ;; ===== Host Import =====
  (import "host" "create_element" (func $create_element (param i32) (result i32)))
  (import "host" "set_text" (func $set_text (param i32 i64)))
  (import "host" "bind_click" (func $bind_click (param i32 i32)))
  (import "host" "gc_alloc" (func $gc_alloc (param i32) (result i32)))
  (import "host" "gc_mark" (func $gc_mark (param i32)))
  (import "host" "schedule_render" (func $schedule_render))

  ;; 状态数组：每个slot存一个 nan-boxed u64
  (global $state_ptr (mut i32) (i32.const 0))
  ;; 导出渲染入口
  (func $render (result i32)
    local.get $state_ptr
    {body}
  )
  (export "render" (func $render))
  ;; 状态setter，slot编号入参
  (func $set_state (param $slot i32) (param $val i64)
    local.get $slot
    i32.const 8
    i32.mul
    local.get $state_ptr
    i32.add
    local.get $val
    i64.store
    call $schedule_render
  )
  (export "set_state" (func $set_state))

  ;; GC根标记入口，渲染时标记所有活跃state
  (func $mark_roots
    local.get $state_ptr
    ;; 简单遍历slot标记，原型
  )
  (memory $mem 2)
  (export "memory" (memory $mem))
)"#
    )
}

fn gen_expr(e: &HirExpr) -> String {
    match e {
        HirExpr::Number(v) => {
            // NaN-box number: tag=0
            let boxed = f64::from_bits(((v.to_bits() & !(0x7FF << 52)) | ((TAG_NUM as u64) << 52)));
            format!("f64.const {}\n", boxed)
        }
        HirExpr::Ident(name) => {
            // 读state slot，简化：count -> 取当前slot
            format!(r#";; read ident {name}
local.get $state_ptr
i32.const 0
i32.const 8
i32.mul
i32.add
i64.load
"#)
        }
        HirExpr::BinOp(op, l, r) => {
            let left = gen_expr(l);
            let right = gen_expr(r);
            let instr = match op {
                BinOp::Add => "f64.add",
                BinOp::Sub => "f64.sub",
                BinOp::Mul => "f64.mul",
            };
            format!("{left}{right}{instr}\n")
        }
        HirExpr::UseState(init) => {
            let init_code = gen_expr(init);
            let slot = unsafe {
                let s = STATE_SLOT;
                STATE_SLOT += 1;
                s
            };
            format!(r#"{init_code}
;; useState slot={slot}
local.get $state_ptr
i32.const {slot}
i32.const 8
i32.mul
i32.add
i64.store
;; 返回 [val, setter]，这里简化：把slot编号压栈给setter捕获
i32.const {slot}
"#)
        }
        HirExpr::Call(fname, args) => {
            let mut s = String::new();
            for a in args {
                s.push_str(&gen_expr(a));
            }
            if fname == "setCount" {
                s.push_str("call $set_state\n");
            }
            s
        }
        HirExpr::Callback(_params, body) => {
            let inner = gen_expr(body);
            format!(r#";; callback
{inner}
"#)
        }
        HirExpr::JsxElement(tag, props, children) => {
            let tag_id = match tag.as_str() {
                "div" => 0,
                "span" => 1,
                _ => 99,
            };
            let mut s = format!("i32.const {tag_id}\ncall $create_element\n");
            for (k, v) in props {
                if k == "onClick" {
                    s.push_str(&gen_expr(v));
                    s.push_str("call $bind_click\n");
                }
            }
            for c in children {
                s.push_str(&gen_expr(c));
                s.push_str("call $set_text\n");
            }
            s
        }
        HirExpr::Text(txt) => {
            // 文本字符串分配到GC堆
            format!(r#"i32.const {}
call $gc_alloc
i64.const {}
"#, txt.len(), txt.as_bytes().iter().fold(0u64, |acc,b| (acc << 8) | *b as u64))
        }
    }
}
