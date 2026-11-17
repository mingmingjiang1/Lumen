//! Android 后端：共享 IR 在这里落成 JNI 调用序列，编译成 liblumen.so，
//! 由 `MainActivity.nativeCreate` 在 Android 上搭出原生 View 树。
//!
//! 布局语义映射（与 iOS / macOS 共享同一套 flex 规则）：
//! - 容器（`View` / `VStack`）→ `LinearLayout`，`flexDirection` → orientation；
//! - 带绝对定位子级的容器 → `FrameLayout` 托底，流式子级放进内部的 `LinearLayout`；
//! - 子级尺寸 / 外边距 → `LinearLayout.LayoutParams`，`flexGrow` → weight；
//! - `justifyContent` → 容器 gravity，`space-between/around` 用 spacer + weight 近似；
//! - `alignItems` → 子级 gravity，`stretch` → match_parent。
//!
//! 状态更新沿用 iOS 的「状态单元 + 标签表」：每个单元有自己的全局变量和引用它的
//! TextView 列表，写入时只刷新这一组；条件分支各自有待替换的槽位。

use super::*;
use inkwell::basic_block::BasicBlock;
use inkwell::types::{FloatType, IntType, PointerType, VoidType};
use inkwell::values::{FloatValue, IntValue};

// ---------- JNI 函数表下标 ----------
// `JNINativeInterface` 的字段顺序由 JNI 规范固定；这些值用 NDK jni.h 的 offsetof
// 核对过。写错不会报错，只会静默调到别的函数（例如把 NewStringUTF 写成 137 会走
// CallStaticFloatMethodA，然后崩在 ART 里）。
const JNI_FIND_CLASS: u64 = 6;
const JNI_EXCEPTION_CLEAR: u64 = 17;
const JNI_PUSH_LOCAL_FRAME: u64 = 19;
const JNI_POP_LOCAL_FRAME: u64 = 20;
const JNI_NEW_GLOBAL_REF: u64 = 21;
const JNI_DELETE_GLOBAL_REF: u64 = 22;
const JNI_NEW_OBJECT_A: u64 = 30;
const JNI_GET_METHOD_ID: u64 = 33;
const JNI_CALL_OBJECT_A: u64 = 36;
const JNI_CALL_VOID_A: u64 = 63;
const JNI_GET_FIELD_ID: u64 = 94;
const JNI_GET_FLOAT_FIELD: u64 = 102;
const JNI_SET_INT_FIELD: u64 = 109;
const JNI_SET_FLOAT_FIELD: u64 = 111;
const JNI_GET_STATIC_METHOD_ID: u64 = 113;
const JNI_CALL_STATIC_INT_A: u64 = 131;
// jni.h 的静态调用块从 GetStaticMethodID(113) 之后开始：
// Object(114/115/116) Boolean(117~119) Byte(120~122) Char(123~125) Short(126~128)
// Int(129~131) Long(132~134) Float(135~137) Double(138~140) Void(141/142/143)
const JNI_CALL_STATIC_OBJECT_A: u64 = 116;
const JNI_CALL_STATIC_VOID_A: u64 = 143;
const JNI_GET_STATIC_FIELD_ID: u64 = 144;
const JNI_GET_STATIC_OBJECT_FIELD: u64 = 145;
const JNI_NEW_STRING_UTF: u64 = 167;
const JNI_GET_STRING_UTF_CHARS: u64 = 169;
const JNI_RELEASE_STRING_UTF_CHARS: u64 = 170;

// ---------- Android 常量 ----------
const MATCH_PARENT: i32 = -1;
const WRAP_CONTENT: i32 = -2;

const LINEAR_VERTICAL: i32 = 1;
const LINEAR_HORIZONTAL: i32 = 0;

// android.view.Gravity
const GRAVITY_CENTER_HORIZONTAL: i32 = 0x01;
const GRAVITY_LEFT: i32 = 0x03;
const GRAVITY_RIGHT: i32 = 0x05;
const GRAVITY_CENTER_VERTICAL: i32 = 0x10;
const GRAVITY_CENTER: i32 = 0x11;
const GRAVITY_TOP: i32 = 0x30;
const GRAVITY_BOTTOM: i32 = 0x50;

// android.util.TypedValue.COMPLEX_UNIT_DIP / _SP
const UNIT_DIP: i32 = 1;

/// 行方向给标签的明确高度按字号乘以这个系数（Roboto 的行高约 1.32 倍，留一点余量）。
const LABEL_LINE_HEIGHT: f64 = 1.2;

/// UIButton 的标题默认字号比正文小，按 iOS 的 15pt 走，否则「Show component」在
/// 140dp 宽里会被 ellipsize 掉。
const DEFAULT_BUTTON_FONT_SIZE: f64 = 15.0;

/// 界面构建期间会攒下几百个 JNI 局部引用，进门先开一个够大的局部引用帧。
const LOCAL_FRAME_CAPACITY: i32 = 4096;

/// 要缓存的 Java 类（别名, JNI 类名）。
const CLASSES: &[(&str, &str)] = &[
    ("Activity", "android/app/Activity"),
    ("Text", "android/widget/TextView"),
    ("Button", "android/widget/Button"),
    ("Input", "android/widget/EditText"),
    ("Image", "android/widget/ImageView"),
    ("Linear", "android/widget/LinearLayout"),
    ("Frame", "android/widget/FrameLayout"),
    ("Scroll", "android/widget/ScrollView"),
    ("View", "android/view/View"),
    ("ViewGroup", "android/view/ViewGroup"),
    ("Drawable", "android/graphics/drawable/GradientDrawable"),
    ("LinearParams", "android/widget/LinearLayout$LayoutParams"),
    ("FrameParams", "android/widget/FrameLayout$LayoutParams"),
    ("Resources", "android/content/res/Resources"),
    ("Metrics", "android/util/DisplayMetrics"),
    ("Log", "android/util/Log"),
    ("TruncateAt", "android/text/TextUtils$TruncateAt"),
    ("ClickBridge", "com/lumen/app/MainActivity$ClickBridge"),
    ("InputBridge", "com/lumen/app/MainActivity$InputBridge"),
    ("FocusBridge", "com/lumen/app/MainActivity$FocusBridge"),
    ("ScrollBridge", "com/lumen/app/MainActivity$ScrollBridge"),
    ("LumenRow", "com/lumen/app/LumenRow"),
];

/// 要缓存的方法 ID（别名, 类别名, 方法名, 签名）。
const METHODS: &[(&str, &str, &str, &str)] = &[
    // 容器
    (
        "linearCtor",
        "Linear",
        "<init>",
        "(Landroid/content/Context;)V",
    ),
    ("setOrientation", "Linear", "setOrientation", "(I)V"),
    ("setLinearGravity", "Linear", "setGravity", "(I)V"),
    (
        "frameCtor",
        "Frame",
        "<init>",
        "(Landroid/content/Context;)V",
    ),
    (
        "scrollCtor",
        "Scroll",
        "<init>",
        "(Landroid/content/Context;)V",
    ),
    ("viewCtor", "View", "<init>", "(Landroid/content/Context;)V"),
    // 文本
    ("textCtor", "Text", "<init>", "(Landroid/content/Context;)V"),
    ("setText", "Text", "setText", "(Ljava/lang/CharSequence;)V"),
    ("setSingleLine", "Text", "setSingleLine", "(Z)V"),
    ("setIncludeFontPadding", "Text", "setIncludeFontPadding", "(Z)V"),
    (
        "setEllipsize",
        "Text",
        "setEllipsize",
        "(Landroid/text/TextUtils$TruncateAt;)V",
    ),
    ("setTextSize", "Text", "setTextSize", "(IF)V"),
    ("setTextColor", "Text", "setTextColor", "(I)V"),
    ("setTextGravity", "Text", "setGravity", "(I)V"),
    (
        "addTextChanged",
        "Text",
        "addTextChangedListener",
        "(Landroid/text/TextWatcher;)V",
    ),
    // 按钮
    (
        "buttonCtor",
        "Button",
        "<init>",
        "(Landroid/content/Context;)V",
    ),
    ("setAllCaps", "Button", "setAllCaps", "(Z)V"),
    // 输入框
    (
        "inputCtor",
        "Input",
        "<init>",
        "(Landroid/content/Context;)V",
    ),
    ("setHint", "Input", "setHint", "(Ljava/lang/CharSequence;)V"),
    ("setHintTextColor", "Input", "setHintTextColor", "(I)V"),
    // 图片（暂未接入资源加载，只占位）
    (
        "imageCtor",
        "Image",
        "<init>",
        "(Landroid/content/Context;)V",
    ),
    // 视图通用
    ("setPadding", "View", "setPadding", "(IIII)V"),
    ("setBackgroundColor", "View", "setBackgroundColor", "(I)V"),
    (
        "setBackground",
        "View",
        "setBackground",
        "(Landroid/graphics/drawable/Drawable;)V",
    ),
    ("setAlpha", "View", "setAlpha", "(F)V"),
    (
        "setLayoutParams",
        "View",
        "setLayoutParams",
        "(Landroid/view/ViewGroup$LayoutParams;)V",
    ),
    (
        "setOnClickListener",
        "View",
        "setOnClickListener",
        "(Landroid/view/View$OnClickListener;)V",
    ),
    (
        "setOnFocusChangeListener",
        "View",
        "setOnFocusChangeListener",
        "(Landroid/view/View$OnFocusChangeListener;)V",
    ),
    ("setClipChildren", "ViewGroup", "setClipChildren", "(Z)V"),
    ("setClipToPadding", "ViewGroup", "setClipToPadding", "(Z)V"),
    // 行内控件按 id 定位，rebind 时只改文字/点击 id，不重建控件
    ("setId", "View", "setId", "(I)V"),
    (
        "findViewById",
        "View",
        "findViewById",
        "(I)Landroid/view/View;",
    ),
    ("addView", "ViewGroup", "addView", "(Landroid/view/View;)V"),
    ("removeAllViews", "ViewGroup", "removeAllViews", "()V"),
    (
        "addViewAt",
        "ViewGroup",
        "addView",
        "(Landroid/view/View;I)V",
    ),
    (
        "removeView",
        "ViewGroup",
        "removeView",
        "(Landroid/view/View;)V",
    ),
    (
        "setOnScrollChange",
        "Scroll",
        "setOnScrollChangeListener",
        "(Landroid/view/View$OnScrollChangeListener;)V",
    ),
    // 布局参数
    ("linearParamsCtor", "LinearParams", "<init>", "(II)V"),
    ("frameParamsCtor", "FrameParams", "<init>", "(II)V"),
    ("setMargins", "LinearParams", "setMargins", "(IIII)V"),
    ("setFrameMargins", "FrameParams", "setMargins", "(IIII)V"),
    // 圆角 / 描边背景
    ("drawableCtor", "Drawable", "<init>", "()V"),
    ("setColor", "Drawable", "setColor", "(I)V"),
    ("setCornerRadius", "Drawable", "setCornerRadius", "(F)V"),
    ("setStroke", "Drawable", "setStroke", "(II)V"),
    // Activity / 资源
    (
        "setContentView",
        "Activity",
        "setContentView",
        "(Landroid/view/View;)V",
    ),
    (
        "getResources",
        "Activity",
        "getResources",
        "()Landroid/content/res/Resources;",
    ),
    (
        "getDisplayMetrics",
        "Resources",
        "getDisplayMetrics",
        "()Landroid/util/DisplayMetrics;",
    ),
    // 事件桥
    ("clickBridgeCtor", "ClickBridge", "<init>", "(I)V"),
    ("inputBridgeCtor", "InputBridge", "<init>", "(I)V"),
    ("focusBridgeCtor", "FocusBridge", "<init>", "(I)V"),
    ("scrollBridgeCtor", "ScrollBridge", "<init>", "(I)V"),
];

/// 要缓存的字段 ID（别名, 类别名, 字段名, 签名）。
const FIELDS: &[(&str, &str, &str, &str)] = &[
    ("density", "Metrics", "density", "F"),
    ("linearWeight", "LinearParams", "weight", "F"),
    ("linearGravity", "LinearParams", "gravity", "I"),
    ("frameGravity", "FrameParams", "gravity", "I"),
];

/// 要缓存的静态字段（别名, 类别名, 字段名, 签名）。
const STATIC_FIELDS: &[(&str, &str, &str, &str)] = &[
    (
        "truncateEnd",
        "TruncateAt",
        "END",
        "Landroid/text/TextUtils$TruncateAt;",
    ),
    (
        "truncateMiddle",
        "TruncateAt",
        "MIDDLE",
        "Landroid/text/TextUtils$TruncateAt;",
    ),
];

/// 要缓存的静态方法（别名, 类别名, 方法名, 签名）。
const STATIC_METHODS: &[(&str, &str, &str, &str)] = &[
    (
        "logError",
        "Log",
        "e",
        "(Ljava/lang/String;Ljava/lang/String;)I",
    ),
    (
        "lumenRowParse",
        "LumenRow",
        "parse",
        "(Ljava/lang/String;)Lcom/lumen/app/LumenRow$Template;",
    ),
    (
        "lumenRowBuild",
        "LumenRow",
        "build",
        "(Landroid/app/Activity;Lcom/lumen/app/LumenRow$Template;Landroid/view/ViewGroup;Ljava/lang/String;II)V",
    ),
    (
        "lumenRowBegin",
        "LumenRow",
        "begin",
        "(Landroid/view/ViewGroup;)V",
    ),
    (
        "lumenRowEnd",
        "LumenRow",
        "end",
        "(Landroid/view/ViewGroup;I)V",
    ),
];

/// 一次成员查找走 JNI 的哪个入口（静态方法、实例方法和字段各不相同）。
#[derive(Clone, Copy)]
enum MemberKind {
    Instance,
    Static,
    Field,
    StaticField,
}

impl MemberKind {
    fn lookup_index(self) -> u64 {
        match self {
            Self::Instance => JNI_GET_METHOD_ID,
            Self::Static => JNI_GET_STATIC_METHOD_ID,
            Self::Field => JNI_GET_FIELD_ID,
            Self::StaticField => JNI_GET_STATIC_FIELD_ID,
        }
    }
}

/// 状态单元在 native 侧的存储类型。
#[derive(Clone, Copy, PartialEq, Eq)]
enum CellKind {
    Number,
    Bool,
    String,
}

/// 一次 JNI 调用的实参，按 jvalue 的 8 字节槽写入。
#[derive(Clone, Copy)]
enum JArg<'ctx> {
    I32(IntValue<'ctx>),
    F32(FloatValue<'ctx>),
    Obj(PointerValue<'ctx>),
}

/// 子级在 `LinearLayout` / `FrameLayout` 里的布局参数。
struct ChildParams<'ctx> {
    width: IntValue<'ctx>,
    height: IntValue<'ctx>,
    weight: Option<FloatValue<'ctx>>,
    gravity: i32,
    /// left, top, right, bottom
    margins: [IntValue<'ctx>; 4],
}

/// `build_node` 的结果：视图本身，以及它是否是还没登记的槽位。
struct Built<'ctx> {
    view: PointerValue<'ctx>,
    conditional: Option<PendingConditional<'ctx>>,
}

struct PendingConditional<'ctx> {
    cell: String,
    initial: bool,
    slot: PointerValue<'ctx>,
    true_global: GlobalValue<'ctx>,
    false_global: Option<GlobalValue<'ctx>>,
    true_mount: Vec<usize>,
    true_unmount: Vec<usize>,
    false_mount: Vec<usize>,
    false_unmount: Vec<usize>,
}

#[derive(Clone)]
struct ConditionalRecord<'ctx> {
    cell: usize,
    slot: GlobalValue<'ctx>,
    true_global: GlobalValue<'ctx>,
    false_global: Option<GlobalValue<'ctx>>,
    active: GlobalValue<'ctx>,
    true_mount: Vec<usize>,
    true_unmount: Vec<usize>,
    false_mount: Vec<usize>,
    false_unmount: Vec<usize>,
}

/// 把 Android 后端的整套函数生成进模块；落盘由调用方决定，测试里可以直接检查 IR。
pub(crate) fn emit<'ctx>(codegen: &IosCodeGen<'ctx>, program: &UiProgram) -> Result<(), String> {
    if !program.errors.is_empty() {
        return Err(program.errors.join("\n"));
    }
    if program.states.is_empty() && program.nodes.iter().any(node_has_input_binding) {
        return Err("Input onInput requires a state(...) declaration".into());
    }
    let android = AndroidGen::new(codegen, program, state_cells(&program.states));
    android.run()?;
    // IosCodeGen::new_for 会预先声明 ObjC 运行时函数，Android 侧一个都不用，
    // 删掉后模块里只剩 JNI 和 libc 依赖。
    for name in ["objc_getClass", "sel_registerName", "objc_msgSend"] {
        if let Some(function) = codegen.module.get_function(name) {
            unsafe { function.delete() };
        }
    }
    codegen.module.verify().map_err(|error| error.to_string())?;
    Ok(())
}

struct AndroidGen<'a, 'ctx> {
    codegen: &'a IosCodeGen<'ctx>,
    program: &'a UiProgram,
    cells: Vec<(String, StateValue)>,
    kinds: Vec<CellKind>,
    activity: RefCell<Option<PointerValue<'ctx>>>,
    jvalues: Cell<Option<PointerValue<'ctx>>>,
    /// 每单元一组引用它的 TextView 全局。
    text_globals: RefCell<Vec<Vec<GlobalValue<'ctx>>>>,
    conditionals: RefCell<Vec<ConditionalRecord<'ctx>>>,
    branch_counter: Cell<usize>,
    refresh_fns: Vec<FunctionValue<'ctx>>,
    toggle_fns: Vec<FunctionValue<'ctx>>,
    action_fns: Vec<FunctionValue<'ctx>>,
    /// 列表渲染槽位（同一个列表可以渲染多次）
    list_slots: RefCell<HashMap<String, Vec<AndroidListSlot<'ctx>>>>,
    /// 行模板构建时的当前行
    list_row_ctx: RefCell<Option<AndroidRowCtx<'ctx>>>,
    list_counter: Cell<usize>,
    /// 行内按钮点击时带出来的行号（-1 表示不是行内点击）
    active_row: GlobalValue<'ctx>,
    /// 绑定了透明度的视图（按单元分组）
    opacity_globals: RefCell<Vec<Vec<GlobalValue<'ctx>>>>,
    /// 列表槽位 id → 字段顺序（行内控件按「字段序号」分配 view id，rebind 时靠它定位）
    list_fields: RefCell<HashMap<String, Vec<String>>>,
}

/// 行内控件的 view id 空间：字面量 k → 1000+k，按钮 action a → 2000+a。
/// id 只在单行子树里唯一，跨行重复没问题（`findViewById` 从行根往下找）。
const ROW_LABEL_ID_BASE: i32 = 1000;
const ROW_ACTION_ID_BASE: i32 = 2000;

/// 行视图复用开关（实验结论：默认关闭）。
///
/// 做过一版「按位置复用行视图 + 只 rebind 文字/点击 id」的实现，在同机 `bench` 示例上
/// 逐个对比「删一行」的 `gfxinfo` 50th percentile：
///   复用（rebind）：300 / 400 / 650 / 1200ms
///   全清重建      ：350 / 400 / 600 / 950 / 1000ms
/// 两者在噪声内打平：建行约 8 次 JNI，rebind 约 12 次 JNI（findViewById、setText、
/// 新建 ClickBridge），真正的成本是「每行十几次 JNI 往返」，而不是重建视图本身。
/// 要让这块真正变快，得把整行的构建/绑定下沉到 Java 侧，一次调用处理一行。
const ROW_REUSE: bool = false;

#[derive(Clone)]
struct AndroidListSlot<'ctx> {
    id: String,
    count_global: GlobalValue<'ctx>,
    container_global: GlobalValue<'ctx>,
    columns: HashMap<String, GlobalValue<'ctx>>,
    rebuild: FunctionValue<'ctx>,
    /// 行模板里的行内条件个数：删除行时这些标记列要跟着一起挪。
    conditionals: usize,
}

#[derive(Clone)]
struct AndroidRowCtx<'ctx> {
    id: String,
    index: IntValue<'ctx>,
}

impl<'a, 'ctx> AndroidGen<'a, 'ctx> {
    fn new(
        codegen: &'a IosCodeGen<'ctx>,
        program: &'a UiProgram,
        cells: Vec<(String, StateValue)>,
    ) -> Self {
        let kinds = cells
            .iter()
            .map(|(_, value)| match value {
                StateValue::Number(_) => CellKind::Number,
                StateValue::Boolean(_) => CellKind::Bool,
                StateValue::String(_) => CellKind::String,
            })
            .collect::<Vec<_>>();

        let ptr_type = codegen.context.ptr_type(AddressSpace::default());
        let env = codegen.module.add_global(ptr_type, None, "g_env");
        env.set_initializer(&ptr_type.const_null());
        let failed = codegen
            .module
            .add_global(codegen.context.i32_type(), None, "g_failed");
        failed.set_initializer(&codegen.context.i32_type().const_zero());
        let density = codegen
            .module
            .add_global(codegen.context.f32_type(), None, "g_density");
        density.set_initializer(&codegen.context.f32_type().const_float(0.0));
        let fade_view = codegen.module.add_global(ptr_type, None, "g_fade_view");
        fade_view.set_initializer(&ptr_type.const_null());
        let activity_global = codegen.module.add_global(ptr_type, None, "g_activity");
        activity_global.set_initializer(&ptr_type.const_null());
        let active_row = codegen.module.add_global(codegen.context.i64_type(), None, "g_active_row");
        active_row.set_initializer(&codegen.context.i64_type().const_int(u64::MAX, true));

        for (index, kind) in kinds.iter().enumerate() {
            if *kind == CellKind::String {
                let name = format!("g_cell_{index}_text");
                let global = codegen.module.add_global(ptr_type, None, &name);
                global.set_initializer(&ptr_type.const_null());
            } else {
                let name = format!("g_cell_{index}");
                let global = codegen
                    .module
                    .add_global(codegen.context.i64_type(), None, &name);
                global.set_initializer(&codegen.context.i64_type().const_zero());
            }
        }

        let void_type = codegen.context.void_type();
        let refresh_fns = (0..kinds.len())
            .map(|index| {
                codegen.module.add_function(
                    &format!("lumen_refresh_{index}"),
                    void_type.fn_type(&[], false),
                    None,
                )
            })
            .collect();
        let toggle_fns = (0..count_conditionals(&program.nodes))
            .map(|index| {
                codegen.module.add_function(
                    &format!("lumen_toggle_{index}"),
                    void_type.fn_type(&[], false),
                    None,
                )
            })
            .collect();
        let action_fns = (0..program.state_actions.len())
            .map(|index| {
                codegen.module.add_function(
                    &format!("lumen_action_{index}"),
                    void_type.fn_type(&[ptr_type.into()], false),
                    None,
                )
            })
            .collect();

        Self {
            codegen,
            program,
            cells,
            kinds,
            activity: RefCell::new(None),
            jvalues: Cell::new(None),
            text_globals: RefCell::new(Vec::new()),
            conditionals: RefCell::new(Vec::new()),
            branch_counter: Cell::new(0),
            refresh_fns,
            toggle_fns,
            action_fns,
            list_slots: RefCell::new(HashMap::new()),
            list_row_ctx: RefCell::new(None),
            list_counter: Cell::new(0),
            active_row,
            opacity_globals: RefCell::new(Vec::new()),
            list_fields: RefCell::new(HashMap::new()),
        }
    }

