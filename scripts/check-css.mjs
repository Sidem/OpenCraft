// Each CSS class belongs to one stylesheet: a top-level rule whose selector is that class alone (`.mf-chip`,
// `.mf-chip:hover`) defines it, and two files defining the same class fail, since the later one silently restyles the
// other's elements (the map filter's chips once all landed on the close button that way). Adjusting another file's
// class is fine: a state on it (`.slot.offhand`), a context (`.offhand .slot`) or a rule inside `@media`.
//   node scripts/check-css.mjs
// Prints nothing but a count when clean; else each clash and exits 1. Run by `npm run check`.
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const WEB = path.join(root, 'web/src');

/** The classes a stylesheet defines: every top-level selector that is one class, perhaps with pseudo-classes. */
function definedClasses(css) {
  const owned = new Set();
  const text = css.replace(/\/\*[\s\S]*?\*\//g, '');
  let depth = 0;
  // Each `{` closes a rule head: the text since the last brace.
  for (const m of text.matchAll(/([^{}]*)([{}])/g)) {
    if (m[2] === '}') {
      depth = Math.max(0, depth - 1);
      continue;
    }
    if (depth++ > 0) continue;
    for (const selector of m[1].split(',')) {
      const own = selector.trim().match(/^\.([a-zA-Z][\w-]*)(?::{1,2}[\w-]+(?:\([^)]*\))?)*$/);
      if (own) owned.add(own[1]);
    }
  }
  return owned;
}

function stylesheets(dir) {
  return fs.readdirSync(dir, { withFileTypes: true }).flatMap((e) => {
    const p = path.join(dir, e.name);
    if (e.isDirectory()) return e.name === 'wasm' ? [] : stylesheets(p);
    return e.name.endsWith('.css') ? [p] : [];
  });
}

const owners = new Map();
for (const file of stylesheets(WEB)) {
  for (const c of definedClasses(fs.readFileSync(file, 'utf8'))) {
    owners.set(c, [...(owners.get(c) ?? []), path.relative(root, file).replaceAll('\\', '/')]);
  }
}
const clashes = [...owners].filter(([, files]) => files.length > 1);
if (clashes.length === 0) {
  console.log(`${owners.size} classes, each defined once`);
} else {
  for (const [c, files] of clashes) console.log(`.${c} is defined in ${files.join(' and ')}: give one its own name`);
  process.exit(1);
}
