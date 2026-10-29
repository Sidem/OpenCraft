// The world map (M): everywhere the player has been (the engine's explored map, `world_map_draw`),
// north up, with the engine's marks (ore seen at the surface, prospected deposits, machines, towns), players
// and pins, and beside it the filter chips (map-filter-panel.ts), the pin list and the ore guide (ore-guide.ts). While
// a town depot is held, the places each kind of town fits are shaded in its colour, with a legend (`world_map_sites`).
// Drag to move, wheel or the buttons to zoom, click to pin a spot or edit a pin; resting on a mark or pin shows what it
// is (map-info.ts). The image is redrawn only when the view or the explored map changes; marks, pins and players at most
// 10 times a second. Pins live in pins.ts, their list in map-pin-list.ts. `showMachines` (analytics' "Show on map")
// rings every machine of a kind and fits the view to them (map-focus-layer.ts).
// The dialog frame reuses the machine panel's styles (machine.css).

import './machine.css';
import './worldmap.css';
import type { Game } from '../wasm/engine.js';
import { button, h } from './dom';
import { FilterPanel } from './map-filter-panel';
import { fitView } from './map-focus';
import { MapFocus } from './map-focus-layer';
import { infoAt, type MapInfo } from './map-info';
import type { MapFilter } from './map-filter';
import { fillPinList } from './map-pin-list';
import { OreGuide } from './ore-guide';
import { arrow, drawMark, drawPin, hex, type Pin, type Pins } from './pins';

/** Blocks per screen pixel at each zoom step. */
const ZOOMS = [0.25, 0.5, 1, 2, 4, 8, 16];
const START_ZOOM = 1;
const MARKS_MS = 100;
/** A press that moves less than this (screen px) is a click, not a drag. */
const CLICK_SLOP = 4;
/** How near (screen px) a click must be to a pin's point to pick it. */
const PIN_REACH = 14;
/** How near (screen px) the pointer must rest to a mark for the info box. */
const HOVER_REACH = 8;
/** How strongly the places a town fits are shaded. */
const SITE_ALPHA = 0.4;
/** Screen px kept clear around the machines "Show on map" fits into view. */
const FOCUS_PAD = 48;

export class WorldMap {
  /** Called after the map closes. `resume` is true when play should continue (M, or a click outside). */
  onClose: (resume: boolean) => void = () => {};

  private readonly backdrop = h('div', 'mp-backdrop hidden');
  private readonly view = h('div', 'wm-view');
  private readonly image = h('canvas', 'wm-image');
  private readonly overlay = h('canvas', 'wm-marks');
  private readonly coords = h('span', 'wm-coords');
  private readonly scaleLabel = h('span', 'wm-scale');
  private readonly pinList = h('div', 'wm-pins');
  private readonly editor = h('div', 'wm-editor hidden');
  private readonly legend = h('div', 'wm-legend hidden');
  private readonly ctx: CanvasRenderingContext2D;
  private readonly focus: MapFocus;
  private centre = { x: 0, z: 0 };
  private zoom = START_ZOOM;
  private drawnKey = '';
  private lastMarks = -Infinity;
  private press: { x: number; y: number; cx: number; cz: number; moved: boolean } | null = null;
  private editing: { pin: Pin | null; x: number; z: number; kind: string } | null = null;

  constructor(
    private readonly game: Game,
    private readonly memory: WebAssembly.Memory,
    private readonly pins: Pins,
    private readonly filter: MapFilter,
    private readonly info: MapInfo,
  ) {
    this.ctx = this.image.getContext('2d')!;
    this.focus = new MapFocus(game);
    this.focus.onClear = () => {
      this.focus.clear();
      this.lastMarks = -Infinity;
    };
    const dialog = h('div', 'mp wm');
    dialog.setAttribute('role', 'dialog');
    dialog.setAttribute('aria-modal', 'true');
    dialog.tabIndex = -1;
    const head = h('div', 'mp-head');
    const close = button('close-btn', '×', () => this.close(true));
    close.setAttribute('aria-label', 'Close');
    head.append(h('h2', '', 'Map'), h('span', 'mp-keys', 'Drag to move · wheel to zoom · click to pin · M to return'), close);

    const tools = h('div', 'wm-tools');
    tools.append(
      button('secondary-btn', '−', () => this.zoomBy(1)),
      button('secondary-btn', '+', () => this.zoomBy(-1)),
      this.scaleLabel,
      button('secondary-btn', 'Centre on me', () => this.centreOnPlayer()),
      button('secondary-btn', 'Pin where I stand', () => this.editAt(this.game.player_x(), this.game.player_z(), null)),
      this.coords,
    );
    this.legend.append(h('strong', '', 'Where a town fits'));
    for (let k = 0; k < game.town_kind_count(); k++) {
      const swatch = h('span', 'wm-swatch');
      swatch.style.background = hex(game.town_kind_colour(k));
      const row = h('span', 'wm-legend-row');
      row.append(swatch, game.town_kind_name(k));
      this.legend.append(row);
    }
    this.view.append(this.image, this.overlay, this.legend, this.focus.chip, this.editor);
    const side = h('aside', 'wm-side');
    side.append(new FilterPanel(game, filter, pins.kinds).el, h('h3', '', 'Pins'), this.pinList, new OreGuide(game).el);
    const main = h('div', 'wm-main');
    main.append(tools, this.view);
    const body = h('div', 'wm-body');
    body.append(main, side);
    dialog.append(head, body);
    this.backdrop.append(dialog);
    document.body.append(this.backdrop);

    this.listen();
    const changed = () => {
      this.fillPinList();
      this.lastMarks = -Infinity;
    };
    pins.subscribe(changed);
    filter.subscribe(changed);
    this.fillPinList();
  }

