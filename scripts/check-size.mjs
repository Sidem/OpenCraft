// Line-count budgets from docs/DEV_PLAN.md section 3.1. Counts every line, blank ones included.
// Prints only files over a limit (warn at the soft limit, FAIL at the hard one) and exits 1 on any
// FAIL. Tests (tests.rs) and generated wasm bindings are exempt.
//   node scripts/check-size.mjs
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');

/** First matching rule wins. */
const RULES = [
  { kind: 'source', soft: 400, hard: 600, test: (p) => /^crates\/.+\/src\/.+\.rs$/.test(p) && !p.endsWith('/tests.rs') },
  { kind: 'source', soft: 400, hard: 600, test: (p) => /^web\/src\/.+\.ts$/.test(p) && !p.startsWith('web/src/wasm/') },
  { kind: 'css', soft: 300, hard: 500, test: (p) => /^web\/src\/.+\.css$/.test(p) },
  { kind: 'CLAUDE.md', soft: 60, hard: 100, test: (p) => path.basename(p) === 'CLAUDE.md' },
  { kind: 'plan', soft: 600, hard: 800, test: (p) => p === 'docs/DEV_PLAN.md' },
  { kind: 'doc', soft: 300, hard: 500, test: (p) => /^docs\/[^/]+\.md$/.test(p) },
];
const SKIP_DIRS = new Set(['node_modules', 'target', 'dist', '.git']);

function* walk(dir) {
  if (!fs.existsSync(dir)) return;
  for (const e of fs.readdirSync(dir, { withFileTypes: true })) {
    const p = path.join(dir, e.name);
    if (e.isDirectory()) {
      if (!SKIP_DIRS.has(e.name)) yield* walk(p);
    } else {
      yield p;
    }
  }
}

function lineCount(file) {
  const lines = fs.readFileSync(file, 'utf8').split('\n');
  if (lines[lines.length - 1] === '') lines.pop();
  return lines.length;
}

const files = [path.join(root, 'CLAUDE.md'), ...['crates', 'web/src', 'docs'].flatMap((d) => [...walk(path.join(root, d))])];
let checked = 0;
let failed = 0;
const report = [];
for (const file of files) {
  const rel = path.relative(root, file).split(path.sep).join('/');
  const rule = RULES.find((r) => r.test(rel));
  if (!rule || !fs.existsSync(file)) continue;
  checked++;
  const n = lineCount(file);
  if (n > rule.hard) {
    failed++;
    report.push(`FAIL ${rel}: ${n} lines (hard limit ${rule.hard} for ${rule.kind})`);
  } else if (n > rule.soft) {
    report.push(`warn ${rel}: ${n} lines (soft limit ${rule.soft} for ${rule.kind}; split before adding)`);
  }
}
for (const line of report) console.log(line);
if (report.length === 0) console.log(`all ${checked} files within budget`);
process.exit(failed > 0 ? 1 : 0);
