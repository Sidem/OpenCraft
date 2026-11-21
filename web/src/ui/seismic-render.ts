// Seismic cross-section canvas renderer (F-SEIS): draws Tomography and Seismogram wiggle-trace views, animated
// P-wavefront sweeps, surface contour, sea-level datum, thumper/player markers, top-down radar inset, caliper,
// crosshair, rulers and the live mini-seismograph strip. Data and palettes live in `seismic-data.ts`.

import {
  cellToCanvas, CLASS_SKY, colorForCell, compassEnds, dirAxis, perpLabel, plotRect,
  type DisplayMode, type GridPoint, type PlotRect, type SliceData,
} from './seismic-data';

export interface RenderState {
  mode: DisplayMode;
  waveProgress: number;
  hover: GridPoint | null;
  focus: GridPoint | null;
  caliperA: GridPoint | null;
  caliperB: GridPoint | null;
  oreColors: Map<number, [number, number, number]>;
}

/** Offscreen pixel buffer cached per instance to avoid allocations on every animation frame. */
export class SeismicRenderer {
  private off = document.createElement('canvas');
  private offCtx = this.off.getContext('2d')!;
  private img: ImageData | null = null;

  draw(ctx: CanvasRenderingContext2D, w: number, h: number, d: SliceData, st: RenderState): PlotRect {
    const plot = plotRect(w, h);
    ctx.clearRect(0, 0, w, h);
    ctx.fillStyle = '#0a0e16';
    ctx.fillRect(0, 0, w, h);

    this.drawSliceBitmap(ctx, plot, d, st);
    if (st.mode === 'wiggle') this.drawWiggleTraces(ctx, plot, d, st.waveProgress);
    this.drawGraticule(ctx, plot, d);
    this.drawSurfaceAndDatums(ctx, plot, d, st);
    if (st.waveProgress < 1) this.drawWavefront(ctx, plot, d, st.waveProgress);
    this.drawCaliper(ctx, plot, d, st);
    this.drawCrosshair(ctx, plot, d, st);
    this.drawRadarInset(ctx, plot, d);
    this.drawRulers(ctx, plot, d);
    return plot;
  }

  private drawSliceBitmap(ctx: CanvasRenderingContext2D, plot: PlotRect, d: SliceData, st: RenderState): void {
    if (this.off.width !== d.width || this.off.height !== d.height) {
      this.off.width = d.width;
      this.off.height = d.height;
      this.img = this.offCtx.createImageData(d.width, d.height);
    }
    const out = this.img!.data;
    const waveR = st.waveProgress < 1 ? st.waveProgress * Math.hypot(d.width * 0.55, d.height * 1.05) : 1e9;
    const originS = d.width >> 1;
    const originY = d.surfaceAtThumper;

    for (let y = 0; y < d.height; y++) {
      const row = d.height - 1 - y;
      for (let s = 0; s < d.width; s++) {
        const idx = (y * d.width + s) * 4;
        const pIdx = (row * d.width + s) * 4;
        const cls = d.cells[idx + 2];
        const dist = Math.hypot(s - originS, y - originY);
        if (cls === CLASS_SKY) {
          const skyT = Math.min(1, Math.max(0, (y - d.surface[s]) / 16));
          out[pIdx] = Math.round(13 - skyT * 4);
          out[pIdx + 1] = Math.round(20 - skyT * 6);
          out[pIdx + 2] = Math.round(33 - skyT * 8);
          out[pIdx + 3] = 255;
          continue;
        }
        if (dist > waveR) {
          out[pIdx] = 12;
          out[pIdx + 1] = 17;
          out[pIdx + 2] = 26;
          out[pIdx + 3] = 255;
          continue;
        }
        const [r, g, b] = colorForCell(st.mode, d.cells[idx], cls, d.cells[idx + 3], st.oreColors);
        const rim = st.waveProgress < 1 && waveR - dist < 3.5 ? (1 - (waveR - dist) / 3.5) * 0.45 : 0;
        out[pIdx] = Math.min(255, Math.round(r + (250 - r) * rim));
        out[pIdx + 1] = Math.min(255, Math.round(g + (165 - g) * rim));
        out[pIdx + 2] = Math.min(255, Math.round(b + (95 - b) * rim));
        out[pIdx + 3] = 255;
      }
    }
    this.offCtx.putImageData(this.img!, 0, 0);
    ctx.save();
    ctx.imageSmoothingEnabled = d.tier < 2;
    ctx.drawImage(this.off, plot.x, plot.y, plot.w, plot.h);
    ctx.restore();
  }

