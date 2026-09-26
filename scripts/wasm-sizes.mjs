// Lists the largest functions in the engine's wasm, to find what made it grow.
//   npm run build:wasm, then: node scripts/wasm-sizes.mjs [count] [name filter]
// Reads cargo's output before wasm-bindgen and wasm-opt, which still has function names. Sizes are
// before wasm-opt, so they only compare with each other (and with the same list from another commit).
import { readFileSync } from 'node:fs';

const file = 'target/wasm32-unknown-unknown/release/opencraft_engine.wasm';
const count = Number(process.argv[2] ?? 30);
const filter = process.argv[3] ?? '';
const b = readFileSync(file);
let p = 8;
const leb = () => {
  let r = 0, s = 0, x;
  do {
    x = b[p++];
    r += (x & 0x7f) * 2 ** s;
    s += 7;
  } while (x & 0x80);
  return r;
};
const skipName = () => {
  const len = leb();
  p += len;
};

let imports = 0;
const sizes = [];
const names = new Map();
while (p < b.length) {
  const id = b[p++];
  const end = leb() + p;
  if (id === 2) {
    for (let i = leb(); i > 0; i--) {
      skipName();
      skipName();
      const kind = b[p++];
      if (kind === 0) (leb(), imports++);
      else if (kind === 1) (p++, b[p++] & 1 ? (leb(), leb()) : leb());
      else if (kind === 2) (b[p++] & 1 ? (leb(), leb()) : leb());
      else p += 2;
    }
  } else if (id === 10) {
    for (let i = leb(); i > 0; i--) {
      const size = leb();
      sizes.push(size);
      p += size;
    }
  } else if (id === 0 && b.subarray(p + 1, p + 1 + b[p]).toString() === 'name') {
    p += 1 + b[p];
    while (p < end) {
      const sub = b[p++];
      const subEnd = leb() + p;
      if (sub === 1) {
        for (let i = leb(); i > 0; i--) {
          const index = leb();
          const len = leb();
          names.set(index, b.subarray(p, p + len).toString());
          p += len;
        }
      }
      p = subEnd;
    }
  }
  p = end;
}

const rows = sizes
  .map((size, i) => [size, names.get(i + imports) ?? `#${i + imports}`])
  .filter(([, name]) => name.includes(filter))
  .sort((a, c) => c[0] - a[0])
  .slice(0, count);
for (const [size, name] of rows) console.log(String(size).padStart(7), name.replace(/17h[0-9a-f]{16}E$/, ''));
