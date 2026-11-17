//! 视图后端层：视图代码生成和平台之间只留三个入口。
//!
//! 1. [`ViewBackend::new_object`] / [`ViewBackend::new_view`]：建对象（视图、颜色、字体、字符串…）；
//! 2. [`ViewBackend::call_ref`]：取值型消息（返回句柄）；
//! 3. [`ViewBackend::call_void`]：置值型消息（无返回值）。
//!
//! Objective-C 后端把三个入口都发到 `objc_msgSend`（iOS / macOS 行为与以前完全一致）；
//! DOM 后端（wasm 目标）按 [`ROUTES`] 把选择器翻译成 `dom.*` import 调用。
//!
//! 映射表就是后端契约的清单：表里没有的选择器直接报错，不会生成一个「链得出来但什么都不画」
//! 的假成功产物。表里登记成 `None` 的选择器是**明确忽略**的（例如 Auto Layout 约束：
//! DOM 目标用编译期算好的 frame 绝对定位，不重复走一遍约束系统）。

use std::cell::RefCell;
use std::collections::HashMap;

use inkwell::AddressSpace;
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::types::{BasicMetadataTypeEnum, BasicType, BasicTypeEnum, StructType};
use inkwell::values::{
    BasicMetadataValueEnum, BasicValueEnum, FloatValue, FunctionValue, PointerValue,
};

/// wasm 目标的逻辑视口。布局在编译期按它算 frame，浏览器里由 CSS 绝对定位落位。
pub(crate) const DOM_VIEWPORT_WIDTH: f64 = 412.0;
pub(crate) const DOM_VIEWPORT_HEIGHT: f64 = 900.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Kind {
    Objc,
    Dom,
}

/// DOM 桥一个入参的翻译方式。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Arg {
    /// 句柄（视图 / 颜色 / 字体 / 字符串…）：指针按 i32 传给桥
    Handle,
    /// 选择器句柄（`stateActionN:` 之类）：DOM 侧就是动作编号
    Selector,
    /// 全局 C 字符串常量，桥按 NUL 结尾读
    CStr,
    F64,
    I32,
    /// CGRect → 四个 f64（x, y, width, height）
    Rect,
    /// CGSize → 两个 f64（width, height）
    Size,
    /// UIEdgeInsets → 四个 f64（top, left, bottom, right）
    Edges,
    /// 这个实参不传给桥（目标、事件掩码之类）
    Ignore,
}

/// 选择器 → DOM 桥操作。
pub(crate) struct Route {
    pub selector: &'static str,
    /// `None`：明确忽略（约束、原生镀铬等）
    pub op: Option<&'static str>,
    pub args: &'static [Arg],
    /// 桥函数末尾再加一个参数字符串（选择器名）：命名色、锚点边这类需要知道
    /// 「调用的是哪个选择器」的映射用它。
    pub name_arg: bool,
}

const fn route(selector: &'static str, op: &'static str, args: &'static [Arg]) -> Route {
    Route {
        selector,
        op: Some(op),
        args,
        name_arg: false,
    }
}

const fn named(selector: &'static str, op: &'static str, args: &'static [Arg]) -> Route {
    Route {
        selector,
        op: Some(op),
        args,
        name_arg: true,
    }
}

const fn ignore(selector: &'static str) -> Route {
    Route {
        selector,
        op: None,
        args: &[],
        name_arg: false,
    }
}

