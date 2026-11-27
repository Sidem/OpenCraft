// Interactive bird's-eye survey viewport for the Site Grader Console (ui/grader.ts).
// Renders the live top-down terrain scan from `game.grader_survey(x, y, z)` (machine-forward = UP) inside a framed
// tactical viewport with floating translucent HUD pills (area summary / tile hover inspector, compass badge, color key).
// Players can drag any edge/corner, slide the box left/right, or sweep a new custom rectangle directly on the map.

import type { Game } from '../wasm/engine.js';
import { h } from './dom';

const CELL_PX = 8;
const REFRESH_MS = 300;

const FLAG_CUT = 1;
const FLAG_FILL = 2;
const FLAG_WATER = 4;
const FLAG_TREE = 8;
const FLAG_ORE = 16;
const FLAG_MACHINE = 32;
const FLAG_PAVED = 64;

const COMPASS_UP = ['N', 'E', 'S', 'W'];

export interface GraderArea {
  width: number;
  depth: number;
  offset: number;
  spareOres: boolean;
  floor: number;
}

type DragMode = 'new' | 'top' | 'left' | 'right' | 'move';

export class GraderSurveyMap {
  readonly el = h('div', 'gr-map-frame');
  private readonly canvas: HTMLCanvasElement;
  private readonly ctx: CanvasRenderingContext2D;
  private readonly infoChip = h('span', 'gr-hud-chip');
  private readonly compassChip = h('span', 'gr-hud-chip', '↑ N');
  private survey: Uint8Array = new Uint8Array(0);
  private pos: [number, number, number] = [0, 0, 0];
  private area: GraderArea = { width: 19, depth: 18, offset: 0, spareOres: true, floor: 0 };
  private bounds: [number, number, number, number, number];
  private readAt = 0;
  private hover: { ahead: number; side: number } | null = null;
  private drag: {
    mode: DragMode;
    anchorSide: number;
    startSide: number;
    startOffset: number;
    fixedLeft: number;
    fixedRight: number;
  } | null = null;

  constructor(
    private readonly game: Game,
    bounds: [number, number, number, number, number],
    private readonly onAreaChange: (patch: { width?: number; depth?: number; offset?: number }) => void,
  ) {
    this.bounds = bounds;
    this.canvas = document.createElement('canvas');
    this.canvas.className = 'gr-map-canvas';
    this.canvas.width = 65 * CELL_PX;
    this.canvas.height = 47 * CELL_PX;
    this.ctx = this.canvas.getContext('2d')!;

    const hudTop = h('div', 'gr-hud-top');
    hudTop.append(this.infoChip, this.compassChip);

    const hudBottom = h('div', 'gr-hud-bottom');
    const key = (color: string, label: string) => {
      const item = h('span', 'gr-key-item');
      const sw = h('span', 'gr-key-swatch');
      sw.style.background = color;
      item.append(sw, label);
      return item;
    };
    hudBottom.append(
      key('#facc15', 'Grader'),
      key('#f97316', 'Wheel'),
      key('#6ee7b7', 'Ore'),
      key('#e18a3c', 'Factory'),
    );

    this.el.append(this.canvas, hudTop, hudBottom);
    this.bindPointer();
  }

  /** Refreshes the survey scan periodically and repaints whenever the area, hover, or world scan updates. */
  update(pos: [number, number, number], area: GraderArea, force = false): void {
    const now = performance.now();
    const moved = pos.some((v, i) => v !== this.pos[i]);
    const areaChanged =
      area.width !== this.area.width ||
      area.depth !== this.area.depth ||
      area.offset !== this.area.offset ||
      area.spareOres !== this.area.spareOres ||
      area.floor !== this.area.floor;
    this.area = { ...area };
    if (force || moved || now - this.readAt >= REFRESH_MS) {
      this.pos = [...pos];
      this.survey = this.game.grader_survey(...pos);
      this.readAt = now;
      this.render();
      return;
    }
    if (areaChanged) this.render();
  }

