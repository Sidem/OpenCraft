// Lints the music library (part of `npm run check`): every file in web/src/audio/music is a row in music-pieces.ts and
// every row has its file; each file is small (warn at 900 KB, fail at 1.3 MB: the old loops were 2.9 MB); and, when ffmpeg
// is installed, each file's integrated loudness is within 2 LU of the library's target (-18 LUFS), so no piece startles.
// Exits 1 on any failure. A new piece: drop the .ogg in the folder, add its row, run this.
import { spawnSync } from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const dir = path.join(root, 'web/src/audio/music');
const rows = fs.readFileSync(path.join(root, 'web/src/audio/music-pieces.ts'), 'utf8');
const WARN_BYTES = 900 * 1024;
const FAIL_BYTES = 1300 * 1024;
const TARGET_LUFS = -18;
const TOLERANCE_LU = 2;

const files = fs.readdirSync(dir).filter((f) => f.endsWith('.ogg')).sort();
const named = [...rows.matchAll(/file\('([^']+)'\)/g)].map((m) => `${m[1]}.ogg`);
const problems = [];
const notes = [];

for (const f of files) if (!named.includes(f)) problems.push(`${f} is not a row in music-pieces.ts`);
for (const n of named) if (!files.includes(n)) problems.push(`music-pieces.ts names ${n}, which is missing`);

const ffmpeg = spawnSync('ffmpeg', ['-version'], { encoding: 'utf8' });
const haveFfmpeg = ffmpeg.status === 0;
let total = 0;
for (const f of files) {
  const bytes = fs.statSync(path.join(dir, f)).size;
  total += bytes;
  if (bytes > FAIL_BYTES) problems.push(`${f} is ${Math.round(bytes / 1024)} KB (limit ${FAIL_BYTES / 1024})`);
  else if (bytes > WARN_BYTES) notes.push(`${f} is ${Math.round(bytes / 1024)} KB (warn at ${WARN_BYTES / 1024})`);
  if (!haveFfmpeg) continue;
  const r = spawnSync('ffmpeg', ['-nostats', '-i', path.join(dir, f), '-af', 'ebur128', '-f', 'null', '-'], {
    encoding: 'utf8',
    maxBuffer: 64 * 1024 * 1024,
  });
  const lufs = [...`${r.stderr}`.matchAll(/^\s+I:\s+(-?[\d.]+) LUFS/gm)].pop();
  if (!lufs) problems.push(`${f}: no loudness reading`);
  else if (Math.abs(Number(lufs[1]) - TARGET_LUFS) > TOLERANCE_LU) {
    problems.push(`${f} is ${lufs[1]} LUFS (target ${TARGET_LUFS} ± ${TOLERANCE_LU})`);
  }
}

for (const n of notes) console.log(`warning: ${n}`);
if (problems.length) {
  for (const p of problems) console.log(p);
  process.exit(1);
}
console.log(
  `${files.length} pieces, ${Math.round(total / 1024)} KB${haveFfmpeg ? ', loudness within 2 LU' : ' (loudness not checked: no ffmpeg)'}`,
);