/// 选择器映射表。新增选择器时先想清楚它在 DOM 上是什么，再进表。
pub(crate) const ROUTES: &[Route] = &[
    // ---- 建对象 ----
    route("alloc", "alloc", &[]),
    route("init", "init", &[]),
    route("initWithFrame:", "initView", &[Arg::Rect]),
    route("initWithUTF8String:", "string", &[Arg::CStr]),
    route("initWithString:", "stringCopy", &[Arg::Handle]),
    route("initWithImage:", "initImage", &[Arg::Handle]),
    route("copy", "copy", &[]),
    route("buttonWithType:", "button", &[Arg::I32]),
    route("imageNamed:", "image", &[Arg::Handle]),
    route("titleLabel", "titleLabel", &[]),
    route("stringValue", "stringValue", &[]),
    route("text", "textValue", &[]),
    route("longLongValue", "numberValue", &[]),
    route("boolValue", "boolValue", &[]),
    route("isEqualToString:", "isEqual", &[Arg::Handle]),
    route(
        "dictionaryWithObject:forKey:",
        "attributes",
        &[Arg::Handle, Arg::Handle],
    ),
    route(
        "initWithString:attributes:",
        "attributed",
        &[Arg::Handle, Arg::Handle],
    ),
    // ---- 颜色 / 字体 ----
    route(
        "colorWithRed:green:blue:alpha:",
        "color",
        &[Arg::F64, Arg::F64, Arg::F64, Arg::F64],
    ),
    named("blackColor", "colorNamed", &[]),
    named("whiteColor", "colorNamed", &[]),
    named("redColor", "colorNamed", &[]),
    named("blueColor", "colorNamed", &[]),
    named("greenColor", "colorNamed", &[]),
    named("grayColor", "colorNamed", &[]),
    named("clearColor", "colorNamed", &[]),
    route("systemFontOfSize:", "fontOfSize", &[Arg::F64]),
    route("boldSystemFontOfSize:", "fontOfSize", &[Arg::F64]),
    // ---- layer / 颜色转换 ----
    route("layer", "layer", &[]),
    route("CGColor", "cgColor", &[]),
    // ---- Auto Layout 锚点（DOM 侧只记录，不参与布局）----
    named("heightAnchor", "anchor", &[]),
    named("widthAnchor", "anchor", &[]),
    named("leadingAnchor", "anchor", &[]),
    named("trailingAnchor", "anchor", &[]),
    named("topAnchor", "anchor", &[]),
    named("bottomAnchor", "anchor", &[]),
    named("centerXAnchor", "anchor", &[]),
    named("centerYAnchor", "anchor", &[]),
    route("safeAreaLayoutGuide", "identity", &[]),
    route("contentLayoutGuide", "identity", &[]),
    route("frameLayoutGuide", "identity", &[]),
    route("contentView", "identity", &[]),
    route("documentView", "identity", &[]),
    route("constraintEqualToConstant:", "constraintConst", &[Arg::F64]),
    route("setConstant:", "constraintConstant", &[Arg::F64]),
    route(
        "constraintEqualToAnchor:",
        "constraintAnchor",
        &[Arg::Handle],
    ),
    route(
        "constraintEqualToAnchor:constant:",
        "constraintAnchorConst",
        &[Arg::Handle, Arg::F64],
    ),
    route(
        "constraintEqualToAnchor:multiplier:",
        "constraintAnchorMul",
        &[Arg::Handle, Arg::F64],
    ),
    route(
        "constraintEqualToAnchor:multiplier:constant:",
        "constraintAnchorMulConst",
        &[Arg::Handle, Arg::F64, Arg::F64],
    ),
    // ---- 视图树 ----
    route("addSubview:", "addChild", &[Arg::Handle]),
    route("addArrangedSubview:", "addFlowChild", &[Arg::Handle]),
    route("subviews", "subviews", &[]),
    // 列表重建的前半步：把旧行整批摘掉，DOM 侧就是清空子节点
    route("makeObjectsPerformSelector:", "clearChildren", &[Arg::Ignore]),
    route("removeFromSuperview", "remove", &[]),
    route("setHidden:", "hidden", &[Arg::I32]),
    route("setTag:", "tag", &[Arg::I32]),
    // ---- 文本与样式 ----
    route("setText:", "setText", &[Arg::Handle]),
    route("setStringValue:", "setText", &[Arg::Handle]),
    route("setTitle:", "setText", &[Arg::Handle]),
    route("setTitle:forState:", "setText", &[Arg::Handle, Arg::Ignore]),
    route("setTextColor:", "textColor", &[Arg::Handle]),
    route("setTitleColor:forState:", "textColor", &[Arg::Handle, Arg::Ignore]),
    route("setContentTintColor:", "textColor", &[Arg::Handle]),
    route("setPlaceholderString:", "placeholder", &[Arg::Handle]),
    route("setBackgroundColor:", "backgroundColor", &[Arg::Handle]),
    route("setTextAlignment:", "textAlign", &[Arg::I32]),
    route("setFont:", "setFont", &[Arg::Handle]),
    route("setAlpha:", "alpha", &[Arg::F64]),
    route("setAlphaValue:", "alpha", &[Arg::F64]),
    route("setPlaceholder:", "placeholder", &[Arg::Handle]),
    route("setAttributedPlaceholder:", "placeholder", &[Arg::Handle]),
    route("setBorderWidth:", "borderWidth", &[Arg::F64]),
    route("setBorderColor:", "borderColor", &[Arg::Handle]),
    route("setCornerRadius:", "cornerRadius", &[Arg::F64]),
    route("setFrame:", "frame", &[Arg::Rect]),
    route("setContentSize:", "contentSize", &[Arg::Size]),
    // ---- 容器（UIStackView 的排布在 DOM 上就是 flex）----
    route("setAxis:", "stackAxis", &[Arg::I32]),
    route("setSpacing:", "stackSpacing", &[Arg::F64]),
    route("setDistribution:", "stackDistribution", &[Arg::I32]),
    route("setLayoutMargins:", "layoutMargins", &[Arg::Edges]),
    // ---- 事件 ----
    route(
        "addTarget:action:forControlEvents:",
        "callback",
        &[Arg::Ignore, Arg::Selector, Arg::Ignore],
    ),
    // ---- 明确忽略：框架已经用 frame 布局，这些原生细节在 DOM 上没有对应物 ----
    ignore("setTranslatesAutoresizingMaskIntoConstraints:"),
    ignore("addConstraint:"),
    ignore("addConstraints:"),
    ignore("setActive:"),
    ignore("activateConstraints:"),
    ignore("setDelegate:"),
    ignore("setTarget:"),
    ignore("setAction:"),
    ignore("setContentEdgeInsets:"),
    ignore("setBorderStyle:"),
    ignore("setBordered:"),
    ignore("setBezeled:"),
    ignore("setEditable:"),
    ignore("setSelectable:"),
    ignore("setDrawsBackground:"),
    ignore("setFocusRingType:"),
    ignore("setWantsLayer:"),
    ignore("setAlignment:"),
    ignore("setContentMode:"),
    ignore("setClipsToBounds:"),
    ignore("setUserInteractionEnabled:"),
    ignore("setAdjustsFontSizeToFitWidth:"),
    ignore("setMinimumScaleFactor:"),
    ignore("setNumberOfLines:"),
    ignore("setLineBreakMode:"),
    ignore("sizeToFit"),
    ignore("layoutIfNeeded"),
    ignore("setNeedsLayout"),
    ignore("setScrollEnabled:"),
    ignore("setShowsVerticalScrollIndicator:"),
    ignore("setAlwaysBounceVertical:"),
    ignore("setBounces:"),
    ignore("setContentOffset:"),
    ignore("setContentInset:"),
    ignore("setScrollIndicatorInsets:"),
    ignore("setShowsHorizontalScrollIndicator:"),
    ignore("setDirectionalLockEnabled:"),
    ignore("setLayoutMarginsRelativeArrangement:"),
];