  private gridMeta() {
    if (this.survey.length < 8) return null;
    const cols = this.survey[0];
    const rows = this.survey[1];
    const aheadMax = this.survey[3];
    const sideHalf = this.survey[4];
    const curX = this.survey[5];
    const curY = this.survey[6];
    const dir = this.survey[7];
    return { cols, rows, aheadMax, sideHalf, curX, curY, dir };
  }

  private areaBounds() {
    const left = -Math.floor(this.area.width / 2) + this.area.offset;
    const right = left + this.area.width - 1;
    const near = 3;
    const far = 3 + this.area.depth - 1;
    return { left, right, near, far };
  }

  private cellFromEvent(e: PointerEvent): { ahead: number; side: number } | null {
    const m = this.gridMeta();
    if (!m) return null;
    const rect = this.canvas.getBoundingClientRect();
    if (rect.width <= 0 || rect.height <= 0) return null;
    // Account for `object-fit: contain` letterboxing if the container aspect differs slightly.
    const canvasAspect = this.canvas.width / this.canvas.height;
    const boxAspect = rect.width / rect.height;
    let drawW = rect.width;
    let drawH = rect.height;
    let ox = rect.left;
    let oy = rect.top;
    if (boxAspect > canvasAspect) {
      drawW = rect.height * canvasAspect;
      ox = rect.left + (rect.width - drawW) * 0.5;
    } else {
      drawH = rect.width / canvasAspect;
      oy = rect.top + (rect.height - drawH) * 0.5;
    }
    const gx = Math.floor(((e.clientX - ox) / drawW) * m.cols);
    const gy = Math.floor(((e.clientY - oy) / drawH) * m.rows);
    const side = Math.max(-m.sideHalf, Math.min(m.sideHalf, gx - m.sideHalf));
    const ahead = Math.max(m.aheadMax - (m.rows - 1), Math.min(m.aheadMax, m.aheadMax - gy));
    return { ahead, side };
  }

  private bindPointer(): void {
    this.canvas.addEventListener('pointerdown', (e) => {
      if (e.button !== 0) return;
      const cell = this.cellFromEvent(e);
      if (!cell) return;
      e.preventDefault();
      this.canvas.setPointerCapture(e.pointerId);
      const { left, right, near, far } = this.areaBounds();
      const onVertSpan = cell.ahead >= near - 1 && cell.ahead <= far + 1;
      const onHorizSpan = cell.side >= left - 1 && cell.side <= right + 1;

      let mode: DragMode = 'new';
      if (onVertSpan && Math.abs(cell.side - left) <= 1) mode = 'left';
      else if (onVertSpan && Math.abs(cell.side - right) <= 1) mode = 'right';
      else if (onHorizSpan && Math.abs(cell.ahead - far) <= 1) mode = 'top';
      else if (cell.ahead >= near && cell.ahead <= far && cell.side >= left && cell.side <= right) mode = 'move';

      this.drag = {
        mode,
        anchorSide: cell.side,
        startSide: cell.side,
        startOffset: this.area.offset,
        fixedLeft: left,
        fixedRight: right,
      };
      if (mode === 'new' && cell.ahead >= near) {
        this.applyDrag(cell);
      }
    });

    this.canvas.addEventListener('pointermove', (e) => {
      const cell = this.cellFromEvent(e);
      this.hover = cell;
      if (this.drag && cell) {
        this.applyDrag(cell);
      } else if (cell) {
        this.updateCursor(cell);
        this.updateHud();
      }
    });

    const endDrag = () => {
      this.drag = null;
    };
    this.canvas.addEventListener('pointerup', endDrag);
    this.canvas.addEventListener('pointercancel', endDrag);
    this.canvas.addEventListener('pointerleave', () => {
      this.hover = null;
      if (!this.drag) this.updateHud();
    });
  }

  private updateCursor(cell: { ahead: number; side: number }): void {
    const { left, right, near, far } = this.areaBounds();
    const onVertSpan = cell.ahead >= near - 1 && cell.ahead <= far + 1;
    const onHorizSpan = cell.side >= left - 1 && cell.side <= right + 1;
    if (onVertSpan && (Math.abs(cell.side - left) <= 1 || Math.abs(cell.side - right) <= 1)) {
      this.canvas.style.cursor = 'ew-resize';
    } else if (onHorizSpan && Math.abs(cell.ahead - far) <= 1) {
      this.canvas.style.cursor = 'ns-resize';
    } else if (cell.ahead >= near && cell.ahead <= far && cell.side >= left && cell.side <= right) {
      this.canvas.style.cursor = 'grab';
    } else {
      this.canvas.style.cursor = 'crosshair';
    }
  }