  private drawWiggleTraces(ctx: CanvasRenderingContext2D, plot: PlotRect, d: SliceData, waveProgress: number): void {
    const amp = (plot.w / d.width) * 1.45;
    const waveR = waveProgress < 1 ? waveProgress * Math.hypot(d.width * 0.55, d.height * 1.05) : 1e9;
    const originS = d.width >> 1;
    const originY = d.surfaceAtThumper;

    ctx.save();
    ctx.beginPath();
    ctx.rect(plot.x, plot.y, plot.w, plot.h);
    ctx.clip();

    for (let s = 1; s < d.width; s += 2) {
      const baseX = plot.x + ((s + 0.5) / d.width) * plot.w;
      const surf = Math.min(d.height - 1, d.surface[s]);
      const [, topPy] = cellToCanvas(plot, d, s, surf);
      ctx.strokeStyle = 'rgba(148, 163, 184, 0.14)';
      ctx.lineWidth = 1;
      ctx.beginPath();
      ctx.moveTo(baseX, topPy);
      ctx.lineTo(baseX, plot.y + plot.h);
      ctx.stroke();

      ctx.beginPath();
      let started = false;
      for (let y = surf; y >= 0; y--) {
        if (Math.hypot(s - originS, y - originY) > waveR) continue;
        const grad = (d.cells[(y * d.width + s) * 4 + 1] - 128) / 127;
        const py = plot.y + (1 - (y + 0.5) / d.height) * plot.h;
        const wx = baseX + grad * amp;
        if (!started) { ctx.moveTo(wx, py); started = true; } else ctx.lineTo(wx, py);
        if (Math.abs(grad) > 0.18) {
          ctx.fillStyle = grad > 0 ? 'rgba(250, 149, 73, 0.42)' : 'rgba(56, 189, 248, 0.36)';
          const rh = Math.max(1.5, plot.h / d.height);
          ctx.fillRect(grad > 0 ? baseX : wx, py - rh * 0.5, Math.abs(wx - baseX), rh);
        }
      }
      ctx.strokeStyle = 'rgba(238, 244, 255, 0.72)';
      ctx.lineWidth = 1.1;
      ctx.stroke();
    }
    ctx.restore();
  }

  private drawGraticule(ctx: CanvasRenderingContext2D, plot: PlotRect, d: SliceData): void {
    ctx.save();
    ctx.strokeStyle = 'rgba(255, 255, 255, 0.065)';
    ctx.lineWidth = 1;
    for (let off = -24; off <= 24; off += 8) {
      const x = Math.round(plot.x + ((off + (d.width >> 1) + 0.5) / d.width) * plot.w) + 0.5;
      ctx.beginPath();
      ctx.moveTo(x, plot.y);
      ctx.lineTo(x, plot.y + plot.h);
      ctx.stroke();
    }
    for (let y = 16; y < d.height; y += 16) {
      const py = Math.round(plot.y + (1 - y / d.height) * plot.h) + 0.5;
      ctx.beginPath();
      ctx.moveTo(plot.x, py);
      ctx.lineTo(plot.x + plot.w, py);
      ctx.stroke();
    }
    ctx.strokeStyle = 'rgba(255, 255, 255, 0.16)';
    ctx.strokeRect(plot.x + 0.5, plot.y + 0.5, plot.w - 1, plot.h - 1);
    ctx.restore();
  }

