// Site panel: opens when the planner (right-click two corner blocks, or a marked site) asks for it
// (`game.take_site_request()`, polled in main.ts). A new area picks a job (dig, fill, flatten) and the level the
// ground ends at, shows what that would move (`game.site_survey`, loaded chunks only) and marks it
// (`game.mark_site`); the Tunnel job instead picks a section and bores from the first block towards the second
// (`tunnel_survey`, `mark_tunnel`). A marked site shows its figures and can be removed (`game.remove_site`).
// Drone ports within reach do the work. The frame is the machine panel's (machine.css). To add a figure: append it to the survey in
// `api/sites.rs` and name it in `FIGURES`.

import './machine.css';
import './site.css';
import type { Game } from '../wasm/engine.js';
import { button, h } from './dom';

const JOBS = [
  ['Dig', 'Cut everything above the level away.'],
  ['Fill', 'Build every column up to the level.'],
  ['Flatten', 'Cut what is above and fill what is below: the ground ends level.'],
  ['Tunnel', 'Bore from the first block towards the second, along the longer side, rising or falling 1 block in 2 at most.'],
];
const TUNNEL = 3;
/** Survey numbers in order (see `site_survey`), the last one (spoil) is worded separately. */
const FIGURES = ['Blocks to dig', 'Blocks to fill', 'Ore among them', 'Trees', 'Water', 'Unseen columns'];
const MAX_LEVEL = 250;

export class SitePanel {
  /** Called after the screen closes. `resume` is true when play should continue. */
  onClose: (resume: boolean) => void = () => {};

  private readonly backdrop = h('div', 'mp-backdrop hidden');
  private readonly dialog = h('div', 'mp site');
  private readonly title = h('h2', '', 'Site');
  private readonly body = h('div', 'mp-body');
  // The corner blocks being marked: x and z of both, then the height of each (a tunnel's ends).
  private ends = [0, 0, 0, 0, 0, 0];
  private job = 2;
  private level = 64;
  private section = 1;

  constructor(private readonly game: Game) {
    this.dialog.setAttribute('role', 'dialog');
    this.dialog.setAttribute('aria-modal', 'true');
    this.dialog.tabIndex = -1;
    const head = h('div', 'mp-head');
    const close = button('close-btn', '×', () => this.close(true));
    close.setAttribute('aria-label', 'Close');
    head.append(this.title, h('span', 'mp-keys', 'E or click outside to return'), close);
    this.dialog.append(head, this.body);
    this.backdrop.append(this.dialog);
    this.backdrop.addEventListener('pointerdown', (e) => {
      if (e.target === this.backdrop) this.close(true);
    });
    this.backdrop.addEventListener('contextmenu', (e) => e.preventDefault());
    window.addEventListener('keydown', (e) => {
      if (!this.isOpen || e.repeat) return;
      const typing = e.target instanceof HTMLInputElement;
      if (!typing && e.code === 'KeyE') {
        e.preventDefault();
        this.close(true);
      } else if (e.key === 'Escape') {
        e.preventDefault();
        this.close(false);
      }
    });
    document.body.append(this.backdrop);
  }

  get isOpen(): boolean {
    return !this.backdrop.classList.contains('hidden');
  }

  /** Opens for a planner request: `[0, ax, az, bx, bz, ay, by]` (a new area) or `[1, id]` (a marked site). */
  open(request: number[]): void {
    this.backdrop.classList.remove('hidden');
    if (request[0] === 1) this.showSite(request[1]);
    else {
      this.ends = request.slice(1, 7);
      this.level = Math.min(request[5], request[6]);
      this.showNew();
    }
    this.dialog.focus();
  }

  close(resume: boolean): void {
    if (!this.isOpen) return;
    this.backdrop.classList.add('hidden');
    this.onClose(resume);
  }