  private applyDrag(cell: { ahead: number; side: number }): void {
    if (!this.drag) return;
    const [minW, maxW, minD, maxD, maxOff] = this.bounds;
    const clamp = (v: number, lo: number, hi: number) => Math.max(lo, Math.min(hi, v));
    const fromEdges = (lo: number, hi: number) => {
      const w = clamp(hi - lo + 1, minW, maxW);
      const off = clamp(lo + Math.floor(w / 2), -maxOff, maxOff);
      return { width: w, offset: off };
    };

    if (this.drag.mode === 'top') {
      const depth = clamp(cell.ahead - 2, minD, maxD);
      this.onAreaChange({ depth });
    } else if (this.drag.mode === 'left') {
      const lo = Math.min(cell.side, this.drag.fixedRight - (minW - 1));
      const { width, offset } = fromEdges(lo, this.drag.fixedRight);
      this.onAreaChange({ width, offset });
    } else if (this.drag.mode === 'right') {
      const hi = Math.max(cell.side, this.drag.fixedLeft + (minW - 1));
      const { width, offset } = fromEdges(this.drag.fixedLeft, hi);
      this.onAreaChange({ width, offset });
    } else if (this.drag.mode === 'move') {
      const delta = cell.side - this.drag.startSide;
      const offset = clamp(this.drag.startOffset + delta, -maxOff, maxOff);
      this.onAreaChange({ offset });
    } else {
      const lo = Math.min(this.drag.anchorSide, cell.side);
      const hi = Math.max(this.drag.anchorSide, cell.side);
      const { width, offset } = fromEdges(lo, hi);
      const depth = clamp(cell.ahead - 2, minD, maxD);
      this.onAreaChange({ width, depth, offset });
    }
  }

