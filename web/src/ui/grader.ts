// Site Grader Control Console: bespoke 2-column dark-glass modal (`grader-panel.css`) that opens when right-clicking
// a Telescoping Bucket-Wheel Site Grader (`main.ts` via `take_panel_request()`).
// Left column: live interactive bird's-eye survey map (`grader-map.ts`) + quick size presets.
// Right column: custom dark sliders + micro-steppers for Width/Depth/Shift, segmented Ore Deposit mode, 3×3 Floor
// Material visual swatch grid, and live Earthworks Hopper / Buffer quick actions — zero scrollbars, zero wasted space.

import './machine.css';
import './grader-panel.css';
import type { Game } from '../wasm/engine.js';
import { button, h } from './dom';
import { GraderSurveyMap } from './grader-map';

const REFRESH_MS = 160;
const ICON_PX = 64;

type Pos = [number, number, number];

interface AreaPreset {
  label: string;
  w: number;
  d: number;
}

const PRESETS: AreaPreset[] = [
  { label: '11×10', w: 11, d: 10 },
  { label: '19×18', w: 19, d: 18 },
  { label: '29×26', w: 29, d: 26 },
  { label: '41×36', w: 41, d: 36 },
];

export class GraderPanel {
  onClose: (resume: boolean) => void = () => {};
  onOpenSlots: ((x: number, y: number, z: number) => void) | null = null;

  private readonly backdrop = h('div', 'mp-backdrop hidden');
  private readonly dialog = h('div', 'mp gr-dialog');
  private readonly statusPill = h('span', 'gr-pill');
  private readonly statusDot = h('span', 'gr-pill-dot');
  private readonly statusText = h('span', '', '');
  private readonly areaPill = h('span', 'gr-pill', '19×18');
  private readonly powerBtn: HTMLButtonElement;
  private readonly powerDot = h('span', 'mp-switch-dot');
  private readonly powerText = h('span', 'mp-switch-label', 'On');
  private readonly map: GraderSurveyMap;
  private readonly presetBtns: { p: AreaPreset; el: HTMLButtonElement }[];
  private readonly widthInput: HTMLInputElement;
  private readonly widthVal = h('span', 'gr-dim-val', '19');
  private readonly depthInput: HTMLInputElement;
  private readonly depthVal = h('span', 'gr-dim-val', '18');
  private readonly offsetInput: HTMLInputElement;
  private readonly offsetVal = h('span', 'gr-dim-val', '0');
  private readonly dimsMeta = h('span', 'gr-card-meta', '342 tiles');
  private readonly spareBtn: HTMLButtonElement;
  private readonly mineBtn: HTMLButtonElement;
  private readonly floorButtons: { block: number; el: HTMLButtonElement }[];
  private readonly floorMeta = h('span', 'gr-card-meta', 'Natural');
  private readonly hopperMeta = h('span', 'gr-card-meta', '0 / 64');
  private readonly hopperFill = h('div', 'gr-hopper-fill');
  private readonly statCut = h('span', 'gr-stat-v', '0');
  private readonly statFill = h('span', 'gr-stat-v', '0');
  private readonly statPave = h('span', 'gr-stat-v', '0');
  private readonly statTree = h('span', 'gr-stat-v', '0');
  private readonly loadBtn: HTMLButtonElement;
  private readonly takeBtn: HTMLButtonElement;

  private readonly bounds: [number, number, number, number, number];
  private pos: Pos = [0, 0, 0];
  private data: Uint32Array = new Uint32Array(0);
  private readAt = 0;
  private drawn = '';