    fn run(&self) -> Result<(), String> {
        self.validate()?;
        *self.text_globals.borrow_mut() = (0..self.cells.len()).map(|_| Vec::new()).collect();
        *self.opacity_globals.borrow_mut() = (0..self.cells.len()).map(|_| Vec::new()).collect();

        self.emit_require()?;
        self.emit_jni_init()?;
        self.emit_build_ui()?;
        self.emit_refresh_fns()?;
        self.emit_toggle_fns()?;
        self.emit_action_fns()?;
        self.emit_entries()?;
        Ok(())
    }

    fn validate(&self) -> Result<(), String> {
        Ok(())
    }

    // ---------- 基础访问 ----------

    fn context(&self) -> &'ctx Context {
        self.codegen.context
    }

    fn module(&self) -> &Module<'ctx> {
        &self.codegen.module
    }

    fn builder(&self) -> &Builder<'ctx> {
        &self.codegen.builder
    }

    fn ptr_type(&self) -> PointerType<'ctx> {
        self.context().ptr_type(AddressSpace::default())
    }

    fn i32_type(&self) -> IntType<'ctx> {
        self.context().i32_type()
    }

    fn i64_type(&self) -> IntType<'ctx> {
        self.context().i64_type()
    }

    fn f32_type(&self) -> FloatType<'ctx> {
        self.context().f32_type()
    }

    fn void_type(&self) -> VoidType<'ctx> {
        self.context().void_type()
    }

    fn i32c(&self, value: i32) -> IntValue<'ctx> {
        self.i32_type().const_int(value as i64 as u64, false)
    }

    fn f32c(&self, value: f32) -> FloatValue<'ctx> {
        self.f32_type().const_float(value as f64)
    }

    fn activity(&self) -> Result<PointerValue<'ctx>, String> {
        if let Some(global) = self.codegen.module.get_global("g_activity") {
            return Ok(self
                .builder()
                .build_load(self.ptr_type(), global.as_pointer_value(), "activity")
                .map_err(|error| error.to_string())?
                .into_pointer_value());
        }
        self.activity
            .borrow()
            .ok_or_else(|| "activity is only available while building the UI".to_string())
    }

    /// 进入一个新函数：定位到 entry 块，并在块首分配各 JNI 调用共用的 jvalue 数组。
    /// alloca 必须落在 entry 里，建在分支块上会破坏支配关系。
    fn begin_function(&self, function: FunctionValue<'ctx>) -> Result<(), String> {
        let entry = self.context().append_basic_block(function, "entry");
        self.builder().position_at_end(entry);
        let slots = self
            .builder()
            .build_array_alloca(
                self.i64_type(),
                self.i64_type().const_int(16, false),
                "jvalues",
            )
            .map_err(|error| error.to_string())?;
        self.jvalues.set(Some(slots));
        Ok(())
    }

    /// jvalue 数组：所有调用共用一块栈空间，每次调用按顺序写入 8 字节槽。
    fn jvalue_array(&self, args: &[JArg<'ctx>]) -> Result<PointerValue<'ctx>, String> {
        let slots = self
            .jvalues
            .get()
            .ok_or("jvalue array is only available inside a generated function")?;
        for (index, arg) in args.iter().enumerate() {
            let slot = unsafe {
                self.builder().build_gep(
                    self.i64_type(),
                    slots,
                    &[self.i64_type().const_int(index as u64, false)],
                    "jvalue_slot",
                )
            }
            .map_err(|error| error.to_string())?;
            let value = match arg {
                JArg::I32(value) => self
                    .builder()
                    .build_int_s_extend(*value, self.i64_type(), "jint_slot")
                    .map_err(|error| error.to_string())?,
                JArg::F32(value) => {
                    let bits = self
                        .builder()
                        .build_bit_cast(*value, self.i32_type(), "jfloat_bits")
                        .map_err(|error| error.to_string())?
                        .into_int_value();
                    self.builder()
                        .build_int_z_extend(bits, self.i64_type(), "jfloat_slot")
                        .map_err(|error| error.to_string())?
                }
                JArg::Obj(value) => self
                    .builder()
                    .build_ptr_to_int(*value, self.i64_type(), "jobject_slot")
                    .map_err(|error| error.to_string())?,
            };
            self.builder()
                .build_store(slot, value)
                .map_err(|error| error.to_string())?;
        }
        Ok(slots)
    }

    fn jni_fn(&self, env: PointerValue<'ctx>, index: u64) -> Result<PointerValue<'ctx>, String> {
        let ptr_type = self.ptr_type();
        let table = self
            .builder()
            .build_load(ptr_type, env, "jni_table")
            .map_err(|error| error.to_string())?
            .into_pointer_value();
        let slot = unsafe {
            self.builder().build_gep(
                ptr_type,
                table,
                &[self.i64_type().const_int(index, false)],
                "jni_slot",
            )
        }
        .map_err(|error| error.to_string())?;
        Ok(self
            .builder()
            .build_load(ptr_type, slot, "jni_fn")
            .map_err(|error| error.to_string())?
            .into_pointer_value())
    }

    fn env(&self) -> Result<PointerValue<'ctx>, String> {
        let global = self
            .module()
            .get_global("g_env")
            .ok_or("g_env was not created")?;
        Ok(self
            .builder()
            .build_load(self.ptr_type(), global.as_pointer_value(), "env")
            .map_err(|error| error.to_string())?
            .into_pointer_value())
    }

    fn invoke(
        &self,
        index: u64,
        ret: Option<BasicTypeEnum<'ctx>>,
        params: &[BasicMetadataTypeEnum<'ctx>],
        args: &[BasicMetadataValueEnum<'ctx>],
    ) -> Result<Option<BasicValueEnum<'ctx>>, String> {
        let env = self.env()?;
        let fn_ptr = self.jni_fn(env, index)?;
        let fn_type = match ret {
            Some(ret) => ret.fn_type(params, false),
            None => self.void_type().fn_type(params, false),
        };
        let call = self
            .builder()
            .build_indirect_call(fn_type, fn_ptr, args, "jni_result")
            .map_err(|error| error.to_string())?;
        Ok(match call.try_as_basic_value() {
            inkwell::values::ValueKind::Basic(value) => Some(value),
            inkwell::values::ValueKind::Instruction(_) => None,
        })
    }

    /// `Call*MethodA`：(env, obj, mid, jvalue*)。
    fn call_method(
        &self,
        index: u64,
        ret: Option<BasicTypeEnum<'ctx>>,
        object: PointerValue<'ctx>,
        method: PointerValue<'ctx>,
        args: &[JArg<'ctx>],
    ) -> Result<Option<BasicValueEnum<'ctx>>, String> {
        let env = self.env()?;
        let values = self.jvalue_array(args)?;
        let ptr_type = self.ptr_type();
        let params = [
            ptr_type.into(),
            ptr_type.into(),
            ptr_type.into(),
            ptr_type.into(),
        ];
        self.invoke(
            index,
            ret,
            &params,
            &[env.into(), object.into(), method.into(), values.into()],
        )
    }

    /// `CallStatic*MethodA`：(env, class, mid, jvalue*)。
    fn call_static(
        &self,
        index: u64,
        ret: Option<BasicTypeEnum<'ctx>>,
        class: PointerValue<'ctx>,
        method: PointerValue<'ctx>,
        args: &[JArg<'ctx>],
    ) -> Result<Option<BasicValueEnum<'ctx>>, String> {
        let env = self.env()?;
        let values = self.jvalue_array(args)?;
        let ptr_type = self.ptr_type();
        let params = [
            ptr_type.into(),
            ptr_type.into(),
            ptr_type.into(),
            ptr_type.into(),
        ];
        self.invoke(
            index,
            ret,
            &params,
            &[env.into(), class.into(), method.into(), values.into()],
        )
    }

    fn call_void(
        &self,
        object: PointerValue<'ctx>,
        member: &str,
        args: &[JArg<'ctx>],
    ) -> Result<(), String> {
        let method = self.member(member)?;
        self.call_method(JNI_CALL_VOID_A, None, object, method, args)?;
        Ok(())
    }

    /// 静态方法调用：返回 jobject（`LumenRow.parse` 这类工厂）。
    fn call_static_object(
        &self,
        class: &str,
        member: &str,
        args: &[JArg<'ctx>],
    ) -> Result<PointerValue<'ctx>, String> {
        let class = self.class(class)?;
        let method = self.member(member)?;
        match self.call_static(
            JNI_CALL_STATIC_OBJECT_A,
            Some(self.ptr_type().into()),
            class,
            method,
            args,
        )? {
            Some(value) => Ok(value.into_pointer_value()),
            None => Err(format!("{member} returned void")),
        }
    }

    fn call_static_void(
        &self,
        class: &str,
        member: &str,
        args: &[JArg<'ctx>],
    ) -> Result<(), String> {
        let class = self.class(class)?;
        let method = self.member(member)?;
        self.call_static(JNI_CALL_STATIC_VOID_A, None, class, method, args)?;
        Ok(())
    }

    fn call_object(
        &self,
        object: PointerValue<'ctx>,
        member: &str,
        args: &[JArg<'ctx>],
    ) -> Result<PointerValue<'ctx>, String> {
        let method = self.member(member)?;
        match self.call_method(
            JNI_CALL_OBJECT_A,
            Some(self.ptr_type().into()),
            object,
            method,
            args,
        )? {
            Some(value) => Ok(value.into_pointer_value()),
            None => Err(format!("{member} returned void")),
        }
    }

    fn new_object(
        &self,
        class: &str,
        ctor: &str,
        args: &[JArg<'ctx>],
    ) -> Result<PointerValue<'ctx>, String> {
        let env = self.env()?;
        let class = self.class(class)?;
        let ctor = self.member(ctor)?;
        let values = self.jvalue_array(args)?;
        let ptr_type = self.ptr_type();
        let params = [
            ptr_type.into(),
            ptr_type.into(),
            ptr_type.into(),
            ptr_type.into(),
        ];
        match self.invoke(
            JNI_NEW_OBJECT_A,
            Some(ptr_type.into()),
            &params,
            &[env.into(), class.into(), ctor.into(), values.into()],
        )? {
            Some(value) => Ok(value.into_pointer_value()),
            None => Err("NewObjectA returned void".into()),
        }
    }

    fn new_string(&self, c_string: PointerValue<'ctx>) -> Result<PointerValue<'ctx>, String> {
        let env = self.env()?;
        let ptr_type = self.ptr_type();
        let params = [ptr_type.into(), ptr_type.into()];
        match self.invoke(
            JNI_NEW_STRING_UTF,
            Some(ptr_type.into()),
            &params,
            &[env.into(), c_string.into()],
        )? {
            Some(value) => Ok(value.into_pointer_value()),
            None => Err("NewStringUTF returned void".into()),
        }
    }

    /// 字符串字面量 → jstring。
    fn text_string(&self, text: &str) -> Result<PointerValue<'ctx>, String> {
        let c_string = self.cstr(text)?;
        self.new_string(c_string)
    }

    fn global_ref(&self, object: PointerValue<'ctx>) -> Result<PointerValue<'ctx>, String> {
        let env = self.env()?;
        let ptr_type = self.ptr_type();
        let params = [ptr_type.into(), ptr_type.into()];
        match self.invoke(
            JNI_NEW_GLOBAL_REF,
            Some(ptr_type.into()),
            &params,
            &[env.into(), object.into()],
        )? {
            Some(value) => Ok(value.into_pointer_value()),
            None => Err("NewGlobalRef returned void".into()),
        }
    }

    fn set_float_field(
        &self,
        object: PointerValue<'ctx>,
        field: &str,
        value: FloatValue<'ctx>,
    ) -> Result<(), String> {
        let env = self.env()?;
        let field = self.member(field)?;
        let ptr_type = self.ptr_type();
        let params = [
            ptr_type.into(),
            ptr_type.into(),
            ptr_type.into(),
            self.f32_type().into(),
        ];
        self.invoke(
            JNI_SET_FLOAT_FIELD,
            None,
            &params,
            &[env.into(), object.into(), field.into(), value.into()],
        )?;
        Ok(())
    }

    fn set_int_field(
        &self,
        object: PointerValue<'ctx>,
        field: &str,
        value: IntValue<'ctx>,
    ) -> Result<(), String> {
        let env = self.env()?;
        let field = self.member(field)?;
        let ptr_type = self.ptr_type();
        let params = [
            ptr_type.into(),
            ptr_type.into(),
            ptr_type.into(),
            self.i32_type().into(),
        ];
        self.invoke(
            JNI_SET_INT_FIELD,
            None,
            &params,
            &[env.into(), object.into(), field.into(), value.into()],
        )?;
        Ok(())
    }

    fn class_global(&self, alias: &str) -> GlobalValue<'ctx> {
        let name = format!("g_cls_{alias}");
        if let Some(global) = self.module().get_global(&name) {
            return global;
        }
        let global = self.module().add_global(self.ptr_type(), None, &name);
        global.set_initializer(&self.ptr_type().const_null());
        global
    }

    fn member_global(&self, alias: &str) -> GlobalValue<'ctx> {
        let name = format!("g_mem_{alias}");
        if let Some(global) = self.module().get_global(&name) {
            return global;
        }
        let global = self.module().add_global(self.ptr_type(), None, &name);
        global.set_initializer(&self.ptr_type().const_null());
        global
    }

    fn class(&self, alias: &str) -> Result<PointerValue<'ctx>, String> {
        if !CLASSES.iter().any(|(known, _)| *known == alias) {
            return Err(format!("unknown JNI class alias '{alias}'"));
        }
        let global = self.class_global(alias);
        Ok(self
            .builder()
            .build_load(self.ptr_type(), global.as_pointer_value(), alias)
            .map_err(|error| error.to_string())?
            .into_pointer_value())
    }

    fn member(&self, alias: &str) -> Result<PointerValue<'ctx>, String> {
        let known = METHODS.iter().any(|(known, ..)| *known == alias)
            || FIELDS.iter().any(|(known, ..)| *known == alias)
            || STATIC_METHODS.iter().any(|(known, ..)| *known == alias)
            || STATIC_FIELDS.iter().any(|(known, ..)| *known == alias);
        if !known {
            // 漏登记的话生成期就报错，而不是运行时拿空 mid 崩在 ART 里
            return Err(format!("unknown JNI member alias '{alias}'"));
        }
        let global = self.member_global(alias);
        Ok(self
            .builder()
            .build_load(self.ptr_type(), global.as_pointer_value(), alias)
            .map_err(|error| error.to_string())?
            .into_pointer_value())
    }

    fn store_global(&self, name: &str, value: PointerValue<'ctx>) -> Result<(), String> {
        let global = self
            .module()
            .get_global(name)
            .ok_or_else(|| format!("{name} was not created"))?;
        self.builder()
            .build_store(global.as_pointer_value(), value)
            .map_err(|error| error.to_string())?;
        Ok(())
    }

    fn load_ptr_global(&self, name: &str) -> Result<PointerValue<'ctx>, String> {
        let global = self
            .module()
            .get_global(name)
            .ok_or_else(|| format!("{name} was not created"))?;
        Ok(self
            .builder()
            .build_load(self.ptr_type(), global.as_pointer_value(), name)
            .map_err(|error| error.to_string())?
            .into_pointer_value())
    }

    /// dp → 像素（样式里的数值按 iOS 的 points 理解，Android 侧要换成像素）。
    /// 换算走 double：clang 对 `float` 类型的十进制常量要求精确可表示，像 1.4×22=30.8
    /// 这种值 LLVM 会打印成 `3.08e+01` 导致链接失败，double 没有这个限制。
    fn dp(&self, value: f64) -> Result<IntValue<'ctx>, String> {
        let scaled = self.dp_scaled(value)?;
        self.builder()
            .build_float_to_signed_int(scaled, self.i32_type(), "dp_px")
            .map_err(|error| error.to_string())
    }

    fn dp_f32(&self, value: f64) -> Result<FloatValue<'ctx>, String> {
        let scaled = self.dp_scaled(value)?;
        self.builder()
            .build_float_cast(scaled, self.f32_type(), "dp_f32")
            .map_err(|error| error.to_string())
    }

    fn dp_scaled(&self, value: f64) -> Result<FloatValue<'ctx>, String> {
        let global = self
            .module()
            .get_global("g_density")
            .ok_or("g_density was not created")?;
        let density = self
            .builder()
            .build_load(self.f32_type(), global.as_pointer_value(), "density")
            .map_err(|error| error.to_string())?
            .into_float_value();
        let density = self
            .builder()
            .build_float_ext(density, self.context().f64_type(), "density_f64")
            .map_err(|error| error.to_string())?;
        self.builder()
            .build_float_mul(density, self.context().f64_type().const_float(value), "dp_scaled")
            .map_err(|error| error.to_string())
    }

    // ---------- 初始化：类 / 方法 ID / 屏幕密度 ----------

    /// 记录一次查找失败：打 log 并置失败标记，避免后面拿空 ID 把进程打崩。
    fn emit_require(&self) -> Result<(), String> {
        let ptr_type = self.ptr_type();
        let fn_type = self
            .void_type()
            .fn_type(&[ptr_type.into(), ptr_type.into()], false);
        let function = self.module().add_function("lumen_require", fn_type, None);
        self.begin_function(function)?;

        let member = function.get_nth_param(0).unwrap().into_pointer_value();
        let name = function.get_nth_param(1).unwrap().into_pointer_value();
        let missing = self.context().append_basic_block(function, "missing");
        let done = self.context().append_basic_block(function, "done");
        let is_null = self
            .builder()
            .build_is_null(member, "is_null")
            .map_err(|error| error.to_string())?;
        self.builder()
            .build_conditional_branch(is_null, missing, done)
            .map_err(|error| error.to_string())?;

        self.builder().position_at_end(missing);
        // 查找失败会在 Java 侧挂一个 NoSuchMethod/NoClassDef 异常；带挂起异常调
        // Java 方法本身就会被 ART 判违法，所以先清掉再打日志。
        let env = self.env()?;
        self.invoke(JNI_EXCEPTION_CLEAR, None, &[ptr_type.into()], &[env.into()])?;
        let tag = self.text_string("Lumen")?;
        let message = self.new_string(name)?;
        let log_class = self.class("Log")?;
        let log_error = self.member("logError")?;
        self.call_static(
            JNI_CALL_STATIC_INT_A,
            Some(self.i32_type().into()),
            log_class,
            log_error,
            &[JArg::Obj(tag), JArg::Obj(message)],
        )?;
        let failed = self
            .module()
            .get_global("g_failed")
            .ok_or("g_failed was not created")?;
        self.builder()
            .build_store(failed.as_pointer_value(), self.i32c(1))
            .map_err(|error| error.to_string())?;
        self.builder()
            .build_unconditional_branch(done)
            .map_err(|error| error.to_string())?;

        self.builder().position_at_end(done);
        self.builder()
            .build_return(None)
            .map_err(|error| error.to_string())?;
        Ok(())
    }

    fn emit_jni_init(&self) -> Result<(), String> {
        let ptr_type = self.ptr_type();
        let fn_type = self.void_type().fn_type(&[ptr_type.into()], false);
        let function = self.module().add_function("lumen_jni_init", fn_type, None);
        self.begin_function(function)?;
        let missing_class = self.context().append_basic_block(function, "class_missing");

        let activity = function.get_nth_param(0).unwrap().into_pointer_value();
        let require = self
            .module()
            .get_function("lumen_require")
            .ok_or("lumen_require was not created")?;

        // Log.e 自己就是失败诊断用的方法，先单独解析：它拿不到就没什么可诊断的了，
        // 也不能让它依赖 lumen_require（require 里会调它）。
        let log_class_name = CLASSES
            .iter()
            .find(|(alias, _)| *alias == "Log")
            .map(|(_, name)| *name)
            .ok_or("Log class is missing from the table")?;
        self.emit_find_class("Log", log_class_name, None, missing_class)?;
        let (log_alias, log_class_alias, log_method, log_signature) = STATIC_METHODS[0];
        let log_class = self.class(log_class_alias)?;
        let name = self.cstr(log_method)?;
        let signature = self.cstr(log_signature)?;
        let log_error = self.emit_member_lookup(log_class, name, signature, MemberKind::Static)?;
        self.builder()
            .build_store(self.member_global(log_alias).as_pointer_value(), log_error)
            .map_err(|error| error.to_string())?;
        self.branch_when_null(log_error, missing_class)?;

        for (alias, name) in CLASSES {
            if *alias == "Log" {
                continue;
            }
            self.emit_find_class(alias, name, Some(require), missing_class)?;
        }

        for (alias, class_alias, name, signature) in METHODS {
            let class = self.class(class_alias)?;
            let name = self.cstr(name)?;
            let signature = self.cstr(signature)?;
            let method = self.emit_member_lookup(class, name, signature, MemberKind::Instance)?;
            self.builder()
                .build_store(self.member_global(alias).as_pointer_value(), method)
                .map_err(|error| error.to_string())?;
            let label = self.cstr(alias)?;
            self.builder()
                .build_call(require, &[method.into(), label.into()], "")
                .map_err(|error| error.to_string())?;
        }

        for (alias, class_alias, name, signature) in FIELDS {
            let class = self.class(class_alias)?;
            let name = self.cstr(name)?;
            let signature = self.cstr(signature)?;
            let field = self.emit_member_lookup(class, name, signature, MemberKind::Field)?;
            self.builder()
                .build_store(self.member_global(alias).as_pointer_value(), field)
                .map_err(|error| error.to_string())?;
            let label = self.cstr(alias)?;
            self.builder()
                .build_call(require, &[field.into(), label.into()], "")
                .map_err(|error| error.to_string())?;
        }

        for (alias, class_alias, name, signature) in STATIC_FIELDS {
            let class = self.class(class_alias)?;
            let name = self.cstr(name)?;
            let signature = self.cstr(signature)?;
            let field = self.emit_member_lookup(class, name, signature, MemberKind::StaticField)?;
            self.builder()
                .build_store(self.member_global(alias).as_pointer_value(), field)
                .map_err(|error| error.to_string())?;
            let label = self.cstr(alias)?;
            self.builder()
                .build_call(require, &[field.into(), label.into()], "")
                .map_err(|error| error.to_string())?;
        }

        for (alias, class_alias, name, signature) in STATIC_METHODS {
            // 第一个条目是错误日志方法本身，它走单独的一次查找，这里跳过
            if *alias == "logError" {
                continue;
            }
            let class = self.class(class_alias)?;
            let name = self.cstr(name)?;
            let signature = self.cstr(signature)?;
            let method = self.emit_member_lookup(class, name, signature, MemberKind::Static)?;
            self.builder()
                .build_store(self.member_global(alias).as_pointer_value(), method)
                .map_err(|error| error.to_string())?;
            let label = self.cstr(alias)?;
            self.builder()
                .build_call(require, &[method.into(), label.into()], "")
                .map_err(|error| error.to_string())?;
        }

        // 屏幕密度：dp 尺寸、字号换算和圆角半径都靠它。
        let resources = self.call_object(activity, "getResources", &[])?;
        let metrics = self.call_object(resources, "getDisplayMetrics", &[])?;
        let density_field = self.member("density")?;
        let env = self.env()?;
        let params = [ptr_type.into(), ptr_type.into(), ptr_type.into()];
        let density = self
            .invoke(
                JNI_GET_FLOAT_FIELD,
                Some(self.f32_type().into()),
                &params,
                &[env.into(), metrics.into(), density_field.into()],
            )?
            .ok_or("GetFloatField returned void")?
            .into_float_value();
        self.builder()
            .build_store(
                self.module()
                    .get_global("g_density")
                    .ok_or("g_density was not created")?
                    .as_pointer_value(),
                density,
            )
            .map_err(|error| error.to_string())?;

        self.builder()
            .build_return(None)
            .map_err(|error| error.to_string())?;
        self.builder().position_at_end(missing_class);
        self.builder()
            .build_return(None)
            .map_err(|error| error.to_string())?;
        Ok(())
    }

    fn emit_find_class(
        &self,
        alias: &str,
        jni_name: &str,
        require: Option<FunctionValue<'ctx>>,
        missing: BasicBlock<'ctx>,
    ) -> Result<(), String> {
        let ptr_type = self.ptr_type();
        let name = self.cstr(jni_name)?;
        let env = self.env()?;
        let params = [ptr_type.into(), ptr_type.into()];
        let found = self
            .invoke(
                JNI_FIND_CLASS,
                Some(ptr_type.into()),
                &params,
                &[env.into(), name.into()],
            )?
            .ok_or("FindClass returned void")?
            .into_pointer_value();
        let reference = self.global_ref(found)?;
        self.builder()
            .build_store(self.class_global(alias).as_pointer_value(), reference)
            .map_err(|error| error.to_string())?;
        if let Some(require) = require {
            let label = self.cstr(alias)?;
            self.builder()
                .build_call(require, &[reference.into(), label.into()], "")
                .map_err(|error| error.to_string())?;
        }
        self.branch_when_null(reference, missing)
    }

    /// 值为空时跳到失败块，否则继续；失败块由调用方收尾。
    fn branch_when_null(
        &self,
        value: PointerValue<'ctx>,
        missing: BasicBlock<'ctx>,
    ) -> Result<(), String> {
        let current = self
            .builder()
            .get_insert_block()
            .ok_or("missing insert block")?;
        let function = current.get_parent().ok_or("missing parent function")?;
        let is_null = self
            .builder()
            .build_is_null(value, "is_null")
            .map_err(|error| error.to_string())?;
        let next = self.context().append_basic_block(function, "present");
        self.builder()
            .build_conditional_branch(is_null, missing, next)
            .map_err(|error| error.to_string())?;
        self.builder().position_at_end(next);
        Ok(())
    }

    fn emit_member_lookup(
        &self,
        class: PointerValue<'ctx>,
        name: PointerValue<'ctx>,
        signature: PointerValue<'ctx>,
        kind: MemberKind,
    ) -> Result<PointerValue<'ctx>, String> {
        let ptr_type = self.ptr_type();
        let env = self.env()?;
        let params = [
            ptr_type.into(),
            ptr_type.into(),
            ptr_type.into(),
            ptr_type.into(),
        ];
        Ok(self
            .invoke(
                kind.lookup_index(),
                Some(ptr_type.into()),
                &params,
                &[env.into(), class.into(), name.into(), signature.into()],
            )?
            .ok_or("member lookup returned void")?
            .into_pointer_value())
    }

    /// 读一个静态对象字段（例如 TextUtils.TruncateAt.END 这类枚举常量）。
    fn static_object_field(
        &self,
        class_alias: &str,
        field_alias: &str,
    ) -> Result<PointerValue<'ctx>, String> {
        let env = self.env()?;
        let class = self.class(class_alias)?;
        let field = self.member(field_alias)?;
        let ptr_type = self.ptr_type();
        let params = [ptr_type.into(), ptr_type.into(), ptr_type.into()];
        Ok(self
            .invoke(
                JNI_GET_STATIC_OBJECT_FIELD,
                Some(ptr_type.into()),
                &params,
                &[env.into(), class.into(), field.into()],
            )?
            .ok_or("GetStaticObjectField returned void")?
            .into_pointer_value())
    }

    fn cstr(&self, value: &str) -> Result<PointerValue<'ctx>, String> {
        Ok(self
            .builder()
            .build_global_string_ptr(value, "lumen_cstr")
            .map_err(|error| error.to_string())?
            .as_pointer_value())
    }

    // ---------- 界面构建 ----------

    fn emit_build_ui(&self) -> Result<(), String> {
        let ptr_type = self.ptr_type();
        let fn_type = self.void_type().fn_type(&[ptr_type.into()], false);
        let function = self.module().add_function("lumen_build_ui", fn_type, None);
        self.begin_function(function)?;

        let activity = function.get_nth_param(0).unwrap().into_pointer_value();
        *self.activity.borrow_mut() = Some(activity);
        // 行重建在点击回调里也会用到底层的 Context，必须持有全局引用
        let activity_ref = self.global_ref(activity)?;
        self.store_global("g_activity", activity_ref)?;

        self.emit_cell_init()?;

        let root = match self.program.nodes.first() {
            Some(node) => self.build_node(node)?,
            None => Built {
                view: self.new_object("Frame", "frameCtor", &[JArg::Obj(activity)])?,
                conditional: None,
            },
        };
        if root.conditional.is_some() {
            return Err("a conditional cannot be the root view".into());
        }
        self.call_void(activity, "setContentView", &[JArg::Obj(root.view)])?;


        let mut mounts = Vec::new();
        collect_initial_mount_actions(&self.program.nodes, &mut mounts);
        for action in mounts {
            let function = self.action_fns[action];
            self.builder()
                .build_call(function, &[ptr_type.const_null().into()], "")
                .map_err(|error| error.to_string())?;
        }

        self.builder()
            .build_return(None)
            .map_err(|error| error.to_string())?;
        Ok(())
    }

    fn emit_cell_init(&self) -> Result<(), String> {
        for (index, (_, value)) in self.cells.iter().enumerate() {
            match self.kinds[index] {
                CellKind::Number => {
                    let number = match value {
                        StateValue::Number(number) => *number,
                        _ => 0,
                    };
                    self.builder()
                        .build_store(
                            self.numeric_global(index)?.as_pointer_value(),
                            self.i64_type().const_int(number as u64, false),
                        )
                        .map_err(|error| error.to_string())?;
                }
                CellKind::Bool => {
                    let flag = matches!(value, StateValue::Boolean(true));
                    self.builder()
                        .build_store(
                            self.numeric_global(index)?.as_pointer_value(),
                            self.i64_type().const_int(flag as u64, false),
                        )
                        .map_err(|error| error.to_string())?;
                }
                CellKind::String => {
                    let text = match value {
                        StateValue::String(text) => text.clone(),
                        other => other.display_value(),
                    };
                    let c_string = self.cstr(&text)?;
                    self.builder()
                        .build_store(self.string_global(index)?.as_pointer_value(), c_string)
                        .map_err(|error| error.to_string())?;
                }
            }
        }
        Ok(())
    }

    /// 渐显目标初始透明；目标没登记就跳过。

    fn build_node(&self, node: &IRNode) -> Result<Built<'ctx>, String> {
        match node {
            IRNode::View { style, children } => {
                self.build_container(style, children, FlexDirection::Column, 0.0)
            }
            IRNode::VStack { style, children } => {
                self.build_container(style, children, FlexDirection::Column, 8.0)
            }
            IRNode::ScrollView { style, children } => self.build_scroll(style, children),
            IRNode::Text { content, style } => Ok(Built {
                view: self.build_text(content, style)?,
                conditional: None,
            }),
            IRNode::StateText {
                cell,
                initial,
                style,
            } => self.build_state_text(cell, initial, style),
            IRNode::Image { style, source, .. } => {
                eprintln!(
                    "warning: Android 后端暂未接入图片资源，<Image source=\"{source}\"> 只渲染占位视图"
                );
                Ok(Built {
                    view: self.new_view_of("Image", "imageCtor", style)?,
                    conditional: None,
                })
            }
            IRNode::Input {
                placeholder,
                style,
                on_input,
                on_focus,
                ..
            } => self.build_input(placeholder, style, *on_input, *on_focus),
            IRNode::Button {
                title,
                style,
                state_action,
                ..
            } => Ok(Built {
                view: self.build_button(title, style, *state_action)?,
                conditional: None,
            }),
            IRNode::Conditional {
                cell,
                when_true,
                when_false,
                initial,
                true_mount,
                true_unmount,
                false_mount,
                false_unmount,
                ..
            } => self.build_conditional(
                cell,
                when_true,
                when_false.as_deref(),
                *initial,
                true_mount,
                true_unmount,
                false_mount,
                false_unmount,
            ),
            IRNode::List {
                style,
                list,
                item,
                fields,
                rows,
                parents,
            } => self.build_list(style, list, item, fields, rows, parents),
            IRNode::RecursiveComponent { component, .. } => Err(format!(
                "recursive component slot '{component}' must be a direct child of the row template root"
            )),
            IRNode::ItemText { field, style, .. } => {
                let ctx = self.list_row_ctx.borrow().clone().ok_or_else(|| {
                    "list item text is only valid inside items.map(...)".to_string()
                })?;
                let view = self.new_view_of("Text", "textCtor", style)?;
                let value = self.load_list_column(&ctx.id, field, ctx.index)?;
                let text = self.new_string(value)?;
                self.call_void(view, "setText", &[JArg::Obj(text)])?;
                // 字段序号写进 view id：行复用后 rebind 靠 findViewById 找到这个 TextView
                if let Some(index) = self
                    .list_fields
                    .borrow()
                    .get(&ctx.id)
                    .and_then(|fields| fields.iter().position(|name| name == field))
                {
                    self.call_void(
                        view,
                        "setId",
                        &[JArg::I32(self.i32c(ROW_LABEL_ID_BASE + index as i32))],
                    )?;
                }
                self.call_void(view, "setSingleLine", &[JArg::I32(self.i32c(1))])?;
                self.call_void(view, "setIncludeFontPadding", &[JArg::I32(self.i32c(0))])?;
                self.apply_text_style(
                    view,
                    style,
                    GRAVITY_LEFT | GRAVITY_CENTER_VERTICAL,
                    DEFAULT_FONT_SIZE,
                )?;
                Ok(Built {
                    view,
                    conditional: None,
                })
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
                let function = self
                    .builder()
                    .get_insert_block()
                    .and_then(|block| block.get_parent())
                    .ok_or("missing insertion block")?;
                let value = self.load_list_column(&ctx.id, field, ctx.index)?;
                let literal = self.cstr("true")?;
                let compare = self.strcmp()?;
                let result = self
                    .builder()
                    .build_call(compare, &[value.into(), literal.into()], "item_cmp")
                    .map_err(|error| error.to_string())?;
                let result = match result.try_as_basic_value() {
                    inkwell::values::ValueKind::Basic(value) => value.into_int_value(),
                    inkwell::values::ValueKind::Instruction(_) => {
                        return Err("strcmp returned void".into());
                    }
                };
                let matched = self
                    .builder()
                    .build_int_compare(IntPredicate::EQ, result, self.i32c(0), "item_condition")
                    .map_err(|error| error.to_string())?;
                let true_block = self.codegen.context.append_basic_block(function, "item_true");
                let false_block = self.codegen.context.append_basic_block(function, "item_false");
                let done_block = self.codegen.context.append_basic_block(function, "item_join");
                self.builder()
                    .build_conditional_branch(matched, true_block, false_block)
                    .map_err(|error| error.to_string())?;

                self.builder().position_at_end(true_block);
                let true_view = self.build_node(when_true)?.view;
                self.builder()
                    .build_unconditional_branch(done_block)
                    .map_err(|error| error.to_string())?;

                self.builder().position_at_end(false_block);
                let false_view = match when_false {
                    Some(node) => self.build_node(node)?.view,
                    None => self.ptr_type().const_null(),
                };
                self.builder()
                    .build_unconditional_branch(done_block)
                    .map_err(|error| error.to_string())?;

                self.builder().position_at_end(done_block);
                let phi = self
                    .builder()
                    .build_phi(self.ptr_type(), "item_branch")
                    .map_err(|error| error.to_string())?;
                phi.add_incoming(&[(&true_view, true_block)]);
                phi.add_incoming(&[(&false_view, false_block)]);
                Ok(Built {
                    view: phi.as_basic_value().into_pointer_value(),
                    conditional: None,
                })
            }
            IRNode::ExprText { initial, style, .. } => Ok(Built {
                view: self.build_text(initial, style)?,
                conditional: None,
            }),
        }
    }

    /// 新建一个控件：背景（含圆角 / 描边）、内边距，并登记 id（渐显目标要用）。
    fn new_view_of(
        &self,
        class: &str,
        ctor: &str,
        style: &InlineStyle,
    ) -> Result<PointerValue<'ctx>, String> {
        let activity = self.activity()?;
        let view = self.new_object(class, ctor, &[JArg::Obj(activity)])?;
        self.apply_box(view, style)?;
        self.apply_padding(view, style)?;
        // 透明度：常量直接设，绑了单元就登记（单元变化时由 refresh 更新）
        if let Some(cell) = &style.opacity_cell {
            let initial = self
                .cell_index(cell)
                .map(|index| {
                    self.builder()
                        .build_load(
                            self.i64_type(),
                            self.numeric_global(index)
                                .map(|global| global.as_pointer_value())
                                .unwrap_or(self.ptr_type().const_null()),
                            "opacity_initial",
                        )
                })
                .transpose()
                .map_err(|error| error.to_string())?;
            let alpha = match initial {
                Some(value) => {
                    let raw = self
                        .builder()
                        .build_signed_int_to_float(value.into_int_value(), self.f32_type(), "opacity_raw")
                        .map_err(|error| error.to_string())?;
                    self.builder()
                        .build_float_div(raw, self.f32c(100.0), "opacity_ratio")
                        .map_err(|error| error.to_string())?
                }
                None => self.f32c(1.0),
            };
            self.call_void(view, "setAlpha", &[JArg::F32(alpha)])?;
        } else if let Some(alpha) = style.opacity_static {
            self.call_void(view, "setAlpha", &[JArg::F32(self.f32c(alpha as f32))])?;
        }
        if let Some(cell) = &style.opacity_cell {
            if let Some(index) = self.cell_index(cell) {
                let slot = self.opacity_globals.borrow()[index].len();
                let global = self.ptr_global(&format!("g_opacity_{index}_{slot}"));
                // 刷新函数在别的 native 调用里跑，这里必须存全局引用
                let reference = self.global_ref(view)?;
                self.builder()
                    .build_store(global.as_pointer_value(), reference)
                    .map_err(|error| error.to_string())?;
                self.opacity_globals.borrow_mut()[index].push(global);
            }
        }
        Ok(view)
    }

    fn apply_padding(&self, view: PointerValue<'ctx>, style: &InlineStyle) -> Result<(), String> {
        let padding = node_padding(style);
        if padding.is_zero() {
            return Ok(());
        }
        let left = self.dp(padding.left)?;
        let top = self.dp(padding.top)?;
        let right = self.dp(padding.right)?;
        let bottom = self.dp(padding.bottom)?;
        self.call_void(
            view,
            "setPadding",
            &[
                JArg::I32(left),
                JArg::I32(top),
                JArg::I32(right),
                JArg::I32(bottom),
            ],
        )
    }

    /// 背景：纯色走 setBackgroundColor，圆角 / 描边要用 GradientDrawable 当背景。
    fn apply_box(&self, view: PointerValue<'ctx>, style: &InlineStyle) -> Result<(), String> {
        let radius = style.border_radius.unwrap_or(0.0);
        let border = style.border_width.unwrap_or(0.0);
        let background = match &style.background_color {
            Some(value) => Some(color_argb(value)?),
            None => None,
        };
        if radius <= 0.0 && border <= 0.0 {
            if let Some(color) = background {
                self.call_void(view, "setBackgroundColor", &[JArg::I32(self.i32c(color))])?;
            }
            return Ok(());
        }
        let drawable = self.new_object("Drawable", "drawableCtor", &[])?;
        self.call_void(
            drawable,
            "setColor",
            &[JArg::I32(self.i32c(background.unwrap_or(0)))],
        )?;
        if radius > 0.0 {
            self.call_void(
                drawable,
                "setCornerRadius",
                &[JArg::F32(self.dp_f32(radius)?)],
            )?;
        }
        if border > 0.0 {
            let color = match &style.border_color {
                Some(value) => color_argb(value)?,
                None => 0xFF00_0000u32 as i32,
            };
            self.call_void(
                drawable,
                "setStroke",
                &[JArg::I32(self.dp(border)?), JArg::I32(self.i32c(color))],
            )?;
        }
        self.call_void(view, "setBackground", &[JArg::Obj(drawable)])
    }

    fn i64_global(&self, name: &str) -> GlobalValue<'ctx> {
        if let Some(global) = self.codegen.module.get_global(name) {
            return global;
        }
        let global = self
            .codegen
            .module
            .add_global(self.i64_type(), None, name);
        global.set_initializer(&self.i64_type().const_zero());
        global
    }

    fn ptr_global(&self, name: &str) -> GlobalValue<'ctx> {
        if let Some(global) = self.codegen.module.get_global(name) {
            return global;
        }
        let global = self.codegen.module.add_global(self.ptr_type(), None, name);
        global.set_initializer(&self.ptr_type().const_null());
        global
    }

    fn list_column(&self, id: &str, field: &str) -> GlobalValue<'ctx> {        let name = format!("g_list_{id}_col_{field}");
        if let Some(global) = self.codegen.module.get_global(&name) {
            return global;
        }
        let array = self.ptr_type().array_type(LIST_CAPACITY as u32);
        let global = self.codegen.module.add_global(array, None, &name);
        global.set_initializer(&array.const_zero());
        global
    }

    fn store_list_column(
        &self,
        id: &str,
        field: &str,
        index: IntValue<'ctx>,
        value: PointerValue<'ctx>,
    ) -> Result<(), String> {
        let global = self.list_column(id, field);
        let array = self.ptr_type().array_type(LIST_CAPACITY as u32);
        let slot = unsafe {
            self.builder().build_gep(
                array,
                global.as_pointer_value(),
                &[self.i64_type().const_zero(), index],
                "list_col_slot",
            )
        }
        .map_err(|error| error.to_string())?;
        self.builder()
            .build_store(slot, value)
            .map_err(|error| error.to_string())?;
        Ok(())
    }

    fn load_list_column(
        &self,
        id: &str,
        field: &str,
        index: IntValue<'ctx>,
    ) -> Result<PointerValue<'ctx>, String> {
        let global = self.list_column(id, field);
        let array = self.ptr_type().array_type(LIST_CAPACITY as u32);
        let slot = unsafe {
            self.builder().build_gep(
                array,
                global.as_pointer_value(),
                &[self.i64_type().const_zero(), index],
                "list_col_slot",
            )
        }
        .map_err(|error| error.to_string())?;
        Ok(self
            .builder()
            .build_load(self.ptr_type(), slot, "list_col_value")
            .map_err(|error| error.to_string())?
            .into_pointer_value())
    }

    /// 行表的数值列（父行下标 / 深度），和 iOS 的 parent、depth 列一一对应。
    /// 行视图缓存：`[LIST_CAPACITY x jobject]`（存 global ref，行复用靠它）。
    fn ptr_array_global(&self, name: &str) -> GlobalValue<'ctx> {
        if let Some(global) = self.codegen.module.get_global(name) {
            return global;
        }
        let array = self.ptr_type().array_type(LIST_CAPACITY as u32);
        let global = self.codegen.module.add_global(array, None, name);
        global.set_initializer(&array.const_zero());
        global
    }

    fn store_row_view(
        &self,
        id: &str,
        index: IntValue<'ctx>,
        value: PointerValue<'ctx>,
    ) -> Result<(), String> {
        let global = self.ptr_array_global(&format!("g_list_{id}_rows"));
        let array = self.ptr_type().array_type(LIST_CAPACITY as u32);
        let slot = unsafe {
            self.builder().build_gep(
                array,
                global.as_pointer_value(),
                &[self.i64_type().const_zero(), index],
                "row_view_slot",
            )
        }
        .map_err(|error| error.to_string())?;
        self.builder()
            .build_store(slot, value)
            .map_err(|error| error.to_string())?;
        Ok(())
    }

    fn load_row_view(&self, id: &str, index: IntValue<'ctx>) -> Result<PointerValue<'ctx>, String> {
        let global = self.ptr_array_global(&format!("g_list_{id}_rows"));
        let array = self.ptr_type().array_type(LIST_CAPACITY as u32);
        let slot = unsafe {
            self.builder().build_gep(
                array,
                global.as_pointer_value(),
                &[self.i64_type().const_zero(), index],
                "row_view_slot",
            )
        }
        .map_err(|error| error.to_string())?;
        Ok(self
            .builder()
            .build_load(self.ptr_type(), slot, "row_view")
            .map_err(|error| error.to_string())?
            .into_pointer_value())
    }

    fn delete_global_ref(&self, object: PointerValue<'ctx>) -> Result<(), String> {
        let env = self.env()?;
        let ptr_type = self.ptr_type();
        let params = [ptr_type.into(), ptr_type.into()];
        self.invoke(
            JNI_DELETE_GLOBAL_REF,
            None,
            &params,
            &[env.into(), object.into()],
        )?;
        Ok(())
    }

    fn list_number_column(&self, id: &str, column: &str) -> GlobalValue<'ctx> {
        let name = format!("g_list_{id}_{column}");
        if let Some(global) = self.codegen.module.get_global(&name) {
            return global;
        }
        let array = self.i64_type().array_type(LIST_CAPACITY as u32);
        let global = self.codegen.module.add_global(array, None, &name);
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
        let array = self.i64_type().array_type(LIST_CAPACITY as u32);
        let slot = unsafe {
            self.builder().build_gep(
                array,
                global.as_pointer_value(),
                &[self.i64_type().const_zero(), index],
                "list_num_slot",
            )
        }
        .map_err(|error| error.to_string())?;
        self.builder()
            .build_store(slot, value)
            .map_err(|error| error.to_string())?;
        Ok(())
    }

    fn load_list_number(
        &self,
        id: &str,
        column: &str,
        index: IntValue<'ctx>,
    ) -> Result<IntValue<'ctx>, String> {
        let global = self.list_number_column(id, column);
        let array = self.i64_type().array_type(LIST_CAPACITY as u32);
        let slot = unsafe {
            self.builder().build_gep(
                array,
                global.as_pointer_value(),
                &[self.i64_type().const_zero(), index],
                "list_num_slot",
            )
        }
        .map_err(|error| error.to_string())?;
        Ok(self
            .builder()
            .build_load(self.i64_type(), slot, "list_num_value")
            .map_err(|error| error.to_string())?
            .into_int_value())
    }

    fn list_slots_for(&self, list: &str) -> Result<Vec<AndroidListSlot<'ctx>>, String> {
        self.list_slots
            .borrow()
            .get(list)
            .cloned()
            .filter(|slots| !slots.is_empty())
            .ok_or_else(|| format!("list state '{list}' is not rendered by this App"))
    }

    fn sync_list_length(&self, list: &str, count: IntValue<'ctx>) -> Result<(), String> {
        let cell = format!("{list}.length");
        let Some(index) = self.cell_index(&cell) else {
            return Ok(());
        };
        if self.kinds[index] != CellKind::Number {
            return Ok(());
        }
        self.builder()
            .build_store(self.numeric_global(index)?.as_pointer_value(), count)
            .map_err(|error| error.to_string())?;
        let refresh = self.refresh_fns[index];
        self.builder()
            .build_call(refresh, &[], "")
            .map_err(|error| error.to_string())?;
        Ok(())
    }

    fn rebuild_lists(&self, slots: &[AndroidListSlot<'ctx>]) -> Result<(), String> {
        for slot in slots {
            self.builder()
                .build_call(slot.rebuild, &[], "")
                .map_err(|error| error.to_string())?;
        }
        Ok(())
    }

    /// 列表渲染：容器 + native 数据列 + 运行期建行。
    fn build_list(
        &self,
        style: &InlineStyle,
        list: &str,
        item: &IRNode,
        fields: &[(String, StateValue)],
        rows: &[Vec<StateValue>],
        parents: &[Option<usize>],
    ) -> Result<Built<'ctx>, String> {
        let index = self.list_counter.get();
        self.list_counter.set(index + 1);
        let id = format!("{list}_{index}");
        let container = self.new_linear(style, FlexDirection::Column)?;
        let container_global = self.ptr_global(&format!("g_list_{id}_container"));
        // 局部引用只在当前 native 调用里有效，跨调用重建必须先转成全局引用
        let container_ref = self.global_ref(container)?;
        self.builder()
            .build_store(container_global.as_pointer_value(), container_ref)
            .map_err(|error| error.to_string())?;
        let count_global = self.i64_global(&format!("g_list_{id}_count"));
        self.builder()
            .build_store(
                count_global.as_pointer_value(),
                self.i64_type().const_int(rows.len() as u64, false),
            )
            .map_err(|error| error.to_string())?;

        let mut columns = HashMap::new();
        let mut names = Vec::new();
        for (field, _) in fields {
            columns.insert(field.clone(), self.list_column(&id, field));
            names.push(field.clone());
        }
        // 行控件按字段序号分配 view id，rebind 时按这张表定位；行视图缓存用于复用。
        self.list_fields.borrow_mut().insert(id.clone(), names.clone());
        // 行视图缓存与已渲染行数（由重建函数维护，这里先建好带初始值的全局）
        let _ = self.ptr_array_global(&format!("g_list_{id}_rows"));
        let _ = self.i64_global(&format!("g_list_{id}_rendered"));
        for (row, values) in rows.iter().enumerate() {
            for (field, value) in names.iter().zip(values) {
                let raw = self.cstr(&value.display_value())?;
                self.store_list_column(&id, field, self.i64_type().const_int(row as u64, false), raw)?;
            }
        }
        // 父子关系与深度：递归组件按这张表在运行期展开子树。
        for (row, parent) in parents.iter().enumerate() {
            let parent_value = parent.map_or(-1, |parent| parent as i64);
            self.store_list_number(
                &id,
                "parent",
                self.i64_type().const_int(row as u64, false),
                self.i64_type().const_int(parent_value as u64, true),
            )?;
            self.store_list_number(
                &id,
                "depth",
                self.i64_type().const_int(row as u64, false),
                self.i64_type().const_int(depth_of(parents, row) as u64, false),
            )?;
        }

        let recursive = row_template_recurses(item);
        let nested = parents.iter().any(Option::is_some);
        // 简单行模板走 Java 侧建行（每行 2 次 JNI）；其余形态回退到 native 建行。
        let spec = if recursive {
            None
        } else {
            row_spec(item, &names)
        };
        if let Some(spec) = &spec {
            let spec_c = self.cstr(spec)?;
            let spec_js = self.new_string(spec_c)?;
            let parsed =
                self.call_static_object("LumenRow", "lumenRowParse", &[JArg::Obj(spec_js)])?;
            let parsed_ref = self.global_ref(parsed)?;
            let global = self.ptr_global(&format!("g_list_{id}_row_template"));
            self.builder()
                .build_store(global.as_pointer_value(), parsed_ref)
                .map_err(|error| error.to_string())?;
        }
        let rebuild = if spec.is_some() {
            self.build_list_rebuild_java(&id, &names, nested)?
        } else if recursive {
            let body = strip_recursive_slots(item);
            self.build_list_rebuild_recursive(&id, &body)?
        } else {
            self.build_list_rebuild(&id, item, nested)?
        };
        self.builder()
            .build_call(rebuild, &[], "")
            .map_err(|error| error.to_string())?;
        self.list_slots
            .borrow_mut()
            .entry(list.to_string())
            .or_default()
            .push(AndroidListSlot {
                id,
                count_global,
                container_global,
                columns,
                rebuild,
                conditionals: row_conditional_fields(item).len(),
            });
        Ok(Built {
            view: container,
            conditional: None,
        })
    }

    /// Java 侧建行的快路径：每行只做「拼字段文本 + 一次静态调用」。
    fn build_list_rebuild_java(
        &self,
        id: &str,
        fields: &[String],
        indent_by_depth: bool,
    ) -> Result<FunctionValue<'ctx>, String> {
        let function = self.codegen.module.add_function(
            &format!("list_rebuild_{id}"),
            self.void_type().fn_type(&[], false),
            None,
        );
        let saved_block = self.builder().get_insert_block();
        let saved_jvalues = self.jvalues.get();
        self.begin_function(function)?;
        let env = self.env()?;
        let ptr_type = self.ptr_type();
        let params = [ptr_type.into(), self.i32_type().into()];
        self.invoke(
            JNI_PUSH_LOCAL_FRAME,
            Some(self.i32_type().into()),
            &params,
            &[env.into(), self.i32c(LOCAL_FRAME_CAPACITY).into()],
        )?;
        let i64_type = self.i64_type();
        let activity = self.activity()?;
        let container = self
            .builder()
            .build_load(
                ptr_type,
                self.ptr_global(&format!("g_list_{id}_container"))
                    .as_pointer_value(),
                "list_container",
            )
            .map_err(|error| error.to_string())?
            .into_pointer_value();
        let template = self
            .builder()
            .build_load(
                ptr_type,
                self.ptr_global(&format!("g_list_{id}_row_template"))
                    .as_pointer_value(),
                "row_template",
            )
            .map_err(|error| error.to_string())?
            .into_pointer_value();
        let count = self
            .builder()
            .build_load(
                i64_type,
                self.i64_global(&format!("g_list_{id}_count"))
                    .as_pointer_value(),
                "list_count",
            )
            .map_err(|error| error.to_string())?
            .into_int_value();
        // 一行所有字段值拼进一块栈缓冲，避免每个字段一次 JNI
        let buffer = self
            .builder()
            .build_array_alloca(
                self.codegen.context.i8_type(),
                self.i32c(1024),
                "row_values",
            )
            .map_err(|error| error.to_string())?;
        let separator = self.cstr("\u{1}")?;
        // 这一轮复用从第 0 行开始
        self.call_static_void("LumenRow", "lumenRowBegin", &[JArg::Obj(container)])?;
        let cursor = self
            .builder()
            .build_alloca(i64_type, "list_cursor")
            .map_err(|error| error.to_string())?;
        self.builder()
            .build_store(cursor, i64_type.const_zero())
            .map_err(|error| error.to_string())?;
        let loop_block = self.codegen.context.append_basic_block(function, "list_loop");
        let body = self.codegen.context.append_basic_block(function, "list_body");
        let done = self.codegen.context.append_basic_block(function, "list_done");
        self.builder()
            .build_unconditional_branch(loop_block)
            .map_err(|error| error.to_string())?;

        self.builder().position_at_end(loop_block);
        let current = self
            .builder()
            .build_load(i64_type, cursor, "list_cursor_value")
            .map_err(|error| error.to_string())?
            .into_int_value();
        let more = self
            .builder()
            .build_int_compare(IntPredicate::SLT, current, count, "list_has_more")
            .map_err(|error| error.to_string())?;
        self.builder()
            .build_conditional_branch(more, body, done)
            .map_err(|error| error.to_string())?;

        self.builder().position_at_end(body);
        for (position, field) in fields.iter().enumerate() {
            let cell = self.load_list_column(id, field, current)?;
            let function_value = if position == 0 {
                self.strcpy()?
            } else {
                self.builder()
                    .build_call(self.strcat()?, &[buffer.into(), separator.into()], "")
                    .map_err(|error| error.to_string())?;
                self.strcat()?
            };
            self.builder()
                .build_call(function_value, &[buffer.into(), cell.into()], "")
                .map_err(|error| error.to_string())?;
        }
        let values = self.new_string(buffer)?;
        let row_index = self
            .builder()
            .build_int_truncate(current, self.i32_type(), "row_index")
            .map_err(|error| error.to_string())?;
        let indent = if indent_by_depth {
            let depth = self.load_list_number(id, "depth", current)?;
            let depth_i32 = self
                .builder()
                .build_int_truncate(depth, self.i32_type(), "row_depth")
                .map_err(|error| error.to_string())?;
            self.builder()
                .build_int_mul(self.dp(16.0)?, depth_i32, "row_indent")
                .map_err(|error| error.to_string())?
        } else {
            self.i32c(0)
        };
        self.call_static_void(
            "LumenRow",
            "lumenRowBuild",
            &[
                JArg::Obj(activity),
                JArg::Obj(template),
                JArg::Obj(container),
                JArg::Obj(values),
                JArg::I32(row_index),
                JArg::I32(indent),
            ],
        )?;
        let advanced = self
            .builder()
            .build_int_add(current, i64_type.const_int(1, false), "list_cursor_next")
            .map_err(|error| error.to_string())?;
        self.builder()
            .build_store(cursor, advanced)
            .map_err(|error| error.to_string())?;
        self.builder()
            .build_unconditional_branch(loop_block)
            .map_err(|error| error.to_string())?;

        self.builder().position_at_end(done);
        let env = self.env()?;
        let params = [ptr_type.into(), ptr_type.into()];
        self.invoke(
            JNI_POP_LOCAL_FRAME,
            Some(ptr_type.into()),
            &params,
            &[env.into(), ptr_type.const_null().into()],
        )?;
        self.builder()
            .build_return(None)
            .map_err(|error| error.to_string())?;
        self.jvalues.set(saved_jvalues);
        if let Some(saved) = saved_block {
            self.builder().position_at_end(saved);
        }
        Ok(function)
    }

    fn build_list_rebuild(
        &self,
        id: &str,
        item: &IRNode,
        indent_by_depth: bool,
    ) -> Result<FunctionValue<'ctx>, String> {
        let function = self.codegen.module.add_function(
            &format!("list_rebuild_{id}"),
            self.void_type().fn_type(&[], false),
            None,
        );
        let saved_block = self.builder().get_insert_block();
        let saved_jvalues = self.jvalues.get();
        self.begin_function(function)?;
        // 新建行会产生局部引用，整段包进局部引用帧，避免累积到引用表上限。
        let env = self.env()?;
        let ptr_type = self.ptr_type();
        let params = [ptr_type.into(), self.i32_type().into()];
        self.invoke(
            JNI_PUSH_LOCAL_FRAME,
            Some(self.i32_type().into()),
            &params,
            &[env.into(), self.i32c(LOCAL_FRAME_CAPACITY).into()],
        )?;
        let rebind = self.build_list_rebind(id, item)?;
        let conditionals = row_conditional_fields(item);
        let i64_type = self.i64_type();
        let container_global = self.ptr_global(&format!("g_list_{id}_container"));
        let count_global = self.i64_global(&format!("g_list_{id}_count"));
        let rendered_global = self.i64_global(&format!("g_list_{id}_rendered"));
        let container = self
            .builder()
            .build_load(self.ptr_type(), container_global.as_pointer_value(), "list_container")
            .map_err(|error| error.to_string())?
            .into_pointer_value();
        let count = self
            .builder()
            .build_load(i64_type, count_global.as_pointer_value(), "list_count")
            .map_err(|error| error.to_string())?
            .into_int_value();
        let rendered = if ROW_REUSE {
            self.builder()
                .build_load(i64_type, rendered_global.as_pointer_value(), "list_rendered")
                .map_err(|error| error.to_string())?
                .into_int_value()
        } else {
            // 关掉复用：先把视图全摘掉，等价于原来的「全清重建」
            self.call_void(container, "removeAllViews", &[])?;
            i64_type.const_zero()
        };
        let cursor = self
            .builder()
            .build_alloca(i64_type, "list_cursor")
            .map_err(|error| error.to_string())?;
        self.builder()
            .build_store(cursor, i64_type.const_zero())
            .map_err(|error| error.to_string())?;
        let loop_block = self.codegen.context.append_basic_block(function, "list_loop");
        let body = self.codegen.context.append_basic_block(function, "list_body");
        let check_block = self
            .codegen
            .context
            .append_basic_block(function, "list_check_cond");
        let drop_block = self
            .codegen
            .context
            .append_basic_block(function, "list_drop_row");
        let build_block = self
            .codegen
            .context
            .append_basic_block(function, "list_build_row");
        let reuse_block = self
            .codegen
            .context
            .append_basic_block(function, "list_reuse_row");
        let next = self.codegen.context.append_basic_block(function, "list_next");
        let trim_head = self
            .codegen
            .context
            .append_basic_block(function, "list_trim_head");
        let trim_body = self
            .codegen
            .context
            .append_basic_block(function, "list_trim_body");
        let trim_last = self
            .codegen
            .context
            .append_basic_block(function, "list_trim_row");
        let done = self.codegen.context.append_basic_block(function, "list_done");
        self.builder()
            .build_unconditional_branch(loop_block)
            .map_err(|error| error.to_string())?;

        self.builder().position_at_end(loop_block);
        let current = self
            .builder()
            .build_load(i64_type, cursor, "list_cursor_value")
            .map_err(|error| error.to_string())?
            .into_int_value();
        let more = self
            .builder()
            .build_int_compare(IntPredicate::SLT, current, count, "list_has_more")
            .map_err(|error| error.to_string())?;
        self.builder()
            .build_conditional_branch(more, body, trim_head)
            .map_err(|error| error.to_string())?;

        // 位置复用：这一行已经有视图，先看条件字段有没有变，没变就只 rebind。
        self.builder().position_at_end(body);
        let exists = if ROW_REUSE {
            self.builder()
                .build_int_compare(IntPredicate::SLT, current, rendered, "row_exists")
                .map_err(|error| error.to_string())?
        } else {
            self.builder()
                .build_int_compare(
                    IntPredicate::EQ,
                    i64_type.const_zero(),
                    i64_type.const_int(1, false),
                    "row_exists",
                )
                .map_err(|error| error.to_string())?
        };
        self.builder()
            .build_conditional_branch(exists, check_block, build_block)
            .map_err(|error| error.to_string())?;

        self.builder().position_at_end(check_block);
        if conditionals.is_empty() {
            self.builder()
                .build_unconditional_branch(reuse_block)
                .map_err(|error| error.to_string())?;
        } else {
            let mut changed = self
                .builder()
                .build_int_compare(
                    IntPredicate::EQ,
                    i64_type.const_zero(),
                    i64_type.const_zero(),
                    "cond_same",
                )
                .map_err(|error| error.to_string())?;
            for (position, field) in conditionals.iter().enumerate() {
                let previous = self.load_list_number(id, &format!("cond_{position}"), current)?;
                let next_flag = self.load_conditional_flag(id, field, current)?;
                let differs = self
                    .builder()
                    .build_int_compare(IntPredicate::NE, previous, next_flag, "cond_differs")
                    .map_err(|error| error.to_string())?;
                changed = self
                    .builder()
                    .build_or(changed, differs, "cond_changed")
                    .map_err(|error| error.to_string())?;
            }
            self.builder()
                .build_conditional_branch(changed, drop_block, reuse_block)
                .map_err(|error| error.to_string())?;
        }

        // 条件分支换了：这一行整行重建（先摘掉旧视图、放掉它的全局引用）。
        self.builder().position_at_end(drop_block);
        let stale = self.load_row_view(id, current)?;
        self.call_void(container, "removeView", &[JArg::Obj(stale)])?;
        self.delete_global_ref(stale)?;
        self.store_row_view(id, current, ptr_type.const_null())?;
        self.builder()
            .build_unconditional_branch(build_block)
            .map_err(|error| error.to_string())?;

        self.builder().position_at_end(build_block);
        *self.list_row_ctx.borrow_mut() = Some(AndroidRowCtx {
            id: id.to_string(),
            index: current,
        });
        let row = self.build_node(item)?.view;
        *self.list_row_ctx.borrow_mut() = None;
        let row_ref = self.global_ref(row)?;
        self.store_row_view(id, current, row_ref)?;
        self.apply_row_layout_params(id, item, row, current, indent_by_depth)?;
        self.call_void(container, "addView", &[JArg::Obj(row)])?;
        for (position, field) in conditionals.iter().enumerate() {
            let flag = self.load_conditional_flag(id, field, current)?;
            self.store_list_number(id, &format!("cond_{position}"), current, flag)?;
        }
        self.builder()
            .build_unconditional_branch(next)
            .map_err(|error| error.to_string())?;

        self.builder().position_at_end(reuse_block);
        let reused = self.load_row_view(id, current)?;
        self.builder()
            .build_call(rebind, &[reused.into(), current.into()], "")
            .map_err(|error| error.to_string())?;
        self.apply_row_layout_params(id, item, reused, current, indent_by_depth)?;
        self.builder()
            .build_unconditional_branch(next)
            .map_err(|error| error.to_string())?;

        self.builder().position_at_end(next);
        let advanced = self
            .builder()
            .build_int_add(current, i64_type.const_int(1, false), "list_cursor_next")
            .map_err(|error| error.to_string())?;
        self.builder()
            .build_store(cursor, advanced)
            .map_err(|error| error.to_string())?;
        self.builder()
            .build_unconditional_branch(loop_block)
            .map_err(|error| error.to_string())?;

        // 行数变少：把多出来的视图摘掉，并放掉它们的全局引用。
        self.builder().position_at_end(trim_head);
        self.builder()
            .build_unconditional_branch(trim_body)
            .map_err(|error| error.to_string())?;
        self.builder().position_at_end(trim_body);
        let trim_index = self
            .builder()
            .build_load(i64_type, cursor, "list_trim_value")
            .map_err(|error| error.to_string())?
            .into_int_value();
        let has_extra = self
            .builder()
            .build_int_compare(IntPredicate::SLT, trim_index, rendered, "list_has_extra")
            .map_err(|error| error.to_string())?;
        self.builder()
            .build_conditional_branch(has_extra, trim_last, done)
            .map_err(|error| error.to_string())?;
        self.builder().position_at_end(trim_last);
        let extra = self.load_row_view(id, trim_index)?;
        self.call_void(container, "removeView", &[JArg::Obj(extra)])?;
        self.delete_global_ref(extra)?;
        self.store_row_view(id, trim_index, ptr_type.const_null())?;
        let trimmed = self
            .builder()
            .build_int_add(trim_index, i64_type.const_int(1, false), "list_trim_next")
            .map_err(|error| error.to_string())?;
        self.builder()
            .build_store(cursor, trimmed)
            .map_err(|error| error.to_string())?;
        self.builder()
            .build_unconditional_branch(trim_body)
            .map_err(|error| error.to_string())?;

        self.builder().position_at_end(done);
        self.builder()
            .build_store(rendered_global.as_pointer_value(), count)
            .map_err(|error| error.to_string())?;
        let env = self.env()?;
        // jobject PopLocalFrame(JNIEnv*, jobject result)
        let params = [ptr_type.into(), ptr_type.into()];
        self.invoke(
            JNI_POP_LOCAL_FRAME,
            Some(ptr_type.into()),
            &params,
            &[env.into(), ptr_type.const_null().into()],
        )?;
        self.builder()
            .build_return(None)
            .map_err(|error| error.to_string())?;
        self.jvalues.set(saved_jvalues);
        if let Some(saved) = saved_block {
            self.builder().position_at_end(saved);
        }
        Ok(function)
    }

    /// 行参数：模板自己的尺寸（`height: 52` 这类）加上按深度的缩进。
    fn apply_row_layout_params(
        &self,
        id: &str,
        item: &IRNode,
        view: PointerValue<'ctx>,
        index: IntValue<'ctx>,
        indent_by_depth: bool,
    ) -> Result<(), String> {
        let mut params =
            self.linear_params(item, FlexDirection::Column, AlignItems::Stretch, 0.0, true)?;
        params.weight = None;
        if indent_by_depth {
            let depth = self.load_list_number(id, "depth", index)?;
            let depth_dp = self
                .builder()
                .build_int_mul(
                    self.builder()
                        .build_int_truncate(depth, self.i32_type(), "row_depth")
                        .map_err(|error| error.to_string())?,
                    self.dp(16.0)?,
                    "row_indent",
                )
                .map_err(|error| error.to_string())?;
            params.margins[0] = self
                .builder()
                .build_int_add(params.margins[0], depth_dp, "row_left")
                .map_err(|error| error.to_string())?;
        }
        let params = self.linear_params_object(&params)?;
        self.call_void(view, "setLayoutParams", &[JArg::Obj(params)])
    }

    /// 行 rebind：只改文字和点击 id，不重建控件。
    fn build_list_rebind(&self, id: &str, item: &IRNode) -> Result<FunctionValue<'ctx>, String> {
        let fields = self
            .list_fields
            .borrow()
            .get(id)
            .cloned()
            .unwrap_or_default();
        let actions = row_action_indices(item);
        let ptr_type = self.ptr_type();
        let function = self.codegen.module.add_function(
            &format!("list_rebind_{id}"),
            self.void_type()
                .fn_type(&[ptr_type.into(), self.i64_type().into()], false),
            None,
        );
        let saved_block = self.builder().get_insert_block();
        let saved_jvalues = self.jvalues.get();
        self.begin_function(function)?;
        let env = self.env()?;
        let params = [ptr_type.into(), self.i32_type().into()];
        self.invoke(
            JNI_PUSH_LOCAL_FRAME,
            Some(self.i32_type().into()),
            &params,
            &[env.into(), self.i32c(LOCAL_FRAME_CAPACITY).into()],
        )?;
        let row = function
            .get_nth_param(0)
            .ok_or("rebind row parameter is missing")?
            .into_pointer_value();
        let index = function
            .get_nth_param(1)
            .ok_or("rebind index parameter is missing")?
            .into_int_value();

        for (position, field) in fields.iter().enumerate() {
            let target = self.call_object(
                row,
                "findViewById",
                &[JArg::I32(self.i32c(ROW_LABEL_ID_BASE + position as i32))],
            )?;
            let is_null = self
                .builder()
                .build_is_null(target, "label_missing")
                .map_err(|error| error.to_string())?;
            let skip = self
                .codegen
                .context
                .append_basic_block(function, "rebind_label_skip");
            let apply = self
                .codegen
                .context
                .append_basic_block(function, "rebind_label");
            self.builder()
                .build_conditional_branch(is_null, skip, apply)
                .map_err(|error| error.to_string())?;
            self.builder().position_at_end(apply);
            let value = self.load_list_column(id, field, index)?;
            let text = self.new_string(value)?;
            self.call_void(target, "setText", &[JArg::Obj(text)])?;
            self.builder()
                .build_unconditional_branch(skip)
                .map_err(|error| error.to_string())?;
            self.builder().position_at_end(skip);
        }

        for action in actions {
            let target = self.call_object(
                row,
                "findViewById",
                &[JArg::I32(self.i32c(ROW_ACTION_ID_BASE + action as i32))],
            )?;
            let is_null = self
                .builder()
                .build_is_null(target, "action_missing")
                .map_err(|error| error.to_string())?;
            let skip = self
                .codegen
                .context
                .append_basic_block(function, "rebind_action_skip");
            let apply = self
                .codegen
                .context
                .append_basic_block(function, "rebind_action");
            self.builder()
                .build_conditional_branch(is_null, skip, apply)
                .map_err(|error| error.to_string())?;
            self.builder().position_at_end(apply);
            // 点击 id = 1_000_000 + action * 1024 + 行号，行位置变了必须重算
            let row_index = self
                .builder()
                .build_int_truncate(index, self.i32_type(), "row_index")
                .map_err(|error| error.to_string())?;
            let with_action = self
                .builder()
                .build_int_add(
                    self.i32c((action as i32) * 1024),
                    row_index,
                    "row_action_id",
                )
                .map_err(|error| error.to_string())?;
            let id_value = self
                .builder()
                .build_int_add(self.i32c(1_000_000), with_action, "row_click_id")
                .map_err(|error| error.to_string())?;
            let bridge = self.new_object("ClickBridge", "clickBridgeCtor", &[JArg::I32(id_value)])?;
            self.call_void(target, "setOnClickListener", &[JArg::Obj(bridge)])?;
            self.builder()
                .build_unconditional_branch(skip)
                .map_err(|error| error.to_string())?;
            self.builder().position_at_end(skip);
        }

        let env = self.env()?;
        let params = [ptr_type.into(), ptr_type.into()];
        self.invoke(
            JNI_POP_LOCAL_FRAME,
            Some(ptr_type.into()),
            &params,
            &[env.into(), ptr_type.const_null().into()],
        )?;
        self.builder()
            .build_return(None)
            .map_err(|error| error.to_string())?;
        self.jvalues.set(saved_jvalues);
        if let Some(saved) = saved_block {
            self.builder().position_at_end(saved);
        }
        Ok(function)
    }

    /// 列里的布尔字段值换算成 0/1，用于判断条件分支是否需要重建。
    fn load_conditional_flag(
        &self,
        id: &str,
        field: &str,
        index: IntValue<'ctx>,
    ) -> Result<IntValue<'ctx>, String> {
        let value = self.load_list_column(id, field, index)?;
        let expected = self.cstr("true")?;
        let compared = self
            .builder()
            .build_call(self.strcmp()?, &[value.into(), expected.into()], "cond_cmp")
            .map_err(|error| error.to_string())?;
        let compared = match compared.try_as_basic_value() {
            inkwell::values::ValueKind::Basic(value) => value.into_int_value(),
            inkwell::values::ValueKind::Instruction(_) => return Err("strcmp returned void".into()),
        };
        let is_true = self
            .builder()
            .build_int_compare(IntPredicate::EQ, compared, self.i32c(0), "cond_is_true")
            .map_err(|error| error.to_string())?;
        self.builder()
            .build_int_z_extend(is_true, self.i64_type(), "cond_flag")
            .map_err(|error| error.to_string())
    }

    /// 递归组件的重建：只从根层起手，子树由行函数自己递归建出来。
    fn build_list_rebuild_recursive(
        &self,
        id: &str,
        body: &IRNode,
    ) -> Result<FunctionValue<'ctx>, String> {
        let function = self.codegen.module.add_function(
            &format!("list_rebuild_{id}"),
            self.void_type().fn_type(&[], false),
            None,
        );
        let saved_block = self.builder().get_insert_block();
        let saved_jvalues = self.jvalues.get();
        self.begin_function(function)?;
        let env = self.env()?;
        let ptr_type = self.ptr_type();
        let params = [ptr_type.into(), self.i32_type().into()];
        self.invoke(
            JNI_PUSH_LOCAL_FRAME,
            Some(self.i32_type().into()),
            &params,
            &[env.into(), self.i32c(LOCAL_FRAME_CAPACITY).into()],
        )?;
        let row_fn = self.build_recursive_row_fn(id, body)?;
        let container_global = self.ptr_global(&format!("g_list_{id}_container"));
        let count_global = self.i64_global(&format!("g_list_{id}_count"));
        let container = self
            .builder()
            .build_load(
                self.ptr_type(),
                container_global.as_pointer_value(),
                "list_container",
            )
            .map_err(|error| error.to_string())?
            .into_pointer_value();
        self.call_void(container, "removeAllViews", &[])?;
        let count = self
            .builder()
            .build_load(self.i64_type(), count_global.as_pointer_value(), "list_count")
            .map_err(|error| error.to_string())?
            .into_int_value();
        let cursor = self
            .builder()
            .build_alloca(self.i64_type(), "list_cursor")
            .map_err(|error| error.to_string())?;
        self.builder()
            .build_store(cursor, self.i64_type().const_zero())
            .map_err(|error| error.to_string())?;
        let loop_block = self.codegen.context.append_basic_block(function, "list_loop");
        let body_block = self.codegen.context.append_basic_block(function, "list_body");
        let root_block = self.codegen.context.append_basic_block(function, "list_root");
        let next_block = self.codegen.context.append_basic_block(function, "list_next");
        let done = self.codegen.context.append_basic_block(function, "list_done");
        self.builder()
            .build_unconditional_branch(loop_block)
            .map_err(|error| error.to_string())?;

        self.builder().position_at_end(loop_block);
        let current = self
            .builder()
            .build_load(self.i64_type(), cursor, "list_cursor_value")
            .map_err(|error| error.to_string())?
            .into_int_value();
        let more = self
            .builder()
            .build_int_compare(IntPredicate::SLT, current, count, "list_has_more")
            .map_err(|error| error.to_string())?;
        self.builder()
            .build_conditional_branch(more, body_block, done)
            .map_err(|error| error.to_string())?;

        // 只把根层交给行函数，子行由行函数按 parent 列递归挂载。
        self.builder().position_at_end(body_block);
        let parent = self.load_list_number(id, "parent", current)?;
        let is_root = self
            .builder()
            .build_int_compare(
                IntPredicate::EQ,
                parent,
                self.i64_type().const_int(u64::MAX, true),
                "list_is_root",
            )
            .map_err(|error| error.to_string())?;
        self.builder()
            .build_conditional_branch(is_root, root_block, next_block)
            .map_err(|error| error.to_string())?;

        self.builder().position_at_end(root_block);
        self.builder()
            .build_call(
                row_fn,
                &[
                    container.into(),
                    current.into(),
                    self.i64_type().const_zero().into(),
                ],
                "",
            )
            .map_err(|error| error.to_string())?;
        self.builder()
            .build_unconditional_branch(next_block)
            .map_err(|error| error.to_string())?;

        self.builder().position_at_end(next_block);
        let advanced = self
            .builder()
            .build_int_add(current, self.i64_type().const_int(1, false), "list_cursor_next")
            .map_err(|error| error.to_string())?;
        self.builder()
            .build_store(cursor, advanced)
            .map_err(|error| error.to_string())?;
        self.builder()
            .build_unconditional_branch(loop_block)
            .map_err(|error| error.to_string())?;

        self.builder().position_at_end(done);
        let env = self.env()?;
        let params = [ptr_type.into(), ptr_type.into()];
        self.invoke(
            JNI_POP_LOCAL_FRAME,
            Some(ptr_type.into()),
            &params,
            &[env.into(), ptr_type.const_null().into()],
        )?;
        self.builder()
            .build_return(None)
            .map_err(|error| error.to_string())?;
        self.jvalues.set(saved_jvalues);
        if let Some(saved) = saved_block {
            self.builder().position_at_end(saved);
        }
        Ok(function)
    }

    /// 递归组件行：`list_row_<id>(parent_container, index, depth)`，
    /// 建完行体后把「父行 = 当前行」的子行递归挂进来。
    fn build_recursive_row_fn(
        &self,
        id: &str,
        body: &IRNode,
    ) -> Result<FunctionValue<'ctx>, String> {
        let function = self.codegen.module.add_function(
            &format!("list_row_{id}"),
            self.void_type().fn_type(
                &[
                    self.ptr_type().into(),
                    self.i64_type().into(),
                    self.i64_type().into(),
                ],
                false,
            ),
            None,
        );
        let saved_block = self.builder().get_insert_block();
        let saved_jvalues = self.jvalues.get();
        self.begin_function(function)?;
        let env = self.env()?;
        let ptr_type = self.ptr_type();
        let params = [ptr_type.into(), self.i32_type().into()];
        self.invoke(
            JNI_PUSH_LOCAL_FRAME,
            Some(self.i32_type().into()),
            &params,
            &[env.into(), self.i32c(LOCAL_FRAME_CAPACITY).into()],
        )?;
        let parent_container = function
            .get_nth_param(0)
            .ok_or("recursive row parent parameter is missing")?
            .into_pointer_value();
        let index = function
            .get_nth_param(1)
            .ok_or("recursive row index parameter is missing")?
            .into_int_value();
        let depth = function
            .get_nth_param(2)
            .ok_or("recursive row depth parameter is missing")?
            .into_int_value();

        *self.list_row_ctx.borrow_mut() = Some(AndroidRowCtx {
            id: id.to_string(),
            index,
        });
        let row = self.build_node(body)?.view;
        *self.list_row_ctx.borrow_mut() = None;

        // 行槽位：行体在上，子行依次挂在下面。
        let slot = self.new_linear(&InlineStyle::default(), FlexDirection::Column)?;
        // 行体尺寸同样按模板样式来（`height: 40` 之类的行高要生效）。
        let mut params =
            self.linear_params(body, FlexDirection::Column, AlignItems::Stretch, 0.0, true)?;
        params.weight = None;
        let params = self.linear_params_object(&params)?;
        self.call_void(row, "setLayoutParams", &[JArg::Obj(params)])?;
        self.call_void(slot, "addView", &[JArg::Obj(row)])?;

        // 子行：父列等于本行下标的行，逐个递归建出来。
        let count = self
            .builder()
            .build_load(
                self.i64_type(),
                self.i64_global(&format!("g_list_{id}_count"))
                    .as_pointer_value(),
                "row_count",
            )
            .map_err(|error| error.to_string())?
            .into_int_value();
        let cursor = self
            .builder()
            .build_alloca(self.i64_type(), "child_cursor")
            .map_err(|error| error.to_string())?;
        self.builder()
            .build_store(cursor, self.i64_type().const_zero())
            .map_err(|error| error.to_string())?;
        let loop_block = self.codegen.context.append_basic_block(function, "child_loop");
        let match_block = self.codegen.context.append_basic_block(function, "child_match");
        let build_block = self.codegen.context.append_basic_block(function, "child_build");
        let next_block = self.codegen.context.append_basic_block(function, "child_next");
        let done = self.codegen.context.append_basic_block(function, "child_done");
        self.builder()
            .build_unconditional_branch(loop_block)
            .map_err(|error| error.to_string())?;

        self.builder().position_at_end(loop_block);
        let current = self
            .builder()
            .build_load(self.i64_type(), cursor, "child_index")
            .map_err(|error| error.to_string())?
            .into_int_value();
        let more = self
            .builder()
            .build_int_compare(IntPredicate::SLT, current, count, "child_has_more")
            .map_err(|error| error.to_string())?;
        self.builder()
            .build_conditional_branch(more, match_block, done)
            .map_err(|error| error.to_string())?;

        self.builder().position_at_end(match_block);
        let parent = self.load_list_number(id, "parent", current)?;
        let is_child = self
            .builder()
            .build_int_compare(IntPredicate::EQ, parent, index, "child_is_direct")
            .map_err(|error| error.to_string())?;
        self.builder()
            .build_conditional_branch(is_child, build_block, next_block)
            .map_err(|error| error.to_string())?;

        self.builder().position_at_end(build_block);
        let child_depth = self
            .builder()
            .build_int_add(depth, self.i64_type().const_int(1, false), "child_depth")
            .map_err(|error| error.to_string())?;
        self.builder()
            .build_call(
                function,
                &[slot.into(), current.into(), child_depth.into()],
                "",
            )
            .map_err(|error| error.to_string())?;
        self.builder()
            .build_unconditional_branch(next_block)
            .map_err(|error| error.to_string())?;

        self.builder().position_at_end(next_block);
        let advanced = self
            .builder()
            .build_int_add(current, self.i64_type().const_int(1, false), "child_next")
            .map_err(|error| error.to_string())?;
        self.builder()
            .build_store(cursor, advanced)
            .map_err(|error| error.to_string())?;
        self.builder()
            .build_unconditional_branch(loop_block)
            .map_err(|error| error.to_string())?;

        // 非根行整体右移 16dp，层级缩进靠嵌套累积。
        self.builder().position_at_end(done);
        let has_depth = self
            .builder()
            .build_int_compare(
                IntPredicate::SGT,
                depth,
                self.i64_type().const_zero(),
                "row_has_depth",
            )
            .map_err(|error| error.to_string())?;
        let indent = self
            .builder()
            .build_select(has_depth, self.dp(16.0)?, self.i32c(0), "row_indent")
            .map_err(|error| error.to_string())?
            .into_int_value();
        let slot_params = ChildParams {
            width: self.i32c(MATCH_PARENT),
            height: self.i32c(WRAP_CONTENT),
            weight: None,
            gravity: 0,
            margins: [indent, self.i32c(0), self.i32c(0), self.i32c(0)],
        };
        let slot_params = self.linear_params_object(&slot_params)?;
        self.call_void(slot, "setLayoutParams", &[JArg::Obj(slot_params)])?;
        self.call_void(parent_container, "addView", &[JArg::Obj(slot)])?;
        let env = self.env()?;
        let params = [ptr_type.into(), ptr_type.into()];
        self.invoke(
            JNI_POP_LOCAL_FRAME,
            Some(ptr_type.into()),
            &params,
            &[env.into(), ptr_type.const_null().into()],
        )?;
        self.builder()
            .build_return(None)
            .map_err(|error| error.to_string())?;
        self.jvalues.set(saved_jvalues);
        if let Some(saved) = saved_block {
            self.builder().position_at_end(saved);
        }
        Ok(function)
    }

    fn build_text(&self, content: &str, style: &InlineStyle) -> Result<PointerValue<'ctx>, String> {
        let view = self.new_view_of("Text", "textCtor", style)?;
        let text = self.text_string(content)?;
        self.call_void(view, "setText", &[JArg::Obj(text)])?;
        // UILabel 默认单行、超长省略；Android TextView 默认换行会把后续内容顶下去。
        self.call_void(view, "setSingleLine", &[JArg::I32(self.i32c(1))])?;
        self.call_void(view, "setIncludeFontPadding", &[JArg::I32(self.i32c(0))])?;
        // UILabel 默认单行 + 尾部省略
        let truncate = self.static_object_field("TruncateAt", "truncateEnd")?;
        self.call_void(view, "setEllipsize", &[JArg::Obj(truncate)])?;
        self.apply_text_style(
            view,
            style,
            GRAVITY_LEFT | GRAVITY_CENTER_VERTICAL,
            DEFAULT_FONT_SIZE,
        )?;
        Ok(view)
    }

    fn apply_text_style(
        &self,
        view: PointerValue<'ctx>,
        style: &InlineStyle,
        default_gravity: i32,
        default_size: f64,
    ) -> Result<(), String> {
        let size = style.font_size.unwrap_or(default_size);
        self.call_void(
            view,
            "setTextSize",
            &[
                JArg::I32(self.i32c(UNIT_DIP)),
                JArg::F32(self.f32c(size as f32)),
            ],
        )?;
        if let Some(value) = &style.color {
            let color = color_argb(value)?;
            self.call_void(view, "setTextColor", &[JArg::I32(self.i32c(color))])?;
        }
        let gravity = match style.text_align {
            Some(TextAlignment::Left) => GRAVITY_LEFT | GRAVITY_CENTER_VERTICAL,
            Some(TextAlignment::Center) => GRAVITY_CENTER,
            Some(TextAlignment::Right) => GRAVITY_RIGHT | GRAVITY_CENTER_VERTICAL,
            None => default_gravity,
        };
        self.call_void(view, "setTextGravity", &[JArg::I32(self.i32c(gravity))])
    }

    fn build_state_text(
        &self,
        cell: &str,
        initial: &StateValue,
        style: &InlineStyle,
    ) -> Result<Built<'ctx>, String> {
        let view = self.build_text(&initial.display_value(), style)?;
        let index = self
            .cell_index(cell)
            .ok_or_else(|| format!("unknown state cell '{cell}'"))?;
        let count = self.text_globals.borrow()[index].len();
        let name = format!("g_text_{index}_{count}");
        let global = self.module().add_global(self.ptr_type(), None, &name);
        global.set_initializer(&self.ptr_type().const_null());
        let reference = self.global_ref(view)?;
        self.builder()
            .build_store(global.as_pointer_value(), reference)
            .map_err(|error| error.to_string())?;
        self.text_globals.borrow_mut()[index].push(global);
        Ok(Built {
            view,
            conditional: None,
        })
    }

    fn build_button(
        &self,
        title: &str,
        style: &InlineStyle,
        state_action: Option<usize>,
    ) -> Result<PointerValue<'ctx>, String> {
        let view = self.new_view_of("Button", "buttonCtor", style)?;
        // Material 主题会把按钮文案转成大写，这里保持和 iOS 一致。
        self.call_void(view, "setAllCaps", &[JArg::I32(self.i32c(0))])?;
        // UIButton 的标题从 contentEdgeInsets（默认 0）开始排；Android Button 自带
        // Material 的左右各 16dp 内边距，窄按钮（52x28 这类）扣完就没有内容区了，
        // 再叠加单行 + 中间省略，文案会被压成一条缝。这里显式清掉默认内边距。
        if node_padding(style).is_zero() {
            self.call_void(
                view,
                "setPadding",
                &[JArg::I32(self.i32c(0)); 4],
            )?;
        }
        let text = self.text_string(title)?;
        self.call_void(view, "setText", &[JArg::Obj(text)])?;
        self.call_void(view, "setSingleLine", &[JArg::I32(self.i32c(1))])?;
        self.call_void(view, "setIncludeFontPadding", &[JArg::I32(self.i32c(0))])?;
        self.apply_text_style(view, style, GRAVITY_CENTER, DEFAULT_BUTTON_FONT_SIZE)?;
        // UIKit 里 alloc/init 出来的 UIButton 是 custom 类型，标题默认白色；
        // Material 的 Button 默认用主题 onSurface（近黑）。没有显式颜色时统一成白色。
        if style.color.is_none() {
            self.call_void(
                view,
                "setTextColor",
                &[JArg::I32(self.i32c(0xFFFF_FFFFu32 as i32))],
            )?;
        }
        // UIButton 的标题是单行 + 中间省略
        let truncate = self.static_object_field("TruncateAt", "truncateMiddle")?;
        self.call_void(view, "setEllipsize", &[JArg::Obj(truncate)])?;
        // 行内按钮按 action 编号分配 view id，rebind 时用它定位并换上新的行号
        if let (Some(action), true) = (state_action, self.list_row_ctx.borrow().is_some()) {
            self.call_void(
                view,
                "setId",
                &[JArg::I32(
                    self.i32c(ROW_ACTION_ID_BASE + action as i32),
                )],
            )?;
        }
        if let Some(action) = state_action {
            let id_value = match self.list_row_ctx.borrow().clone() {
                Some(ctx) => {
                    // 行内按钮：id = 1_000_000 + action * 1024 + 行号
                    let row = self
                        .builder()
                        .build_int_truncate(ctx.index, self.i32_type(), "row_index")
                        .map_err(|error| error.to_string())?;
                    let with_action = self
                        .builder()
                        .build_int_add(
                            self.i32c((action as i32) * 1024),
                            row,
                            "row_action_id",
                        )
                        .map_err(|error| error.to_string())?;
                    self.builder()
                        .build_int_add(self.i32c(1_000_000), with_action, "row_click_id")
                        .map_err(|error| error.to_string())?
                }
                None => self.i32c(action as i32),
            };
            let bridge = self.new_object("ClickBridge", "clickBridgeCtor", &[JArg::I32(id_value)])?;
            self.call_void(view, "setOnClickListener", &[JArg::Obj(bridge)])?;
        }
        Ok(view)
    }

    fn build_input(
        &self,
        placeholder: &str,
        style: &InlineStyle,
        on_input: Option<usize>,
        on_focus: Option<usize>,
    ) -> Result<Built<'ctx>, String> {
        let view = self.new_view_of("Input", "inputCtor", style)?;
        if !placeholder.is_empty() {
            let hint = self.text_string(placeholder)?;
            self.call_void(view, "setHint", &[JArg::Obj(hint)])?;
            // 占位符颜色统一由样式控制（默认与 iOS / macOS 一致）
            let hint_color = color_argb(
                style
                    .placeholder_color
                    .as_deref()
                    .unwrap_or(DEFAULT_PLACEHOLDER_COLOR),
            )?;
            self.call_void(view, "setHintTextColor", &[JArg::I32(self.i32c(hint_color))])?;
        }
        self.apply_text_style(
            view,
            style,
            GRAVITY_LEFT | GRAVITY_CENTER_VERTICAL,
            DEFAULT_FONT_SIZE,
        )?;
        if let Some(action) = on_input {
            let bridge = self.new_object(
                "InputBridge",
                "inputBridgeCtor",
                &[JArg::I32(self.i32c(action as i32))],
            )?;
            self.call_void(view, "addTextChanged", &[JArg::Obj(bridge)])?;
        }
        if let Some(action) = on_focus {
            if !matches!(self.program.state_actions[action], StateMutation::Noop) {
                let bridge = self.new_object(
                    "FocusBridge",
                    "focusBridgeCtor",
                    &[JArg::I32(self.i32c(action as i32))],
                )?;
                self.call_void(view, "setOnFocusChangeListener", &[JArg::Obj(bridge)])?;
            }
        }
        Ok(Built {
            view,
            conditional: None,
        })
    }

    fn build_scroll(
        &self,
        style: &InlineStyle,
        children: &[IRNode],
    ) -> Result<Built<'ctx>, String> {
        let activity = self.activity()?;
        let scroll = self.new_object("Scroll", "scrollCtor", &[JArg::Obj(activity)])?;
        self.apply_box(scroll, style)?;

        let positioned: Vec<&IRNode> = children
            .iter()
            .filter(|child| is_positioned(child))
            .collect();
        let flow: Vec<&IRNode> = children
            .iter()
            .filter(|child| !is_positioned(child))
            .collect();

        // ScrollView 只能有一个直接子级：内容层包住所有子级，内边距跟着内容一起滚。
        // 有绝对定位子级时内容层用 FrameLayout，流式子级再套一层 LinearLayout。
        let (content, flow_parent) = if positioned.is_empty() {
            let content = self.new_object("Linear", "linearCtor", &[JArg::Obj(activity)])?;
            self.call_void(
                content,
                "setOrientation",
                &[JArg::I32(self.i32c(LINEAR_VERTICAL))],
            )?;
            self.apply_padding(content, style)?;
            (content, content)
        } else {
            let content = self.new_object("Frame", "frameCtor", &[JArg::Obj(activity)])?;
            self.allow_child_overflow(content)?;
            self.apply_padding(content, style)?;
            let linear = self.new_object("Linear", "linearCtor", &[JArg::Obj(activity)])?;
            self.call_void(
                linear,
                "setOrientation",
                &[JArg::I32(self.i32c(LINEAR_VERTICAL))],
            )?;
            let params = ChildParams {
                width: self.i32c(MATCH_PARENT),
                height: self.i32c(WRAP_CONTENT),
                weight: None,
                gravity: 0,
                margins: [self.i32c(0); 4],
            };
            let params = self.frame_params_object(&params)?;
            self.call_void(linear, "setLayoutParams", &[JArg::Obj(params)])?;
            self.call_void(content, "addView", &[JArg::Obj(linear)])?;
            (content, linear)
        };

        self.add_flow_children(
            flow_parent,
            style,
            &flow,
            FlexDirection::Column,
            style.spacing.unwrap_or(0.0),
        )?;
        for (index, child) in positioned.iter().enumerate() {
            let params = self.frame_params(child)?;
            let built = self.build_node(child)?;
            let params = self.frame_params_object(&params)?;
            self.call_void(built.view, "setLayoutParams", &[JArg::Obj(params)])?;
            self.call_view_at(content, built.view, index + 1)?;
            if let Some(pending) = built.conditional {
                self.finalize_conditional(pending)?;
            }
        }

        let params = ChildParams {
            width: self.i32c(MATCH_PARENT),
            height: self.i32c(WRAP_CONTENT),
            weight: None,
            gravity: 0,
            margins: [self.i32c(0); 4],
        };
        let params = self.linear_params_object(&params)?;
        self.call_void(content, "setLayoutParams", &[JArg::Obj(params)])?;
        self.call_void(scroll, "addView", &[JArg::Obj(content)])?;

        if style.scroll_action.is_some() {
            let bridge = self.new_object(
                "ScrollBridge",
                "scrollBridgeCtor",
                &[JArg::I32(self.i32c(0))],
            )?;
            self.call_void(scroll, "setOnScrollChange", &[JArg::Obj(bridge)])?;
        }
        Ok(Built {
            view: scroll,
            conditional: None,
        })
    }

    fn build_container(
        &self,
        style: &InlineStyle,
        children: &[IRNode],
        default_direction: FlexDirection,
        default_gap: f64,
    ) -> Result<Built<'ctx>, String> {
        let direction = style.flex_direction.unwrap_or(default_direction);
        let gap = style.spacing.unwrap_or(default_gap);
        let positioned: Vec<&IRNode> = children
            .iter()
            .filter(|child| is_positioned(child))
            .collect();
        let flow: Vec<&IRNode> = children
            .iter()
            .filter(|child| !is_positioned(child))
            .collect();

        if positioned.is_empty() {
            let linear = self.new_linear(style, direction)?;
            self.add_flow_children(linear, style, &flow, direction, gap)?;
            return Ok(Built {
                view: linear,
                conditional: None,
            });
        }

        // 有绝对定位子级：FrameLayout 托底，流式子级放进内部的 LinearLayout。
        let activity = self.activity()?;
        let frame = self.new_object("Frame", "frameCtor", &[JArg::Obj(activity)])?;
        self.allow_child_overflow(frame)?;
        self.apply_box(frame, style)?;
        self.apply_padding(frame, style)?;

        let linear = self.new_object("Linear", "linearCtor", &[JArg::Obj(activity)])?;
        self.call_void(
            linear,
            "setOrientation",
            &[JArg::I32(self.i32c(match direction {
                FlexDirection::Row => LINEAR_HORIZONTAL,
                FlexDirection::Column => LINEAR_VERTICAL,
            }))],
        )?;
        self.add_flow_children(linear, style, &flow, direction, gap)?;
        let flow_params = ChildParams {
            width: self.i32c(MATCH_PARENT),
            height: self.i32c(WRAP_CONTENT),
            weight: None,
            gravity: 0,
            margins: [self.i32c(0); 4],
        };
        let flow_params = self.linear_params_object(&flow_params)?;
        self.call_void(linear, "setLayoutParams", &[JArg::Obj(flow_params)])?;
        self.call_void(frame, "addView", &[JArg::Obj(linear)])?;

        for (index, child) in positioned.iter().enumerate() {
            let params = self.frame_params(child)?;
            let built = self.build_node(child)?;
            let params = self.frame_params_object(&params)?;
            self.call_void(built.view, "setLayoutParams", &[JArg::Obj(params)])?;
            // 绝对定位子级必须排在流式内容（索引 0）之后，否则会被白色卡片压住上边框
            self.call_view_at(frame, built.view, index + 1)?;
            if let Some(pending) = built.conditional {
                self.finalize_conditional(pending)?;
            }
        }

        Ok(Built {
            view: frame,
            conditional: None,
        })
    }

    /// UIKit 的 UIView 默认不裁剪子视图，Android 的容器默认裁到自身（含 padding）区域。
    /// 顶部栏这种「固定高度 + 大字号」的写法要靠溢出可见才能显示完整，这里对齐 UIKit。
    fn allow_child_overflow(&self, view: PointerValue<'ctx>) -> Result<(), String> {
        self.call_void(view, "setClipChildren", &[JArg::I32(self.i32c(0))])?;
        self.call_void(view, "setClipToPadding", &[JArg::I32(self.i32c(0))])
    }

    fn new_linear(
        &self,
        style: &InlineStyle,
        direction: FlexDirection,
    ) -> Result<PointerValue<'ctx>, String> {
        let view = self.new_view_of("Linear", "linearCtor", style)?;
        self.allow_child_overflow(view)?;
        self.call_void(
            view,
            "setOrientation",
            &[JArg::I32(self.i32c(match direction {
                FlexDirection::Row => LINEAR_HORIZONTAL,
                FlexDirection::Column => LINEAR_VERTICAL,
            }))],
        )?;
        Ok(view)
    }

    fn add_flow_children(
        &self,
        parent: PointerValue<'ctx>,
        style: &InlineStyle,
        children: &[&IRNode],
        direction: FlexDirection,
        gap: f64,
    ) -> Result<(), String> {
        let justify = style.justify_content.unwrap_or(JustifyContent::Start);
        let align = style.align_items.unwrap_or(AlignItems::Stretch);
        let vertical = direction == FlexDirection::Column;

        // 主轴对齐：space-* 用 spacer + weight 近似（容器主轴尺寸不定时会退化成 start）。
        let container_gravity = match justify {
            JustifyContent::Start | JustifyContent::SpaceBetween | JustifyContent::SpaceAround => 0,
            JustifyContent::Center => {
                if vertical {
                    GRAVITY_CENTER_VERTICAL
                } else {
                    GRAVITY_CENTER_HORIZONTAL
                }
            }
            JustifyContent::End => {
                if vertical {
                    GRAVITY_BOTTOM
                } else {
                    GRAVITY_RIGHT
                }
            }
        };
        if container_gravity != 0 {
            self.call_void(
                parent,
                "setLinearGravity",
                &[JArg::I32(self.i32c(container_gravity))],
            )?;
        }

        let spaced = matches!(
            justify,
            JustifyContent::SpaceBetween | JustifyContent::SpaceAround
        );
        let mut index = 0usize;
        if spaced && justify == JustifyContent::SpaceAround && !children.is_empty() {
            self.add_spacer(parent, direction, index)?;
            index += 1;
        }
        for (position, child) in children.iter().enumerate() {
            if spaced && position > 0 {
                self.add_spacer(parent, direction, index)?;
                index += 1;
            }
            let params = if matches!(child, IRNode::Conditional { .. }) {
                self.slot_params(child, direction, align, gap, position == 0)?
            } else {
                self.linear_params(child, direction, align, gap, position == 0)?
            };
            let built = self.build_node(child)?;
            let params = self.linear_params_object(&params)?;
            self.call_void(built.view, "setLayoutParams", &[JArg::Obj(params)])?;
            self.call_view_at(parent, built.view, index)?;
            if let Some(pending) = built.conditional {
                self.finalize_conditional(pending)?;
            }
            index += 1;
        }
        if spaced && justify == JustifyContent::SpaceAround && !children.is_empty() {
            self.add_spacer(parent, direction, index)?;
        }
        Ok(())
    }

    fn add_spacer(
        &self,
        parent: PointerValue<'ctx>,
        direction: FlexDirection,
        index: usize,
    ) -> Result<(), String> {
        let activity = self.activity()?;
        let spacer = self.new_object("View", "viewCtor", &[JArg::Obj(activity)])?;
        let params = match direction {
            FlexDirection::Column => ChildParams {
                width: self.i32c(MATCH_PARENT),
                height: self.i32c(0),
                weight: Some(self.f32c(1.0)),
                gravity: 0,
                margins: [self.i32c(0); 4],
            },
            FlexDirection::Row => ChildParams {
                width: self.i32c(0),
                height: self.i32c(0),
                weight: Some(self.f32c(1.0)),
                gravity: 0,
                margins: [self.i32c(0); 4],
            },
        };
        let params = self.linear_params_object(&params)?;
        self.call_void(spacer, "setLayoutParams", &[JArg::Obj(params)])?;
        self.call_view_at(parent, spacer, index)
    }

    fn call_view_at(
        &self,
        parent: PointerValue<'ctx>,
        child: PointerValue<'ctx>,
        index: usize,
    ) -> Result<(), String> {
        self.call_void(
            parent,
            "addViewAt",
            &[JArg::Obj(child), JArg::I32(self.i32c(index as i32))],
        )
    }

    fn linear_params(
        &self,
        node: &IRNode,
        direction: FlexDirection,
        align: AlignItems,
        gap: f64,
        first: bool,
    ) -> Result<ChildParams<'ctx>, String> {
        let style = node_style(node);
        let horizontal_main = direction == FlexDirection::Row;
        let stretch = align == AlignItems::Stretch;
        let flex = node_flex_grow(node);
        // 行方向下交叉轴是高度。UIKit 的 UILabel 不裁字形，拉伸到容器高度也没问题；
        // Android 的 TextView 会把自己的字形裁到测量高度，而 WRAP_CONTENT 又会被父容器的
        // AT_MOST 压到可用高度（顶部栏 44dp 减 12dp padding 只剩 20dp，装不下 22pt 字），
        // 所以标签直接按字号给一个明确高度，再用 gravity 居中，观感与 iOS 一致。
        let label_height = if horizontal_main
            && style.height.is_none()
            && matches!(node, IRNode::Text { .. } | IRNode::StateText { .. })
        {
            Some(
                style
                    .font_size
                    .unwrap_or(DEFAULT_FONT_SIZE)
                    * LABEL_LINE_HEIGHT,
            )
        } else {
            None
        };
        let wrap_cross = stretch && horizontal_main && node_uses_intrinsic_dimension(node, true);

        // 主轴上的百分比和 flexGrow 都交给 weight；交叉轴的百分比退化成 match_parent。
        let width = match style.width {
            Some(Dimension::Points(value)) => self.dp(value)?,
            Some(Dimension::Percent(fraction)) if fraction >= 0.999 => self.i32c(MATCH_PARENT),
            Some(Dimension::Percent(_)) if horizontal_main => self.i32c(0),
            Some(Dimension::Percent(_)) => self.i32c(MATCH_PARENT),
            None if horizontal_main && flex > 0.0 => self.i32c(0),
            None if horizontal_main => self.i32c(WRAP_CONTENT),
            None => self.i32c(if stretch { MATCH_PARENT } else { WRAP_CONTENT }),
        };
        let height = match style.height {
            Some(Dimension::Points(value)) => self.dp(value)?,
            Some(Dimension::Percent(fraction)) if fraction >= 0.999 => self.i32c(MATCH_PARENT),
            Some(Dimension::Percent(_)) if !horizontal_main => self.i32c(0),
            Some(Dimension::Percent(_)) => self.i32c(MATCH_PARENT),
            None if !horizontal_main && flex > 0.0 => self.i32c(0),
            None if !horizontal_main => self.i32c(WRAP_CONTENT),
            None if stretch && !wrap_cross => self.i32c(MATCH_PARENT),
            None => match label_height {
                Some(points) => self.dp(points)?,
                None => self.i32c(WRAP_CONTENT),
            },
        };
        let weight = if horizontal_main {
            match style.width {
                Some(Dimension::Percent(fraction)) if fraction < 0.999 => {
                    Some(self.f32c(fraction as f32))
                }
                None if flex > 0.0 => Some(self.f32c(flex as f32)),
                _ => None,
            }
        } else {
            match style.height {
                Some(Dimension::Percent(fraction)) if fraction < 0.999 => {
                    Some(self.f32c(fraction as f32))
                }
                None if flex > 0.0 => Some(self.f32c(flex as f32)),
                _ => None,
            }
        };

        let gravity = match (direction, align) {
            (_, AlignItems::Stretch) if wrap_cross || label_height.is_some() => GRAVITY_CENTER_VERTICAL,
            (_, AlignItems::Stretch) => 0,
            (FlexDirection::Row, AlignItems::Center) => GRAVITY_CENTER_VERTICAL,
            (FlexDirection::Row, AlignItems::End) => GRAVITY_BOTTOM,
            (FlexDirection::Row, AlignItems::Start) => GRAVITY_TOP,
            (FlexDirection::Column, AlignItems::Center) => GRAVITY_CENTER_HORIZONTAL,
            (FlexDirection::Column, AlignItems::End) => GRAVITY_RIGHT,
            (FlexDirection::Column, AlignItems::Start) => GRAVITY_LEFT,
        };

        let margins = node_margins(node);
        let extra = if first { 0.0 } else { gap };
        let (top, left) = match direction {
            FlexDirection::Row => (margins.top, margins.left + extra),
            FlexDirection::Column => (margins.top + extra, margins.left),
        };
        Ok(ChildParams {
            width,
            height,
            weight,
            gravity,
            margins: [
                self.dp(left)?,
                self.dp(top)?,
                self.dp(margins.right)?,
                self.dp(margins.bottom)?,
            ],
        })
    }

    /// 条件节点的槽位：主轴尺寸交给内容决定，分支移除后自然收成 0。
    fn slot_params(
        &self,
        node: &IRNode,
        direction: FlexDirection,
        align: AlignItems,
        gap: f64,
        first: bool,
    ) -> Result<ChildParams<'ctx>, String> {
        let mut params = self.linear_params(node, direction, align, gap, first)?;
        params.weight = None;
        match direction {
            FlexDirection::Column => params.height = self.i32c(WRAP_CONTENT),
            FlexDirection::Row => params.width = self.i32c(WRAP_CONTENT),
        }
        Ok(params)
    }

    fn frame_params(&self, node: &IRNode) -> Result<ChildParams<'ctx>, String> {
        let style = node_style(node);
        let left = self.inset(style.left)?;
        let right = self.inset(style.right)?;
        let top = self.inset(style.top)?;
        let bottom = self.inset(style.bottom)?;

        let width = match style.width {
            Some(Dimension::Points(value)) => self.dp(value)?,
            Some(Dimension::Percent(_)) => self.i32c(MATCH_PARENT),
            None if left.is_some() && right.is_some() => self.i32c(MATCH_PARENT),
            None => self.i32c(WRAP_CONTENT),
        };
        let height = match style.height {
            Some(Dimension::Points(value)) => self.dp(value)?,
            Some(Dimension::Percent(_)) => self.i32c(MATCH_PARENT),
            None if top.is_some() && bottom.is_some() => self.i32c(MATCH_PARENT),
            None => self.i32c(WRAP_CONTENT),
        };
        let gravity = if bottom.is_some() && top.is_none() {
            GRAVITY_BOTTOM
        } else {
            GRAVITY_TOP
        } | if right.is_some() && left.is_none() {
            GRAVITY_RIGHT
        } else {
            GRAVITY_LEFT
        };

        Ok(ChildParams {
            width,
            height,
            weight: None,
            gravity,
            margins: [
                left.unwrap_or_else(|| self.i32c(0)),
                top.unwrap_or_else(|| self.i32c(0)),
                right.unwrap_or_else(|| self.i32c(0)),
                bottom.unwrap_or_else(|| self.i32c(0)),
            ],
        })
    }

    fn inset(&self, value: Option<Dimension>) -> Result<Option<IntValue<'ctx>>, String> {
        match value {
            None => Ok(None),
            Some(Dimension::Points(points)) => Ok(Some(self.dp(points)?)),
            // 百分比偏移在 FrameLayout 里要靠测量才知道，这里退化成 0
            Some(Dimension::Percent(_)) => Ok(Some(self.i32c(0))),
        }
    }

    fn linear_params_object(
        &self,
        params: &ChildParams<'ctx>,
    ) -> Result<PointerValue<'ctx>, String> {
        let object = self.new_object(
            "LinearParams",
            "linearParamsCtor",
            &[JArg::I32(params.width), JArg::I32(params.height)],
        )?;
        let [left, top, right, bottom] = params.margins;
        self.call_void(
            object,
            "setMargins",
            &[
                JArg::I32(left),
                JArg::I32(top),
                JArg::I32(right),
                JArg::I32(bottom),
            ],
        )?;
        if let Some(weight) = params.weight {
            self.set_float_field(object, "linearWeight", weight)?;
        }
        if params.gravity != 0 {
            self.set_int_field(object, "linearGravity", self.i32c(params.gravity))?;
        }
        Ok(object)
    }

    fn frame_params_object(
        &self,
        params: &ChildParams<'ctx>,
    ) -> Result<PointerValue<'ctx>, String> {
        let object = self.new_object(
            "FrameParams",
            "frameParamsCtor",
            &[JArg::I32(params.width), JArg::I32(params.height)],
        )?;
        let [left, top, right, bottom] = params.margins;
        self.call_void(
            object,
            "setFrameMargins",
            &[
                JArg::I32(left),
                JArg::I32(top),
                JArg::I32(right),
                JArg::I32(bottom),
            ],
        )?;
        if params.gravity != 0 {
            self.set_int_field(object, "frameGravity", self.i32c(params.gravity))?;
        }
        Ok(object)
    }

    fn build_conditional(
        &self,
        cell: &str,
        when_true: &IRNode,
        when_false: Option<&IRNode>,
        initial: bool,
        true_mount: &[usize],
        true_unmount: &[usize],
        false_mount: &[usize],
        false_unmount: &[usize],
    ) -> Result<Built<'ctx>, String> {
        let activity = self.activity()?;
        let slot = self.new_object("Frame", "frameCtor", &[JArg::Obj(activity)])?;
        self.allow_child_overflow(slot)?;

        let true_branch = self.emit_branch(when_true)?;
        let false_branch = match when_false {
            Some(node) => Some(self.emit_branch(node)?),
            None => None,
        };

        // 初始只挂当前状态对应的那一支
        let visible = if initial {
            Some(true_branch.0)
        } else {
            false_branch.map(|branch| branch.0)
        };
        if let Some(view) = visible {
            self.call_view_at(slot, view, 0)?;
        }

        Ok(Built {
            view: slot,
            conditional: Some(PendingConditional {
                cell: cell.to_string(),
                initial,
                slot,
                true_global: true_branch.1,
                false_global: false_branch.map(|branch| branch.1),
                true_mount: true_mount.to_vec(),
                true_unmount: true_unmount.to_vec(),
                false_mount: false_mount.to_vec(),
                false_unmount: false_unmount.to_vec(),
            }),
        })
    }

    /// 构建一个分支视图，并把它登记进全局（切换时要用同一个对象做增删）。
    fn emit_branch(
        &self,
        node: &IRNode,
    ) -> Result<(PointerValue<'ctx>, GlobalValue<'ctx>), String> {
        let built = self.build_node(node)?;
        if let Some(pending) = built.conditional {
            // 分支本身也是条件节点：它的槽位就是这一支的视图，正常登记即可
            self.finalize_conditional(pending)?;
        }
        let params = self.frame_params(node)?;
        let params = self.frame_params_object(&params)?;
        self.call_void(built.view, "setLayoutParams", &[JArg::Obj(params)])?;
        let index = self.branch_counter.get();
        self.branch_counter.set(index + 1);
        let global = self
            .module()
            .add_global(self.ptr_type(), None, &format!("g_branch_{index}"));
        global.set_initializer(&self.ptr_type().const_null());
        let reference = self.global_ref(built.view)?;
        self.builder()
            .build_store(global.as_pointer_value(), reference)
            .map_err(|error| error.to_string())?;
        Ok((built.view, global))
    }

    /// 槽位挂到父容器后登记：槽位、两支分支、当前状态都进全局，切换函数再操作它们。
    fn finalize_conditional(&self, pending: PendingConditional<'ctx>) -> Result<(), String> {
        let cell = self
            .cell_index(&pending.cell)
            .ok_or_else(|| format!("unknown state cell '{}'", pending.cell))?;
        if self.kinds[cell] == CellKind::String {
            return Err(format!(
                "conditional branches need a boolean state cell, '{}' is a string",
                pending.cell
            ));
        }
        let index = self.conditionals.borrow().len();

        let slot_name = format!("g_cond_{index}_slot");
        let slot = self.module().add_global(self.ptr_type(), None, &slot_name);
        slot.set_initializer(&self.ptr_type().const_null());
        let slot_reference = self.global_ref(pending.slot)?;
        self.builder()
            .build_store(slot.as_pointer_value(), slot_reference)
            .map_err(|error| error.to_string())?;

        let active_name = format!("g_cond_{index}_active");
        let active = self
            .module()
            .add_global(self.i64_type(), None, &active_name);
        active.set_initializer(&self.i64_type().const_zero());
        self.builder()
            .build_store(
                active.as_pointer_value(),
                self.i64_type().const_int(pending.initial as u64, false),
            )
            .map_err(|error| error.to_string())?;

        self.conditionals.borrow_mut().push(ConditionalRecord {
            cell,
            slot,
            true_global: pending.true_global,
            false_global: pending.false_global,
            active,
            true_mount: pending.true_mount,
            true_unmount: pending.true_unmount,
            false_mount: pending.false_mount,
            false_unmount: pending.false_unmount,
        });
        Ok(())
    }

    // ---------- 状态刷新与动作 ----------

    fn cell_index(&self, cell: &str) -> Option<usize> {
        self.cells.iter().position(|(name, _)| name == cell)
    }

    fn numeric_global(&self, cell: usize) -> Result<GlobalValue<'ctx>, String> {
        self.module()
            .get_global(&format!("g_cell_{cell}"))
            .ok_or_else(|| "numeric cell global was not created".to_string())
    }

    fn string_global(&self, cell: usize) -> Result<GlobalValue<'ctx>, String> {
        self.module()
            .get_global(&format!("g_cell_{cell}_text"))
            .ok_or_else(|| "string cell global was not created".to_string())
    }

    fn emit_refresh_fns(&self) -> Result<(), String> {
        let conditionals = self.conditionals.borrow().clone();
        for (index, function) in self.refresh_fns.iter().enumerate() {
            self.begin_function(*function)?;

            let text = self.cell_text(index)?;
            for global in self.text_globals.borrow()[index].clone() {
                let view = self
                    .builder()
                    .build_load(self.ptr_type(), global.as_pointer_value(), "text_view")
                    .map_err(|error| error.to_string())?
                    .into_pointer_value();
                self.call_void(view, "setText", &[JArg::Obj(text)])?;
            }
            for (position, record) in conditionals.iter().enumerate() {
                if record.cell != index {
                    continue;
                }
                let toggle = self.toggle_fns[position];
                self.builder()
                    .build_call(toggle, &[], "")
                    .map_err(|error| error.to_string())?;
            }
            // 绑定了透明度的视图：单元里的整数 /100 后 setAlpha
            let opacity_views = self
                .opacity_globals
                .borrow()
                .get(index)
                .cloned()
                .unwrap_or_default();
            if !opacity_views.is_empty() && self.kinds[index] == CellKind::Number {
                let value = self
                    .builder()
                    .build_load(
                        self.i64_type(),
                        self.numeric_global(index)?.as_pointer_value(),
                        "opacity_cell",
                    )
                    .map_err(|error| error.to_string())?
                    .into_int_value();
                let raw = self
                    .builder()
                    .build_signed_int_to_float(value, self.f32_type(), "opacity_raw")
                    .map_err(|error| error.to_string())?;
                let ratio = self
                    .builder()
                    .build_float_div(raw, self.f32c(100.0), "opacity_ratio")
                    .map_err(|error| error.to_string())?;
                let zero = self.f32c(0.0);
                let one = self.f32c(1.0);
                let positive = self
                    .builder()
                    .build_float_compare(FloatPredicate::OGT, ratio, zero, "opacity_positive")
                    .map_err(|error| error.to_string())?;
                let lower = self
                    .builder()
                    .build_select(positive, ratio, zero, "opacity_lower")
                    .map_err(|error| error.to_string())?
                    .into_float_value();
                let small = self
                    .builder()
                    .build_float_compare(FloatPredicate::OLT, lower, one, "opacity_small")
                    .map_err(|error| error.to_string())?;
                let alpha = self
                    .builder()
                    .build_select(small, lower, one, "opacity_alpha")
                    .map_err(|error| error.to_string())?
                    .into_float_value();
                for global in opacity_views {
                    let view = self
                        .builder()
                        .build_load(self.ptr_type(), global.as_pointer_value(), "opacity_view")
                        .map_err(|error| error.to_string())?
                        .into_pointer_value();
                    self.call_void(view, "setAlpha", &[JArg::F32(alpha)])?;
                }
            }

            self.builder()
                .build_return(None)
                .map_err(|error| error.to_string())?;
        }
        Ok(())
    }

    /// 把一个单元的值变成 jstring：数字走 snprintf，布尔选字面量，字符串直接用 C 串。
    fn cell_text(&self, index: usize) -> Result<PointerValue<'ctx>, String> {
        match self.kinds[index] {
            CellKind::Number => {
                let value = self
                    .builder()
                    .build_load(
                        self.i64_type(),
                        self.numeric_global(index)?.as_pointer_value(),
                        "cell",
                    )
                    .map_err(|error| error.to_string())?
                    .into_int_value();
                let buffer = self.cell_buffer()?;
                let format = self.cstr("%lld")?;
                let snprintf = self.snprintf()?;
                self.builder()
                    .build_call(
                        snprintf,
                        &[
                            buffer.into(),
                            self.i64_type().const_int(32, false).into(),
                            format.into(),
                            value.into(),
                        ],
                        "",
                    )
                    .map_err(|error| error.to_string())?;
                self.new_string(buffer)
            }
            CellKind::Bool => {
                let value = self
                    .builder()
                    .build_load(
                        self.i64_type(),
                        self.numeric_global(index)?.as_pointer_value(),
                        "cell",
                    )
                    .map_err(|error| error.to_string())?
                    .into_int_value();
                let is_false = self
                    .builder()
                    .build_int_compare(
                        IntPredicate::EQ,
                        value,
                        self.i64_type().const_zero(),
                        "cell_false",
                    )
                    .map_err(|error| error.to_string())?;
                let false_text = self.cstr("false")?;
                let true_text = self.cstr("true")?;
                let selected = self
                    .builder()
                    .build_select(is_false, false_text, true_text, "cell_text")
                    .map_err(|error| error.to_string())?
                    .into_pointer_value();
                self.new_string(selected)
            }
            CellKind::String => {
                let pointer = self
                    .builder()
                    .build_load(
                        self.ptr_type(),
                        self.string_global(index)?.as_pointer_value(),
                        "cell_text",
                    )
                    .map_err(|error| error.to_string())?
                    .into_pointer_value();
                self.new_string(pointer)
            }
        }
    }

    fn cell_buffer(&self) -> Result<PointerValue<'ctx>, String> {
        let name = "g_cell_buffer";
        if let Some(global) = self.module().get_global(name) {
            return Ok(global.as_pointer_value());
        }
        let array = self.context().i8_type().array_type(32);
        let global = self.module().add_global(array, None, name);
        global.set_initializer(&array.const_zero());
        Ok(global.as_pointer_value())
    }

    fn snprintf(&self) -> Result<FunctionValue<'ctx>, String> {
        if let Some(function) = self.module().get_function("snprintf") {
            return Ok(function);
        }
        let ptr_type = self.ptr_type();
        let fn_type = self.i32_type().fn_type(
            &[ptr_type.into(), self.i64_type().into(), ptr_type.into()],
            true,
        );
        Ok(self.module().add_function("snprintf", fn_type, None))
    }

    fn strcmp(&self) -> Result<FunctionValue<'ctx>, String> {
        if let Some(function) = self.module().get_function("strcmp") {
            return Ok(function);
        }
        let ptr_type = self.ptr_type();
        Ok(self.module().add_function(
            "strcmp",
            self.i32_type().fn_type(&[ptr_type.into(), ptr_type.into()], false),
            None,
        ))
    }

    fn strdup(&self) -> Result<FunctionValue<'ctx>, String> {
        if let Some(function) = self.module().get_function("strdup") {
            return Ok(function);
        }
        let ptr_type = self.ptr_type();
        Ok(self
            .module()
            .add_function("strdup", ptr_type.fn_type(&[ptr_type.into()], false), None))
    }

    fn strcpy(&self) -> Result<FunctionValue<'ctx>, String> {
        if let Some(function) = self.module().get_function("strcpy") {
            return Ok(function);
        }
        let ptr_type = self.ptr_type();
        Ok(self.module().add_function(
            "strcpy",
            ptr_type.fn_type(&[ptr_type.into(), ptr_type.into()], false),
            None,
        ))
    }

    fn strcat(&self) -> Result<FunctionValue<'ctx>, String> {
        if let Some(function) = self.module().get_function("strcat") {
            return Ok(function);
        }
        let ptr_type = self.ptr_type();
        Ok(self.module().add_function(
            "strcat",
            ptr_type.fn_type(&[ptr_type.into(), ptr_type.into()], false),
            None,
        ))
    }

    fn emit_toggle_fns(&self) -> Result<(), String> {
        let records = self.conditionals.borrow().clone();
        for (index, function) in self.toggle_fns.iter().enumerate() {
            let Some(record) = records.get(index) else {
                continue;
            };
            self.begin_function(*function)?;

            let value = self
                .builder()
                .build_load(
                    self.i64_type(),
                    self.numeric_global(record.cell)?.as_pointer_value(),
                    "cond_cell",
                )
                .map_err(|error| error.to_string())?
                .into_int_value();
            let want = self
                .builder()
                .build_int_compare(
                    IntPredicate::NE,
                    value,
                    self.i64_type().const_zero(),
                    "cond_want",
                )
                .map_err(|error| error.to_string())?;
            let want_value = self
                .builder()
                .build_select(
                    want,
                    self.i64_type().const_int(1, false),
                    self.i64_type().const_zero(),
                    "cond_want_value",
                )
                .map_err(|error| error.to_string())?
                .into_int_value();
            let active = self
                .builder()
                .build_load(
                    self.i64_type(),
                    record.active.as_pointer_value(),
                    "cond_active",
                )
                .map_err(|error| error.to_string())?
                .into_int_value();
            let unchanged = self
                .builder()
                .build_int_compare(IntPredicate::EQ, want_value, active, "cond_unchanged")
                .map_err(|error| error.to_string())?;

            let done = self.context().append_basic_block(*function, "done");
            let changed = self.context().append_basic_block(*function, "changed");
            self.builder()
                .build_conditional_branch(unchanged, done, changed)
                .map_err(|error| error.to_string())?;

            self.builder().position_at_end(changed);
            let turn_on = self.context().append_basic_block(*function, "turn_on");
            let turn_off = self.context().append_basic_block(*function, "turn_off");
            self.builder()
                .build_conditional_branch(want, turn_on, turn_off)
                .map_err(|error| error.to_string())?;

            self.builder().position_at_end(turn_on);
            self.emit_branch_switch(
                record,
                record.false_global.as_ref(),
                Some(&record.true_global),
                &record.false_unmount,
                &record.true_mount,
                1,
            )?;
            self.builder()
                .build_unconditional_branch(done)
                .map_err(|error| error.to_string())?;

            self.builder().position_at_end(turn_off);
            self.emit_branch_switch(
                record,
                Some(&record.true_global),
                record.false_global.as_ref(),
                &record.true_unmount,
                &record.false_mount,
                0,
            )?;
            self.builder()
                .build_unconditional_branch(done)
                .map_err(|error| error.to_string())?;

            self.builder().position_at_end(done);
            self.builder()
                .build_return(None)
                .map_err(|error| error.to_string())?;
        }
        Ok(())
    }

    /// 切换槽位内容：移除旧分支 → 跑卸载动作 → 挂新分支 → 跑挂载动作。
    fn emit_branch_switch(
        &self,
        record: &ConditionalRecord<'ctx>,
        remove: Option<&GlobalValue<'ctx>>,
        add: Option<&GlobalValue<'ctx>>,
        unmount: &[usize],
        mount: &[usize],
        active: u64,
    ) -> Result<(), String> {
        let slot = self
            .builder()
            .build_load(self.ptr_type(), record.slot.as_pointer_value(), "slot")
            .map_err(|error| error.to_string())?
            .into_pointer_value();
        if let Some(global) = remove {
            let view = self
                .builder()
                .build_load(self.ptr_type(), global.as_pointer_value(), "gone_branch")
                .map_err(|error| error.to_string())?
                .into_pointer_value();
            self.call_void(slot, "removeView", &[JArg::Obj(view)])?;
        }
        for action in unmount {
            let function = self.action_fns[*action];
            self.builder()
                .build_call(function, &[self.ptr_type().const_null().into()], "")
                .map_err(|error| error.to_string())?;
        }
        if let Some(global) = add {
            let view = self
                .builder()
                .build_load(self.ptr_type(), global.as_pointer_value(), "shown_branch")
                .map_err(|error| error.to_string())?
                .into_pointer_value();
            self.call_view_at(slot, view, 0)?;
        }
        for action in mount {
            let function = self.action_fns[*action];
            self.builder()
                .build_call(function, &[self.ptr_type().const_null().into()], "")
                .map_err(|error| error.to_string())?;
        }
        self.builder()
            .build_store(
                record.active.as_pointer_value(),
                self.i64_type().const_int(active, false),
            )
            .map_err(|error| error.to_string())?;
        Ok(())
    }

    fn emit_action_fns(&self) -> Result<(), String> {
        for (index, function) in self.action_fns.iter().enumerate() {
            self.begin_function(*function)?;
            let text = function.get_nth_param(0).unwrap().into_pointer_value();
            let mutation = self.program.state_actions[index].clone();
            self.emit_action_body(&mutation, text)?;
            self.builder()
                .build_return(None)
                .map_err(|error| error.to_string())?;
        }
        Ok(())
    }

    /// 多语句 handler 按顺序执行，每条只刷新自己写的单元。
    fn list_value_pointer(&self, value: &super::ItemValue) -> Result<PointerValue<'ctx>, String> {
        match value {
            super::ItemValue::Literal(value) => self.cstr(&value.display_value()),
            super::ItemValue::Cell(cell) => {
                let index = self
                    .cell_index(cell)
                    .ok_or_else(|| format!("unknown state cell '{cell}'"))?;
                match self.kinds[index] {
                    CellKind::String => {
                        let source = self
                            .builder()
                            .build_load(
                                self.ptr_type(),
                                self.string_global(index)?.as_pointer_value(),
                                "push_cell_text",
                            )
                            .map_err(|error| error.to_string())?
                            .into_pointer_value();
                        let copy = self
                            .builder()
                            .build_call(self.strdup()?, &[source.into()], "push_cell_copy")
                            .map_err(|error| error.to_string())?;
                        match copy.try_as_basic_value() {
                            inkwell::values::ValueKind::Basic(value) => {
                                Ok(value.into_pointer_value())
                            }
                            inkwell::values::ValueKind::Instruction(_) => {
                                Err("strdup returned void".into())
                            }
                        }
                    }
                    _ => {
                        let number = self
                            .builder()
                            .build_load(
                                self.i64_type(),
                                self.numeric_global(index)?.as_pointer_value(),
                                "push_cell_number",
                            )
                            .map_err(|error| error.to_string())?
                            .into_int_value();
                        self.number_to_text(number)
                    }
                }
            }
        }
    }

    fn number_to_text(&self, value: IntValue<'ctx>) -> Result<PointerValue<'ctx>, String> {
        let buffer_type = self.codegen.context.i8_type().array_type(32);
        let buffer = match self.codegen.module.get_global("g_number_text_buffer") {
            Some(global) => global,
            None => {
                let global = self
                    .codegen
                    .module
                    .add_global(buffer_type, None, "g_number_text_buffer");
                global.set_initializer(&buffer_type.const_zero());
                global
            }
        };
        let format = self
            .builder()
            .build_global_string_ptr("%lld", "number_format")
            .map_err(|error| error.to_string())?;
        let ptr_type = self.ptr_type();
        let snprintf_type = self.i32_type().fn_type(
            &[ptr_type.into(), self.i64_type().into(), ptr_type.into()],
            true,
        );
        let snprintf = self
            .module()
            .get_function("snprintf")
            .unwrap_or_else(|| self.module().add_function("snprintf", snprintf_type, None));
        self.builder()
            .build_call(
                snprintf,
                &[
                    buffer.as_pointer_value().into(),
                    self.i64_type().const_int(32, false).into(),
                    format.as_pointer_value().into(),
                    value.into(),
                ],
                "number_text",
            )
            .map_err(|error| error.to_string())?;
        let copy = self
            .builder()
            .build_call(self.strdup()?, &[buffer.as_pointer_value().into()], "number_copy")
            .map_err(|error| error.to_string())?;
        match copy.try_as_basic_value() {
            inkwell::values::ValueKind::Basic(value) => Ok(value.into_pointer_value()),
            inkwell::values::ValueKind::Instruction(_) => Err("strdup returned void".into()),
        }
    }

    fn emit_push(
        &self,
        list: &str,
        values: &[(String, super::ItemValue)],
    ) -> Result<(), String> {
        let Ok(slots) = self.list_slots_for(list) else {
            // 降级模式下列表没有渲染槽位，动作直接跳过
            return Ok(());
        };
        let slot = slots[0].clone();
        let i64_type = self.i64_type();
        let count = self
            .builder()
            .build_load(i64_type, slot.count_global.as_pointer_value(), "list_count")
            .map_err(|error| error.to_string())?
            .into_int_value();
        let has_room = self
            .builder()
            .build_int_compare(
                IntPredicate::SLT,
                count,
                i64_type.const_int(LIST_CAPACITY as u64, false),
                "list_has_room",
            )
            .map_err(|error| error.to_string())?;
        let function = self
            .builder()
            .get_insert_block()
            .and_then(|block| block.get_parent())
            .ok_or("missing insertion block")?;
        let append = self.codegen.context.append_basic_block(function, "list_append");
        let done = self.codegen.context.append_basic_block(function, "list_append_done");
        self.builder()
            .build_conditional_branch(has_room, append, done)
            .map_err(|error| error.to_string())?;
        self.builder().position_at_end(append);
        let mut columns: Vec<String> = slot.columns.keys().cloned().collect();
        columns.sort();
        for column in &columns {
            let raw = match values.iter().find(|(name, _)| name == column) {
                Some((_, value)) => self.list_value_pointer(value)?,
                None => self.cstr("")?,
            };
            self.store_list_column(&slot.id, column, count, raw)?;
        }
        // 新行挂在根层：和 iOS 一致，push 出来的行 parent = -1、depth = 0。
        self.store_list_number(
            &slot.id,
            "parent",
            count,
            i64_type.const_int(u64::MAX, true),
        )?;
        self.store_list_number(&slot.id, "depth", count, i64_type.const_zero())?;
        let next = self
            .builder()
            .build_int_add(count, i64_type.const_int(1, false), "list_count_next")
            .map_err(|error| error.to_string())?;
        self.builder()
            .build_store(slot.count_global.as_pointer_value(), next)
            .map_err(|error| error.to_string())?;
        self.sync_list_length(list, next)?;
        self.rebuild_lists(&slots)?;
        self.builder()
            .build_unconditional_branch(done)
            .map_err(|error| error.to_string())?;
        self.builder().position_at_end(done);
        Ok(())
    }

    fn emit_set_item(
        &self,
        list: &str,
        field: &str,
        value: &StateValue,
    ) -> Result<(), String> {
        let Ok(slots) = self.list_slots_for(list) else {
            return Ok(());
        };
        let slot = slots[0].clone();
        let row = self
            .builder()
            .build_load(self.i64_type(), self.active_row.as_pointer_value(), "active_row")
            .map_err(|error| error.to_string())?
            .into_int_value();
        let valid = self
            .builder()
            .build_int_compare(
                IntPredicate::SGE,
                row,
                self.i64_type().const_zero(),
                "row_valid",
            )
            .map_err(|error| error.to_string())?;
        let function = self
            .builder()
            .get_insert_block()
            .and_then(|block| block.get_parent())
            .ok_or("missing insertion block")?;
        let write = self.codegen.context.append_basic_block(function, "item_write");
        let done = self.codegen.context.append_basic_block(function, "item_write_done");
        self.builder()
            .build_conditional_branch(valid, write, done)
            .map_err(|error| error.to_string())?;
        self.builder().position_at_end(write);
        let raw = self.cstr(&value.display_value())?;
        self.store_list_column(&slot.id, field, row, raw)?;
        self.rebuild_lists(&slots)?;
        self.builder()
            .build_unconditional_branch(done)
            .map_err(|error| error.to_string())?;
        self.builder().position_at_end(done);
        Ok(())
    }

    fn emit_remove_row(&self, list: &str) -> Result<(), String> {
        let Ok(slots) = self.list_slots_for(list) else {
            // 降级模式下列表没有渲染槽位，动作直接跳过
            return Ok(());
        };
        let slot = slots[0].clone();
        let i64_type = self.i64_type();
        let count = self
            .builder()
            .build_load(i64_type, slot.count_global.as_pointer_value(), "list_count")
            .map_err(|error| error.to_string())?
            .into_int_value();
        let row = self
            .builder()
            .build_load(i64_type, self.active_row.as_pointer_value(), "active_row")
            .map_err(|error| error.to_string())?
            .into_int_value();
        let function = self
            .builder()
            .get_insert_block()
            .and_then(|block| block.get_parent())
            .ok_or("missing insertion block")?;
        let has_rows = self
            .builder()
            .build_int_compare(IntPredicate::SGT, count, i64_type.const_zero(), "list_has_rows")
            .map_err(|error| error.to_string())?;
        let remove = self.codegen.context.append_basic_block(function, "list_remove");
        let done = self.codegen.context.append_basic_block(function, "list_remove_done");
        self.builder()
            .build_conditional_branch(has_rows, remove, done)
            .map_err(|error| error.to_string())?;
        self.builder().position_at_end(remove);

        let mut columns: Vec<String> = slot.columns.keys().cloned().collect();
        columns.sort();
        let cursor = self
            .builder()
            .build_alloca(i64_type, "shift_index")
            .map_err(|error| error.to_string())?;
        self.builder()
            .build_store(cursor, row)
            .map_err(|error| error.to_string())?;
        let loop_block = self.codegen.context.append_basic_block(function, "shift_loop");
        let body = self.codegen.context.append_basic_block(function, "shift_body");
        let end = self.codegen.context.append_basic_block(function, "shift_done");
        self.builder()
            .build_unconditional_branch(loop_block)
            .map_err(|error| error.to_string())?;
        self.builder().position_at_end(loop_block);
        let index = self
            .builder()
            .build_load(i64_type, cursor, "shift_index_value")
            .map_err(|error| error.to_string())?
            .into_int_value();
        let last = self
            .builder()
            .build_int_sub(count, i64_type.const_int(1, false), "shift_last")
            .map_err(|error| error.to_string())?;
        let more = self
            .builder()
            .build_int_compare(IntPredicate::SLT, index, last, "shift_has_more")
            .map_err(|error| error.to_string())?;
        self.builder()
            .build_conditional_branch(more, body, end)
            .map_err(|error| error.to_string())?;
        self.builder().position_at_end(body);
        let next_index = self
            .builder()
            .build_int_add(index, i64_type.const_int(1, false), "shift_next")
            .map_err(|error| error.to_string())?;
        for column in &columns {
            let value = self.load_list_column(&slot.id, column, next_index)?;
            self.store_list_column(&slot.id, column, index, value)?;
        }
        // 父子/深度与行内条件标记也是按行存的，删除时要一起前移
        for column in ["parent", "depth"] {
            let value = self.load_list_number(&slot.id, column, next_index)?;
            self.store_list_number(&slot.id, column, index, value)?;
        }
        for position in 0..slot.conditionals {
            let column = format!("cond_{position}");
            let value = self.load_list_number(&slot.id, &column, next_index)?;
            self.store_list_number(&slot.id, &column, index, value)?;
        }
        self.builder()
            .build_store(cursor, next_index)
            .map_err(|error| error.to_string())?;
        self.builder()
            .build_unconditional_branch(loop_block)
            .map_err(|error| error.to_string())?;
        self.builder().position_at_end(end);

        let next_count = self
            .builder()
            .build_int_sub(count, i64_type.const_int(1, false), "list_count_next")
            .map_err(|error| error.to_string())?;
        self.builder()
            .build_store(slot.count_global.as_pointer_value(), next_count)
            .map_err(|error| error.to_string())?;
        self.sync_list_length(list, next_count)?;
        self.rebuild_lists(&slots)?;
        self.builder()
            .build_unconditional_branch(done)
            .map_err(|error| error.to_string())?;
        self.builder().position_at_end(done);
        Ok(())
    }

    fn emit_clear_list(&self, list: &str) -> Result<(), String> {
        let Ok(slots) = self.list_slots_for(list) else {
            return Ok(());
        };
        let slot = slots[0].clone();
        let zero = self.i64_type().const_zero();
        self.builder()
            .build_store(slot.count_global.as_pointer_value(), zero)
            .map_err(|error| error.to_string())?;
        self.sync_list_length(list, zero)?;
        self.rebuild_lists(&slots)
    }

    fn emit_action_body(
        &self,
        mutation: &StateMutation,
        text: PointerValue<'ctx>,
    ) -> Result<(), String> {
        match mutation {
            StateMutation::Sequence(steps) => {
                for step in steps {
                    self.emit_action_body(step, text)?;
                }
                Ok(())
            }
            StateMutation::SetCells { values } => {
                for (cell, value) in values {
                    if let Some(cell_index) = self.cell_index(cell) {
                        self.emit_mutation(
                            cell_index,
                            &StateMutation::Set(cell.clone(), value.clone()),
                            text,
                        )?;
                        let refresh = self.refresh_fns[cell_index];
                        self.builder()
                            .build_call(refresh, &[], "")
                            .map_err(|error| error.to_string())?;
                    }
                }
                Ok(())
            }
            StateMutation::PushItem { list, values } => self.emit_push(list, values),
            StateMutation::SetItemField { list, field, value } => {
                self.emit_set_item(list, field, value)
            }
            StateMutation::RemoveCurrentItem { list } => self.emit_remove_row(list),
            StateMutation::ClearList { list } => self.emit_clear_list(list),
            scalar => {
                let Some(cell) = scalar.cell() else {
                    return Ok(());
                };
                let cell_index = self
                    .cell_index(cell)
                    .ok_or_else(|| format!("unknown state cell '{cell}'"))?;
                self.emit_mutation(cell_index, scalar, text)?;
                let refresh = self.refresh_fns[cell_index];
                self.builder()
                    .build_call(refresh, &[], "")
                    .map_err(|error| error.to_string())?;
                Ok(())
            }
        }
    }

    fn emit_mutation(
        &self,
        cell: usize,
        mutation: &StateMutation,
        text: PointerValue<'ctx>,
    ) -> Result<(), String> {
        match mutation {
            StateMutation::Noop => Ok(()),
            StateMutation::Sequence(_)
            | StateMutation::PushItem { .. }
            | StateMutation::SetItemField { .. }
            | StateMutation::RemoveCurrentItem { .. }
            | StateMutation::ClearList { .. }
            // 滚动位置由 nativeOnScroll 直接写入，不走控件派发
            | StateMutation::SetFromScroll(_)
            | StateMutation::SetFromScrollClamped { .. }
            | StateMutation::SetCells { .. } => Ok(()),
            StateMutation::SetFromCell { source, delta, .. } => {
                self.require_kind(cell, CellKind::Number, "SetFromCell")?;
                let source_index = self
                    .cell_index(source)
                    .ok_or_else(|| format!("unknown state cell '{source}'"))?;
                self.require_kind(source_index, CellKind::Number, "SetFromCell")?;
                let source_value = self
                    .builder()
                    .build_load(
                        self.i64_type(),
                        self.numeric_global(source_index)?.as_pointer_value(),
                        "source_cell",
                    )
                    .map_err(|error| error.to_string())?
                    .into_int_value();
                let updated = self
                    .builder()
                    .build_int_add(
                        source_value,
                        self.i64_type().const_int(*delta as u64, true),
                        "cell_updated",
                    )
                    .map_err(|error| error.to_string())?;
                self.builder()
                    .build_store(self.numeric_global(cell)?.as_pointer_value(), updated)
                    .map_err(|error| error.to_string())?;
                Ok(())
            }
            StateMutation::Increment(_) => self.emit_numeric_delta(cell, 1),
            StateMutation::Decrement(_) => self.emit_numeric_delta(cell, -1),
            StateMutation::SetTrue(_) => {
                self.require_kind(cell, CellKind::Bool, "SetTrue")?;
                self.builder()
                    .build_store(
                        self.numeric_global(cell)?.as_pointer_value(),
                        self.i64_type().const_int(1, false),
                    )
                    .map_err(|error| error.to_string())?;
                Ok(())
            }
            StateMutation::Set(_, value) => match self.kinds[cell] {
                CellKind::Number => {
                    let number = match value {
                        StateValue::Number(number) => *number,
                        other => {
                            return Err(format!(
                                "state cell '{}' is numeric but was assigned '{}'",
                                self.cells[cell].0,
                                other.display_value()
                            ));
                        }
                    };
                    self.builder()
                        .build_store(
                            self.numeric_global(cell)?.as_pointer_value(),
                            self.i64_type().const_int(number as u64, false),
                        )
                        .map_err(|error| error.to_string())?;
                    Ok(())
                }
                CellKind::Bool => {
                    let flag = matches!(value, StateValue::Boolean(true));
                    self.builder()
                        .build_store(
                            self.numeric_global(cell)?.as_pointer_value(),
                            self.i64_type().const_int(flag as u64, false),
                        )
                        .map_err(|error| error.to_string())?;
                    Ok(())
                }
                CellKind::String => {
                    let literal = match value {
                        StateValue::String(text) => text.clone(),
                        other => other.display_value(),
                    };
                    let c_string = self.cstr(&literal)?;
                    self.builder()
                        .build_store(self.string_global(cell)?.as_pointer_value(), c_string)
                        .map_err(|error| error.to_string())?;
                    Ok(())
                }
            },
            StateMutation::SetFromInput(_) => {
                self.require_kind(cell, CellKind::String, "SetFromInput")?;
                let env = self.env()?;
                let ptr_type = self.ptr_type();
                let params = [ptr_type.into(), ptr_type.into(), ptr_type.into()];
                let chars = self
                    .invoke(
                        JNI_GET_STRING_UTF_CHARS,
                        Some(ptr_type.into()),
                        &params,
                        &[env.into(), text.into(), ptr_type.const_null().into()],
                    )?
                    .ok_or("GetStringUTFChars returned void")?
                    .into_pointer_value();
                // jstring 的字符只在释放前有效，拷贝一份由自己持有（输入框不会改到需要精细回收的规模）
                let call = self
                    .builder()
                    .build_call(self.strdup()?, &[chars.into()], "input_copy")
                    .map_err(|error| error.to_string())?;
                let copy = match call.try_as_basic_value() {
                    inkwell::values::ValueKind::Basic(value) => value.into_pointer_value(),
                    inkwell::values::ValueKind::Instruction(_) => {
                        return Err("strdup returned void".into());
                    }
                };
                self.invoke(
                    JNI_RELEASE_STRING_UTF_CHARS,
                    None,
                    &params,
                    &[env.into(), text.into(), chars.into()],
                )?;
                self.builder()
                    .build_store(self.string_global(cell)?.as_pointer_value(), copy)
                    .map_err(|error| error.to_string())?;
                Ok(())
            }
        }
    }

    fn emit_numeric_delta(&self, cell: usize, delta: i64) -> Result<(), String> {
        self.require_kind(cell, CellKind::Number, "Increment/Decrement")?;
        let global = self.numeric_global(cell)?;
        let current = self
            .builder()
            .build_load(self.i64_type(), global.as_pointer_value(), "cell")
            .map_err(|error| error.to_string())?
            .into_int_value();
        let updated = self
            .builder()
            .build_int_add(
                current,
                self.i64_type().const_int(delta as u64, false),
                "cell_updated",
            )
            .map_err(|error| error.to_string())?;
        self.builder()
            .build_store(global.as_pointer_value(), updated)
            .map_err(|error| error.to_string())?;
        Ok(())
    }

    fn require_kind(&self, cell: usize, kind: CellKind, operation: &str) -> Result<(), String> {
        if self.kinds[cell] == kind {
            return Ok(());
        }
        Err(format!(
            "{operation} cannot be applied to state cell '{}'",
            self.cells[cell].0
        ))
    }

    // ---------- JNI 入口 ----------

    fn emit_entries(&self) -> Result<(), String> {
        let ptr_type = self.ptr_type();

        // nativeCreate(JNIEnv*, jobject, jobject activity)
        let fn_type = self
            .void_type()
            .fn_type(&[ptr_type.into(), ptr_type.into(), ptr_type.into()], false);
        let function = self.module().add_function(
            "Java_com_lumen_app_MainActivity_nativeCreate",
            fn_type,
            None,
        );
        self.begin_function(function)?;
        let env = function.get_nth_param(0).unwrap().into_pointer_value();
        let activity = function.get_nth_param(2).unwrap().into_pointer_value();
        self.store_global("g_env", env)?;
        // 界面构建会攒下几百个局部引用，整段包在一个大局部引用帧里。
        let params = [ptr_type.into(), self.i32_type().into()];
        self.invoke(
            JNI_PUSH_LOCAL_FRAME,
            Some(self.i32_type().into()),
            &params,
            &[env.into(), self.i32c(LOCAL_FRAME_CAPACITY).into()],
        )?;
        let init = self
            .module()
            .get_function("lumen_jni_init")
            .ok_or("lumen_jni_init was not created")?;
        self.builder()
            .build_call(init, &[activity.into()], "")
            .map_err(|error| error.to_string())?;

        // 符号解析失败就不要建界面了，logcat 里能看到缺的是哪一个。
        let failed = self
            .module()
            .get_global("g_failed")
            .ok_or("g_failed was not created")?;
        let flag = self
            .builder()
            .build_load(self.i32_type(), failed.as_pointer_value(), "failed")
            .map_err(|error| error.to_string())?
            .into_int_value();
        let broken = self
            .builder()
            .build_int_compare(IntPredicate::NE, flag, self.i32c(0), "jni_broken")
            .map_err(|error| error.to_string())?;
        let bail = self.context().append_basic_block(function, "bail");
        let build = self.context().append_basic_block(function, "build");
        self.builder()
            .build_conditional_branch(broken, bail, build)
            .map_err(|error| error.to_string())?;

        self.builder().position_at_end(build);
        let build_fn = self
            .module()
            .get_function("lumen_build_ui")
            .ok_or("lumen_build_ui was not created")?;
        self.builder()
            .build_call(build_fn, &[activity.into()], "")
            .map_err(|error| error.to_string())?;
        self.builder()
            .build_unconditional_branch(bail)
            .map_err(|error| error.to_string())?;

        self.builder().position_at_end(bail);
        let params = [ptr_type.into(), ptr_type.into()];
        self.invoke(
            JNI_POP_LOCAL_FRAME,
            Some(ptr_type.into()),
            &params,
            &[env.into(), ptr_type.const_null().into()],
        )?;
        self.builder()
            .build_return(None)
            .map_err(|error| error.to_string())?;

        // nativeOnClick / nativeOnFocus：(JNIEnv*, jclass, jint)
        for name in [
            "Java_com_lumen_app_MainActivity_nativeOnClick",
            "Java_com_lumen_app_MainActivity_nativeOnFocus",
        ] {
            let fn_type = self.void_type().fn_type(
                &[ptr_type.into(), ptr_type.into(), self.i32_type().into()],
                false,
            );
            let function = self.module().add_function(name, fn_type, None);
            self.begin_function(function)?;
            let env = function.get_nth_param(0).unwrap().into_pointer_value();
            let id = function.get_nth_param(2).unwrap().into_int_value();
            self.store_global("g_env", env)?;
            self.emit_action_dispatch(function, id, None)?;
            self.builder()
                .build_return(None)
                .map_err(|error| error.to_string())?;
        }

        // nativeOnInput(JNIEnv*, jclass, jint, jstring)
        let fn_type = self.void_type().fn_type(
            &[
                ptr_type.into(),
                ptr_type.into(),
                self.i32_type().into(),
                ptr_type.into(),
            ],
            false,
        );
        let function = self.module().add_function(
            "Java_com_lumen_app_MainActivity_nativeOnInput",
            fn_type,
            None,
        );
        self.begin_function(function)?;
        let env = function.get_nth_param(0).unwrap().into_pointer_value();
        let id = function.get_nth_param(2).unwrap().into_int_value();
        let text = function.get_nth_param(3).unwrap().into_pointer_value();
        self.store_global("g_env", env)?;
        self.emit_action_dispatch(function, id, Some(text))?;
        self.builder()
            .build_return(None)
            .map_err(|error| error.to_string())?;

        // nativeOnScroll(JNIEnv*, jclass, jint scrollY)
        let fn_type = self.void_type().fn_type(
            &[ptr_type.into(), ptr_type.into(), self.i32_type().into()],
            false,
        );
        let function = self.module().add_function(
            "Java_com_lumen_app_MainActivity_nativeOnScroll",
            fn_type,
            None,
        );
        self.begin_function(function)?;
        let env = function.get_nth_param(0).unwrap().into_pointer_value();
        let scroll = function.get_nth_param(2).unwrap().into_int_value();
        self.store_global("g_env", env)?;
        // onScroll={(y) => 单元 = y}：滚动位置写进数值单元并刷新引用它的控件
        if let Some((cell, bounds)) = scroll_action_cell(self.program) {
            if let Some(index) = self.cell_index(cell) {
                if self.kinds[index] == CellKind::Number {
                    let value = self
                        .builder()
                        .build_int_s_extend(scroll, self.i64_type(), "scroll_offset")
                        .map_err(|error| error.to_string())?;
                    // clamp(y, low, high)：写入前夹到区间内
                    let value = match bounds {
                        Some((low, high, shift)) => {
                            let i64_type = self.i64_type();
                            // clamp(y - shift, low, high)
                            let value = if shift == 0 {
                                value
                            } else {
                                self.builder()
                                    .build_int_add(
                                        value,
                                        i64_type.const_int((-shift) as u64, true),
                                        "scroll_shifted",
                                    )
                                    .map_err(|error| error.to_string())?
                            };
                            let above_low = self
                                .builder()
                                .build_int_compare(
                                    IntPredicate::SGT,
                                    value,
                                    i64_type.const_int(low as u64, true),
                                    "scroll_above_low",
                                )
                                .map_err(|error| error.to_string())?;
                            let lower = self
                                .builder()
                                .build_select(
                                    above_low,
                                    value,
                                    i64_type.const_int(low as u64, true),
                                    "scroll_lower",
                                )
                                .map_err(|error| error.to_string())?
                                .into_int_value();
                            let below_high = self
                                .builder()
                                .build_int_compare(
                                    IntPredicate::SLT,
                                    lower,
                                    i64_type.const_int(high as u64, true),
                                    "scroll_below_high",
                                )
                                .map_err(|error| error.to_string())?;
                            self.builder()
                                .build_select(
                                    below_high,
                                    lower,
                                    i64_type.const_int(high as u64, true),
                                    "scroll_clamped",
                                )
                                .map_err(|error| error.to_string())?
                                .into_int_value()
                        }
                        None => value,
                    };
                    self.builder()
                        .build_store(self.numeric_global(index)?.as_pointer_value(), value)
                        .map_err(|error| error.to_string())?;
                    let refresh = self.refresh_fns[index];
                    self.builder()
                        .build_call(refresh, &[], "")
                        .map_err(|error| error.to_string())?;
                }
            }
        }
        self.builder()
            .build_return(None)
            .map_err(|error| error.to_string())?;
        Ok(())
    }

    fn emit_action_dispatch(
        &self,
        function: FunctionValue<'ctx>,
        id: IntValue<'ctx>,
        text: Option<PointerValue<'ctx>>,
    ) -> Result<(), String> {
        let done = self.context().append_basic_block(function, "dispatch_done");
        // 行内点击的 id 里带着行号，先解码再匹配动作
        let base = self.i32c(1_000_000);
        let is_row = self
            .builder()
            .build_int_compare(IntPredicate::UGE, id, base, "is_row_click")
            .map_err(|error| error.to_string())?;
        let offset = self
            .builder()
            .build_int_sub(id, base, "row_click_offset")
            .map_err(|error| error.to_string())?;
        let row_action = self
            .builder()
            .build_int_signed_div(offset, self.i32c(1024), "row_click_action")
            .map_err(|error| error.to_string())?;
        let row_index = self
            .builder()
            .build_int_signed_rem(offset, self.i32c(1024), "row_click_row")
            .map_err(|error| error.to_string())?;
        let action_id = self
            .builder()
            .build_select(is_row, row_action, id, "dispatch_action_id")
            .map_err(|error| error.to_string())?
            .into_int_value();
        let row_value = self
            .builder()
            .build_select(is_row, row_index, self.i32c(-1), "dispatch_row")
            .map_err(|error| error.to_string())?
            .into_int_value();
        let row_value = self
            .builder()
            .build_int_s_extend(row_value, self.i64_type(), "dispatch_row_i64")
            .map_err(|error| error.to_string())?;
        self.builder()
            .build_store(self.active_row.as_pointer_value(), row_value)
            .map_err(|error| error.to_string())?;
        for (index, action) in self.action_fns.iter().enumerate() {
            let hit = self
                .context()
                .append_basic_block(function, &format!("hit_{index}"));
            let next = self
                .context()
                .append_basic_block(function, &format!("next_{index}"));
            let matched = self
                .builder()
                .build_int_compare(IntPredicate::EQ, action_id, self.i32c(index as i32), "id_matched")
                .map_err(|error| error.to_string())?;
            self.builder()
                .build_conditional_branch(matched, hit, next)
                .map_err(|error| error.to_string())?;
            self.builder().position_at_end(hit);
            let argument = match text {
                Some(value) => value,
                None => self.ptr_type().const_null(),
            };
            self.builder()
                .build_call(*action, &[argument.into()], "")
                .map_err(|error| error.to_string())?;
            self.builder()
                .build_unconditional_branch(done)
                .map_err(|error| error.to_string())?;
            self.builder().position_at_end(next);
        }
        self.builder()
            .build_unconditional_branch(done)
            .map_err(|error| error.to_string())?;
        self.builder().position_at_end(done);
        Ok(())
    }
}

