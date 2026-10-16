// In-game HUD: crosshair, the top-centre column (#hud-top: helper status line, target name with detail text and
// mining bar; the scanner pointer joins it from prospect.ts), hotbar, pickup toasts (and a town's news when it stalls
// or is supplied again: `town_states` compared with the last look, `town_news` for the words),
// the muted badge and the F3 debug overlay. Reads from the engine each frame; hotbar slots redraw only
// when `inventory_version` changes. `itemIcon` renders the item icons other panels reuse, `showAmount` a
// slot's count or, for a tool (whose count is its uses left), its wear bar.

import './hud.css';
import './hud-columns.css';
import type { Game } from '../wasm/engine.js';

const ICON_PX = 64;
const TOAST_MS = 2600;
/** A toast count that means "went into orbit" (keys below it are launch toasts, one per satellite kind). */
const LAUNCHED = -100000;
/** How often the target's detail text (deposit size, machine status) is refreshed. */
const DETAIL_MS = 150;
/** Toast keys below this are towns' news, one per town; how often the towns' states are read; `town_states`'s values
 * (api/towns.rs) for a stalled and a dormant town. */
const TOWN_KEY = -200000;
const TOWN_MS = 1000;
const STALLED = 1;
const DORMANT = 2;

interface Slot {
  root: HTMLDivElement;
  icon: HTMLCanvasElement;
  count: HTMLSpanElement;
  item: number;
  n: number;
}

interface Toast {
  el: HTMLDivElement;
  label: HTMLSpanElement;
  count: number;
  until: number;
}

export interface DebugInfo {
  fps: number;
  frameMs: number;
  workMs: number;
  meshes: number;
  visible: number;
  drawCalls: number;
  quads: number;
}

const $ = <T extends HTMLElement>(id: string) => document.getElementById(id) as T;

/** Shows a slot's amount: the count, or a wear bar for a tool (its count is the uses left; hud.css). */
export function showAmount(game: Game, root: HTMLElement, count: HTMLElement, item: number, n: number): void {
  const uses = n > 0 ? game.tool_uses(item) : 0;
  root.classList.toggle('tool', uses > 0);
  if (uses > 0) root.style.setProperty('--wear', String(n / uses));
  count.textContent = uses === 0 && n > 1 ? String(n) : '';
}

/** DOM overlay: hotbar, target readout, mining progress, pickup toasts and the F3 debug panel. */
export class Hud {
  private readonly slots: Slot[] = [];
  private readonly icons = new Map<number, HTMLCanvasElement>();
  private readonly layers: HTMLCanvasElement[] = [];
  /** Block names, for the target readout. */
  private readonly names: string[] = [];
  private readonly toasts = new Map<number, Toast>();
  private invVersion = -1;
  private debugVisible = false;
  private lastDebug = 0;
  private lastDetail = 0;
  private detailKey = '';

  private readonly target = $<HTMLDivElement>('target');
  private readonly targetName = $<HTMLSpanElement>('target-name');
  private readonly targetDetail = $<HTMLDivElement>('target-detail');
  private readonly mineFill = $<HTMLDivElement>('mine-fill');
  private readonly helperStatus = $<HTMLDivElement>('helper-status');
  private helperText = '';
  private townStates: number[] | null = null;
  private lastTowns = 0;
  private readonly toastBox = $<HTMLDivElement>('toasts');
  private readonly debug = $<HTMLPreElement>('debug');
  private readonly mutedBadge = $<HTMLDivElement>('muted');

  constructor(private readonly game: Game, texPixels: Uint8Array, texSize: number) {
    const layerBytes = texSize * texSize * 4;
    for (let i = 0; i * layerBytes < texPixels.length; i++) {
      const c = document.createElement('canvas');
      c.width = c.height = texSize;
      const data = new ImageData(new Uint8ClampedArray(texPixels.slice(i * layerBytes, (i + 1) * layerBytes)), texSize, texSize);
      c.getContext('2d')!.putImageData(data, 0, 0);
      this.layers.push(c);
    }
    for (let id = 0; id < game.block_count(); id++) this.names.push(game.block_name(id));

    const bar = $<HTMLDivElement>('hotbar');
    for (let i = 0; i < game.hotbar_size(); i++) {
      const root = document.createElement('div');
      root.className = 'slot';
      const key = document.createElement('span');
      key.className = 'key';
      key.textContent = String(i + 1);
      const icon = document.createElement('canvas');
      icon.width = icon.height = ICON_PX;
      const count = document.createElement('span');
      count.className = 'count';
      root.append(key, icon, count);
      bar.append(root);
      this.slots.push({ root, icon, count, item: -1, n: -1 });
    }
  }

