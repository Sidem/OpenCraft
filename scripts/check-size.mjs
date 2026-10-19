// Budgets from docs/DEV_PLAN.md section 3.1: lines for code (every line, blank ones included), bytes for docs
// (a line budget only made doc lines longer), and bytes for the session-start read. Prints only what is over a
// limit (warn at the soft limit, FAIL at the hard one) and exits 1 on any FAIL. Tests (tests.rs), generated wasm
// bindings and docs/history/ are exempt.
//   node scripts/check-size.mjs
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');

/** First matching rule wins. */
const RULES = [
  { kind: 'source', soft: 400, hard: 600, test: (p) => /^crates\/.+\/src\/.+\.rs$/.test(p) && !p.endsWith('/tests.rs') },
  { kind: 'source', soft: 400, hard: 600, test: (p) => /^web\/src\/.+\.ts$/.test(p) && !p.startsWith('web/src/wasm/') },
  { kind: 'source', soft: 400, hard: 600, test: (p) => /^signal\/src\/.+\.ts$/.test(p) },
  { kind: 'css', soft: 300, hard: 500, test: (p) => /^web\/src\/.+\.css$/.test(p) },
  { kind: 'CLAUDE.md', soft: 60, hard: 100, test: (p) => path.basename(p) === 'CLAUDE.md' },
  { kind: 'doc', unit: 'KB', soft: 24, hard: 48, test: (p) => /^docs\/[^/]+\.md$/.test(p) },
];
/** Read at the start of every session (root CLAUDE.md says so), in KB. */
const STARTUP = { files: ['CLAUDE.md', 'docs/DEV_PLAN.md', 'docs/CODEMAP.md'], soft: 48, hard: 64 };
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

function kb(file) {
  return Math.round(fs.statSync(file).size / 1024);
}

function lineCount(file) {
  const lines = fs.readFileSync(file, 'utf8').split('\n');
  if (lines[lines.length - 1] === '') lines.pop();
  return lines.length;
}

const files = [path.join(root, 'CLAUDE.md'), ...['crates', 'web/src', 'signal/src', 'docs'].flatMap((d) => [...walk(path.join(root, d))])];
let checked = 0;
let failed = 0;
const report = [];
for (const file of files) {
  const rel = path.relative(root, file).split(path.sep).join('/');
  const rule = RULES.find((r) => r.test(rel));
  if (!rule || !fs.existsSync(file)) continue;
  checked++;
  const unit = rule.unit ?? 'lines';
  const n = unit === 'KB' ? kb(file) : lineCount(file);
  if (n > rule.hard) {
    failed++;
    report.push(`FAIL ${rel}: ${n} ${unit} (hard limit ${rule.hard} for ${rule.kind})`);
  } else if (n > rule.soft) {
    report.push(`warn ${rel}: ${n} ${unit} (soft limit ${rule.soft} for ${rule.kind}; split before adding)`);
  }
}
const startup = Math.round(STARTUP.files.reduce((sum, f) => sum + fs.statSync(path.join(root, f)).size, 0) / 1024);
if (startup > STARTUP.hard) {
  failed++;
  report.push(`FAIL session-start read: ${startup} KB (hard limit ${STARTUP.hard}: ${STARTUP.files.join(', ')})`);
} else if (startup > STARTUP.soft) {
  report.push(`warn session-start read: ${startup} KB (soft limit ${STARTUP.soft}: ${STARTUP.files.join(', ')})`);
}
for (const line of report) console.log(line);
console.log(`${report.length === 0 ? `all ${checked} files within budget; ` : ''}session-start read ${startup} KB`);
process.exit(failed > 0 ? 1 : 0);
