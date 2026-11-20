// Visual target inspector card (#target in hud.ts): parses structured `@` directives from engine readouts and renders
// them as tier badges, glowing status pills, isometric recipe flow diagrams, mini inventory slots, segmented efficiency
// and load bars, connection pips, and stat chips. Pure parser (`parseTargetCard`) has no DOM imports so it is tested in
// web/test/target-card.test.mjs.

import { type HudControl, parseHudText } from './target-controls.ts';

export type Tone = 'ok' | 'warn' | 'err' | 'off' | 'info' | 'cyan' | 'accent';

export interface TierBadge {
  mk: number;
  itemId: number;
  kitId: number;
  kits: number;
}

export interface GridBadge {
  label: string;
  colour: string;
}

export interface ItemAmount {
  itemId: number;
  amount: number;
}

export type CardRow =
  | { kind: 'status'; tone: Tone; text: string }
  | { kind: 'recipe'; inputs: ItemAmount[]; outputs: ItemAmount[] }
  | { kind: 'items'; group: string; items: ItemAmount[] }
  | { kind: 'bar'; label: string; value: number; max: number; right: string; tone: Tone }
  | { kind: 'eff'; idle: boolean; pct: number; cause: string; lossPct: number }
  | { kind: 'pips'; label: string; used: number; total: number }
  | { kind: 'stats'; chips: string[] }
  | { kind: 'text'; text: string };

export interface ParsedTargetCard {
  tier: TierBadge | null;
  grid: GridBadge | null;
  rows: CardRow[];
  controls: HudControl[];
}

export interface CardRendererHooks {
  itemIcon(itemId: number): HTMLCanvasElement;
  itemName(itemId: number): string;
  tierColour(itemId: number): number;
}

/** Strips a trailing `" Mk2"` suffix when a visual tier badge already shows the Mk. */
export function stripMkSuffix(title: string, mk: number): string {
  const suffix = ` Mk${mk}`;
  return title.endsWith(suffix) ? title.slice(0, -suffix.length) : title;
}

/** Formats tenths of an item per minute (`300` -> `"30/m"`, `125` -> `"12.5/m"`). */
export function formatTenthsRate(tenths: number): string {
  return tenths % 10 === 0 ? `${tenths / 10}/m` : `${(tenths / 10).toFixed(1)}/m`;
}

/** Compact count for mini inventory slots (`64` -> `"64"`, `1400` -> `"1.4k"`). */
export function formatSlotCount(n: number): string {
  if (n >= 10_000) return `${Math.round(n / 1000)}k`;
  if (n >= 1000) return `${(n / 1000).toFixed(1).replace(/\.0$/, '')}k`;
  return String(n);
}

/** Parses engine target detail / line label body into header badges, visual rows, and control chips. */
export function parseTargetCard(raw: string): ParsedTargetCard {
  const { detail, controls } = parseHudText(raw);
  let tier: TierBadge | null = null;
  let grid: GridBadge | null = null;
  const rows: CardRow[] = [];
  if (detail) {
    for (const line of detail.split('\n')) {
      if (!line) continue;
      if (!line.startsWith('@')) {
        rows.push({ kind: 'text', text: line });
        continue;
      }
      const p = line.split('|');
      switch (p[0]) {
        case '@tier':
          tier = { mk: Number(p[1]) || 1, itemId: Number(p[2]) || 0, kitId: Number(p[3]) || 0, kits: Number(p[4]) || 0 };
          break;
        case '@grid':
          grid = { label: p[1] ?? '', colour: p[2] ?? '#0072b2' };
          break;
        case '@status':
          rows.push({ kind: 'status', tone: asTone(p[1]), text: p[2] ?? '' });
          break;
        case '@recipe':
          rows.push({ kind: 'recipe', inputs: parseItemList(p[1]), outputs: parseItemList(p[2]) });
          break;
        case '@items': {
          const items = parseItemList(p[2]);
          if (items.length) rows.push({ kind: 'items', group: p[1] ?? '', items });
          break;
        }
        case '@bar':
          rows.push({
            kind: 'bar',
            label: p[1] ?? '',
            value: Number(p[2]) || 0,
            max: Math.max(1, Number(p[3]) || 1),
            right: p[4] ?? '',
            tone: asTone(p[5]),
          });
          break;
        case '@eff':
          rows.push({
            kind: 'eff',
            idle: p[1] === 'idle',
            pct: Number(p[1]) || 0,
            cause: p[2] ?? '',
            lossPct: Number(p[3]) || 0,
          });
          break;
        case '@pips':
          rows.push({ kind: 'pips', label: p[1] ?? '', used: Number(p[2]) || 0, total: Number(p[3]) || 0 });
          break;
        case '@stats': {
          const chips = p.slice(1).filter(Boolean);
          if (chips.length) rows.push({ kind: 'stats', chips });
          break;
        }
        default:
          rows.push({ kind: 'text', text: line });
      }
    }
  }
  return { tier, grid, rows, controls };
}