  private render(): void {
    const m = this.gridMeta();
    if (!m) return;
    const { cols, rows, aheadMax, sideHalf, curX, curY, dir } = m;
    if (this.canvas.width !== cols * CELL_PX || this.canvas.height !== rows * CELL_PX) {
      this.canvas.width = cols * CELL_PX;
      this.canvas.height = rows * CELL_PX;
    }
    const ctx = this.ctx;
    const { left, right, near, far } = this.areaBounds();

    // 1. Shaded top-down terrain tiles with crisp outside-pad dimming.
    for (let gy = 0; gy < rows; gy++) {
      const ahead = aheadMax - gy;
      for (let gx = 0; gx < cols; gx++) {
        const side = gx - sideHalf;
        const idx = 8 + (gy * cols + gx) * 5;
        let r = this.survey[idx];
        let g = this.survey[idx + 1];
        let b = this.survey[idx + 2];
        const flags = this.survey[idx + 4];
        const inside = ahead >= near && ahead <= far && side >= left && side <= right;

        if (!inside) {
          r = Math.round(r * 0.52);
          g = Math.round(g * 0.54);
          b = Math.round(b * 0.6);
        }
        ctx.fillStyle = `rgb(${r},${g},${b})`;
        ctx.fillRect(gx * CELL_PX, gy * CELL_PX, CELL_PX, CELL_PX);

        // Elevation contour bevel.
        if (gx > 0 && gy > 0) {
          const dHere = this.survey[idx + 3];
          const dUp = this.survey[8 + ((gy - 1) * cols + gx) * 5 + 3];
          const dLeft = this.survey[8 + (gy * cols + (gx - 1)) * 5 + 3];
          if (dHere !== dUp) {
            ctx.fillStyle = dHere > dUp ? 'rgba(255,255,255,0.16)' : 'rgba(0,0,0,0.22)';
            ctx.fillRect(gx * CELL_PX, gy * CELL_PX, CELL_PX, 1);
          }
          if (dHere !== dLeft) {
            ctx.fillStyle = dHere > dLeft ? 'rgba(255,255,255,0.12)' : 'rgba(0,0,0,0.18)';
            ctx.fillRect(gx * CELL_PX, gy * CELL_PX, 1, CELL_PX);
          }
        }

        // Subtle paved check inside pad.
        if (inside && (flags & FLAG_PAVED) !== 0) {
          ctx.fillStyle = 'rgba(255,255,255,0.12)';
          ctx.fillRect(gx * CELL_PX + 1, gy * CELL_PX + 1, CELL_PX - 2, CELL_PX - 2);
        }

        // Ore deposit diamond marker (mint green when spared, amber when mined).
        if ((flags & FLAG_ORE) !== 0) {
          const cx = gx * CELL_PX + CELL_PX * 0.5;
          const cy = gy * CELL_PX + CELL_PX * 0.5;
          ctx.fillStyle = this.area.spareOres ? '#6ee7b7' : '#f59e0b';
          ctx.beginPath();
          ctx.moveTo(cx, cy - 3);
          ctx.lineTo(cx + 3, cy);
          ctx.lineTo(cx, cy + 3);
          ctx.lineTo(cx - 3, cy);
          ctx.closePath();
          ctx.fill();
        }
      }
    }

    // 2. Clean 5-block survey grid inside the active grading area.
    const rx0 = (left + sideHalf) * CELL_PX;
    const ry0 = (aheadMax - far) * CELL_PX;
    const rw = this.area.width * CELL_PX;
    const rh = this.area.depth * CELL_PX;
    ctx.strokeStyle = 'rgba(251, 191, 36, 0.2)';
    ctx.lineWidth = 1;
    for (let sx = 5; sx < this.area.width; sx += 5) {
      ctx.beginPath();
      ctx.moveTo(rx0 + sx * CELL_PX + 0.5, ry0);
      ctx.lineTo(rx0 + sx * CELL_PX + 0.5, ry0 + rh);
      ctx.stroke();
    }
    for (let sy = 5; sy < this.area.depth; sy += 5) {
      ctx.beginPath();
      ctx.moveTo(rx0, ry0 + sy * CELL_PX + 0.5);
      ctx.lineTo(rx0 + rw, ry0 + sy * CELL_PX + 0.5);
      ctx.stroke();
    }

    // 3. Crisp glowing gold selection rectangle + corner & edge drag handles.
    ctx.strokeStyle = '#fbbf24';
    ctx.lineWidth = 2;
    ctx.strokeRect(rx0 + 1, ry0 + 1, rw - 2, rh - 2);
    ctx.fillStyle = '#fef08a';
    for (const [bx, by] of [
      [rx0, ry0],
      [rx0 + rw, ry0],
      [rx0, ry0 + rh],
      [rx0 + rw, ry0 + rh],
    ]) {
      ctx.fillRect(bx - 3.5, by - 3.5, 7, 7);
    }
    // Mid-edge drag pips (top, left, right) so drag affordance is unmistakable.
    ctx.fillStyle = '#fbbf24';
    ctx.fillRect(rx0 + rw * 0.5 - 8, ry0 - 2, 16, 4);
    ctx.fillRect(rx0 - 2, ry0 + rh * 0.5 - 8, 4, 16);
    ctx.fillRect(rx0 + rw - 2, ry0 + rh * 0.5 - 8, 4, 16);

    // 4. 3×3 Site Grader footprint + intake/output arrows + live boom & wheel crosshair.
    const gx0 = (-1 + sideHalf) * CELL_PX;
    const gy0 = (aheadMax - 2) * CELL_PX;
    const gw = 3 * CELL_PX;
    ctx.fillStyle = '#1f242d';
    ctx.fillRect(gx0, gy0, gw, gw);
    ctx.strokeStyle = '#facc15';
    ctx.lineWidth = 2;
    ctx.strokeRect(gx0 + 1, gy0 + 1, gw - 2, gw - 2);
    // Left floor intake port (sky blue) & rear spoil discharge port (orange).
    ctx.fillStyle = '#38bdf8';
    ctx.fillRect(gx0 - 4, gy0 + gw / 2 - 2.5, 4, 5);
    ctx.fillStyle = '#fb923c';
    ctx.fillRect(gx0 + gw / 2 - 2.5, gy0 + gw, 5, 4);

    if (curX < cols && curY < rows) {
      const px = (sideHalf + 0.5) * CELL_PX;
      const py = (aheadMax - 1 + 0.5) * CELL_PX;
      const tx = (curX + 0.5) * CELL_PX;
      const ty = (curY + 0.5) * CELL_PX;
      ctx.strokeStyle = 'rgba(0, 0, 0, 0.55)';
      ctx.lineWidth = 4;
      ctx.beginPath();
      ctx.moveTo(px, py);
      ctx.lineTo(tx, ty);
      ctx.stroke();
      ctx.strokeStyle = '#fde047';
      ctx.lineWidth = 2.2;
      ctx.beginPath();
      ctx.moveTo(px, py);
      ctx.lineTo(tx, ty);
      ctx.stroke();
      ctx.fillStyle = '#f97316';
      ctx.beginPath();
      ctx.arc(tx, ty, 4.5, 0, Math.PI * 2);
      ctx.fill();
      ctx.strokeStyle = '#ffffff';
      ctx.lineWidth = 1.2;
      ctx.stroke();
    }

    this.compassChip.textContent = `↑ ${COMPASS_UP[dir % 4] ?? 'N'}`;
    this.updateHud();
  }

