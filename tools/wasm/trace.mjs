// 在 node 里跑一遍 out.wasm，用打点桩记录 dom.* 调用，排查渲染为空的原因。
import { readFileSync } from 'node:fs';

const bytes = readFileSync('out.wasm');
let nextId = 1;
const calls = [];
const handlers = {};
const dom = new Proxy(
  {},
  {
    get(_, name) {
      if (handlers[name]) return handlers[name];
      return (...args) => {
        calls.push([name, args]);
        if (name === 'class') return nextId++;
        if (name === 'alloc' || name === 'string' || name === 'color' || name === 'anchor') {
          return nextId++;
        }
        return nextId++;
      };
    },
  }
);
handlers.attach = (id) => {
  calls.push(['attach', [id]]);
};

const imports = {
  dom,
  env: new Proxy({}, {
    get(_, name) {
      if (typeof name === 'string' && !name.startsWith('dom.')) console.log('ENVGET', name);
      const op = typeof name === 'string' && name.startsWith('dom.') ? name.slice(4) : null;
      if (op) return dom[op];
      if (name === 'snprintf') {
        return (buf, size, fmt, ...rest) => {
          console.log('SNPRINTF', buf, String(size), fmt, rest.map(String).join(','));
          return 0;
        };
      }
      return () => 0;
    },
  }),
  wasi_snapshot_preview1: new Proxy({}, { get: () => () => 0 }),
};

try {
  const { instance } = await WebAssembly.instantiate(bytes, imports);
  console.log('exports:', Object.keys(instance.exports).slice(0, 20).join(', '));
  const main = instance.exports.main;
  if (typeof main !== 'function') throw new Error('no main export');
  const rc = main();
  console.log('main rc =', rc);
  console.log('dom calls:', calls.length);
  for (const [name, args] of calls.slice(0, 60)) {
    console.log(name, JSON.stringify(args));
  }
} catch (error) {
  console.error('FAILED:', error.message);
  console.error('calls so far:', calls.length, JSON.stringify(calls.slice(-8)));
}
