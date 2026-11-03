// Checks the built bundle (dist/) against what itch.io's HTML5 hosting needs, then writes itch/build/opencraft-html5.zip
// for a manual upload (the CI deploy pushes the dist folder itself with butler).
//   npm run build && node itch/pack.mjs
// itch rules: index.html at the zip root, at most 1000 files, 500 MB unpacked, no file over 200 MB, and every asset
// URL relative, because the game runs in an iframe on another host.
import { deflateRawSync } from 'node:zlib';
import { mkdirSync, readFileSync, readdirSync, statSync, writeFileSync } from 'node:fs';
import { dirname, join, relative, resolve, sep } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const dist = join(root, 'dist');
const out = join(root, 'itch', 'build', 'opencraft-html5.zip');
const MB = 1024 * 1024;
const TABLE = Uint32Array.from({ length: 256 }, (_, n) => {
  for (let k = 0; k < 8; k++) n = n & 1 ? 0xedb88320 ^ (n >>> 1) : n >>> 1;
  return n >>> 0;
});

const files = walk(dist).sort();
const fail = (msg) => {
  console.error(`itch pack: ${msg}`);
  process.exit(1);
};
if (!files.length) fail('dist/ is empty; run `npm run build` first');
if (!files.includes(join(dist, 'index.html'))) fail('dist/index.html is missing');
if (files.length > 1000) fail(`${files.length} files; itch allows 1000`);
let total = 0;
for (const f of files) {
  const size = statSync(f).size;
  total += size;
  if (size > 200 * MB) fail(`${relative(dist, f)} is over itch's 200 MB file limit`);
}
if (total > 500 * MB) fail(`${(total / MB).toFixed(0)} MB unpacked; itch allows 500 MB`);

const absolute = /(?:src|href)\s*=\s*["']\/(?!\/)|url\(\s*["']?\/(?!\/)/;
for (const f of files.filter((p) => /\.(html|css)$/.test(p))) {
  if (absolute.test(readFileSync(f, 'utf8'))) fail(`${relative(dist, f)} has a root-absolute URL; itch serves from a sub-path`);
}

mkdirSync(dirname(out), { recursive: true });
writeFileSync(out, zip(files));
console.log(`itch pack: ${files.length} files, ${(total / MB).toFixed(1)} MB unpacked, zip ${(statSync(out).size / MB).toFixed(1)} MB -> ${relative(root, out)}`);

function walk(dir) {
  return readdirSync(dir, { withFileTypes: true }).flatMap((e) => (e.isDirectory() ? walk(join(dir, e.name)) : [join(dir, e.name)]));
}

/** A zip with forward-slash names (Windows archivers write backslashes, which itch rejects). */
function zip(paths) {
  const parts = [];
  const central = [];
  let offset = 0;
  for (const p of paths) {
    const name = Buffer.from(relative(dist, p).split(sep).join('/'));
    const data = readFileSync(p);
    const packed = deflateRawSync(data, { level: 9 });
    const crc = crc32(data);
    const head = Buffer.alloc(30);
    head.writeUInt32LE(0x04034b50, 0);
    head.writeUInt16LE(20, 4);
    head.writeUInt16LE(0x0800, 6); // UTF-8 names
    head.writeUInt16LE(8, 8); // deflate
    head.writeUInt16LE(0, 10); // time 00:00
    head.writeUInt16LE(0x5821, 12); // date 2024-01-01, fixed so the zip is reproducible
    head.writeUInt32LE(crc, 14);
    head.writeUInt32LE(packed.length, 18);
    head.writeUInt32LE(data.length, 22);
    head.writeUInt16LE(name.length, 26);
    parts.push(head, name, packed);
    const entry = Buffer.alloc(46);
    entry.writeUInt32LE(0x02014b50, 0);
    entry.writeUInt16LE(20, 4);
    head.copy(entry, 6, 4, 30); // version needed .. name length, as in the local header
    entry.writeUInt32LE(offset, 42);
    central.push(entry, name);
    offset += head.length + name.length + packed.length;
  }
  const dirSize = central.reduce((n, b) => n + b.length, 0);
  const end = Buffer.alloc(22);
  end.writeUInt32LE(0x06054b50, 0);
  end.writeUInt16LE(paths.length, 8);
  end.writeUInt16LE(paths.length, 10);
  end.writeUInt32LE(dirSize, 12);
  end.writeUInt32LE(offset, 16);
  return Buffer.concat([...parts, ...central, end]);
}

function crc32(buf) {
  let c = 0xffffffff;
  for (const b of buf) c = TABLE[(c ^ b) & 0xff] ^ (c >>> 8);
  return (c ^ 0xffffffff) >>> 0;
}
