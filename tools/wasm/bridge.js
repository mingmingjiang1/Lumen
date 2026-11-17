// Lumen 的 DOM 桥：wasm 里的视图调用全部落在这里。
// 句柄是 i32，用 Map 映射到真实 DOM 节点；状态与刷新逻辑仍在 wasm 内，
// 这里只负责「建节点 / 改属性 / 挂事件」，不持有任何应用状态。
(function () {
  const nodes = new Map();
  let nextId = 1;
  let memory = null;
  const encoder = new TextEncoder();
  const decoder = new TextDecoder();

  const root = document.getElementById('root');
  const byTag = { 0: 'div', 1: 'span', 2: 'button', 3: 'input' };

  function readString(ptr, len) {
    if (!memory || !ptr) return '';
    return decoder.decode(new Uint8Array(memory.buffer, ptr, len));
  }

  const dom = {
    createView(tag) {
      const el = document.createElement(byTag[tag] || 'div');
      const id = nextId++;
      nodes.set(id, el);
      if (el.tagName === 'DIV') el.style.display = 'flex';
      return id;
    },
    setText(id, ptr, len) {
      const el = nodes.get(id);
      if (el) el.textContent = readString(ptr, len);
    },
    setStyle(id, keyPtr, keyLen, valPtr, valLen) {
      const el = nodes.get(id);
      if (!el) return;
      const key = readString(keyPtr, keyLen);
      const value = readString(valPtr, valLen);
      if (key === 'background-color') el.style.backgroundColor = value;
      else if (key === 'color') el.style.color = value;
      else if (key === 'opacity') el.style.opacity = value;
      else if (key === 'border-radius') el.style.borderRadius = value;
      else if (key === 'font-size') el.style.fontSize = value;
      else if (key === 'padding') el.style.padding = value;
      else if (key === 'flex-direction') el.style.flexDirection = value;
      else el.style.setProperty(key, value);
    },
    addChild(parentId, childId) {
      const parent = parentId ? nodes.get(parentId) : root;
      const child = nodes.get(childId);
      if (parent && child) parent.appendChild(child);
    },
    setCallback(id, action) {
      const el = nodes.get(id);
      if (!el) return;
      el.addEventListener('click', () => {
        if (instance && instance.exports.lumen_action) {
          instance.exports.lumen_action(action);
        }
      });
    },
    requestFrame() {
      // 目前是同步重建模型，占位即可
    },
  };

  let instance = null;

  window.lumenStart = async function (wasmUrl) {
    const response = await fetch(wasmUrl);
    const bytes = await response.arrayBuffer();
    const imports = {
      dom,
      env: new Proxy({}, { get: () => () => 0 }),
      wasi_snapshot_preview1: new Proxy({}, { get: () => () => 0 }),
    };
    const { instance: inst } = await WebAssembly.instantiate(bytes, imports);
    instance = inst;
    memory = inst.exports.memory;
    for (const name of ['main', '_start', 'lumen_build_ui', 'lumen_create']) {
      if (typeof inst.exports[name] === 'function') {
        inst.exports[name]();
        break;
      }
    }
    if (inst.exports.malloc) window.lumenMalloc = inst.exports.malloc;
  };
})();