/// 子级是否脱离 flex 流：显式 absolute，或给了任意一个方向的偏移。
/// 行模板 → Java 侧模板描述；不支持的形态返回 None（回退到 native 建行）。
///
/// 只覆盖「横向容器 + 一排叶子控件」这种最常见的行：列表行、设置项、清单项都是这个形态。
fn row_spec(item: &IRNode, fields: &[String]) -> Option<String> {
    let (style, children) = match item {
        IRNode::View { style, children } | IRNode::VStack { style, children } => (style, children),
        _ => return None,
    };
    if style.flex_direction.unwrap_or(FlexDirection::Column) != FlexDirection::Row
        || style.align_items != Some(AlignItems::Center)
        || style.position != Position::Relative
    {
        return None;
    }
    let space_between = matches!(style.justify_content, Some(JustifyContent::SpaceBetween));
    if !matches!(
        style.justify_content,
        None | Some(JustifyContent::Start) | Some(JustifyContent::SpaceBetween)
    ) {
        return None;
    }

    let padding = node_padding(style);
    let margins = node_margins(item);
    let mut spec = String::from("L dir=0");
    if let Some(gap) = style.spacing.filter(|gap| *gap > 0.0) {
        spec.push_str(&format!(" gap={}", gap.round() as i32));
    }
    spec.push_str(&format!(
        " pl={} pt={} pr={} pb={}",
        padding.left.round() as i32,
        padding.top.round() as i32,
        padding.right.round() as i32,
        padding.bottom.round() as i32
    ));
    spec.push_str(&format!(
        " ml={} mt={} mr={} mb={}",
        margins.left.round() as i32,
        margins.top.round() as i32,
        margins.right.round() as i32,
        margins.bottom.round() as i32
    ));
    spec.push_str(&format!(" w={}", spec_length(style.width, -1)?));
    spec.push_str(&format!(" h={}", spec_length(style.height, -2)?));
    if space_between {
        spec.push_str(" sb=1");
    }
    if let Some(background) = &style.background_color {
        spec.push_str(&format!(" bg={}", color_argb(background).ok()?));
    }
    if let Some(radius) = style.border_radius.filter(|radius| *radius > 0.0) {
        spec.push_str(&format!(" rad={}", radius.round() as i32));
    }

    for child in children {
        let line = match child {
            IRNode::ItemText { field, style, .. } => {
                let index = fields.iter().position(|name| name == field)?;
                let mut line = format!("T f={index}");
                line.push_str(&spec_slot_box(style, true)?);
                line.push_str(&spec_text_tokens(style)?);
                line
            }
            IRNode::Button {
                title,
                style,
                state_action: Some(action),
                unsupported_handler: false,
            } => {
                if title.contains(' ') || title.contains('|') || title.is_empty() {
                    return None;
                }
                let mut line = format!("B a={action}");
                line.push_str(&spec_slot_box(style, false)?);
                line.push_str(&spec_text_tokens(style)?);
                if let Some(background) = &style.background_color {
                    line.push_str(&format!(" bg={}", color_argb(background).ok()?));
                }
                if let Some(radius) = style.border_radius.filter(|radius| *radius > 0.0) {
                    line.push_str(&format!(" rad={}", radius.round() as i32));
                }
                line.push_str(&format!(" title={title}"));
                line
            }
            IRNode::ItemConditional {
                field,
                when_true,
                when_false,
                ..
            } => {
                let index = fields.iter().position(|name| name == field)?;
                let (true_text, true_style) = spec_text_literal(when_true)?;
                let (false_text, false_style) = spec_text_literal(when_false.as_deref()?)?;
                if true_text.contains(' ') || false_text.contains(' ') {
                    return None;
                }
                let mut line = format!("C f={index}");
                line.push_str(&spec_slot_box(true_style, true)?);
                line.push_str(&format!(" ts={}", true_style.font_size.unwrap_or(DEFAULT_FONT_SIZE)));
                line.push_str(&format!(" t={true_text} ft={false_text}"));
                if let Some(color) = &true_style.color {
                    line.push_str(&format!(" tcol={}", color_argb(color).ok()?));
                }
                if let Some(color) = &false_style.color {
                    line.push_str(&format!(" fcol={}", color_argb(color).ok()?));
                }
                line
            }
            _ => return None,
        };
        spec.push('\n');
        spec.push_str(&line);
    }
    Some(spec)
}