  private drawSurfaceAndDatums(ctx: CanvasRenderingContext2D, plot: PlotRect, d: SliceData, st: RenderState): void {
    ctx.save();
    if (d.seaLevel > 0 && d.seaLevel < d.height) {
      const sy = Math.round(plot.y + (1 - (d.seaLevel + 0.5) / d.height) * plot.h) + 0.5;
      ctx.setLineDash([5, 5]);
      ctx.strokeStyle = 'rgba(56, 189, 248, 0.48)';
      ctx.beginPath();
      ctx.moveTo(plot.x, sy);
      ctx.lineTo(plot.x + plot.w, sy);
      ctx.stroke();
      ctx.setLineDash([]);
      ctx.fillStyle = 'rgba(56, 189, 248, 0.85)';
      ctx.font = '600 10px var(--mono, monospace)';
      ctx.fillText(`SEA ${d.seaLevel}`, plot.x + plot.w - 46, sy - 4);
    }

    ctx.beginPath();
    for (let s = 0; s < d.width; s++) {
      const [cx, cy] = cellToCanvas(plot, d, s, Math.min(d.height - 1, d.surface[s]));
      if (s === 0) ctx.moveTo(cx, cy); else ctx.lineTo(cx, cy);
    }
    ctx.strokeStyle = 'rgba(226, 232, 240, 0.65)';
    ctx.lineWidth = 1.6;
    ctx.stroke();

    const [tx, ty] = cellToCanvas(plot, d, d.width >> 1, Math.min(d.height - 1, d.anchorY));
    ctx.fillStyle = '#fa9549';
    ctx.beginPath();
    ctx.moveTo(tx, ty + 3);
    ctx.lineTo(tx - 6, ty - 7);
    ctx.lineTo(tx + 6, ty - 7);
    ctx.closePath();
    ctx.fill();
    ctx.strokeStyle = '#0a0e16';
    ctx.lineWidth = 1.2;
    ctx.stroke();

    const pS = d.playerS + (d.width >> 1);
    if (d.playerPerp <= 12 && pS >= 0 && pS < d.width && d.playerY >= 0 && d.playerY < d.height) {
      const [px, py] = cellToCanvas(plot, d, pS, d.playerY);
      ctx.fillStyle = '#f8fafc';
      ctx.beginPath();
      ctx.arc(px, py, 4, 0, Math.PI * 2);
      ctx.fill();
      ctx.strokeStyle = '#fa9549';
      ctx.lineWidth = 1.6;
      ctx.stroke();
    }

    if (st.focus) {
      const [fx, fy] = cellToCanvas(plot, d, st.focus.s, st.focus.y);
      ctx.strokeStyle = '#fde047';
      ctx.lineWidth = 1.8;
      ctx.beginPath();
      ctx.arc(fx, fy, 9, 0, Math.PI * 2);
      ctx.stroke();
      ctx.strokeStyle = 'rgba(253, 224, 71, 0.35)';
      ctx.lineWidth = 1;
      ctx.beginPath();
      ctx.arc(fx, fy, 15, 0, Math.PI * 2);
      ctx.stroke();
    }
    ctx.restore();
  }

  private drawWavefront(ctx: CanvasRenderingContext2D, plot: PlotRect, d: SliceData, progress: number): void {
    const [ox, oy] = cellToCanvas(plot, d, d.width >> 1, Math.min(d.height - 1, d.surfaceAtThumper));
    const r = progress * Math.hypot(plot.w * 0.58, plot.h * 1.05);
    ctx.save();
    ctx.beginPath();
    ctx.rect(plot.x, plot.y, plot.w, plot.h);
    ctx.clip();
    ctx.strokeStyle = `rgba(250, 165, 85, ${(1 - progress * 0.7) * 0.9})`;
    ctx.lineWidth = 2.4;
    ctx.beginPath();
    ctx.arc(ox, oy, r, 0, Math.PI);
    ctx.stroke();
    if (r > 18) {
      ctx.strokeStyle = `rgba(56, 189, 248, ${(1 - progress * 0.8) * 0.55})`;
      ctx.lineWidth = 1.2;
      ctx.beginPath();
      ctx.arc(ox, oy, r - 14, 0, Math.PI);
      ctx.stroke();
    }
    ctx.restore();
  }