  constructor(
    private readonly game: Game,
    private readonly icon: (item: number) => HTMLCanvasElement,
  ) {
    const b = game.grader_bounds();
    this.bounds = [b[0] ?? 5, b[1] ?? 41, b[2] ?? 5, b[3] ?? 36, b[4] ?? 20];
    const [minW, maxW, minD, maxD, maxOff] = this.bounds;

    this.dialog.setAttribute('role', 'dialog');
    this.dialog.setAttribute('aria-modal', 'true');
    this.dialog.tabIndex = -1;

    // ---- Header bar
    const head = h('div', 'gr-head');
    const headLeft = h('div', 'gr-head-left');
    this.statusPill.append(this.statusDot, this.statusText);
    headLeft.append(h('h2', 'gr-title', 'Site Grader'), this.statusPill, this.areaPill);

    const headRight = h('div', 'gr-head-right');
    const slotsBtn = button('secondary-btn gr-act-btn', 'Buffers', () => {
      const [x, y, z] = this.pos;
      this.close(false);
      this.onOpenSlots?.(x, y, z);
    });
    slotsBtn.title = 'Open raw machine input/output slots beside your inventory';
    this.powerBtn = button('secondary-btn mp-switch', '', () => {
      this.game.set_machine_enabled(...this.pos, !this.game.machine_enabled(...this.pos));
      this.readAt = 0;
    });
    this.powerBtn.append(this.powerDot, this.powerText);
    const closeBtn = button('close-btn', '×', () => this.close(true));
    closeBtn.setAttribute('aria-label', 'Close');
    headRight.append(slotsBtn, this.powerBtn);
    head.append(headLeft, headRight, closeBtn);

    // ---- Left column: Interactive Bird's-Eye Survey Map + Preset strip
    this.map = new GraderSurveyMap(game, this.bounds, (patch) => this.apply(patch));
    const presetSeg = h('div', 'gr-seg');
    this.presetBtns = PRESETS.map((p) => {
      const btn = button('gr-seg-btn', p.label, () => this.apply({ width: p.w, depth: p.d, offset: 0 }));
      presetSeg.append(btn);
      return { p, el: btn };
    });
    const mapFooter = h('div', 'gr-map-footer');
    mapFooter.append(presetSeg, h('span', 'gr-map-hint', 'Drag edges or box to adjust'));
    const mapCol = h('div', 'gr-map-col');
    mapCol.append(this.map.el, mapFooter);

    // ---- Right column: 4 compact dark-glass control cards
    // Card 1: Custom Dimensions
    const dimCard = h('section', 'gr-card');
    const dimHead = h('div', 'gr-card-head');
    dimHead.append(h('h3', 'gr-card-title', 'Area Dimensions'), this.dimsMeta);
    const wRow = this.dimRow('Width', minW, maxW, this.widthVal, (v) => this.apply({ width: v }));
    const dRow = this.dimRow('Depth', minD, maxD, this.depthVal, (v) => this.apply({ depth: v }));
    const oRow = this.dimRow('Shift', -maxOff, maxOff, this.offsetVal, (v) => this.apply({ offset: v }));
    this.widthInput = wRow.input;
    this.depthInput = dRow.input;
    this.offsetInput = oRow.input;
    this.offsetVal.title = 'Click to re-center (0)';
    this.offsetVal.addEventListener('click', () => {
      if (this.current().offset !== 0) this.apply({ offset: 0 });
    });
    dimCard.append(dimHead, wRow.row, dRow.row, oRow.row);

    // Card 2: Ore Deposits
    const oreCard = h('section', 'gr-card');
    const oreHead = h('div', 'gr-card-head');
    oreHead.append(h('h3', 'gr-card-title', 'Ore Deposits'));
    this.spareBtn = button('gr-seg-btn', 'Spare Ores', () => this.apply({ spare: true }));
    this.spareBtn.dataset.tone = 'mint';
    this.mineBtn = button('gr-seg-btn', 'Mine Ores', () => this.apply({ spare: false }));
    const oreSeg = h('div', 'gr-seg gr-seg-full');
    oreSeg.append(this.spareBtn, this.mineBtn);
    oreCard.append(oreHead, oreSeg);

    // Card 3: Floor Material (3×3 visual swatch grid)
    const floorCard = h('section', 'gr-card');
    const floorHead = h('div', 'gr-card-head');
    floorHead.append(h('h3', 'gr-card-title', 'Floor Material'), this.floorMeta);
    const floorGrid = h('div', 'gr-floors');
    this.floorButtons = Array.from(game.grader_floors(), (block) => {
      const label = block === 0 ? 'Natural' : shortBlockName(game.block_name(block));
      const btn = button('gr-floor-btn', '', () => this.apply({ floor: block }));
      btn.title = block === 0 ? 'Natural ground (level & fill only, no paving)' : `Pave floor with ${game.block_name(block)} (left port)`;
      if (block === 0) {
        btn.append(h('span', 'gr-floor-nat'));
      } else {
        const item = this.itemForBlock(block);
        if (item > 0) btn.append(this.iconCanvas(item));
      }
      btn.append(h('span', 'gr-floor-name', label));
      floorGrid.append(btn);
      return { block, el: btn };
    });
    floorCard.append(floorHead, floorGrid);

    // Card 4: Earthworks Hopper & Quick Actions
    const hopCard = h('section', 'gr-card');
    const hopHead = h('div', 'gr-card-head');
    hopHead.append(h('h3', 'gr-card-title', 'Earthworks'), this.hopperMeta);
    const hopBar = h('div', 'gr-hopper-bar');
    hopBar.append(this.hopperFill);
    const stats = h('div', 'gr-stats');
    stats.append(
      this.statCell('Cut', this.statCut),
      this.statCell('Filled', this.statFill),
      this.statCell('Paved', this.statPave),
      this.statCell('Trees', this.statTree),
    );
    this.loadBtn = button('secondary-btn gr-act-btn', 'Load floor', () => {
      const floor = this.current().floor;
      if (floor === 0) return;
      const item = this.itemForBlock(floor);
      if (item > 0) this.game.insert_into_machine(...this.pos, item);
      this.readAt = 0;
    });
    this.takeBtn = button('secondary-btn gr-act-btn', 'Take output', () => {
      this.game.take_machine_output(...this.pos);
      this.readAt = 0;
    });
    const actions = h('div', 'gr-actions');
    actions.append(this.loadBtn, this.takeBtn);
    hopCard.append(hopHead, hopBar, stats, actions);

    const side = h('div', 'gr-side');
    side.append(dimCard, oreCard, floorCard, hopCard);

    const body = h('div', 'gr-body');
    body.append(mapCol, side);

    this.dialog.append(head, body);
    this.backdrop.append(this.dialog);
    document.body.append(this.backdrop);

    this.backdrop.addEventListener('pointerdown', (e) => {
      if (e.target === this.backdrop) this.close(true);
    });
    this.backdrop.addEventListener('contextmenu', (e) => e.preventDefault());
    window.addEventListener('keydown', (e) => {
      if (this.isOpen && e.key === 'Escape') {
        e.preventDefault();
        this.close(true);
      }
    });
  }