  get isOpen(): boolean {
    return !this.backdrop.classList.contains('hidden');
  }

  open(): void {
    this.backdrop.classList.remove('hidden');
    this.centreOnPlayer();
    this.fillPinList();
    (this.backdrop.firstElementChild as HTMLElement).focus();
  }

  /** Opens the map on every machine of kind `block` (analytics' "Show on map"): all in view, or centred on `at`. */
  showMachines(block: number, at?: { x: number; z: number }): void {
    this.open();
    this.focus.show(block, at);
    const v = this.frame();
    const fit = at ? { x: at.x + 0.5, z: at.z + 0.5, zoom: START_ZOOM } : v && fitView(this.focus.machines, v.w, v.h, ZOOMS, START_ZOOM, FOCUS_PAD);
    if (!fit) return;
    this.centre = { x: fit.x, z: fit.z };
    this.zoom = fit.zoom;
    this.scaleLabel.textContent = scaleText(ZOOMS[fit.zoom]);
  }

  close(resume: boolean): void {
    if (!this.isOpen) return;
    this.focus.clear();
    this.closeEditor();
    this.info.hide();
    this.backdrop.classList.add('hidden');
    this.onClose(resume);
  }

  /** Call every frame: redraws what changed while open. */
  update(now: number): void {
    if (!this.isOpen) return;
    const v = this.frame();
    if (!v) return;
    const scale = Math.max(1, v.bpp);
    const perPixel = scale / v.bpp; // screen px per image pixel
    const w = Math.min(1024, Math.ceil(v.w / perPixel) + 2), hgt = Math.min(1024, Math.ceil(v.h / perPixel) + 2);
    const x0 = Math.floor(v.left / scale) * scale, z0 = Math.floor(v.top / scale) * scale;
    const key = `${x0},${z0},${scale},${w},${hgt},${this.game.world_map_version()}`;
    if (key !== this.drawnKey) {
      this.drawnKey = key;
      this.game.world_map_draw(x0, z0, scale, w, hgt);
      this.image.width = w;
      this.image.height = hgt;
      const view = new Uint8ClampedArray(this.memory.buffer, this.game.world_map_ptr(), w * hgt * 4);
      this.ctx.putImageData(new ImageData(view, w, hgt), 0, 0);
      const s = this.image.style;
      s.left = `${(x0 - v.left) / v.bpp}px`;
      s.top = `${(z0 - v.top) / v.bpp}px`;
      s.width = `${w * perPixel}px`;
      s.height = `${hgt * perPixel}px`;
      this.lastMarks = -Infinity;
    }
    if (now - this.lastMarks >= MARKS_MS) {
      this.lastMarks = now;
      this.drawOverlay(v);
    }
  }

  /** The view's size and where it looks: its top-left world column and blocks per screen pixel. */
  private frame(): { w: number; h: number; bpp: number; left: number; top: number } | null {
    const w = this.view.clientWidth, h = this.view.clientHeight;
    if (w === 0 || h === 0) return null;
    const bpp = ZOOMS[this.zoom];
    return { w, h, bpp, left: this.centre.x - (w * bpp) / 2, top: this.centre.z - (h * bpp) / 2 };
  }