/// 尺寸变成「dp 整数」：百分比 100% → MATCH_PARENT(-1)，写成 wrap(-2) 之外的空缺由调用方给默认值。
fn spec_length(dimension: Option<Dimension>, fallback: i32) -> Option<i32> {
    match dimension {
        None => Some(fallback),
        Some(Dimension::Points(points)) => Some(points.round() as i32),
        Some(Dimension::Percent(fraction)) if fraction >= 0.999 => Some(-1),
        Some(Dimension::Percent(_)) => None,
    }
}

/// 行内控件的宽高：文字没写高度时按「字号 × 1.2」给固定高度，和 native 对齐。
fn spec_slot_box(style: &InlineStyle, label: bool) -> Option<String> {
    let width = spec_length(style.width, -2)?;
    let height = match style.height {
        Some(_) => spec_length(style.height, -2)?,
        None if label => {
            (style.font_size.unwrap_or(DEFAULT_FONT_SIZE) * LABEL_LINE_HEIGHT).round() as i32
        }
        None => -2,
    };
    Some(format!(" w={width} h={height}"))
}

fn spec_text_tokens(style: &InlineStyle) -> Option<String> {
    let mut tokens = String::new();
    tokens.push_str(&format!(
        " ts={}",
        style.font_size.unwrap_or(DEFAULT_FONT_SIZE)
    ));
    if let Some(color) = &style.color {
        tokens.push_str(&format!(" col={}", color_argb(color).ok()?));
    }
    Some(tokens)
}

