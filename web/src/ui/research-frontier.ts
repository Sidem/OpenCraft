// What the research screen shows (`docs/TECH_TREE.md` section 10): the frontier grid's cells, which techs the player
// has discovered, which rings open up wide, and the prerequisite chain behind a tech. Pure functions over plain
// arrays, no DOM or engine: tested in Node (`web/test/research-frontier.test.mjs`).
// Discovery rule: a tech shows once it is done, available, being researched or queued; a locked tech in an open ring
// whose missing needs are all available shows as a faint "next" teaser; everything else stays hidden, so the tree
// reveals itself as the player goes. Rings show up to the first closed one (its gate), never past it.

/** How a tech appears: fully, as a one-step-ahead teaser, or not at all. */
export type Seen = 'shown' | 'next' | 'hidden';

/** The facts discovery needs about one tech. */
export interface TechFacts {
  done: boolean;
  available: boolean;
  /** Being researched or waiting in the queue. */
  planned: boolean;
  ring: number;
  needs: number[];
}

/** Most ring columns open wide at once; the rest fold to a column of dots. */
export const MAX_WIDE = 2;

/** Every tech's place: `cells[wedge][ring]`, each sorted so a prerequisite comes before what needs it. */
export function bucket(needs: number[][], rings: number[], wedges: number[], ringCount: number, wedgeCount: number): number[][][] {
  const depth = Array<number>(needs.length).fill(-1);
  const depthOf = (t: number): number => {
    if (depth[t] >= 0) return depth[t];
    depth[t] = 0;
    for (const p of needs[t]) if (rings[p] === rings[t]) depth[t] = Math.max(depth[t], depthOf(p) + 1);
    return depth[t];
  };
  const cells = Array.from({ length: wedgeCount }, () => Array.from({ length: ringCount }, (): number[] => []));
  for (let t = 0; t < needs.length; t++) {
    const w = Math.min(wedgeCount - 1, Math.max(0, wedges[t]));
    const r = Math.min(ringCount - 1, Math.max(0, rings[t]));
    cells[w][r].push(t);
  }
  for (const row of cells) for (const cell of row) cell.sort((a, b) => depthOf(a) - depthOf(b) || a - b);
  return cells;
}

/** Whether each tech is shown, teased as next, or hidden (see the header). */
export function discover(facts: TechFacts[], ringOpen: boolean[]): Seen[] {
  const shown = facts.map((f) => f.done || f.available || f.planned);
  return facts.map((f, t) => {
    if (shown[t]) return 'shown';
    if (!ringOpen[f.ring]) return 'hidden';
    const missing = f.needs.filter((n) => !facts[n].done);
    return missing.length > 0 && missing.every((n) => facts[n].available || facts[n].planned) ? 'next' : 'hidden';
  });
}

/** The rings the grid has columns for: every open ring and the first closed one (its gate); none past it. */
export function visibleRings(ringOpen: boolean[]): number[] {
  const out: number[] = [];
  for (let r = 0; r < ringOpen.length; r++) {
    out.push(r);
    if (!ringOpen[r]) break;
  }
  return out;
}

/**
 * Which visible rings open wide: open rings with something left to do, newest first, at most `MAX_WIDE`; then the
 * closed gate ring if there is room; if none qualifies, the newest open ring. `left[r]`: unfinished techs in ring r.
 */
export function autoWide(ringOpen: boolean[], left: number[]): Set<number> {
  const rings = visibleRings(ringOpen);
  const busy = rings.filter((r) => ringOpen[r] && left[r] > 0).reverse().slice(0, MAX_WIDE);
  const wide = new Set(busy);
  const gate = rings.find((r) => !ringOpen[r]);
  if (gate !== undefined && wide.size < MAX_WIDE) wide.add(gate);
  if (wide.size === 0 && rings.length > 0) wide.add(rings[rings.length - 1]);
  return wide;
}

/** Every tech `t` needs, directly or through others (not `t` itself). */
export function ancestors(needs: number[][], t: number): Set<number> {
  const seen = new Set<number>();
  const walk = (o: number) => {
    for (const p of needs[o]) if (!seen.has(p)) (seen.add(p), walk(p));
  };
  walk(t);
  return seen;
}

/** The techs that need `t` directly. */
export function dependents(needs: number[][], t: number): number[] {
  return needs.flatMap((ns, o) => (ns.includes(t) ? [o] : []));
}

/** The prerequisite links to draw for `t`: every link inside its chain, then `t` to what it leads to. */
export function chainLinks(needs: number[][], t: number): { from: number; to: number; ahead: boolean }[] {
  const chain = ancestors(needs, t);
  chain.add(t);
  const links: { from: number; to: number; ahead: boolean }[] = [];
  for (const c of chain) for (const p of needs[c]) if (chain.has(p)) links.push({ from: p, to: c, ahead: false });
  for (const d of dependents(needs, t)) links.push({ from: t, to: d, ahead: true });
  return links;
}