  private drawOverlay(v: { w: number; h: number; bpp: number; left: number; top: number }): void {
    const dpr = window.devicePixelRatio || 1;
    const o = this.overlay, c = o.getContext('2d')!;
    if (o.width !== Math.round(v.w * dpr) || o.height !== Math.round(v.h * dpr)) {
      o.width = Math.round(v.w * dpr);
      o.height = Math.round(v.h * dpr);
    }
    c.clearRect(0, 0, o.width, o.height);
    const at = (x: number, z: number): [number, number] => [((x - v.left) / v.bpp) * dpr, ((z - v.top) / v.bpp) * dpr];
    const x1 = Math.ceil(v.left + v.w * v.bpp), z1 = Math.ceil(v.top + v.h * v.bpp);
    const sites = this.game.world_map_sites(Math.floor(v.left), Math.floor(v.top), x1, z1);
    this.legend.classList.toggle('hidden', sites.length === 0);
    c.globalAlpha = SITE_ALPHA;
    for (let i = 0; i < sites.length; i += 4) {
      const [x, y] = at(sites[i], sites[i + 1]);
      const size = (sites[i + 2] / v.bpp) * dpr;
      c.fillStyle = hex(this.game.town_kind_colour(sites[i + 3]));
      c.fillRect(x, y, size, size);
    }
    c.globalAlpha = 1;
    const n = this.game.minimap_mark_fields();
    const marks = this.filter.marks(this.game.world_map_marks(Math.floor(v.left), Math.floor(v.top), x1, z1), n);
    c.lineWidth = dpr;
    for (let i = 0; i < marks.length; i += n) {
      const [x, y] = at(marks[i] + 0.5, marks[i + 1] + 0.5);
      drawMark(c, x, y, marks[i + 2], marks[i + 3], dpr * 1.3, dpr / v.bpp);
    }
    this.focus.refresh();
    this.focus.draw(c, at, dpr);
    for (const pin of this.pins.list.filter((p) => this.filter.showsPin(p.kind))) {
      const [x, y] = at(pin.x + 0.5, pin.z + 0.5);
      drawPin(c, x, y, this.pins.kind(pin.kind).color, dpr, this.pins.label(pin));
    }
    const px = this.game.player_x(), pz = this.game.player_z();
    const others = this.game.minimap_players();
    for (let i = 0; i < others.length; i += 3) arrow(c, ...at(px + others[i], pz + others[i + 1]), others[i + 2], dpr, '#fa9549');
    arrow(c, ...at(px, pz), this.game.yaw(), dpr, '#ffffff');
  }

  private listen(): void {
    this.backdrop.addEventListener('pointerdown', (e) => {
      if (e.target === this.backdrop) this.close(true);
    });
    this.backdrop.addEventListener('contextmenu', (e) => e.preventDefault());
    window.addEventListener('keydown', (e) => {
      if (!this.isOpen || e.repeat || e.target instanceof HTMLInputElement) return;
      if (e.code === 'KeyM') {
        e.preventDefault();
        this.close(true);
      } else if (e.key === 'Escape') {
        e.preventDefault();
        if (this.editing) this.closeEditor();
        else this.close(false);
      }
    });
    const o = this.overlay;
    o.addEventListener('pointerdown', (e) => {
      o.setPointerCapture(e.pointerId);
      this.press = { x: e.clientX, y: e.clientY, cx: this.centre.x, cz: this.centre.z, moved: false };
    });
    o.addEventListener('pointermove', (e) => {
      const world = this.toWorld(e.clientX, e.clientY);
      if (world) this.showCoords(world.x, world.z);
      const p = this.press;
      if (!p) return this.hover(e.clientX, e.clientY);
      const dx = e.clientX - p.x, dy = e.clientY - p.y;
      if (!p.moved && Math.hypot(dx, dy) < CLICK_SLOP) return;
      this.info.hide();
      p.moved = true;
      this.centre = { x: p.cx - dx * ZOOMS[this.zoom], z: p.cz - dy * ZOOMS[this.zoom] };
    });
    o.addEventListener('pointerup', (e) => {
      const p = this.press;
      this.press = null;
      if (!p || p.moved) return;
      const world = this.toWorld(e.clientX, e.clientY);
      if (world) this.editAt(world.x, world.z, this.pinNear(e.clientX, e.clientY));
    });
    o.addEventListener('pointerleave', () => this.info.hide());
    o.addEventListener('wheel', (e) => {
      e.preventDefault();
      this.zoomBy(Math.sign(e.deltaY), e.clientX, e.clientY);
    }, { passive: false });
  }

  /** The info box for the pin or marks under the pointer. */
  private hover(clientX: number, clientY: number): void {
    const world = this.toWorld(clientX, clientY);
    if (!world) return;
    const reach = HOVER_REACH * ZOOMS[this.zoom];
    const ringed = this.focus.near(world.x, world.z, reach);
    // A ringed machine's line replaces the engine's plain line for its kind.
    const rest = infoAt(this.game, this.filter, this.pins, this.pinNear(clientX, clientY), world.x, world.z, reach);
    const lines = [...ringed, ...rest.filter((l) => ringed.length === 0 || l.title !== this.focus.name)].slice(0, 3);
    this.info.show(lines, clientX, clientY);
  }

  /** The world column under a screen point. */
  private toWorld(clientX: number, clientY: number): { x: number; z: number } | null {
    const v = this.frame();
    if (!v) return null;
    const r = this.view.getBoundingClientRect();
    return { x: v.left + (clientX - r.left) * v.bpp, z: v.top + (clientY - r.top) * v.bpp };
  }