fn route_for(selector: &str) -> Option<&'static Route> {
    ROUTES.iter().find(|route| route.selector == selector)
}

/// 三个入口 + Objective-C runtime / DOM import 的声明。
///
/// 不持有 `Builder`（inkwell 的 Builder 既不是 Copy 也不是 Clone），
/// 由调用方借用传进来，`imports` 缓存放在调用方。
pub(crate) struct ViewBackend<'ctx, 'a> {
    kind: Kind,
    context: &'ctx Context,
    module: &'a Module<'ctx>,
    builder: &'a Builder<'ctx>,
    imports: &'a RefCell<HashMap<&'static str, FunctionValue<'ctx>>>,
}

impl<'ctx, 'a> ViewBackend<'ctx, 'a> {
    pub(crate) fn new(
        kind: Kind,
        context: &'ctx Context,
        module: &'a Module<'ctx>,
        builder: &'a Builder<'ctx>,
        imports: &'a RefCell<HashMap<&'static str, FunctionValue<'ctx>>>,
    ) -> Self {
        Self {
            kind,
            context,
            module,
            builder,
            imports,
        }
    }

    fn objc_get_class_fn(&self) -> FunctionValue<'ctx> {
        let ptr_type = self.ptr_type();
        self.module
            .get_function("objc_getClass")
            .unwrap_or_else(|| {
                self.module.add_function(
                    "objc_getClass",
                    ptr_type.fn_type(&[ptr_type.into()], false),
                    None,
                )
            })
    }

    fn sel_register_name_fn(&self) -> FunctionValue<'ctx> {
        let ptr_type = self.ptr_type();
        self.module
            .get_function("sel_registerName")
            .unwrap_or_else(|| {
                self.module.add_function(
                    "sel_registerName",
                    ptr_type.fn_type(&[ptr_type.into()], false),
                    None,
                )
            })
    }

    fn objc_msg_send_fn(&self) -> FunctionValue<'ctx> {
        let ptr_type = self.ptr_type();
        self.module
            .get_function("objc_msgSend")
            .unwrap_or_else(|| {
                self.module.add_function(
                    "objc_msgSend",
                    ptr_type.fn_type(&[ptr_type.into(), ptr_type.into()], true),
                    None,
                )
            })
    }

    pub(crate) fn is_dom(&self) -> bool {
        self.kind == Kind::Dom
    }

    // ---------- 入口 0：类与选择器句柄 ----------

    /// 类句柄：ObjC 是 `objc_getClass`，DOM 是 `dom.class("UIView")`。
    pub(crate) fn class_handle(&self, name: &str) -> Result<PointerValue<'ctx>, String> {
        if self.is_dom() {
            let text = self
                .builder
                .build_global_string_ptr(name, "dom_class_name")
                .map_err(|error| error.to_string())?
                .as_pointer_value();
            let class_route = Route {
                selector: "class",
                op: Some("class"),
                args: &[Arg::CStr],
                name_arg: false,
            };
            let handle = self
                .dom_op_ref(&class_route, "class", &[text.into()], &[])?
                .ok_or_else(|| format!("dom.class({name}) returned void"))?;
            return Ok(handle);
        }
        let get_class = self.objc_get_class_fn();
        let name_ptr = self
            .builder
            .build_global_string_ptr(name, "class_name")
            .map_err(|error| error.to_string())?;
        let call = self
            .builder
            .build_call(get_class, &[name_ptr.as_pointer_value().into()], "class")
            .map_err(|error| error.to_string())?;
        match call.try_as_basic_value() {
            inkwell::values::ValueKind::Basic(value) => Ok(value.into_pointer_value()),
            inkwell::values::ValueKind::Instruction(_) => Ok(self
                .context
                .ptr_type(AddressSpace::default())
                .const_null()),
        }
    }

    /// 选择器句柄：ObjC 是 `sel_registerName`；DOM 上只有动作选择器有意义，
    /// `stateActionN:` 直接编码成编号 N，其它选择器返回空句柄。
    pub(crate) fn selector_handle(&self, name: &str) -> Result<PointerValue<'ctx>, String> {
        if self.is_dom() {
            let ptr_type = self.context.ptr_type(AddressSpace::default());
            let Some(action) = action_index(name) else {
                return Ok(ptr_type.const_null());
            };
            let handle = self.context.i32_type().const_int(action as u64, false);
            return self
                .builder
                .build_int_to_ptr(handle, ptr_type, "action_selector")
                .map_err(|error| error.to_string());
        }
        let register = self.sel_register_name_fn();
        let name_ptr = self
            .builder
            .build_global_string_ptr(name, "selector_name")
            .map_err(|error| error.to_string())?;
        let call = self
            .builder
            .build_call(register, &[name_ptr.as_pointer_value().into()], "sel")
            .map_err(|error| error.to_string())?;
        match call.try_as_basic_value() {
            inkwell::values::ValueKind::Basic(value) => Ok(value.into_pointer_value()),
            inkwell::values::ValueKind::Instruction(_) => {
                Err("sel_registerName returned void".into())
            }
        }
    }

    pub(crate) fn action_handle(&self, action: usize) -> PointerValue<'ctx> {
        if self.is_dom() {
            let handle = self.context.i32_type().const_int(action as u64, false);
            return self
                .builder
                .build_int_to_ptr(handle, self.context.ptr_type(AddressSpace::default()), "action")
                .unwrap_or_else(|_| self.context.ptr_type(AddressSpace::default()).const_null());
        }
        self.selector_handle(&format!("stateAction{action}:"))
            .unwrap_or_else(|_| self.context.ptr_type(AddressSpace::default()).const_null())
    }

    // ---------- 入口 1：建对象 ----------

    /// `[[Class alloc] init]`
    pub(crate) fn new_object(&self, class_name: &str) -> Result<PointerValue<'ctx>, String> {
        if self.is_dom() {
            let class = self.class_handle(class_name)?;
            let instance = self
                .call_ref(class, "alloc", &[])?
                .ok_or_else(|| format!("dom.alloc({class_name}) returned void"))?;
            return self
                .call_ref(instance, "init", &[])?
                .ok_or_else(|| format!("dom.init({class_name}) returned void"));
        }
        let class = self.class_handle(class_name)?;
        let instance = self
            .call_ref(class, "alloc", &[])?
            .ok_or("alloc returned void")?;
        self.call_ref(instance, "init", &[])?.ok_or("init returned void".into())
    }

    /// `[[Class alloc] initWithFrame:rect]`
    pub(crate) fn new_view(
        &self,
        class_name: &str,
        frame: inkwell::values::StructValue<'ctx>,
    ) -> Result<PointerValue<'ctx>, String> {
        let class = self.class_handle(class_name)?;
        let instance = self
            .call_ref(class, "alloc", &[])?
            .ok_or("alloc returned void")?;
        self.call_ref(instance, "initWithFrame:", &[frame.into()])?
            .ok_or_else(|| format!("{class_name} initWithFrame: returned void"))
    }

    // ---------- 入口 2 / 3：消息 ----------

    /// 取值型消息：返回句柄。
    pub(crate) fn call_ref(
        &self,
        receiver: PointerValue<'ctx>,
        selector: &str,
        args: &[BasicMetadataValueEnum<'ctx>],
    ) -> Result<Option<PointerValue<'ctx>>, String> {
        if self.is_dom() {
            return self.dom_call_ref(selector, args, &[receiver.into()]);
        }
        let selector = self.selector_handle(selector)?;
        self.objc_call(receiver, selector, args, BasicTypeEnum::PointerType(self.ptr_type()))
            .map(|value| value.map(|value| value.into_pointer_value()))
    }

    /// 置值型消息：无返回值。
    pub(crate) fn call_void(
        &self,
        receiver: PointerValue<'ctx>,
        selector: &str,
        args: &[BasicMetadataValueEnum<'ctx>],
    ) -> Result<(), String> {
        if self.is_dom() {
            self.dom_call_void(selector, args, &[receiver.into()])?;
            return Ok(());
        }
        let selector = self.selector_handle(selector)?;
        self.objc_call_void(receiver, selector, args)
    }

    /// 返回值是标量的消息（`isEqualToString:` → BOOL、`longLongValue` → long）。
    pub(crate) fn call_scalar(
        &self,
        receiver: PointerValue<'ctx>,
        selector: &str,
        args: &[BasicMetadataValueEnum<'ctx>],
        return_type: BasicTypeEnum<'ctx>,
    ) -> Result<Option<BasicValueEnum<'ctx>>, String> {
        if self.is_dom() {
            let handle = self.dom_call_ref(selector, args, &[receiver.into()])?;
            let int_value = match handle {
                Some(value) => self
                    .builder
                    .build_ptr_to_int(value, self.context.i32_type(), "scalar")
                    .map_err(|error| error.to_string())?,
                None => self.context.i32_type().const_zero(),
            };
            return Ok(Some(self.scalar_from_i32(int_value, return_type)?));
        }
        let selector = self.selector_handle(selector)?;
        self.objc_call(receiver, selector, args, return_type)
    }

    fn scalar_from_i32(
        &self,
        value: inkwell::values::IntValue<'ctx>,
        return_type: BasicTypeEnum<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, String> {
        match return_type {
            BasicTypeEnum::IntType(int_type) if int_type.get_bit_width() <= 32 => {
                if int_type.get_bit_width() == 1 {
                    let truthy = self
                        .builder
                        .build_int_compare(
                            inkwell::IntPredicate::NE,
                            value,
                            self.context.i32_type().const_zero(),
                            "truthy",
                        )
                        .map_err(|error| error.to_string())?;
                    Ok(truthy.into())
                } else {
                    let truncated = self
                        .builder
                        .build_int_truncate(value, int_type, "scalar_trunc")
                        .map_err(|error| error.to_string())?;
                    Ok(truncated.into())
                }
            }
            BasicTypeEnum::IntType(int_type) => {
                let widened = self
                    .builder
                    .build_int_s_extend(value, int_type, "scalar_ext")
                    .map_err(|error| error.to_string())?;
                Ok(widened.into())
            }
            other => Err(format!("DOM 后端不支持标量返回类型 {other:?}")),
        }
    }

    fn ptr_type(&self) -> inkwell::types::PointerType<'ctx> {
        self.context.ptr_type(AddressSpace::default())
    }

    /// 根据实际参数类型调用 `objc_msgSend`。
    fn objc_call(
        &self,
        receiver: PointerValue<'ctx>,
        selector: PointerValue<'ctx>,
        args: &[BasicMetadataValueEnum<'ctx>],
        return_type: BasicTypeEnum<'ctx>,
    ) -> Result<Option<BasicValueEnum<'ctx>>, String> {
        let msg_send = self.objc_msg_send_fn();
        let mut param_types: Vec<BasicMetadataTypeEnum<'ctx>> =
            vec![self.ptr_type().into(), self.ptr_type().into()];
        for arg in args {
            param_types.push(metadata_type(arg)?);
        }
        let fn_type = return_type.fn_type(&param_types, false);
        let mut call_args: Vec<BasicMetadataValueEnum<'ctx>> =
            vec![receiver.into(), selector.into()];
        call_args.extend(args.iter().cloned());
        let call = self
            .builder
            .build_indirect_call(
                fn_type,
                msg_send.as_global_value().as_pointer_value(),
                &call_args,
                "msg_result",
            )
            .map_err(|error| error.to_string())?;
        Ok(match call.try_as_basic_value() {
            inkwell::values::ValueKind::Basic(value) => Some(value),
            inkwell::values::ValueKind::Instruction(_) => None,
        })
    }

    fn objc_call_void(
        &self,
        receiver: PointerValue<'ctx>,
        selector: PointerValue<'ctx>,
        args: &[BasicMetadataValueEnum<'ctx>],
    ) -> Result<(), String> {
        let msg_send = self.objc_msg_send_fn();
        let mut param_types: Vec<BasicMetadataTypeEnum<'ctx>> =
            vec![self.ptr_type().into(), self.ptr_type().into()];
        for arg in args {
            param_types.push(metadata_type(arg)?);
        }
        let fn_type = self.context.void_type().fn_type(&param_types, false);
        let mut call_args: Vec<BasicMetadataValueEnum<'ctx>> =
            vec![receiver.into(), selector.into()];
        call_args.extend(args.iter().cloned());
        self.builder
            .build_indirect_call(
                fn_type,
                msg_send.as_global_value().as_pointer_value(),
                &call_args,
                "",
            )
            .map_err(|error| error.to_string())?;
        Ok(())
    }

    // ---------- DOM 桥 ----------

    fn dom_call_ref(
        &self,
        selector: &str,
        args: &[BasicMetadataValueEnum<'ctx>],
        prefix: &[BasicMetadataValueEnum<'ctx>],
    ) -> Result<Option<PointerValue<'ctx>>, String> {
        let route = self.dom_route(selector)?;
        let Some(op) = route.op else {
            return Ok(None);
        };
        self.dom_op_ref(route, op, args, prefix)
    }

    fn dom_op_ref(
        &self,
        route: &Route,
        op: &'static str,
        args: &[BasicMetadataValueEnum<'ctx>],
        prefix: &[BasicMetadataValueEnum<'ctx>],
    ) -> Result<Option<PointerValue<'ctx>>, String> {
        let (params, call_args) = self.dom_args(route, args, prefix)?;
        let function = self.dom_import(op, &params, Some(self.context.i32_type().into()));
        let call = self
            .builder
            .build_call(function, &call_args, "dom_result")
            .map_err(|error| error.to_string())?;
        match call.try_as_basic_value() {
            inkwell::values::ValueKind::Basic(value) => {
                let handle = value.into_int_value();
                let pointer = self
                    .builder
                    .build_int_to_ptr(handle, self.ptr_type(), "dom_handle")
                    .map_err(|error| error.to_string())?;
                Ok(Some(pointer))
            }
            inkwell::values::ValueKind::Instruction(_) => Ok(None),
        }
    }

    fn dom_call_void(
        &self,
        selector: &str,
        args: &[BasicMetadataValueEnum<'ctx>],
        prefix: &[BasicMetadataValueEnum<'ctx>],
    ) -> Result<(), String> {
        let route = self.dom_route(selector)?;
        let Some(op) = route.op else {
            return Ok(());
        };
        let (params, call_args) = self.dom_args(route, args, prefix)?;
        let function = self.dom_import(op, &params, None);
        self.builder
            .build_call(function, &call_args, "")
            .map_err(|error| error.to_string())?;
        Ok(())
    }

    fn dom_route(&self, selector: &str) -> Result<&'static Route, String> {
        route_for(selector).ok_or_else(|| {
            format!(
                "wasm 后端还没接选择器 '{selector}'：先到 src/view_backend.rs 的 ROUTES 登记它的 DOM 映射"
            )
        })
    }

    /// 把 LLVM 实参翻译成桥函数的实参，并给出签名。
    fn dom_args(
        &self,
        route: &Route,
        args: &[BasicMetadataValueEnum<'ctx>],
        prefix: &[BasicMetadataValueEnum<'ctx>],
    ) -> Result<(Vec<BasicMetadataTypeEnum<'ctx>>, Vec<BasicMetadataValueEnum<'ctx>>), String> {
        let (mut params, mut call_args) = self.dom_args_inner(route, args, prefix)?;
        if route.name_arg {
            let i32_type = self.context.i32_type();
            let selector_ptr = self
                .builder
                .build_global_string_ptr(route.selector, "dom_selector_name")
                .map_err(|error| error.to_string())?
                .as_pointer_value();
            params.push(i32_type.into());
            call_args.push(self.handle_value(&selector_ptr.into())?.into());
        }
        Ok((params, call_args))
    }

    fn dom_args_inner(
        &self,
        route: &Route,
        args: &[BasicMetadataValueEnum<'ctx>],
        prefix: &[BasicMetadataValueEnum<'ctx>],
    ) -> Result<(Vec<BasicMetadataTypeEnum<'ctx>>, Vec<BasicMetadataValueEnum<'ctx>>), String> {
        if route.args.len() != args.len() {
            return Err(format!(
                "选择器 '{}' 的映射登记了 {} 个参数，实际传了 {} 个",
                route.selector,
                route.args.len(),
                args.len()
            ));
        }
        let i32_type = self.context.i32_type();
        let f64_type = self.context.f64_type();
        let mut params: Vec<BasicMetadataTypeEnum<'ctx>> =
            prefix.iter().map(|_| i32_type.into()).collect();
        let mut call_args: Vec<BasicMetadataValueEnum<'ctx>> = Vec::with_capacity(prefix.len() + 4);
        for value in prefix {
            call_args.push(self.handle_value(value)?.into());
        }
        for (kind, value) in route.args.iter().zip(args) {
            match kind {
                Arg::Ignore => {}
                Arg::Handle | Arg::Selector | Arg::CStr => {
                    params.push(i32_type.into());
                    call_args.push(self.handle_value(value)?.into());
                }
                Arg::F64 => {
                    params.push(f64_type.into());
                    call_args.push(*value);
                }
                Arg::I32 => {
                    params.push(i32_type.into());
                    call_args.push(self.i32_value(value)?.into());
                }
                Arg::Rect => {
                    for _ in 0..4 {
                        params.push(f64_type.into());
                    }
                    for field in self.rect_fields(value)? {
                        call_args.push(field.into());
                    }
                }
                Arg::Size => {
                    for _ in 0..2 {
                        params.push(f64_type.into());
                    }
                    for field in self.size_fields(value)? {
                        call_args.push(field.into());
                    }
                }
                Arg::Edges => {
                    for _ in 0..4 {
                        params.push(f64_type.into());
                    }
                    for field in self.edge_fields(value)? {
                        call_args.push(field.into());
                    }
                }
            }
        }
        Ok((params, call_args))
    }

    /// 指针实参 → i32 句柄（DOM 桥的句柄是 i32）。
    fn handle_value(
        &self,
        value: &BasicMetadataValueEnum<'ctx>,
    ) -> Result<inkwell::values::IntValue<'ctx>, String> {
        let BasicMetadataValueEnum::PointerValue(pointer) = value else {
            return Err(format!("期望句柄实参，实际是 {value:?}"));
        };
        let i32_type = self.context.i32_type();
        self.builder
            .build_ptr_to_int(*pointer, i32_type, "dom_arg")
            .map_err(|error| error.to_string())
    }

    fn i32_value(
        &self,
        value: &BasicMetadataValueEnum<'ctx>,
    ) -> Result<inkwell::values::IntValue<'ctx>, String> {
        let BasicMetadataValueEnum::IntValue(int_value) = value else {
            return Err(format!("期望整数实参，实际是 {value:?}"));
        };
        let width = int_value.get_type().get_bit_width();
        let i32_type = self.context.i32_type();
        if width == 32 {
            Ok(*int_value)
        } else if width < 32 {
            self.builder
                .build_int_z_extend(*int_value, i32_type, "dom_i32")
                .map_err(|error| error.to_string())
        } else {
            self.builder
                .build_int_truncate(*int_value, i32_type, "dom_i32")
                .map_err(|error| error.to_string())
        }
    }

    /// CGRect 常量 → 4 个 f64：先落栈再逐字段读，避免依赖常量结构体的常量折叠。
    fn rect_fields(
        &self,
        value: &BasicMetadataValueEnum<'ctx>,
    ) -> Result<[FloatValue<'ctx>; 4], String> {
        let BasicMetadataValueEnum::StructValue(rect) = value else {
            return Err(format!("期望 CGRect 实参，实际是 {value:?}"));
        };
        let rect_type = rect.get_type();
        let point_type = struct_field(rect_type, 0)?;
        let size_type = struct_field(rect_type, 1)?;
        let slot = self
            .builder
            .build_alloca(rect_type, "rect_slot")
            .map_err(|error| error.to_string())?;
        self.builder
            .build_store(slot, *rect)
            .map_err(|error| error.to_string())?;
        let x = self.load_nested_f64(slot, rect_type, point_type, 0, 0)?;
        let y = self.load_nested_f64(slot, rect_type, point_type, 0, 1)?;
        let width = self.load_nested_f64(slot, rect_type, size_type, 1, 0)?;
        let height = self.load_nested_f64(slot, rect_type, size_type, 1, 1)?;
        Ok([x, y, width, height])
    }

    fn size_fields(
        &self,
        value: &BasicMetadataValueEnum<'ctx>,
    ) -> Result<[FloatValue<'ctx>; 2], String> {
        let BasicMetadataValueEnum::StructValue(size) = value else {
            return Err(format!("期望 CGSize 实参，实际是 {value:?}"));
        };
        let size_type = size.get_type();
        let slot = self
            .builder
            .build_alloca(size_type, "size_slot")
            .map_err(|error| error.to_string())?;
        self.builder
            .build_store(slot, *size)
            .map_err(|error| error.to_string())?;
        let f64_type = self.context.f64_type();
        let width_ptr = self
            .builder
            .build_struct_gep(size_type, slot, 0, "size_w")
            .map_err(|error| error.to_string())?;
        let width = self
            .builder
            .build_load(f64_type, width_ptr, "size_w")
            .map_err(|error| error.to_string())?
            .into_float_value();
        let height_ptr = self
            .builder
            .build_struct_gep(size_type, slot, 1, "size_h")
            .map_err(|error| error.to_string())?;
        let height = self
            .builder
            .build_load(f64_type, height_ptr, "size_h")
            .map_err(|error| error.to_string())?
            .into_float_value();
        Ok([width, height])
    }

    /// UIEdgeInsets（4 个连续的 f64）→ [top, left, bottom, right]
    fn edge_fields(
        &self,
        value: &BasicMetadataValueEnum<'ctx>,
    ) -> Result<[FloatValue<'ctx>; 4], String> {
        let BasicMetadataValueEnum::StructValue(insets) = value else {
            return Err(format!("期望 UIEdgeInsets 实参，实际是 {value:?}"));
        };
        let insets_type = insets.get_type();
        let slot = self
            .builder
            .build_alloca(insets_type, "insets_slot")
            .map_err(|error| error.to_string())?;
        self.builder
            .build_store(slot, *insets)
            .map_err(|error| error.to_string())?;
        let f64_type = self.context.f64_type();
        let mut fields = Vec::with_capacity(4);
        for index in 0..4 {
            let field_ptr = self
                .builder
                .build_struct_gep(insets_type, slot, index, "insets_field")
                .map_err(|error| error.to_string())?;
            let value = self
                .builder
                .build_load(f64_type, field_ptr, "insets_value")
                .map_err(|error| error.to_string())?
                .into_float_value();
            fields.push(value);
        }
        Ok([fields[0], fields[1], fields[2], fields[3]])
    }

    fn load_nested_f64(
        &self,
        slot: PointerValue<'ctx>,
        outer: StructType<'ctx>,
        _inner: StructType<'ctx>,
        outer_index: u32,
        inner_index: u32,
    ) -> Result<FloatValue<'ctx>, String> {
        let f64_type = self.context.f64_type();
        let outer_ptr = self
            .builder
            .build_struct_gep(outer, slot, outer_index, "rect_outer")
            .map_err(|error| error.to_string())?;
        let inner_type = struct_field(outer, outer_index)?;
        let field_ptr = self
            .builder
            .build_struct_gep(inner_type, outer_ptr, inner_index, "rect_field")
            .map_err(|error| error.to_string())?;
        Ok(self
            .builder
            .build_load(f64_type, field_ptr, "rect_value")
            .map_err(|error| error.to_string())?
            .into_float_value())
    }

    fn dom_import(
        &self,
        op: &'static str,
        params: &[BasicMetadataTypeEnum<'ctx>],
        result: Option<BasicTypeEnum<'ctx>>,
    ) -> FunctionValue<'ctx> {
        if let Some(function) = self.imports.borrow().get(op) {
            return *function;
        }
        let name = format!("dom.{op}");
        let function = match self.module.get_function(&name) {
            Some(function) => function,
            None => {
                let fn_type = match result {
                    Some(result) => result.fn_type(params, false),
                    None => self.context.void_type().fn_type(params, false),
                };
                self.module.add_function(&name, fn_type, None)
            }
        };
        self.imports.borrow_mut().insert(op, function);
        function
    }

    /// 滚动事件：把容器的滚动距离回传给 wasm（`lumen_scroll(y)`）。
    pub(crate) fn dom_scroll_callback(
        &self,
        view: PointerValue<'ctx>,
        action: usize,
    ) -> Result<(), String> {
        let i32_type = self.context.i32_type();
        let function = self.dom_import(
            "scrollCallback",
            &[i32_type.into(), i32_type.into()],
            None,
        );
        let handle = self.handle_value(&view.into())?;
        self.builder
            .build_call(
                function,
                &[
                    handle.into(),
                    i32_type.const_int(action as u64, false).into(),
                ],
                "",
            )
            .map_err(|error| error.to_string())?;
        Ok(())
    }

    /// 数字 → 字符串句柄。wasm 的 varargs（`snprintf`）要额外的调用约定，
    /// DOM 后端直接把 i64 交给桥转字符串。
    pub(crate) fn dom_number_text(
        &self,
        value: inkwell::values::IntValue<'ctx>,
    ) -> Result<PointerValue<'ctx>, String> {
        let i64_type = self.context.i64_type();
        let function = self.dom_import(
            "numberText",
            &[i64_type.into()],
            Some(self.context.i32_type().into()),
        );
        let call = self
            .builder
            .build_call(function, &[value.into()], "dom_number_text")
            .map_err(|error| error.to_string())?;
        let handle = match call.try_as_basic_value() {
            inkwell::values::ValueKind::Basic(value) => value.into_int_value(),
            inkwell::values::ValueKind::Instruction(_) => {
                return Err("dom.numberText returned void".into());
            }
        };
        self.builder
            .build_int_to_ptr(handle, self.ptr_type(), "number_text")
            .map_err(|error| error.to_string())
    }

    /// 供生成入口使用：直接调一个 DOM import（例如 `dom.attach`）。
    pub(crate) fn dom_statement(
        &self,
        op: &'static str,
        args: &[BasicMetadataValueEnum<'ctx>],
    ) -> Result<(), String> {
        let (params, call_args) = self.dom_args_body(args)?;
        let function = self.dom_import(op, &params, None);
        self.builder
            .build_call(function, &call_args, "")
            .map_err(|error| error.to_string())?;
        Ok(())
    }

    fn dom_args_body(
        &self,
        args: &[BasicMetadataValueEnum<'ctx>],
    ) -> Result<(Vec<BasicMetadataTypeEnum<'ctx>>, Vec<BasicMetadataValueEnum<'ctx>>), String> {
        let i32_type = self.context.i32_type();
        let f64_type = self.context.f64_type();
        let mut params = Vec::with_capacity(args.len());
        let mut call_args = Vec::with_capacity(args.len());
        for arg in args {
            match arg {
                BasicMetadataValueEnum::IntValue(_) => {
                    params.push(i32_type.into());
                    call_args.push(self.i32_value(arg)?.into());
                }
                BasicMetadataValueEnum::FloatValue(_) => {
                    params.push(f64_type.into());
                    call_args.push(*arg);
                }
                BasicMetadataValueEnum::PointerValue(_) => {
                    params.push(i32_type.into());
                    call_args.push(self.handle_value(arg)?.into());
                }
                other => return Err(format!("dom.attach 不支持实参 {other:?}")),
            }
        }
        Ok((params, call_args))
    }
}

fn struct_field<'ctx>(ty: StructType<'ctx>, index: u32) -> Result<StructType<'ctx>, String> {
    match ty.get_field_type_at_index(index) {
        Some(BasicTypeEnum::StructType(field)) => Ok(field),
        _ => Err(format!("CGRect 字段 {index} 不是结构体")),
    }
}

fn metadata_type<'ctx>(
    value: &BasicMetadataValueEnum<'ctx>,
) -> Result<BasicMetadataTypeEnum<'ctx>, String> {
    match value {
        BasicMetadataValueEnum::IntValue(value) => Ok(value.get_type().into()),
        BasicMetadataValueEnum::FloatValue(value) => Ok(value.get_type().into()),
        BasicMetadataValueEnum::PointerValue(value) => Ok(value.get_type().into()),
        BasicMetadataValueEnum::StructValue(value) => Ok(value.get_type().into()),
        BasicMetadataValueEnum::ArrayValue(value) => Ok(value.get_type().into()),
        BasicMetadataValueEnum::VectorValue(value) => Ok(value.get_type().into()),
        BasicMetadataValueEnum::ScalableVectorValue(value) => Ok(value.get_type().into()),
        BasicMetadataValueEnum::MetadataValue(_) => {
            Err("Metadata values cannot be passed as function arguments".into())
        }
    }
}

/// `stateAction3:` → Some(3)
fn action_index(selector: &str) -> Option<usize> {
    selector
        .strip_prefix("stateAction")?
        .strip_suffix(':')?
        .parse()
        .ok()
}
