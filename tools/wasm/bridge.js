// Lumen 的 DOM 桥：wasm 里的视图调用全部落在这里。
//
// 句柄是 i32，用一张表映射到真实 DOM 节点或「值对象」（颜色、字体、字符串、
// 锚点、约束）。状态与刷新逻辑仍在 wasm 内，这里只负责「建节点 / 改属性 /
// 挂事件」，不持有任何应用状态。
//
// 布局来源：各平台的布局在编译期就算好了（frame），所以这里统一用绝对定位落位，
// 不再跑一遍 CSS flex 布局；Auto Layout 的锚点/约束只记录不生效（见 view_backend
// 的 ROUTES 里登记为 ignore 的那批选择器）。
(function () {
  const table = new Map();
  let nextId = 1;
  let memory = null;
  let instance = null;
  const encoder = new TextEncoder();
  const decoder = new TextDecoder();
  const root = document.getElementById('root');

  // 布局模式：CSS 模式下节点是父容器的 flex 子项，位置由浏览器决定；
  // 默认模式沿用编译期算好的 frame，桥按绝对定位落位。
  const CSS_MODE = !!window.__LUMEN_CSS;

  // 类名 → 元素与基础样式。四端观感对齐的默认值都放在这里。
  const VIEW_BASE = CSS_MODE
    ? 'box-sizing:border-box;display:flex;flex-direction:column;align-items:stretch;'
    : 'position:absolute;box-sizing:border-box;display:flex;flex-direction:column;' +
      'align-items:flex-start;';
  const LABEL_BASE =
    (CSS_MODE ? '' : 'position:absolute;') +
    'box-sizing:border-box;display:flex;align-items:center;' +
    'white-space:nowrap;overflow:hidden;text-overflow:ellipsis;line-height:1.2;';
  const BUTTON_BASE =
    (CSS_MODE ? '' : 'position:absolute;') +
    'box-sizing:border-box;display:flex;align-items:center;' +
    'justify-content:center;appearance:none;border:none;margin:0;padding:0;' +
    'background:transparent;font:inherit;cursor:pointer;';
  const INPUT_BASE =
    (CSS_MODE ? '' : 'position:absolute;') +
    'box-sizing:border-box;border:none;outline:none;' +
    'font:inherit;line-height:1.2;background:transparent;';
  const SCROLL_BASE = CSS_MODE
    ? 'box-sizing:border-box;overflow-y:auto;overflow-x:hidden;' +
      'display:flex;flex-direction:column;'
    : 'position:absolute;box-sizing:border-box;overflow-y:auto;overflow-x:hidden;display:block;';

  const NAMED_COLORS = {
    black: '#000000',
    white: '#FFFFFF',
    red: '#FF3B30',
    blue: '#007AFF',
    green: '#34C759',
    gray: '#8E8E93',
    grey: '#8E8E93',
    clear: 'transparent',
    transparent: 'transparent',
  };

  function put(entry) {
    const id = nextId++;
    table.set(id, entry);
    return id;
  }

  function get(id) {
    return table.get(id);
  }

  function element(id) {
    const entry = get(id);
    if (!entry) return null;
    if (entry.el) return entry.el;
    if (entry.view) return element(entry.view);
    if (entry.layerOf) return element(entry.layerOf);
    return null;
  }

  function readCstring(ptr) {
    if (!memory || !ptr) return '';
    const bytes = new Uint8Array(memory.buffer);
    let end = ptr;
    while (end < bytes.length && bytes[end] !== 0) end += 1;
    return decoder.decode(bytes.subarray(ptr, end));
  }

  function cssColor(handle) {
    const entry = get(handle);
    if (!entry) return 'transparent';
    if (entry.kind === 'color') return entry.css;
    return 'transparent';
  }

  // ---------- 建对象 ----------

  const CLASS_ELEMENTS = {
    UIView: ['div', VIEW_BASE],
    NSView: ['div', VIEW_BASE],
    UIStackView: ['div', VIEW_BASE],
    NSStackView: ['div', VIEW_BASE],
    UIScrollView: ['div', SCROLL_BASE],
    NSScrollView: ['div', SCROLL_BASE],
    UILabel: ['div', LABEL_BASE],
    NSTextField: ['div', LABEL_BASE],
    UIButton: ['button', BUTTON_BASE],
    NSButton: ['button', BUTTON_BASE],
    UIImageView: ['div', VIEW_BASE + 'align-items:center;justify-content:center;color:#9aa3b2;font-size:12px;'],
    NSImageView: ['div', VIEW_BASE + 'align-items:center;justify-content:center;color:#9aa3b2;font-size:12px;'],
    UITextField: ['input', INPUT_BASE],
  };

  function makeElement(className) {
    const spec = CLASS_ELEMENTS[className];
    if (!spec) return null;
    const [tag, base] = spec;
    const el = document.createElement(tag);
    el.setAttribute('style', base);
    el.__base = base;
    if (tag === 'input') el.type = 'text';
    if (tag === 'button') el.type = 'button';
    return el;
  }

  // 滚动容器里再套一层内容层：子节点挂内容层，滚动距离由内容高度决定。
  function ensureContent(el) {
    if (el.__content) return el.__content;
    const content = document.createElement('div');
    content.setAttribute(
      'style',
      'position:relative;box-sizing:border-box;display:flex;flex-direction:column;width:100%;'
    );
    el.appendChild(content);
    el.__content = content;
    return content;
  }

  function finishView(entry) {
    const { className, x, y, width, height } = entry;
    const el = makeElement(className);
    if (!el) return put({ kind: 'unknown', className });
    // CSS 模式不落像素位置：位置和尺寸由随后发来的样式决定；默认模式仍按 frame 落位。
    if (!CSS_MODE) {
      el.style.left = `${x}px`;
      el.style.top = `${y}px`;
      el.style.width = `${width}px`;
      el.style.height = `${height}px`;
    }
    return put({ kind: 'view', className, el });
  }

  function numberHandle(ptr) {
    const entry = get(ptr);
    if (!entry) return 0;
    if (entry.kind === 'string') return Number(entry.text) || 0;
    if (entry.kind === 'number') return entry.value;
    return 0;
  }

  // 约束里的宽高真的落到 CSS 上：条件槽位隐藏就是把高度设成 0，
  // 列表容器的高度则决定滚动内容有多长。
  function applyConstraintValue(constraint) {
    if (!constraint) return;
    const anchor = get(constraint.anchor);
    if (!anchor || anchor.kind !== 'anchor') return;
    const el = element(anchor.view);
    if (!el) return;
    if (anchor.edge === 'height') el.style.height = `${constraint.value}px`;
    else if (anchor.edge === 'width') el.style.width = `${constraint.value}px`;
  }

  // 「贴住父视图某条边」是 Auto Layout 里唯一有 CSS 等价物的约束：leading/top 落到
  // left/top，trailing/bottom 落到 right/bottom。两条边都贴住时把尺寸放开成 auto，
  // 和约束里两条边同时生效的结果一致。其余约束（跨层级、和兄弟对齐）继续只记录。
  const ANCHOR_SIDES = {
    leading: 'left',
    trailing: 'right',
    top: 'top',
    bottom: 'bottom',
  };

  function applyAnchorConstraint(anchorHandle, otherHandle, constant) {
    if (CSS_MODE) return;
    const child = get(anchorHandle);
    const other = get(otherHandle);
    if (!child || child.kind !== 'anchor' || !other || other.kind !== 'anchor') return;
    const el = element(child.view);
    const host = element(other.view);
    if (!el || !host || el.parentElement !== host) return;
    const side = ANCHOR_SIDES[child.edge];
    if (!side) return;
    el.style[side] = `${constant}px`;
    const box = el.__anchorBox || (el.__anchorBox = {});
    box[side] = constant;
    if (box.left !== undefined && box.right !== undefined) el.style.width = 'auto';
    if (box.top !== undefined && box.bottom !== undefined) el.style.height = 'auto';
  }

  // 滚动容器内层：承载 padding 与 gap，自身不再设固定高度
  const CONTENT_BASE = 'box-sizing:border-box;';

  const dom = {
    // ---- 类与实例 ----
    class(ptr) {
      return put({ kind: 'class', name: readCstring(ptr) });
    },
    alloc(classHandle) {
      const entry = get(classHandle);
      const className = entry && entry.name ? entry.name : '';
      return put({ kind: 'alloc', className });
    },
    init(instanceHandle) {
      const entry = get(instanceHandle);
      const className = entry ? entry.className : '';
      if (className === 'NSString') return put({ kind: 'string', text: '' });
      if (CLASS_ELEMENTS[className]) {
        return finishView({ className, x: 0, y: 0, width: 0, height: 0 });
      }
      return put({ kind: 'unknown', className });
    },
    initView(instanceHandle, x, y, width, height) {
      const entry = get(instanceHandle);
      const className = entry ? entry.className : '';
      return finishView({ className, x, y, width, height });
    },
    string(instanceHandle, ptr) {
      return put({ kind: 'string', text: readCstring(ptr) });
    },
    stringCopy(instanceHandle, sourceHandle) {
      const entry = get(sourceHandle);
      const text = entry && entry.kind === 'string' ? entry.text : readCstring(sourceHandle);
      return put({ kind: 'string', text });
    },
    copy(instanceHandle, sourceHandle) {
      const entry = get(sourceHandle);
      if (entry && entry.kind === 'string') return put({ kind: 'string', text: entry.text });
      return sourceHandle;
    },
    button(classHandle, type) {
      return finishView({ className: 'UIButton', x: 0, y: 0, width: 0, height: 0 });
    },
    image(classHandle, nameHandle) {
      const entry = get(nameHandle);
      return put({ kind: 'image', name: entry && entry.text ? entry.text : '' });
    },
    initImage(instanceHandle, imageHandle) {
      const entry = get(instanceHandle);
      const className = entry ? entry.className : 'UIImageView';
      const view = finishView({ className, x: 0, y: 0, width: 0, height: 0 });
      const image = get(imageHandle);
      const el = element(view);
      if (el && image && image.name) el.textContent = '';
      return view;
    },
    titleLabel(buttonHandle) {
      return buttonHandle;
    },
    stringValue(viewHandle) {
      const el = element(viewHandle);
      if (el && el.tagName === 'INPUT') return put({ kind: 'string', text: el.value || '' });
      if (el) return put({ kind: 'string', text: el.textContent || '' });
      return put({ kind: 'string', text: '' });
    },
    textValue(viewHandle) {
      const el = element(viewHandle);
      if (el && el.tagName === 'INPUT') return put({ kind: 'string', text: el.value || '' });
      return put({ kind: 'string', text: el ? el.textContent || '' : '' });
    },
    numberText(value) {
      return put({ kind: 'string', text: String(value) });
    },
    numberValue(handle) {
      return numberHandle(handle);
    },
    // 布尔状态单元在 wasm 里是 NSString（"true"/"false"），取值语义跟
    // NSString boolValue 一致：按前缀认 Y/y/T/t/1-9，其余算 false。
    boolValue(handle) {
      const entry = get(handle);
      if (entry && entry.kind === 'string') {
        return /^[ \t\n]*[yYtT1-9]/.test(entry.text) ? 1 : 0;
      }
      const el = element(handle);
      return el ? 1 : 0;
    },
    isEqual(handle, otherHandle) {
      const a = get(handle);
      const b = get(otherHandle);
      const textA = a && a.kind === 'string' ? a.text : '';
      const textB = b && b.kind === 'string' ? b.text : readCstring(otherHandle);
      return textA === textB ? 1 : 0;
    },
    // ---- 颜色 / 字体 ----
    color(classHandle, r, g, b, a) {
      const channel = (v) => Math.max(0, Math.min(255, Math.round(v * 255)));
      return put({
        kind: 'color',
        css: `rgba(${channel(r)}, ${channel(g)}, ${channel(b)}, ${a})`,
      });
    },
    colorNamed(classHandle, namePtr) {
      const name = readCstring(namePtr).replace(/Color$/, '').toLowerCase();
      return put({ kind: 'color', css: NAMED_COLORS[name] || 'transparent' });
    },
    fontOfSize(classHandle, size) {
      return put({ kind: 'font', size });
    },
    layer(viewHandle) {
      return viewHandle;
    },
    cgColor(colorHandle) {
      return colorHandle;
    },
    // ---- Auto Layout：只有「贴父视图」的边约束参与布局，其余只记录 ----
    anchor(viewHandle, namePtr) {
      const edge = readCstring(namePtr).replace(/Anchor$/, '').toLowerCase();
      return put({ kind: 'anchor', view: viewHandle, edge });
    },
    identity(handle) {
      return handle;
    },
    constraintConst(anchorHandle, value) {
      const handle = put({ kind: 'constraint', anchor: anchorHandle, value });
      applyConstraintValue(get(handle));
      return handle;
    },
    constraintConstant(constraintHandle, value) {
      const constraint = get(constraintHandle);
      if (!constraint) return;
      constraint.value = value;
      applyConstraintValue(constraint);
    },
    constraintAnchor(anchorHandle, otherHandle) {
      const handle = put({ kind: 'constraint', anchor: anchorHandle, other: otherHandle, constant: 0 });
      applyAnchorConstraint(anchorHandle, otherHandle, 0);
      return handle;
    },
    constraintAnchorConst(anchorHandle, otherHandle, constant) {
      const handle = put({ kind: 'constraint', anchor: anchorHandle, other: otherHandle, constant });
      applyAnchorConstraint(anchorHandle, otherHandle, constant);
      return handle;
    },
    constraintAnchorMul(anchorHandle, otherHandle, multiplier) {
      return put({ kind: 'constraint', anchor: anchorHandle, other: otherHandle, multiplier });
    },
    constraintAnchorMulConst(anchorHandle, otherHandle, multiplier, constant) {
      return put({
        kind: 'constraint',
        anchor: anchorHandle,
        other: otherHandle,
        multiplier,
        constant,
      });
    },
    // ---- 视图树 ----
    addChild(parentHandle, childHandle) {
      const parent = element(parentHandle);
      const child = element(childHandle);
      if (!parent || !child) return;
      const host = parent.__content ? parent.__content : parent;
      host.appendChild(child);
    },
    // stack 的 arrangedSubview：跟着 flex 流排下去（行就是这样一行行落位的）
    addFlowChild(parentHandle, childHandle) {
      const parent = element(parentHandle);
      const child = element(childHandle);
      if (!parent || !child) return;
      child.style.position = 'relative';
      child.style.left = '0px';
      child.style.top = '0px';
      (parent.__content ? parent.__content : parent).appendChild(child);
    },
    attach(viewHandle) {
      const el = element(viewHandle);
      if (el && root) root.appendChild(el);
    },
    remove(viewHandle) {
      const el = element(viewHandle);
      if (el && el.parentNode) el.parentNode.removeChild(el);
    },
    subviews(viewHandle) {
      return put({ kind: 'children', view: viewHandle });
    },
    clearChildren(childrenHandle) {
      const entry = get(childrenHandle);
      const parent = element(entry && entry.kind === 'children' ? entry.view : childrenHandle);
      if (!parent) return;
      const host = parent.__content ? parent.__content : parent;
      while (host.firstChild) host.removeChild(host.firstChild);
    },
    hidden(viewHandle, hidden) {
      const el = element(viewHandle);
      if (el) el.style.display = hidden ? 'none' : el.__display || 'flex';
    },
    tag(viewHandle, tag) {
      const el = element(viewHandle);
      if (el) el.dataset.rowTag = String(tag);
    },
    // ---- 样式 ----
    // 编译期算好的 CSS 直接落到元素上。追加而不覆盖：盒模型先来，容器/滚动这类
    // 后续声明要能盖住前面的同名属性。
    style(viewHandle, ptr) {
      if (!CSS_MODE) return;
      const el = element(viewHandle);
      if (!el) return;
      const css = readCstring(Number(ptr));
      el.__css = el.__css ? `${el.__css}${css}` : css;
      el.style.cssText = (el.__base || '') + el.__css;
    },
    contentStyle(viewHandle, ptr) {
      if (!CSS_MODE) return;
      const el = element(viewHandle);
      if (!el) return;
      const content = ensureContent(el);
      content.style.cssText = CONTENT_BASE + readCstring(Number(ptr));
    },
    // ---- 文本与样式 ----
    setText(viewHandle, textHandle) {
      const el = element(viewHandle);
      const text = get(textHandle);
      if (!el) return;
      el.textContent = text && text.kind === 'string' ? text.text : '';
    },
    textColor(viewHandle, colorHandle) {
      const el = element(viewHandle);
      if (el) el.style.color = cssColor(colorHandle);
    },
    backgroundColor(viewHandle, colorHandle) {
      const el = element(viewHandle);
      if (el) el.style.backgroundColor = cssColor(colorHandle);
    },
    textAlign(viewHandle, value) {
      const el = element(viewHandle);
      if (!el) return;
      const keywords = ['left', 'center', 'right'];
      el.style.textAlign = keywords[value] || 'left';
      if (el.style.justifyContent === '' || el.style.justifyContent === 'center') {
        el.style.justifyContent =
          value === 1 ? 'center' : value === 2 ? 'flex-end' : 'flex-start';
      }
    },
    setFont(viewHandle, fontHandle) {
      const el = element(viewHandle);
      const font = get(fontHandle);
      if (el && font && font.kind === 'font') el.style.fontSize = `${font.size}px`;
    },
    alpha(viewHandle, value) {
      const el = element(viewHandle);
      if (el) el.style.opacity = String(Math.max(0, Math.min(1, value)));
    },
    placeholder(viewHandle, attributedHandle) {
      const el = element(viewHandle);
      const attributed = get(attributedHandle);
      if (!el) return;
      if (attributed && attributed.kind === 'attributed') {
        el.placeholder = attributed.text;
        if (attributed.color) el.style.setProperty('--ph', attributed.color);
      } else if (attributed && attributed.kind === 'string') {
        el.placeholder = attributed.text;
      }
    },
    attributes(classHandle, colorHandle, keyHandle) {
      return put({ kind: 'attributes', color: cssColor(colorHandle), key: get(keyHandle) });
    },
    attributed(classHandle, textHandle, attributesHandle) {
      const text = get(textHandle);
      const attributes = get(attributesHandle);
      return put({
        kind: 'attributed',
        text: text && text.kind === 'string' ? text.text : readCstring(textHandle),
        color: attributes && attributes.color ? attributes.color : null,
      });
    },
    borderWidth(viewHandle, width) {
      const el = element(viewHandle);
      if (el) el.style.borderWidth = `${width}px`;
    },
    borderColor(viewHandle, colorHandle) {
      const el = element(viewHandle);
      if (!el) return;
      el.style.borderStyle = 'solid';
      el.style.borderColor = cssColor(colorHandle);
    },
    cornerRadius(viewHandle, radius) {
      const el = element(viewHandle);
      if (el) el.style.borderRadius = `${radius}px`;
    },
    frame(viewHandle, x, y, width, height) {
      const el = element(viewHandle);
      if (!el) return;
      el.style.left = `${x}px`;
      el.style.top = `${y}px`;
      el.style.width = `${width}px`;
      el.style.height = `${height}px`;
    },
    contentSize(viewHandle, width, height) {
      const el = element(viewHandle);
      if (!el) return;
      const content = ensureContent(el);
      // CSS 模式只给下限：内容由 flex 子项撑开；默认模式沿用编译期内容高度
      if (CSS_MODE) content.style.minHeight = `${height}px`;
      else content.style.height = `${height}px`;
    },
    // ---- 容器排布 ----
    // CSS 模式下这些属性已经在样式串里了；再按控件消息写一遍会把真实方向盖掉
    // （动态子节点会把行容器当竖栈配置，直接把 row 改成 column）。
    stackAxis(viewHandle, axis) {
      if (CSS_MODE) return;
      const el = element(viewHandle);
      if (el) el.style.flexDirection = axis === 0 ? 'row' : 'column';
    },
    stackSpacing(viewHandle, spacing) {
      if (CSS_MODE) return;
      const el = element(viewHandle);
      if (el) el.style.gap = `${spacing}px`;
    },
    stackDistribution(viewHandle, distribution) {
      if (CSS_MODE) return;
      const el = element(viewHandle);
      if (!el) return;
      const keywords = [
        'flex-start', // Fill
        'space-around', // FillEqually 的近似
        'space-between', // FillProportionally 的近似
        'space-between', // EqualSpacing
        'space-around', // EqualCentering
      ];
      el.style.justifyContent = keywords[distribution] || 'flex-start';
    },
    layoutMargins(viewHandle, top, left, bottom, right) {
      if (CSS_MODE) return;
      const el = element(viewHandle);
      if (el) el.style.padding = `${top}px ${right}px ${bottom}px ${left}px`;
    },
    // ---- 事件 ----
    scrollCallback(viewHandle, action) {
      const el = element(viewHandle);
      if (!el) return;
      el.addEventListener('scroll', () => {
        if (instance && typeof instance.exports.lumen_scroll === 'function') {
          instance.exports.lumen_scroll(Math.round(el.scrollTop));
        }
      });
    },
    callback(viewHandle, action) {
      const el = element(viewHandle);
      if (!el) return;
      el.addEventListener('click', () => {
        if (instance && typeof instance.exports.lumen_action === 'function') {
          instance.exports.lumen_action(action, Number(el.dataset.rowTag || 0));
        }
      });
    },
  };

  // `snprintf` 被用来把数字单元格式化成字符串（`%lld`），wasm 里没有 libc，
  // 这里补一个够用的实现：只处理整数格式化，返回写入的字节数。
  function snprintf(bufPtr, size, formatPtr, ...args) {
    // wasm 的 i64 到 JS 是 BigInt，这里统一转成 Number 再做运算
    const capacity = Number(size);
    const format = readCstring(Number(formatPtr));
    let argIndex = 0;
    const text = format.replace(/%lld|%ld|%d/gu, () => String(args[argIndex++]));
    const bytes = encoder.encode(text);
    const limit = Math.max(0, capacity - 1);
    const base = Number(bufPtr);
    const target = new Uint8Array(memory.buffer, base, Math.min(bytes.length, limit));
    target.set(bytes.subarray(0, target.length));
    new Uint8Array(memory.buffer)[base + target.length] = 0;
    return target.length;
  }

  // rust-lld 会把 `dom.foo` 这类符号放进 "env" 命名空间（name = "dom.foo"）
  const importEnv = new Proxy(
    {},
    {
      get(_, name) {
        if (typeof name === 'string' && name.startsWith('dom.')) {
          const op = name.slice(4);
          if (dom[op]) return dom[op];
        }
        if (name === 'snprintf') return snprintf;
        return () => 0;
      },
    }
  );

  window.lumenStart = async function (wasmBytes) {
    const imports = {
      dom,
      env: importEnv,
      wasi_snapshot_preview1: new Proxy({}, { get: () => () => 0 }),
    };
    const { instance: inst } = await WebAssembly.instantiate(wasmBytes, imports);
    instance = inst;
    memory = inst.exports.memory;
    for (const name of ['main', '_start', 'lumen_build_ui', 'lumen_create']) {
      if (typeof inst.exports[name] === 'function') {
        inst.exports[name]();
        break;
      }
    }
  };

  window.lumenDecodeBase64 = function (text) {
    const binary = atob(text);
    const bytes = new Uint8Array(binary.length);
    for (let i = 0; i < binary.length; i += 1) bytes[i] = binary.charCodeAt(i);
    return bytes;
  };
})();