/** Renders the header badges (`tier`, `grid`) and visual body rows into `#target`. */
export function renderTargetCard(
  titleEl: HTMLElement,
  badgesEl: HTMLElement,
  bodyEl: HTMLElement,
  title: string,
  parsed: ParsedTargetCard,
  hooks: CardRendererHooks,
): void {
  titleEl.textContent = parsed.tier ? stripMkSuffix(title, parsed.tier.mk) : title;

  const badges: HTMLElement[] = [];
  if (parsed.tier) {
    const t = parsed.tier;
    const pill = el('span', 'tcard-tier', `Mk${t.mk}`);
    const rgb = hooks.tierColour(t.itemId);
    if (rgb > 0) pill.style.setProperty('--tier-col', `#${rgb.toString(16).padStart(6, '0')}`);
    badges.push(pill);
    if (t.kitId > 0 && t.kits > 0) {
      const next = el('span', 'tcard-next');
      next.append(el('span', 'tcard-arrow-up', '↗'), iconCanvas(hooks.itemIcon(t.kitId), 15), el('span', '', `×${t.kits}`));
      badges.push(next);
    }
  }
  if (parsed.grid) {
    const g = el('span', 'tcard-grid');
    const dot = el('span', 'tcard-gdot');
    dot.style.background = parsed.grid.colour;
    g.append(dot, el('span', '', parsed.grid.label));
    badges.push(g);
  }
  badgesEl.classList.toggle('hidden', badges.length === 0);
  badgesEl.replaceChildren(...badges);

  const nodes: HTMLElement[] = [];
  for (let i = 0; i < parsed.rows.length; i++) {
    const row = parsed.rows[i];
    // Merge consecutive `@items` rows (e.g. In + Fuel + Out) into one horizontal buffer tray.
    if (row.kind === 'items') {
      const tray = el('div', 'tcard-buffers');
      while (i < parsed.rows.length && parsed.rows[i].kind === 'items') {
        const cur = parsed.rows[i] as Extract<CardRow, { kind: 'items' }>;
        tray.append(renderItemGroup(cur.group, cur.items, hooks));
        i++;
      }
      i--;
      nodes.push(tray);
      continue;
    }
    nodes.push(renderRow(row, hooks));
  }
  bodyEl.classList.toggle('hidden', nodes.length === 0);
  bodyEl.replaceChildren(...nodes);
}

function renderRow(row: Exclude<CardRow, { kind: 'items' }>, hooks: CardRendererHooks): HTMLElement {
  switch (row.kind) {
    case 'status': {
      const wrap = el('div', 'tcard-status-wrap');
      const pill = el('span', `tcard-status tcard-tone-${row.tone}`);
      pill.append(el('span', 'tcard-dot'), el('span', '', row.text));
      wrap.append(pill);
      return wrap;
    }
    case 'recipe':
      return renderRecipe(row.inputs, row.outputs, hooks);
    case 'bar':
      return renderBar(row.label, row.value, row.max, row.right, row.tone);
    case 'eff':
      return renderEfficiency(row);
    case 'pips':
      return renderPips(row.label, row.used, row.total);
    case 'stats': {
      const wrap = el('div', 'tcard-stats');
      for (const chip of row.chips) wrap.append(el('span', 'tcard-chip', chip));
      return wrap;
    }
    case 'text':
      return el('div', 'tcard-text', row.text);
  }
}

function renderRecipe(inputs: ItemAmount[], outputs: ItemAmount[], hooks: CardRendererHooks): HTMLElement {
  const wrap = el('div', 'tcard-recipe');
  const addSide = (list: ItemAmount[], out: boolean) => {
    list.forEach((entry, idx) => {
      if (idx > 0) wrap.append(el('span', 'tcard-op', '+'));
      const node = el('span', out ? 'tcard-rnode tcard-rout' : 'tcard-rnode');
      const meta = el('span', 'tcard-rmeta');
      meta.append(el('span', 'tcard-rrate', formatTenthsRate(entry.amount)), el('span', 'tcard-rname', hooks.itemName(entry.itemId)));
      node.append(iconCanvas(hooks.itemIcon(entry.itemId), 20), meta);
      wrap.append(node);
    });
  };
  addSide(inputs, false);
  wrap.append(el('span', 'tcard-arrow', '→'));
  addSide(outputs, true);
  return wrap;
}