  toggleDebug(): void {
    this.debugVisible = !this.debugVisible;
    this.debug.classList.toggle('hidden', !this.debugVisible);
  }

  setMuted(muted: boolean): void {
    this.mutedBadge.classList.toggle('hidden', !muted);
  }

  update(now: number, info: DebugInfo): void {
    const g = this.game;

    if (g.inventory_version() !== this.invVersion) {
      this.invVersion = g.inventory_version();
      const selected = g.selected_slot();
      this.slots.forEach((s, i) => {
        const item = g.slot_item(i);
        const n = g.slot_count(i);
        s.root.classList.toggle('selected', i === selected);
        s.root.title = n > 0 ? g.item_name(item) : '';
        if (item !== s.item || n !== s.n) {
          const ctx = s.icon.getContext('2d')!;
          ctx.clearRect(0, 0, ICON_PX, ICON_PX);
          if (n > 0) ctx.drawImage(this.itemIcon(item), 0, 0);
          showAmount(g, s.root, s.count, item, n);
          s.item = item;
          s.n = n;
        }
      });
    }

    const line = g.line_label();
    if (line !== '') {
      // Dragging out a belt line or placing a quarry: its plan (a title line, then the rest) replaces
      // the target readout.
      const cut = line.indexOf('\n');
      this.target.classList.remove('hidden', 'mining');
      this.targetName.textContent = line.slice(0, cut);
      this.targetDetail.textContent = line.slice(cut + 1);
      this.targetDetail.classList.remove('hidden');
      this.detailKey = '';
    } else if (g.has_target()) {
      this.target.classList.remove('hidden');
      this.targetName.textContent = this.names[g.target_block()] ?? '';
      const p = g.mine_progress();
      this.mineFill.style.transform = `scaleX(${p})`;
      this.target.classList.toggle('mining', p > 0);
      // Refresh at once on a new target, then a few times a second for live machine status.
      const key = `${g.target_x()},${g.target_y()},${g.target_z()},${g.target_block()}`;
      if (key !== this.detailKey || now - this.lastDetail > DETAIL_MS) {
        this.detailKey = key;
        this.lastDetail = now;
        const text = g.target_detail();
        this.targetDetail.textContent = text;
        this.targetDetail.classList.toggle('hidden', text === '');
      }
    } else {
      this.target.classList.add('hidden');
      this.detailKey = '';
    }

    const helpers = g.helper_status();
    if (helpers !== this.helperText) {
      this.helperText = helpers;
      this.helperStatus.textContent = helpers;
      this.helperStatus.classList.toggle('hidden', helpers === '');
    }

    while (g.next_pickup()) this.pushToast(g.pickup_item(), g.pickup_count(), now);
    for (let sat = g.next_launch(); sat !== 0; sat = g.next_launch()) this.pushToast(sat, LAUNCHED, now);
    if (now - this.lastTowns > TOWN_MS) this.townNews(now);
    for (const [item, t] of this.toasts) {
      if (now > t.until) {
        this.toasts.delete(item);
        t.el.classList.add('out');
        setTimeout(() => t.el.remove(), 300);
      }
    }

    if (this.debugVisible && now - this.lastDebug > 200) {
      this.lastDebug = now;
      const f = (v: number, d = 1) => v.toFixed(d);
      const yawDeg = ((g.yaw() * 180) / Math.PI + 360) % 360;
      const facing = ['N (-Z)', 'E (+X)', 'S (+Z)', 'W (-X)'][Math.round(yawDeg / 90) % 4];
      this.debug.textContent = [
        `OpenCraft · ${f(info.fps, 0)} fps · ${f(info.frameMs, 2)} ms/frame · worldgen+mesh ${f(info.workMs, 2)} ms`,
        `XYZ ${f(g.player_x(), 2)} / ${f(g.player_y(), 2)} / ${f(g.player_z(), 2)}   facing ${facing}`,
        `mode ${g.flying() ? 'fly' : 'walk'}${g.on_ground() ? ' · on ground' : ''}`,
        `chunks ${g.chunks_loaded()} loaded · ${g.chunks_pending()} queued · ${g.chunks_dirty()} awaiting mesh`,
        `meshes ${info.meshes} · drawn ${info.visible} · ${info.drawCalls} draw calls · ${(info.quads / 1000).toFixed(1)}k quads`,
        `items ${g.item_entities()}` +
          (g.has_target() ? ` · target ${this.names[g.target_block()]} @ ${g.target_x()} ${g.target_y()} ${g.target_z()}` : ''),
        `factory ${g.belts()} belts · ${g.miners()} miners · ${g.boxes()} boxes · ${g.deposits_tracked()} deposits tracked`,
      ].join('\n');
    }
  }