/// 行内条件的两个分支必须是字面量 Text，把文字和样式一起取出来。
fn spec_text_literal(node: &IRNode) -> Option<(String, &InlineStyle)> {
    match node {
        IRNode::Text { content, style } => Some((content.clone(), style)),
        _ => None,
    }
}

/// 行模板里用到的按钮动作下标：rebind 时按 action 定位按钮、重算行号。
fn row_action_indices(node: &IRNode) -> Vec<usize> {
    let mut out = Vec::new();
    collect_row_actions(node, &mut out);
    out.sort_unstable();
    out.dedup();
    out
}

fn collect_row_actions(node: &IRNode, out: &mut Vec<usize>) {
    match node {
        IRNode::Button {
            state_action: Some(action),
            ..
        } => out.push(*action),
        IRNode::View { children, .. }
        | IRNode::VStack { children, .. }
        | IRNode::ScrollView { children, .. } => {
            for child in children {
                collect_row_actions(child, out);
            }
        }
        IRNode::ItemConditional {
            when_true,
            when_false,
            ..
        }
        | IRNode::Conditional {
            when_true,
            when_false,
            ..
        } => {
            collect_row_actions(when_true, out);
            if let Some(node) = when_false {
                collect_row_actions(node, out);
            }
        }
        _ => {}
    }
}