  get isOpen(): boolean {
    return !this.backdrop.classList.contains('hidden');
  }

  /** Opens the Site Grader Console if `(x, y, z)` is a Site Grader; returns false for any other machine. */
  open(x: number, y: number, z: number): boolean {
    const data = this.game.grader_panel(x, y, z);
    if (data.length === 0) return false;
    this.pos = [x, y, z];
    this.data = data;
    this.drawn = '';
    this.readAt = 0;
    this.backdrop.classList.remove('hidden');
    this.dialog.focus();
    this.refresh(true);
    return true;
  }

  close(resume: boolean): void {
    if (!this.isOpen) return;
    this.backdrop.classList.add('hidden');
    this.onClose(resume);
  }

  /** Called every frame while open; closes automatically if the machine is broken. */
  update(): void {
    if (!this.isOpen) return;
    this.refresh(false);
  }

  private refresh(force: boolean): void {
    const now = performance.now();
    if (force || now - this.readAt >= REFRESH_MS) {
      const data = this.game.grader_panel(...this.pos);
      if (data.length === 0) {
        this.close(true);
        return;
      }
      this.data = data;
      this.readAt = now;
    }

    const cur = this.current();
    this.map.update(this.pos, {
      width: cur.width,
      depth: cur.depth,
      offset: cur.offset,
      spareOres: cur.spare,
      floor: cur.floor,
    }, force);

    const enabled = this.game.machine_enabled(...this.pos);
    const statusFull = this.game.machine_status(...this.pos);
    const statusShort = statusFull.split(' · ')[0] || statusFull;
    const panelData = this.game.machine_panel(...this.pos);
    let outCount = 0;
    let inCount = 0;
    const floorItem = cur.floor === 0 ? 0 : this.itemForBlock(cur.floor);
    if (panelData.length >= 6) {
      const slots = panelData[5];
      const outRole = this.game.panel_role_output();
      const inRole = this.game.panel_role_input();
      for (let i = 0; i < slots; i++) {
        const role = panelData[6 + i * 3];
        const item = panelData[7 + i * 3];
        const n = panelData[8 + i * 3];
        if (role === outRole) outCount += n;
        else if (role === inRole && (floorItem === 0 || item === floorItem)) inCount += n;
      }
    }
    const invFloorCount = floorItem === 0 ? 0 : this.inventoryCount(floorItem);
    const key = `${this.data.join(',')}|${enabled ? 1 : 0}|${statusShort}|${outCount}|${inCount}|${invFloorCount}`;
    if (key === this.drawn) return;
    this.drawn = key;

    // Header status & power switch
    this.statusText.textContent = statusShort;
    const tone = !enabled
      ? 'err'
      : /no power|not wired/i.test(statusShort)
        ? 'err'
        : /waiting|output full/i.test(statusShort)
          ? 'warn'
          : 'ok';
    this.statusPill.dataset.tone = tone;
    this.powerBtn.dataset.on = enabled ? 'true' : 'false';
    this.powerBtn.setAttribute('aria-pressed', enabled ? 'true' : 'false');
    this.powerText.textContent = enabled ? 'On' : 'Off';
    this.areaPill.textContent = `${cur.width}×${cur.depth}${cur.offset === 0 ? '' : ` (${cur.offset > 0 ? '+' : ''}${cur.offset})`}`;

    // Presets & dimensions
    for (const { p, el } of this.presetBtns) {
      el.classList.toggle('active', cur.width === p.w && cur.depth === p.d && cur.offset === 0);
    }
    this.widthInput.value = String(cur.width);
    this.widthVal.textContent = String(cur.width);
    this.depthInput.value = String(cur.depth);
    this.depthVal.textContent = String(cur.depth);
    this.offsetInput.value = String(cur.offset);
    this.offsetVal.textContent = cur.offset === 0 ? '0' : cur.offset > 0 ? `+${cur.offset}` : String(cur.offset);
    this.offsetVal.dataset.resettable = cur.offset !== 0 ? 'true' : 'false';
    this.dimsMeta.textContent = `${cur.width * cur.depth} tiles`;

    // Ore mode & Floor swatches
    this.spareBtn.classList.toggle('active', cur.spare);
    this.mineBtn.classList.toggle('active', !cur.spare);
    const floorName = cur.floor === 0 ? 'Natural' : this.game.block_name(cur.floor);
    this.floorMeta.textContent = cur.floor === 0 ? 'Keep ground' : `${floorName} (${inCount} in)`;
    for (const f of this.floorButtons) {
      f.el.classList.toggle('active', f.block === cur.floor);
    }

    // Hopper bar & telemetry
    const [, , hopper, cap, cut, filled, paved, trees] = this.data;
    this.hopperMeta.textContent = `Hopper ${hopper} / ${cap}`;
    this.hopperFill.style.width = `${Math.min(100, (hopper / Math.max(1, cap)) * 100)}%`;
    this.statCut.textContent = String(cut);
    this.statFill.textContent = String(filled);
    this.statPave.textContent = String(paved);
    this.statTree.textContent = String(trees);

    // Quick buffer buttons
    if (cur.floor === 0) {
      this.loadBtn.textContent = 'Natural floor';
      this.loadBtn.disabled = true;
    } else if (invFloorCount > 0) {
      this.loadBtn.textContent = `Load ${shortBlockName(floorName)} (${invFloorCount})`;
      this.loadBtn.disabled = false;
    } else {
      this.loadBtn.textContent = `No ${shortBlockName(floorName)} in pack`;
      this.loadBtn.disabled = true;
    }
    this.takeBtn.textContent = outCount > 0 ? `Take output (${outCount})` : 'Output empty';
    this.takeBtn.disabled = outCount === 0;
  }

