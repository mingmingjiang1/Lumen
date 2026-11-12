#[derive(Debug, Clone)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
}

#[derive(Debug, Clone)]
pub enum HirExpr {
    Number(f64),
    Ident(String),
    /// useState(init) -> 返回 [value, setter]
    UseState(Box<HirExpr>),
    /// 二元运算 a + b
    BinOp(BinOp, Box<HirExpr>, Box<HirExpr>),
    /// JSX元素：tag, props(事件), children
    JsxElement(String, Vec<(String, HirExpr)>, Vec<HirExpr>),
    Text(String),
    /// 匿名回调函数（用于 onClick={() => setCount(count+1)}）
    Callback(Vec<String>, Box<HirExpr>),
    /// 调用：fn(arg1, arg2...)
    Call(String, Vec<HirExpr>),
}

#[derive(Debug)]
pub struct HirComponent {
    pub name: String,
    pub body: HirExpr,
}

/// NaN-box 类型标记（和 Perry 对齐）
/// f64 高位尾数存 tag：0=number,1=object,2=string,3=undefined
pub const TAG_NUM: u64 = 0;
pub const TAG_OBJ: u64 = 1;
pub const TAG_STR: u64 = 2;
pub const TAG_UNDEF: u64 = 3;