/// 行模板里的行内条件字段：这些字段一变，行内分支就换了，只能整行重建。
fn row_conditional_fields(node: &IRNode) -> Vec<String> {
    let mut out = Vec::new();
    collect_row_conditionals(node, &mut out);
    out.dedup();
    out
}

fn collect_row_conditionals(node: &IRNode, out: &mut Vec<String>) {
    match node {
        IRNode::ItemConditional {
            field,
            when_true,
            when_false,
            ..
        } => {
            out.push(field.clone());
            collect_row_conditionals(when_true, out);
            if let Some(node) = when_false {
                collect_row_conditionals(node, out);
            }
        }
        IRNode::View { children, .. }
        | IRNode::VStack { children, .. }
        | IRNode::ScrollView { children, .. } => {
            for child in children {
                collect_row_conditionals(child, out);
            }
        }
        IRNode::Conditional {
            when_true,
            when_false,
            ..
        } => {
            collect_row_conditionals(when_true, out);
            if let Some(node) = when_false {
                collect_row_conditionals(node, out);
            }
        }
        _ => {}
    }
}

fn is_positioned(node: &IRNode) -> bool {
    let style = node_style(node);
    style.position == Position::Absolute
        || style.left.is_some()
        || style.right.is_some()
        || style.top.is_some()
        || style.bottom.is_some()
}

