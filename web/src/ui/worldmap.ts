// The world map (M): everywhere the player has been (the engine's explored map, `world_map_draw`),
// north up, with the engine's marks (ore seen at the surface, prospected deposits, machines), players
// and pins, and beside it the pin list and the ore guide (ore-guide.ts). Drag to move, wheel or the
// buttons to zoom, click to pin a spot or edit a pin. The image is redrawn only when the view or the
// explored map changes; marks, pins and players at most 10 times a second. Pins live in pins.ts.
// The dialog frame reuses the machine panel's styles (machine.css).

import './machine.css';
import './worldmap.css';
import type { Game } from '../wasm/engine.js';
import { button, h } from './dom';
import { OreGuide } from './ore-guide';
import { drawMark, drawPin, type Pin, type Pins } from './pins';

/** Blocks per screen pixel at each zoom step. */
const ZOOMS = [0.25, 0.5, 1, 2, 4, 8, 16];
const START_ZOOM = 1;
const MARKS_MS = 100;
/** A press that moves less than this (screen px) is a click, not a drag. */
const CLICK_SLOP = 4;
/** How near (screen px) a click must be to a pin's point to pick it. */
const PIN_REACH = 14;

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
  private readonly ctx: CanvasRenderingContext2D;
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
  ) {
    this.ctx = this.image.getContext('2d')!;
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
    this.view.append(this.image, this.overlay, this.editor);
    const side = h('aside', 'wm-side');
    side.append(h('h3', '', 'Pins'), this.pinList, new OreGuide(game).el);
    const main = h('div', 'wm-main');
    main.append(tools, this.view);
    const body = h('div', 'wm-body');
    body.append(main, side);
    dialog.append(head, body);
    this.backdrop.append(dialog);
    document.body.append(this.backdrop);

    this.listen();
    pins.subscribe(() => {
      this.fillPinList();
      this.lastMarks = -Infinity;
    });
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

  close(resume: boolean): void {
    if (!this.isOpen) return;
    this.closeEditor();
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
    const marks = this.game.world_map_marks(Math.floor(v.left), Math.floor(v.top), x1, z1);
    const n = this.game.minimap_mark_fields();
    c.lineWidth = dpr;
    for (let i = 0; i < marks.length; i += n) {
      const [x, y] = at(marks[i] + 0.5, marks[i + 1] + 0.5);
      drawMark(c, x, y, marks[i + 2], marks[i + 3], dpr * 1.3);
    }
    for (const pin of this.pins.list) {
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
      if (!p) return;
      const dx = e.clientX - p.x, dy = e.clientY - p.y;
      if (!p.moved && Math.hypot(dx, dy) < CLICK_SLOP) return;
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
    o.addEventListener('wheel', (e) => {
      e.preventDefault();
      this.zoomBy(Math.sign(e.deltaY), e.clientX, e.clientY);
    }, { passive: false });
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
    for (const pin of this.pins.list) {
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
    const list = this.pinList;
    list.replaceChildren();
    if (this.pins.list.length === 0) {
      list.append(h('p', 'wm-empty', 'No pins yet. Click the map to pin a spot.'));
      return;
    }
    const px = this.game.player_x(), pz = this.game.player_z();
    for (const pin of this.pins.list) {
      const row = button('wm-pin', '', () => {
        this.centre = { x: pin.x + 0.5, z: pin.z + 0.5 };
        this.editAt(pin.x, pin.z, pin);
      });
      const dot = h('span', 'og-swatch');
      dot.style.background = this.pins.kind(pin.kind).color;
      const dx = pin.x + 0.5 - px, dz = pin.z + 0.5 - pz, d = Math.round(Math.hypot(dx, dz));
      const far = d < 4 ? 'here' : `${d} blocks ${compass(dx, dz)}`;
      row.append(dot, h('span', 'wm-pin-label', this.pins.label(pin)), h('span', 'wm-pin-far', far));
      list.append(row);
    }
  }
}

/** A player arrow at canvas point (x, y), turned by `yaw` (0 faces north, up the map). */
function arrow(c: CanvasRenderingContext2D, x: number, y: number, yaw: number, dpr: number, color: string): void {
  c.save();
  c.translate(x, y);
  c.rotate(yaw);
  c.beginPath();
  c.moveTo(0, -8 * dpr);
  c.lineTo(6 * dpr, 7 * dpr);
  c.lineTo(0, 3 * dpr);
  c.lineTo(-6 * dpr, 7 * dpr);
  c.closePath();
  c.fillStyle = color;
  c.strokeStyle = 'rgba(0, 0, 0, 0.8)';
  c.fill();
  c.stroke();
  c.restore();
}

function scaleText(bpp: number): string {
  return bpp < 1 ? `1 block = ${1 / bpp} px` : `1 px = ${bpp} block${bpp === 1 ? '' : 's'}`;
}

/** The compass direction of an offset (north is -z). */
function compass(dx: number, dz: number): string {
  const names = ['N', 'NE', 'E', 'SE', 'S', 'SW', 'W', 'NW'];
  const turn = Math.atan2(dx, -dz) / (Math.PI / 4);
  return names[(Math.round(turn) + 8) % 8];
}