  /** A toast per item, adding up; a tool shows no amount, a count of 0 means it wore out, and `LAUNCHED` that a
   * satellite went into orbit (`next_launch`). */
  private pushToast(item: number, count: number, now: number): void {
    const tool = this.game.tool_uses(item) > 0;
    if (count === LAUNCHED) {
      const key = LAUNCHED - item;
      const existing = this.toasts.get(key);
      if (existing) existing.until = now + TOAST_MS;
      else this.addToast(key, item, 'in orbit', 0, now);
      return;
    }
    const key = count === 0 ? -1 - item : item;
    const text = (n: number) => (!tool ? `+${n}` : n === 0 ? 'wore out' : '');
    const existing = this.toasts.get(key);
    if (existing) {
      existing.count += count;
      existing.label.textContent = text(existing.count);
      existing.until = now + TOAST_MS;
      existing.el.classList.remove('bump');
      void existing.el.offsetWidth;
      existing.el.classList.add('bump');
      return;
    }
    this.addToast(key, item, text(count), count, now);
  }

  /** Toasts each town that stalled or is supplied again since the last look (the first look only remembers). */
  private townNews(now: number): void {
    this.lastTowns = now;
    const states = Array.from(this.game.town_states());
    const before = this.townStates;
    this.townStates = states;
    if (!before) return;
    states.forEach((state, i) => {
      if (before[i] === undefined || state === before[i] || state === DORMANT || before[i] === DORMANT) return;
      this.toasts.get(TOWN_KEY - i)?.el.remove();
      const stalled = state === STALLED;
      const label = stalled ? 'stalled' : 'supplied';
      this.addToast(TOWN_KEY - i, this.game.town_depot_item(), label, 0, now, this.game.town_news(i));
      this.toasts.get(TOWN_KEY - i)!.el.classList.toggle('warn', stalled);
    });
  }

  private addToast(key: number, item: number, amount: string, count: number, now: number, words = ''): void {
    const el = document.createElement('div');
    el.className = 'toast';
    const icon = document.createElement('canvas');
    icon.width = icon.height = ICON_PX;
    icon.getContext('2d')!.drawImage(this.itemIcon(item), 0, 0);
    const label = document.createElement('span');
    label.className = 'amount';
    label.textContent = amount;
    const name = document.createElement('span');
    name.textContent = words || this.game.item_name(item);
    el.append(icon, label, name);
    this.toastBox.append(el);
    this.toasts.set(key, { el, label, count, until: now + TOAST_MS });
  }

