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

/// UILabel 的默认字号是 17pt，Android TextView 默认 14sp；不写 fontSize 时对齐 iOS。
const DEFAULT_FONT_SIZE: f64 = 17.0;

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
    ("addView", "ViewGroup", "addView", "(Landroid/view/View;)V"),
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
const STATIC_METHODS: &[(&str, &str, &str, &str)] = &[(
    "logError",
    "Log",
    "e",
    "(Ljava/lang/String;Ljava/lang/String;)I",
)];

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
        }
    }

    fn run(&self) -> Result<(), String> {
        self.validate()?;
        *self.text_globals.borrow_mut() = (0..self.cells.len()).map(|_| Vec::new()).collect();

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
        if let Some(config) = &self.program.scroll_fade {
            if !config.distance.is_finite() || config.distance <= 0.0 {
                return Err("ScrollView fadeDistance must be greater than zero".into());
            }
            if count_scroll_fade_targets(&self.program.nodes) > 1 {
                return Err(
                    "this compiler phase supports one ScrollView fadeTarget per App".into(),
                );
            }
            let Some(target) = find_view_by_id(&self.program.nodes, &config.target_id) else {
                return Err(format!(
                    "ScrollView fadeTarget '{}' does not match a View id",
                    config.target_id
                ));
            };
            // 没有背景色就没有可淡入的内容（子级透明度也会一起变，但主体是背景）
            if node_style(target).background_color.is_none() {
                return Err(format!(
                    "View id '{}' needs a backgroundColor for scroll fading",
                    config.target_id
                ));
            }
        }
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

        if self.program.scroll_fade.is_some() {
            self.emit_fade_init()?;
        }

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
    fn emit_fade_init(&self) -> Result<(), String> {
        let view = self.load_ptr_global("g_fade_view")?;
        let current = self
            .builder()
            .get_insert_block()
            .ok_or("missing insert block")?;
        let function = current.get_parent().ok_or("missing parent function")?;
        let skip = self
            .context()
            .append_basic_block(function, "no_fade_target");
        let apply = self.context().append_basic_block(function, "fade_hidden");
        let is_null = self
            .builder()
            .build_is_null(view, "fade_missing")
            .map_err(|error| error.to_string())?;
        self.builder()
            .build_conditional_branch(is_null, skip, apply)
            .map_err(|error| error.to_string())?;
        self.builder().position_at_end(apply);
        self.call_void(view, "setAlpha", &[JArg::F32(self.f32c(0.0))])?;
        self.builder()
            .build_unconditional_branch(skip)
            .map_err(|error| error.to_string())?;
        self.builder().position_at_end(skip);
        Ok(())
    }

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
        self.register_view_id(view, style)?;
        Ok(view)
    }

    fn register_view_id(
        &self,
        view: PointerValue<'ctx>,
        style: &InlineStyle,
    ) -> Result<(), String> {
        let Some(id) = &style.view_id else {
            return Ok(());
        };
        let Some(fade) = &self.program.scroll_fade else {
            return Ok(());
        };
        if &fade.target_id != id {
            return Ok(());
        }
        let reference = self.global_ref(view)?;
        self.store_global("g_fade_view", reference)
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
        let text = self.text_string(title)?;
        self.call_void(view, "setText", &[JArg::Obj(text)])?;
        self.call_void(view, "setSingleLine", &[JArg::I32(self.i32c(1))])?;
        self.call_void(view, "setIncludeFontPadding", &[JArg::I32(self.i32c(0))])?;
        self.apply_text_style(view, style, GRAVITY_CENTER, DEFAULT_BUTTON_FONT_SIZE)?;
        // UIButton 的标题是单行 + 中间省略
        let truncate = self.static_object_field("TruncateAt", "truncateMiddle")?;
        self.call_void(view, "setEllipsize", &[JArg::Obj(truncate)])?;
        if let Some(action) = state_action {
            let bridge = self.new_object(
                "ClickBridge",
                "clickBridgeCtor",
                &[JArg::I32(self.i32c(action as i32))],
            )?;
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
        self.register_view_id(scroll, style)?;

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

        if style.scroll_fade_target.is_some() {
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
        self.register_view_id(frame, style)?;

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

    fn strdup(&self) -> Result<FunctionValue<'ctx>, String> {
        if let Some(function) = self.module().get_function("strdup") {
            return Ok(function);
        }
        let ptr_type = self.ptr_type();
        Ok(self
            .module()
            .add_function("strdup", ptr_type.fn_type(&[ptr_type.into()], false), None))
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
            if let Some(cell) = mutation.cell() {
                let cell_index = self
                    .cell_index(cell)
                    .ok_or_else(|| format!("unknown state cell '{cell}'"))?;
                self.emit_mutation(cell_index, &mutation, text)?;
                let refresh = self.refresh_fns[cell_index];
                self.builder()
                    .build_call(refresh, &[], "")
                    .map_err(|error| error.to_string())?;
            }
            self.builder()
                .build_return(None)
                .map_err(|error| error.to_string())?;
        }
        Ok(())
    }

    fn emit_mutation(
        &self,
        cell: usize,
        mutation: &StateMutation,
        text: PointerValue<'ctx>,
    ) -> Result<(), String> {
        match mutation {
            StateMutation::Noop => Ok(()),
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
        if let Some(fade) = &self.program.scroll_fade {
            let view = self.load_ptr_global("g_fade_view")?;
            let skip = self.context().append_basic_block(function, "no_target");
            let apply = self.context().append_basic_block(function, "apply");
            let is_null = self
                .builder()
                .build_is_null(view, "fade_missing")
                .map_err(|error| error.to_string())?;
            self.builder()
                .build_conditional_branch(is_null, skip, apply)
                .map_err(|error| error.to_string())?;
            self.builder().position_at_end(apply);
            let offset = self
                .builder()
                .build_signed_int_to_float(scroll, self.f32_type(), "scroll_f32")
                .map_err(|error| error.to_string())?;
            let relative = self
                .builder()
                .build_float_sub(offset, self.f32c(fade.start as f32), "fade_offset")
                .map_err(|error| error.to_string())?;
            let raw = self
                .builder()
                .build_float_div(relative, self.f32c(fade.distance as f32), "fade_ratio")
                .map_err(|error| error.to_string())?;
            let zero = self.f32c(0.0);
            let one = self.f32c(1.0);
            let below = self
                .builder()
                .build_float_compare(FloatPredicate::OGT, raw, zero, "fade_below")
                .map_err(|error| error.to_string())?;
            let lower = self
                .builder()
                .build_select(below, raw, zero, "fade_lower")
                .map_err(|error| error.to_string())?
                .into_float_value();
            let above = self
                .builder()
                .build_float_compare(FloatPredicate::OGT, lower, one, "fade_above")
                .map_err(|error| error.to_string())?;
            let alpha = self
                .builder()
                .build_select(above, one, lower, "fade_alpha")
                .map_err(|error| error.to_string())?
                .into_float_value();
            self.call_void(view, "setAlpha", &[JArg::F32(alpha)])?;
            self.builder()
                .build_unconditional_branch(skip)
                .map_err(|error| error.to_string())?;
            self.builder().position_at_end(skip);
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
        for (index, action) in self.action_fns.iter().enumerate() {
            let hit = self
                .context()
                .append_basic_block(function, &format!("hit_{index}"));
            let next = self
                .context()
                .append_basic_block(function, &format!("next_{index}"));
            let matched = self
                .builder()
                .build_int_compare(IntPredicate::EQ, id, self.i32c(index as i32), "id_matched")
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