  private drawCaliper(ctx: CanvasRenderingContext2D, plot: PlotRect, d: SliceData, st: RenderState): void {
    if (!st.caliperA || !st.caliperB) return;
    const [ax, ay] = cellToCanvas(plot, d, st.caliperA.s, st.caliperA.y);
    const [bx, by] = cellToCanvas(plot, d, st.caliperB.s, st.caliperB.y);
    const dh = Math.abs(st.caliperB.s - st.caliperA.s);
    const dy = st.caliperB.y - st.caliperA.y;
    const dist = Math.hypot(dh, dy);
    const angle = Math.round((Math.atan2(Math.abs(dy), Math.max(0.001, dh)) * 180) / Math.PI);

    ctx.save();
    ctx.setLineDash([6, 4]);
    ctx.strokeStyle = '#fde047';
    ctx.lineWidth = 1.8;
    ctx.beginPath();
    ctx.moveTo(ax, ay);
    ctx.lineTo(bx, by);
    ctx.stroke();
    ctx.setLineDash([]);
    for (const [px, py] of [[ax, ay], [bx, by]]) {
      ctx.fillStyle = '#fde047';
      ctx.beginPath();
      ctx.arc(px, py, 3.8, 0, Math.PI * 2);
      ctx.fill();
    }
    const label = `ΔH ${dh}m · ΔY ${dy >= 0 ? '+' : '−'}${Math.abs(dy)}m · ${dist.toFixed(1)}m (${angle}°)`;
    ctx.font = '600 11px var(--mono, monospace)';
    const tw = ctx.measureText(label).width + 14;
    const mx = Math.min(plot.x + plot.w - tw - 6, Math.max(plot.x + 6, (ax + bx) * 0.5 - tw * 0.5));
    const my = Math.min(plot.y + plot.h - 24, Math.max(plot.y + 8, (ay + by) * 0.5 - 22));
    ctx.fillStyle = 'rgba(12, 16, 24, 0.92)';
    ctx.strokeStyle = 'rgba(253, 224, 71, 0.65)';
    ctx.lineWidth = 1;
    ctx.beginPath();
    ctx.roundRect(mx, my, tw, 20, 6);
    ctx.fill();
    ctx.stroke();
    ctx.fillStyle = '#fef08a';
    ctx.fillText(label, mx + 7, my + 14);
    ctx.restore();
  }

  private drawCrosshair(ctx: CanvasRenderingContext2D, plot: PlotRect, d: SliceData, st: RenderState): void {
    if (!st.hover) return;
    const [hx, hy] = cellToCanvas(plot, d, st.hover.s, st.hover.y);
    ctx.save();
    ctx.strokeStyle = 'rgba(248, 250, 252, 0.35)';
    ctx.lineWidth = 1;
    ctx.beginPath();
    ctx.moveTo(plot.x, hy);
    ctx.lineTo(plot.x + plot.w, hy);
    ctx.moveTo(hx, plot.y);
    ctx.lineTo(hx, plot.y + plot.h);
    ctx.stroke();
    ctx.strokeStyle = '#fa9549';
    ctx.lineWidth = 1.4;
    ctx.strokeRect(hx - 4, hy - 4, 8, 8);
    ctx.restore();
  }

  private drawRulers(ctx: CanvasRenderingContext2D, plot: PlotRect, d: SliceData): void {
    ctx.save();
    ctx.fillStyle = '#94a3b8';
    ctx.font = '500 10px var(--mono, monospace)';
    ctx.textAlign = 'right';
    ctx.textBaseline = 'middle';
    for (let y = 0; y < d.height; y += 16) {
      ctx.fillText(y === 0 ? 'Y 0' : `${y}`, plot.x - 7, plot.y + (1 - (y + 0.5) / d.height) * plot.h);
    }
    ctx.textAlign = 'center';
    ctx.textBaseline = 'top';
    for (let off = -32; off <= 32; off += 8) {
      const s = Math.min(d.width - 1, Math.max(0, off + (d.width >> 1)));
      ctx.fillStyle = off === 0 ? '#fa9549' : '#94a3b8';
      ctx.fillText(off === 0 ? '0' : `${off > 0 ? '+' : '−'}${Math.abs(off)}m`, plot.x + ((s + 0.5) / d.width) * plot.w, plot.y + plot.h + 8);
    }
    const [neg, pos] = compassEnds(d.dir);
    ctx.font = '700 11px var(--mono, monospace)';
    ctx.textBaseline = 'bottom';
    ctx.textAlign = 'left';
    ctx.fillStyle = '#cbd5e1';
    ctx.fillText(`◀ ${neg} (−32m)`, plot.x, plot.y - 7);
    ctx.textAlign = 'center';
    ctx.fillStyle = '#fa9549';
    ctx.fillText(`THUMPER (${d.anchorX}, ${d.anchorY}, ${d.anchorZ}) · SLICE ${perpLabel(d.dir, d.perpOffset)}`, plot.x + plot.w * 0.5, plot.y - 7);
    ctx.textAlign = 'right';
    ctx.fillStyle = '#cbd5e1';
    ctx.fillText(`(+32m) ${pos} ▶`, plot.x + plot.w, plot.y - 7);
    ctx.restore();
  }

