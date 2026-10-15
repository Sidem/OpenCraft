// Layout of the research tree (used by research.ts): pure functions from the prerequisite lists and pack counts to
// grid positions, so the screen needs no positions from the engine and a new tech finds its own place.
// Techs are grouped in eras by how many science packs a unit costs (a tech is never earlier than what it needs, so
// every link runs forward). Each era is a band of columns: a tech's column is one past its deepest prerequisite in
// the same era. Within a column the order is tuned to keep links short and few of them crossing.

export interface Era {
  /** Packs per unit of the techs that define the era (the highest in it). */
  packs: number;
  /** Absolute column of the era's first column, and how many columns it has. */
  first: number;
  cols: number;
  techs: number[];
}

export interface TreeLayout {
  /** Column and row of each tech (rows in whole steps, 0 at the top) and the index of its era. */
  col: number[];
  row: number[];
  era: number[];
  eras: Era[];
  columns: number;
  rows: number;
}

const SWEEPS = 6;

/** `needs[t]` lists the techs to finish before `t`; `packs[t]` is how many science packs a unit of `t` costs. */
export function layoutTree(needs: number[][], packs: number[]): TreeLayout {
  const n = needs.length;
  const level: number[] = Array(n).fill(-1);
  const levelOf = (t: number): number => {
    if (level[t] < 0) level[t] = Math.max(packs[t], ...needs[t].map(levelOf));
    return level[t];
  };
  for (let t = 0; t < n; t++) levelOf(t);
  const levels = [...new Set(level)].sort((a, b) => a - b);
  const era = level.map((l) => levels.indexOf(l));

  const depth: number[] = Array(n).fill(-1);
  const depthOf = (t: number): number => {
    if (depth[t] < 0) depth[t] = Math.max(0, ...needs[t].filter((p) => era[p] === era[t]).map((p) => depthOf(p) + 1));
    return depth[t];
  };
  for (let t = 0; t < n; t++) depthOf(t);

  let first = 0;
  const eras: Era[] = levels.map((l, i) => {
    const techs = Array.from({ length: n }, (_, t) => t).filter((t) => era[t] === i);
    const cols = Math.max(...techs.map((t) => depth[t])) + 1;
    first += cols;
    return { packs: l, first: first - cols, cols, techs };
  });
  const col = Array.from({ length: n }, (_, t) => eras[era[t]].first + depth[t]);
  const columns = first;
  const dependents: number[][] = Array.from({ length: n }, () => []);
  needs.forEach((parents, t) => parents.forEach((p) => dependents[p].push(t)));
  const members: number[][] = Array.from({ length: columns }, () => []);
  for (let t = 0; t < n; t++) members[col[t]].push(t);

  // Order each column by the mean row of its neighbours, sweeping forward (by prerequisites) and back (by dependents).
  const row: number[] = Array(n).fill(0);
  members.forEach((m) => m.forEach((t, i) => (row[t] = i)));
  const mean = (near: number[], fallback: number) =>
    near.length > 0 ? near.reduce((s, o) => s + row[o], 0) / near.length : fallback;
  for (let sweep = 0; sweep < SWEEPS; sweep++) {
    const forward = sweep % 2 === 0;
    for (let i = 0; i < columns; i++) {
      const c = forward ? i : columns - 1 - i;
      const link = (t: number) => mean(forward ? needs[t] : dependents[t], row[t]);
      const want = new Map(members[c].map((t) => [t, link(t)]));
      members[c].sort((a, b) => want.get(a)! - want.get(b)! || row[a] - row[b]);
      let next = 0;
      for (const t of members[c]) {
        row[t] = Math.max(want.get(t)!, next);
        next = row[t] + 1;
      }
      // Pushing techs apart only moves them down; bring the column back up to where its neighbours are.
      const m = members[c];
      const drift = m.reduce((s, t) => s + row[t] - want.get(t)!, 0) / m.length;
      const lift = Math.min(Math.max(0, drift), row[m[0]]);
      for (const t of m) row[t] -= lift;
    }
  }
  const top = Math.min(...row);
  for (let t = 0; t < n; t++) row[t] -= top;
  return { col, row, era, eras, columns, rows: Math.ceil(Math.max(0, ...row)) + 1 };
}

/** Every tech reachable from `t` by following prerequisites (`up`) or dependents (not `up`), and `t` itself. */
export function related(needs: number[][], t: number): Set<number> {
  const seen = new Set<number>([t]);
  const walk = (from: number, up: boolean) => {
    for (let o = 0; o < needs.length; o++) {
      const linked = up ? needs[from].includes(o) : needs[o].includes(from);
      if (linked && !seen.has(o)) {
        seen.add(o);
        walk(o, up);
      }
    }
  };
  walk(t, true);
  walk(t, false);
  return seen;
}