function renderItemGroup(group: string, items: ItemAmount[], hooks: CardRendererHooks): HTMLElement {
  const box = el('div', 'tcard-bgroup');
  if (group) box.append(el('span', 'tcard-blabel', group));
  const list = el('div', 'tcard-slots');
  for (const it of items) {
    const slot = el('span', 'tcard-slot');
    slot.title = `${it.amount} ${hooks.itemName(it.itemId)}`;
    slot.append(iconCanvas(hooks.itemIcon(it.itemId), 22), el('span', 'tcard-scount', formatSlotCount(it.amount)));
    list.append(slot);
  }
  box.append(list);
  return box;
}

function renderBar(label: string, value: number, max: number, right: string, tone: Tone): HTMLElement {
  const pct = Math.max(0, Math.min(100, (value / Math.max(1, max)) * 100));
  const box = el('div', 'tcard-bar');
  const head = el('div', 'tcard-bar-head');
  head.append(el('span', 'tcard-bar-label', label), el('span', 'tcard-bar-val', right));
  const track = el('div', 'tcard-track');
  const fill = el('div', `tcard-fill tcard-fill-${tone}`);
  fill.style.width = `${pct.toFixed(1)}%`;
  track.append(fill);
  box.append(head, track);
  return box;
}

function renderEfficiency(row: Extract<CardRow, { kind: 'eff' }>): HTMLElement {
  if (row.idle) {
    const box = el('div', 'tcard-eff-idle');
    box.append(el('span', 'tcard-bar-label', 'Efficiency'), el('span', 'tcard-chip', 'Idle'));
    return box;
  }
  const box = el('div', 'tcard-bar');
  const head = el('div', 'tcard-bar-head');
  const left = el('span', 'tcard-eff-left');
  const pctTone = row.pct >= 90 ? 'ok' : row.pct >= 40 ? 'warn' : 'err';
  left.append(el('span', 'tcard-bar-label', 'Efficiency'), el('span', `tcard-eff-pct tcard-pct-${pctTone}`, `${row.pct}%`));
  head.append(left);
  if (row.cause) {
    const cause = row.cause.charAt(0).toUpperCase() + row.cause.slice(1);
    head.append(el('span', 'tcard-eff-cause', cause));
  }
  const track = el('div', 'tcard-track tcard-track-seg');
  if (row.pct > 0) {
    const work = el('div', 'tcard-fill tcard-fill-ok');
    work.style.width = `${Math.min(100, row.pct)}%`;
    track.append(work);
  }
  const loss = Math.min(100 - row.pct, row.lossPct);
  if (loss > 0) {
    const errCause = /power|fuel|water|compute/i.test(row.cause);
    const lost = el('div', `tcard-fill ${errCause ? 'tcard-fill-err' : 'tcard-fill-warn'}`);
    lost.style.width = `${loss}%`;
    track.append(lost);
  }
  box.append(head, track);
  return box;
}

function renderPips(label: string, used: number, total: number): HTMLElement {
  const row = el('div', 'tcard-pips');
  row.append(el('span', 'tcard-bar-label', label));
  const dots = el('span', 'tcard-pip-row');
  const full = used >= total && total > 0;
  for (let i = 0; i < total; i++) {
    dots.append(el('span', i < used ? (full ? 'tcard-pip tcard-pip-full' : 'tcard-pip tcard-pip-on') : 'tcard-pip'));
  }
  row.append(dots, el('span', 'tcard-bar-val', `${used} / ${total}`));
  return row;
}

function parseItemList(raw: string | undefined): ItemAmount[] {
  if (!raw) return [];
  const out: ItemAmount[] = [];
  for (const part of raw.split(',')) {
    const [id, n] = part.split(':');
    const itemId = Number(id);
    const amount = Number(n);
    if (itemId > 0 && Number.isFinite(amount)) out.push({ itemId, amount });
  }
  return out;
}

function asTone(s: string | undefined): Tone {
  return s === 'ok' || s === 'warn' || s === 'err' || s === 'off' || s === 'info' || s === 'cyan' || s === 'accent'
    ? s
    : 'info';
}

function iconCanvas(src: HTMLCanvasElement, px: number): HTMLCanvasElement {
  const c = document.createElement('canvas');
  c.width = c.height = src.width;
  c.style.width = c.style.height = `${px}px`;
  c.className = 'tcard-icon';
  c.getContext('2d')!.drawImage(src, 0, 0);
  return c;
}

function el<K extends keyof HTMLElementTagNameMap>(tag: K, className = '', text = ''): HTMLElementTagNameMap[K] {
  const node = document.createElement(tag);
  if (className) node.className = className;
  if (text) node.textContent = text;
  return node;
}