  private current(): { spare: boolean; floor: number; width: number; depth: number; offset: number } {
    return {
      spare: (this.data[0] ?? 1) === 1,
      floor: this.data[1] ?? 0,
      width: this.data[8] ?? 19,
      depth: this.data[9] ?? 18,
      offset: (this.data[10] ?? 128) - 128,
    };
  }

  private apply(patch: Partial<{ spare: boolean; floor: number; width: number; depth: number; offset: number }>): void {
    const [minW, maxW, minD, maxD, maxOff] = this.bounds;
    const cur = this.current();
    const spare = patch.spare ?? cur.spare;
    const floor = patch.floor ?? cur.floor;
    const width = Math.max(minW, Math.min(maxW, Math.round(patch.width ?? cur.width)));
    const depth = Math.max(minD, Math.min(maxD, Math.round(patch.depth ?? cur.depth)));
    const offset = Math.max(-maxOff, Math.min(maxOff, Math.round(patch.offset ?? cur.offset)));
    this.game.set_grader(...this.pos, spare, floor, width, depth, offset);
    this.data = Uint32Array.from([
      spare ? 1 : 0,
      floor,
      this.data[2] ?? 0,
      this.data[3] ?? 64,
      this.data[4] ?? 0,
      this.data[5] ?? 0,
      this.data[6] ?? 0,
      this.data[7] ?? 0,
      width,
      depth,
      offset + 128,
    ]);
    this.drawn = '';
    this.refresh(false);
  }