fn count_conditionals(nodes: &[IRNode]) -> usize {
    nodes.iter().map(count_conditional_node).sum()
}

fn count_conditional_node(node: &IRNode) -> usize {
    match node {
        IRNode::Conditional {
            when_true,
            when_false,
            ..
        } => {
            1 + count_conditional_node(when_true)
                + when_false
                    .as_deref()
                    .map(count_conditional_node)
                    .unwrap_or_default()
        }
        IRNode::View { children, .. }
        | IRNode::VStack { children, .. }
        | IRNode::ScrollView { children, .. } => children.iter().map(count_conditional_node).sum(),
        _ => 0,
    }
}

/// CSS 颜色 → Android ARGB；命名色沿用 iOS 系统色的取值，保证两端观感一致。
fn color_argb(value: &str) -> Result<i32, String> {
    if let Some((red, green, blue, alpha)) = parse_functional_color(value)? {
        let channel = |value: f64| (value * 255.0).round().clamp(0.0, 255.0) as u32;
        return Ok(((channel(alpha) << 24)
            | (channel(red) << 16)
            | (channel(green) << 8)
            | channel(blue)) as i32);
    }
    let (red, green, blue) = match value.trim().to_ascii_lowercase().as_str() {
        "black" => (0, 0, 0),
        "white" => (255, 255, 255),
        "red" => (255, 59, 48),
        "blue" => (0, 122, 255),
        "green" => (52, 199, 89),
        "gray" | "grey" => (142, 142, 147),
        "clear" | "transparent" => return Ok(0),
        other => return Err(format!("unsupported inline color: {other}")),
    };
    Ok((0xFF00_0000u32 | (red << 16) | (green << 8) | blue) as i32)
}