  private updateHud(): void {
    const m = this.gridMeta();
    if (!m) return;
    const { cols, aheadMax, sideHalf } = m;
    if (this.hover) {
      const gx = this.hover.side + sideHalf;
      const gy = aheadMax - this.hover.ahead;
      if (gx >= 0 && gx < cols && gy >= 0 && gy < m.rows) {
        const idx = 8 + (gy * cols + gx) * 5;
        const delta = this.survey[idx + 3] - 128;
        const flags = this.survey[idx + 4];
        const tags: string[] = [];
        if ((flags & FLAG_MACHINE) !== 0) tags.push('Factory');
        else if ((flags & FLAG_WATER) !== 0) tags.push(`Water ${delta}m`);
        else if ((flags & FLAG_CUT) !== 0) tags.push(`Hill +${delta}m`);
        else if ((flags & FLAG_FILL) !== 0) tags.push(`Hollow ${delta}m`);
        else if ((flags & FLAG_PAVED) !== 0) tags.push('Paved');
        else tags.push('Level');
        if ((flags & FLAG_TREE) !== 0) tags.push('Tree');
        if ((flags & FLAG_ORE) !== 0) tags.push(this.area.spareOres ? 'Ore (spared)' : 'Ore (mined)');
        const sx = this.hover.side >= 0 ? `+${this.hover.side}` : String(this.hover.side);
        this.infoChip.textContent = `(${sx}, ${this.hover.ahead}m) · ${tags.join(' · ')}`;
        return;
      }
    }

    const { left, right, near, far } = this.areaBounds();
    let hills = 0;
    let hollows = 0;
    let ores = 0;
    let trees = 0;
    for (let ahead = near; ahead <= far; ahead++) {
      const gy = aheadMax - ahead;
      if (gy < 0 || gy >= m.rows) continue;
      for (let side = left; side <= right; side++) {
        const gx = side + sideHalf;
        if (gx < 0 || gx >= cols) continue;
        const flags = this.survey[8 + (gy * cols + gx) * 5 + 4];
        if ((flags & FLAG_CUT) !== 0) hills++;
        if ((flags & FLAG_FILL) !== 0) hollows++;
        if ((flags & FLAG_ORE) !== 0) ores++;
        if ((flags & FLAG_TREE) !== 0) trees++;
      }
    }
    const parts = [`${this.area.width}×${this.area.depth}`, `Hills ${hills}`, `Fill ${hollows}`];
    if (trees > 0) parts.push(`Trees ${trees}`);
    if (ores > 0) parts.push(`Ores ${ores}`);
    this.infoChip.textContent = parts.join(' · ');
  }
}