  private dimRow(
    label: string,
    min: number,
    max: number,
    valEl: HTMLElement,
    onChange: (value: number) => void,
  ): { row: HTMLElement; input: HTMLInputElement } {
    const row = h('div', 'gr-dim-row');
    const input = document.createElement('input');
    input.type = 'range';
    input.className = 'gr-range';
    input.min = String(min);
    input.max = String(max);
    input.step = '1';
    input.addEventListener('input', () => onChange(Number(input.value)));
    const minus = button('gr-step', '−', () => onChange(Number(input.value) - 1));
    const plus = button('gr-step', '+', () => onChange(Number(input.value) + 1));
    row.append(h('span', 'gr-dim-label', label), minus, input, plus, valEl);
    return { row, input };
  }

  private statCell(label: string, valueEl: HTMLElement): HTMLElement {
    const cell = h('div', 'gr-stat');
    cell.append(h('span', 'gr-stat-k', label), valueEl);
    return cell;
  }

  private inventoryCount(item: number): number {
    if (item <= 0) return 0;
    let total = 0;
    for (let s = 0; s < this.game.inventory_size(); s++) {
      if (this.game.slot_item(s) === item) total += this.game.slot_count(s);
    }
    return total;
  }

  private itemForBlock(block: number): number {
    if (block < 256) return block;
    const found = this.game.find_item(this.game.block_name(block));
    return found.length > 0 ? found[0] : 0;
  }

  private iconCanvas(item: number): HTMLCanvasElement {
    const c = h('canvas', 'gr-floor-icon');
    c.width = c.height = ICON_PX;
    c.getContext('2d')!.drawImage(this.icon(item), 0, 0);
    return c;
  }
}

function shortBlockName(name: string): string {
  return name.replace(' Bricks', ' Brk');
}
