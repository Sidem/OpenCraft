// Layout of the research tree (used by research.ts): pure functions from the prerequisite lists to grid
// positions, so the screen needs no positions from the engine and a new tech finds its own place.
// A tech's column is one past its deepest prerequisite; within a column techs sit as near the height of
// their prerequisites as the rows allow.

export interface TreeLayout {
  /** Column and row of each tech (rows in whole steps, 0 at the top). */
  col: number[];
  row: number[];
  columns: number;
  rows: number;
}

/** `needs[t]` lists the techs to finish before `t`. */
export function layoutTree(needs: number[][]): TreeLayout {
  const n = needs.length;
  const col: number[] = Array(n).fill(-1);
  const depth = (t: number): number => {
    if (col[t] < 0) col[t] = needs[t].length > 0 ? 1 + Math.max(...needs[t].map(depth)) : 0;
    return col[t];
  };
  for (let t = 0; t < n; t++) depth(t);
  const columns = Math.max(0, ...col) + 1;
  const row: number[] = Array(n).fill(0);
  for (let c = 0; c < columns; c++) {
    const wanted = (t: number) => (needs[t].length > 0 ? needs[t].reduce((s, p) => s + row[p], 0) / needs[t].length : 0);
    const members = Array.from({ length: n }, (_, t) => t).filter((t) => col[t] === c);
    members.sort((a, b) => wanted(a) - wanted(b) || a - b);
    let next = 0;
    for (const t of members) {
      row[t] = Math.max(wanted(t), next);
      next = row[t] + 1;
    }
  }
  return { col, row, columns, rows: Math.ceil(Math.max(0, ...row)) + 1 };
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
