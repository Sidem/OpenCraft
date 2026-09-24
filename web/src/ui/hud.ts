import type { Game } from '../wasm/engine.js';

const ICON_PX = 64;
const TOAST_MS = 2600;

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

/** DOM overlay: hotbar, target readout, mining progress, pickup toasts and the F3 debug panel. */
export class Hud {
  private readonly slots: Slot[] = [];
  private readonly icons = new Map<number, HTMLCanvasElement>();
  private readonly layers: HTMLCanvasElement[] = [];
  private readonly names: string[] = [];
  private readonly toasts = new Map<number, Toast>();
  private invVersion = -1;
  private debugVisible = false;
  private lastDebug = 0;

  private readonly target = $<HTMLDivElement>('target');
  private readonly targetName = $<HTMLSpanElement>('target-name');
  private readonly mineFill = $<HTMLDivElement>('mine-fill');
  private readonly toastBox = $<HTMLDivElement>('toasts');
  private readonly debug = $<HTMLPreElement>('debug');

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

  update(now: number, info: DebugInfo): void {
    const g = this.game;

    if (g.inventory_version() !== this.invVersion) {
      this.invVersion = g.inventory_version();
      const selected = g.selected_slot();
      this.slots.forEach((s, i) => {
        const item = g.slot_item(i);
        const n = g.slot_count(i);
        s.root.classList.toggle('selected', i === selected);
        s.root.title = n > 0 ? this.names[item] : '';
        if (item !== s.item || n !== s.n) {
          const ctx = s.icon.getContext('2d')!;
          ctx.clearRect(0, 0, ICON_PX, ICON_PX);
          if (n > 0) ctx.drawImage(this.icon(item), 0, 0);
          s.count.textContent = n > 1 ? String(n) : '';
          s.item = item;
          s.n = n;
        }
      });
    }

    if (g.has_target()) {
      this.target.classList.remove('hidden');
      this.targetName.textContent = this.names[g.target_block()] ?? '';
      const p = g.mine_progress();
      this.mineFill.style.transform = `scaleX(${p})`;
      this.target.classList.toggle('mining', p > 0);
    } else {
      this.target.classList.add('hidden');
    }

    while (g.next_pickup()) this.pushToast(g.pickup_item(), g.pickup_count(), now);
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
      ].join('\n');
    }
  }

  private pushToast(item: number, count: number, now: number): void {
    const existing = this.toasts.get(item);
    if (existing) {
      existing.count += count;
      existing.label.textContent = `+${existing.count}`;
      existing.until = now + TOAST_MS;
      existing.el.classList.remove('bump');
      void existing.el.offsetWidth;
      existing.el.classList.add('bump');
      return;
    }
    const el = document.createElement('div');
    el.className = 'toast';
    const icon = document.createElement('canvas');
    icon.width = icon.height = ICON_PX;
    icon.getContext('2d')!.drawImage(this.icon(item), 0, 0);
    const label = document.createElement('span');
    label.className = 'amount';
    label.textContent = `+${count}`;
    const name = document.createElement('span');
    name.textContent = this.names[item];
    el.append(icon, label, name);
    this.toastBox.append(el);
    this.toasts.set(item, { el, label, count, until: now + TOAST_MS });
  }

  /** Isometric block icon drawn from the block's top and side textures. */
  private icon(item: number): HTMLCanvasElement {
    let c = this.icons.get(item);
    if (c) return c;
    c = document.createElement('canvas');
    c.width = c.height = ICON_PX;
    const ctx = c.getContext('2d')!;
    ctx.imageSmoothingEnabled = false;
    const S = ICON_PX * 0.9, o = (ICON_PX - S) / 2, k = S / 16;
    const top = this.layers[this.game.block_face_texture(item, 2)];
    const side = this.layers[this.game.block_face_texture(item, 0)];
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
    // Each face is shaded with `source-atop` inside its own parallelogram, so it only darkens itself.
    face(side, [k / 2, k / 4, 0, k / 2, o, o + S / 4], 0.22);
    face(side, [k / 2, -k / 4, 0, k / 2, o + S / 2, o + S / 2], 0.4);
    face(top, [k / 2, k / 4, -k / 2, k / 4, o + S / 2, o], 0);
    ctx.setTransform(1, 0, 0, 1, 0, 0);
    this.icons.set(item, c);
    return c;
  }
}