  /** Isometric icon using the same box assembly as the loose item; blocks keep their single box. */
  itemIcon(item: number): HTMLCanvasElement {
    let c = this.icons.get(item);
    if (c) return c;
    c = document.createElement('canvas');
    c.width = c.height = ICON_PX;
    const ctx = c.getContext('2d')!;
    ctx.imageSmoothingEnabled = false;
    const info = this.game.item_icon(item);
    this.icons.set(item, c);
    if (info.length < 6) return c;
    // +x runs right-down, +z left-down, +y up. Part coordinates are fractions of one cube.
    const S = ICON_PX * 0.86, k = S / 16;
    const project = (x: number, y: number, z: number): [number, number] =>
      [ICON_PX / 2 + (x - z) * S / 2, ICON_PX * 0.54 + (x + z) * S / 4 - y * S / 2];
    const face = (img: HTMLCanvasElement, m: [number, number, number, number, number, number], shade: number) => {
      ctx.setTransform(...m);
      ctx.drawImage(img, 0, 0);
      if (shade > 0) {
        ctx.globalCompositeOperation = 'source-atop';
        ctx.fillStyle = `rgba(0,0,0,${shade})`;
        ctx.fillRect(0, 0, 16, 16);
        ctx.globalCompositeOperation = 'source-over';
      }
    };
    // A box turned by `yaw` about its vertical axis: its local x runs along (cos, sin) in the xz plane and
    // its local z along (-sin, cos). A world vector (vx, vz) on screen is ((vx - vz) S/2, (vx + vz) S/4); an
    // image is 16 px, so a face's matrix takes its edge vectors over 16.
    const draw = (x: number, y: number, z: number, a: number, h: number, b: number, yaw: number,
      topLayer: number, sideLayer: number) => {
      const top = this.layers[topLayer], side = this.layers[sideLayer];
      const c = Math.cos(yaw), s = Math.sin(yaw);
      const corner = (lx: number, lz: number) => project(x + c * lx - s * lz, y + h / 2, z + s * lx + c * lz);
      const edge = (vx: number, vz: number): [number, number] => [((vx - vz) * S) / 32, ((vx + vz) * S) / 64];
      const down = (h * k) / 2;
      // Side faces seen from outside, left to right: start corner and edge; shaded by which way they face.
      const sides: [number, number, number, number, number, number][] = [
        [-a / 2, b / 2, c * a, s * a, -s, c], // +z face runs along +x
        [a / 2, b / 2, s * b, -c * b, c, s], // +x face runs along -z
        [a / 2, -b / 2, -c * a, -s * a, s, -c], // -z face runs along -x
        [-a / 2, -b / 2, -s * b, c * b, -c, -s], // -x face runs along +z
      ];
      for (const [lx, lz, ex, ez, nx, nz] of sides) {
        if (nx + nz <= 0.01) continue; // faces turned away from the viewer
        const [sx, sy] = edge(ex, ez), [px, py] = corner(lx, lz);
        face(side, [sx, sy, 0, down, px, py], 0.31 + 0.09 * (nx - nz));
      }
      // The top face's texture starts at its back corner, then runs along local +x and +z.
      const [ax, ay] = edge(c * a, s * a), [bx, by] = edge(-s * b, c * b), [qx, qy] = corner(-a / 2, -b / 2);
      face(top, [ax, ay, bx, by, qx, qy], 0);
    };
    const model = this.game.item_model(item);
    if (model.length) {
      for (let i = 0; i < model.length; i += 10) {
        draw(model[i], model[i + 1], model[i + 2], model[i + 3], model[i + 4], model[i + 5],
          model[i + 6], model[i + 7], model[i + 8]);
      }
    } else {
      draw(0, 0, 0, info[3], info[4], info[5], 0, info[0], info[1]);
    }
    ctx.setTransform(1, 0, 0, 1, 0, 0);
    // Tiered machines get a corner chip in their Mk colour with a pip per Mk.
    const tier = this.game.item_tier(item);
    if (tier >= 0) {
      ctx.fillStyle = '#1a1a1a';
      ctx.fillRect(2, 2, 8 + tier * 5, 12);
      ctx.fillStyle = `#${this.game.tier_colour(tier).toString(16).padStart(6, '0')}`;
      ctx.fillRect(3, 3, 6 + tier * 5, 10);
      ctx.fillStyle = '#f5f5f0';
      for (let p = 0; p <= tier; p++) ctx.fillRect(4 + p * 5, 6, 3, 4);
    }
    return c;
  }
}