  private showNew(): void {
    const [ax, az, bx, bz] = this.ends;
    const width = Math.abs(ax - bx) + 1;
    const depth = Math.abs(az - bz) + 1;
    const tunnel = this.job === TUNNEL;
    this.title.textContent = tunnel ? `New tunnel · ${Math.max(width, depth)} blocks` : `New site · ${width} × ${depth}`;
    const jobs = h('div', 'site-jobs');
    JOBS.forEach(([name], i) => {
      const b = button('secondary-btn' + (i === this.job ? ' active' : ''), name, () => {
        this.job = i;
        this.showNew();
      });
      jobs.append(b);
    });
    const level = h('input', 'site-level');
    level.type = 'number';
    level.min = '1';
    level.max = String(MAX_LEVEL);
    level.value = String(this.level);
    level.setAttribute('aria-label', 'Level');
    const set = (n: number) => {
      this.level = Math.max(1, Math.min(MAX_LEVEL, Math.round(n) || this.level));
      this.showNew();
    };
    level.addEventListener('change', () => set(level.valueAsNumber));
    const row = h('div', 'site-row');
    row.append(
      h('span', '', 'Ground ends at height'),
      button('secondary-btn', '−', () => set(this.level - 1)),
      level,
      button('secondary-btn', '+', () => set(this.level + 1)),
    );
    const facts = this.survey();
    const mark = button('secondary-btn', tunnel ? 'Mark tunnel' : 'Mark site', () => {
      if (tunnel) this.game.mark_tunnel(this.tunnelEnds(), this.section);
      else this.game.mark_site(ax, az, bx, bz, this.level, this.job);
      this.close(true);
    });
    mark.disabled = facts.length === 0;
    this.body.replaceChildren(
      h('p', 'site-help', JOBS[this.job][1] + ' Drone ports within reach do the work, using the storage boxes beside their pad.'),
      jobs,
      tunnel ? this.sections() : row,
      ...facts,
      mark,
    );
  }

  /** The two blocks as x, y, z each (the order the engine wants). */
  private tunnelEnds(): Int32Array {
    const [ax, az, bx, bz, ay, by] = this.ends;
    return Int32Array.of(ax, ay, az, bx, by, bz);
  }

  /** The tunnel's section choice; the first block is the middle of its floor. */
  private sections(): HTMLElement {
    const sizes = this.game.tunnel_sections();
    const row = h('div', 'site-jobs');
    for (let i = 0; i + 1 < sizes.length; i += 2) {
      const n = i / 2;
      const name = `${sizes[i]} wide × ${sizes[i + 1]} tall`;
      row.append(
        button('secondary-btn' + (n === this.section ? ' active' : ''), name, () => {
          this.section = n;
          this.showNew();
        }),
      );
    }
    return row;
  }

  private survey(): HTMLElement[] {
    const [ax, az, bx, bz] = this.ends;
    const tunnel = this.job === TUNNEL;
    const s = tunnel
      ? this.game.tunnel_survey(this.tunnelEnds(), this.section)
      : this.game.site_survey(ax, az, bx, bz, this.level, this.job);
    if (s.length === 0) {
      const why = tunnel
        ? 'That tunnel is too long (96 blocks at most), too steep (1 block up or down for every 2 along) or too near the top or bottom of the world, or a site is in the way.'
        : 'That area is too big for one site (64 × 64 at most).';
      return [h('p', 'site-warn', why)];
    }
    const figures = tunnel ? FIGURES.map((name) => (name === 'Water' ? 'Blocks left beside water' : name)) : FIGURES;
    const rows = figures.map((name, i) => [name, s[i]] as const).filter(([, n]) => n > 0);
    const spoil = s[FIGURES.length];
    const out: HTMLElement[] = [];
    for (const [name, n] of rows) out.push(h('p', 'site-fact', `${name}: ${n}`));
    if (spoil > 0) out.push(h('p', 'site-fact', `${spoil} blocks of ground go into the boxes beside the port`));
    if (spoil < 0) out.push(h('p', 'site-fact', `${-spoil} blocks of ground must be in the boxes beside the port`));
    if (rows.length === 0) out.push(h('p', 'site-fact', 'Nothing to move at this level.'));
    return out;
  }

  private showSite(id: number): void {
    const f = this.game.site_fields();
    const all = this.game.sites();
    let row: number[] | undefined;
    for (let i = 0; i + f <= all.length; i += f) if (all[i] === id) row = Array.from(all.slice(i, i + f));
    this.title.textContent = 'Marked site';
    if (!row) {
      this.body.replaceChildren(h('p', 'site-fact', 'This site is already finished.'));
      return;
    }
    const [, x0, z0, x1, z1, level, job, cells, section] = row;
    const remove = button('secondary-btn', 'Remove site', () => {
      this.game.remove_site(id);
      this.close(true);
    });
    const sizes = this.game.tunnel_sections();
    const shape =
      job === TUNNEL
        ? `${sizes[section * 2]} wide × ${sizes[section * 2 + 1]} tall · starts at height ${level}`
        : `${x1 - x0 + 1} × ${z1 - z0 + 1} · ground ends at height ${level}`;
    this.body.replaceChildren(
      h('p', 'site-fact', `${JOBS[job][0]} · ${shape}`),
      h('p', 'site-fact', `${cells} blocks in all; drone ports within reach work on it until it is done.`),
      remove,
    );
  }
}