  private drawRadarInset(ctx: CanvasRenderingContext2D, plot: PlotRect, d: SliceData): void {
    const size = 64, rad = 24, x = plot.x + 10, y = plot.y + 10, cx = x + size * 0.5, cy = y + size * 0.5;
    const [ax, az] = dirAxis(d.dir), [px, pz] = dirAxis(d.dir + 1);
    ctx.save();
    ctx.fillStyle = 'rgba(8, 12, 20, 0.86)';
    ctx.strokeStyle = 'rgba(148, 163, 184, 0.32)';
    ctx.lineWidth = 1;
    ctx.beginPath(); ctx.roundRect(x, y, size, size, 7); ctx.fill(); ctx.stroke();
    ctx.strokeStyle = 'rgba(148, 163, 184, 0.16)';
    ctx.beginPath();
    ctx.moveTo(cx - rad, cy); ctx.lineTo(cx + rad, cy);
    ctx.moveTo(cx, cy - rad); ctx.lineTo(cx, cy + rad);
    ctx.stroke();
    ctx.setLineDash([2, 2]);
    ctx.strokeStyle = 'rgba(56, 189, 248, 0.28)';
    ctx.strokeRect(cx - rad * 0.75, cy - rad * 0.75, rad * 1.5, rad * 1.5);
    ctx.setLineDash([]);
    const sx0 = cx + ((px * d.perpOffset - ax * 32) / 32) * rad, sy0 = cy + ((pz * d.perpOffset - az * 32) / 32) * rad;
    const sx1 = cx + ((px * d.perpOffset + ax * 32) / 32) * rad, sy1 = cy + ((pz * d.perpOffset + az * 32) / 32) * rad;
    ctx.strokeStyle = '#fa9549'; ctx.lineWidth = 2.2;
    ctx.beginPath(); ctx.moveTo(sx0, sy0); ctx.lineTo(sx1, sy1); ctx.stroke();
    ctx.fillStyle = '#fa9549';
    ctx.beginPath(); ctx.arc(cx, cy, 2.6, 0, Math.PI * 2); ctx.fill();
    ctx.fillStyle = '#cbd5e1'; ctx.font = '700 9px var(--mono, monospace)';
    ctx.textAlign = 'center'; ctx.textBaseline = 'top'; ctx.fillText('N', cx, y + 2);
    ctx.restore();
  }
}

/** Renders the live mini-seismograph trace strip (quiet micro-seismic hum that spikes realistically on pulse). */
export function drawSeismoStrip(ctx: CanvasRenderingContext2D, w: number, h: number, nowSec: number, pulseAge: number, active: boolean): void {
  ctx.clearRect(0, 0, w, h);
  ctx.fillStyle = '#090d15';
  ctx.fillRect(0, 0, w, h);
  ctx.strokeStyle = 'rgba(148, 163, 184, 0.16)';
  ctx.beginPath();
  ctx.moveTo(0, h * 0.5);
  ctx.lineTo(w, h * 0.5);
  ctx.stroke();
  if (!active) return;
  ctx.beginPath();
  const mid = h * 0.5;
  for (let x = 0; x < w; x++) {
    const tAgo = (1 - x / w) * 2.4;
    const sampleT = nowSec - tAgo;
    let y = Math.sin(sampleT * 19) * 0.06 + Math.sin(sampleT * 47) * 0.04;
    const dt = pulseAge - tAgo;
    if (dt >= 0 && dt < 1.3) {
      y += Math.sin(dt * 85) * Math.exp(-dt * 6.5) * 0.88;
      if (dt > 0.22) y += Math.sin((dt - 0.22) * 52) * Math.exp(-(dt - 0.22) * 4.2) * 0.5;
    }
    const py = mid - y * (h * 0.44);
    if (x === 0) ctx.moveTo(x, py); else ctx.lineTo(x, py);
  }
  ctx.strokeStyle = pulseAge < 0.9 ? '#fa9549' : '#38bdf8';
  ctx.lineWidth = 1.5;
  ctx.stroke();
}