  private pinNear(clientX: number, clientY: number): Pin | null {
    const v = this.frame();
    if (!v) return null;
    const r = this.view.getBoundingClientRect();
    let best: Pin | null = null, bestD = PIN_REACH;
    for (const pin of this.pins.list.filter((p) => this.filter.showsPin(p.kind))) {
      const sx = r.left + (pin.x + 0.5 - v.left) / v.bpp, sy = r.top + (pin.z + 0.5 - v.top) / v.bpp;
      // The point or the head (drawn above the point).
      const d = Math.min(Math.hypot(clientX - sx, clientY - sy), Math.hypot(clientX - sx, clientY - (sy - 14)));
      if (d < bestD) [best, bestD] = [pin, d];
    }
    return best;
  }

  /** Zooms one step out (+1) or in (-1), keeping the world column under the given point in place. */
  private zoomBy(step: number, clientX?: number, clientY?: number): void {
    const next = Math.min(ZOOMS.length - 1, Math.max(0, this.zoom + step));
    if (next === this.zoom) return;
    const r = this.view.getBoundingClientRect();
    const sx = clientX === undefined ? r.width / 2 : clientX - r.left;
    const sy = clientY === undefined ? r.height / 2 : clientY - r.top;
    const world = this.toWorld(r.left + sx, r.top + sy) ?? this.centre;
    this.zoom = next;
    const bpp = ZOOMS[next];
    this.centre = { x: world.x - (sx - r.width / 2) * bpp, z: world.z - (sy - r.height / 2) * bpp };
    this.scaleLabel.textContent = scaleText(bpp);
  }

  private centreOnPlayer(): void {
    this.centre = { x: this.game.player_x(), z: this.game.player_z() };
    this.scaleLabel.textContent = scaleText(ZOOMS[this.zoom]);
  }

  private showCoords(x: number, z: number): void {
    const d = Math.round(Math.hypot(x - this.game.player_x(), z - this.game.player_z()));
    this.coords.textContent = `x ${Math.floor(x)}, z ${Math.floor(z)} · ${d} blocks from you`;
  }

  /** Opens the pin editor for `pin`, or for a new pin at (x, z). */
  private editAt(x: number, z: number, pin: Pin | null): void {
    const kind = pin?.kind ?? this.editing?.kind ?? this.pins.kinds[1].id;
    this.editing = { pin, x: pin?.x ?? Math.floor(x), z: pin?.z ?? Math.floor(z), kind };
    const e = this.editor;
    e.replaceChildren();
    const kinds = h('div', 'wm-kinds');
    for (const k of this.pins.kinds) {
      const b = button('wm-kind', k.label, () => {
        this.editing!.kind = k.id;
        kinds.querySelectorAll('.wm-kind').forEach((el) => el.classList.toggle('on', el === b));
      });
      b.style.setProperty('--kind', k.color);
      b.classList.toggle('on', k.id === kind);
      kinds.append(b);
    }
    const note = h('input', 'wm-note');
    note.placeholder = 'A few words (optional)';
    note.maxLength = 40;
    note.value = pin?.note ?? '';
    const save = () => {
      const ed = this.editing!;
      if (ed.pin) this.pins.update(ed.pin, ed.kind, note.value.trim());
      else this.pins.add({ x: ed.x, z: ed.z, kind: ed.kind, note: note.value.trim() });
      this.closeEditor();
    };
    note.addEventListener('keydown', (ev) => {
      if (ev.key === 'Enter') save();
      else if (ev.key === 'Escape') {
        ev.preventDefault();
        this.closeEditor();
      }
    });
    const buttons = h('div', 'wm-editor-buttons');
    buttons.append(button('secondary-btn active', pin ? 'Save' : 'Add pin', save));
    if (pin) {
      buttons.append(button('secondary-btn danger', 'Remove', () => {
        this.pins.remove(pin);
        this.closeEditor();
      }));
    }
    buttons.append(button('secondary-btn', 'Cancel', () => this.closeEditor()));
    e.append(h('h4', '', `${pin ? 'Pin' : 'New pin'} at x ${this.editing.x}, z ${this.editing.z}`), kinds, note, buttons);
    e.classList.remove('hidden');
    note.focus();
  }

  private closeEditor(): void {
    this.editing = null;
    this.editor.classList.add('hidden');
  }

  private fillPinList(): void {
    fillPinList(this.pinList, this.game, this.pins, this.filter, (pin) => {
      this.centre = { x: pin.x + 0.5, z: pin.z + 0.5 };
      this.editAt(pin.x, pin.z, pin);
    });
  }
}

function scaleText(bpp: number): string {
  return bpp < 1 ? `1 block = ${1 / bpp} px` : `1 px = ${bpp} block${bpp === 1 ? '' : 's'}`;
}
