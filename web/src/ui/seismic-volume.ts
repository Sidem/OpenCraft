// Interactive 3D subsurface cutaway renderer (F-SEIS): projects the 48×48×H seismic feature volume into an orbitable,
// zoomable 3D tomography cube with shaded 3-face voxels, topographic terrain contour roof, bedrock grid, sea-level
// perimeter, active 2D slice plane indicator, P-wave pulse shell, depth-peeling cutaway and 3D voxel hover picking.

import {
  CLASS_CAVE,
  CLASS_ORE_UNKNOWN,
  CLASS_STRUCTURE,
  CLASS_TUNNEL,
  CLASS_WATER,
  colorForCell,
  dirAxis,
  type VolumeData,
  type VolumeFilters,
  type VolumePick,
} from './seismic-data';

export interface VolumeState {
  yaw: number;
  pitch: number;
  zoom: number;
  peelY: number;
  waveProgress: number;
  sliceDir: number;
  slicePerp: number;
  filters: VolumeFilters;
  oreColors: Map<number, [number, number, number]>;
  pointer: [number, number] | null;
  focus: VolumePick | null;
}

interface ProjectedVoxel {
  sx: number;
  sy: number;
  depth: number;
  size: number;
  idx: number;
  cls: number;
  imp: number;
  det: number;
  lx: number;
  y: number;
  lz: number;
}

/** Projects and renders the 3D subsurface cutaway with painter's-algorithm depth sorting and cursor hit-testing. */
export class SeismicVolumeRenderer {
  private sorted: ProjectedVoxel[] = [];

  draw(ctx: CanvasRenderingContext2D, w: number, h: number, v: VolumeData, st: VolumeState): VolumePick | null {
    ctx.clearRect(0, 0, w, h);
    const bg = ctx.createRadialGradient(w * 0.5, h * 0.45, 24, w * 0.5, h * 0.5, Math.max(w, h) * 0.72);
    bg.addColorStop(0, '#101827');
    bg.addColorStop(1, '#070a11');
    ctx.fillStyle = bg;
    ctx.fillRect(0, 0, w, h);

    const half = v.size * 0.5;
    const midY = v.topY * 0.5;
    const scale = (Math.min(w, h) / Math.max(v.size * 1.48, v.topY * 1.15)) * st.zoom;
    const cx = w * 0.5;
    const cy = h * 0.52;
    const cyaw = Math.cos(st.yaw), syaw = Math.sin(st.yaw);
    const cp = Math.cos(st.pitch), sp = Math.sin(st.pitch);
    const camDist = 175;

    const project = (x: number, y: number, z: number): [number, number, number] => {
      const rx = x * cyaw - z * syaw;
      const rz = x * syaw + z * cyaw;
      const ry = y * cp - rz * sp;
      const depth = y * sp + rz * cp;
      const persp = camDist / Math.max(45, camDist - depth * 0.42);
      return [cx + rx * scale * persp, cy - ry * scale * persp, depth];
    };

    this.drawCage(ctx, v, half, midY, st.peelY, project);
    if (st.filters.surface) this.drawTerrainRoof(ctx, v, half, midY, project);
    if (st.filters.slicePlane) this.drawSlicePlane(ctx, v, half, midY, st.sliceDir, st.slicePerp, project);

    const picked = this.drawVoxels(ctx, v, half, midY, scale, st, project);
    this.drawMarkers(ctx, v, half, midY, st, picked, project);
    if (st.waveProgress < 1) this.drawWaveRing(ctx, v, half, midY, st.waveProgress, project);
    return picked;
  }

  private drawCage(
    ctx: CanvasRenderingContext2D,
    v: VolumeData,
    half: number,
    midY: number,
    peelY: number,
    project: (x: number, y: number, z: number) => [number, number, number],
  ): void {
    ctx.save();
    // Bedrock floor grid (y = 0).
    ctx.strokeStyle = 'rgba(185, 58, 70, 0.22)';
    ctx.lineWidth = 1;
    for (let i = 0; i <= v.size; i += 8) {
      const p0 = project(i - half, -midY, -half);
      const p1 = project(i - half, -midY, half);
      ctx.beginPath();
      ctx.moveTo(p0[0], p0[1]);
      ctx.lineTo(p1[0], p1[1]);
      ctx.stroke();
      const q0 = project(-half, -midY, i - half);
      const q1 = project(half, -midY, i - half);
      ctx.beginPath();
      ctx.moveTo(q0[0], q0[1]);
      ctx.lineTo(q1[0], q1[1]);
      ctx.stroke();
    }

    // Sea-level perimeter ring.
    if (v.seaLevel > 0 && v.seaLevel < v.topY) {
      const sy = v.seaLevel - midY;
      ctx.setLineDash([4, 4]);
      ctx.strokeStyle = 'rgba(56, 189, 248, 0.35)';
      this.strokeLoop(ctx, [[-half, sy, -half], [half, sy, -half], [half, sy, half], [-half, sy, half]], project);
      ctx.setLineDash([]);
    }

    // Active peel-depth ceiling ring when peeling below top.
    if (peelY < v.topY) {
      const py = peelY - midY;
      ctx.strokeStyle = 'rgba(250, 149, 73, 0.55)';
      ctx.lineWidth = 1.4;
      this.strokeLoop(ctx, [[-half, py, -half], [half, py, -half], [half, py, half], [-half, py, half]], project);
    }

    // Vertical corner pillars + top rim.
    const top = v.topY - midY;
    ctx.strokeStyle = 'rgba(148, 163, 184, 0.25)';
    ctx.lineWidth = 1;
    for (const [cx, cz] of [[-half, -half], [half, -half], [half, half], [-half, half]]) {
      const b = project(cx, -midY, cz);
      const t = project(cx, top, cz);
      ctx.beginPath();
      ctx.moveTo(b[0], b[1]);
      ctx.lineTo(t[0], t[1]);
      ctx.stroke();
    }
    this.strokeLoop(ctx, [[-half, top, -half], [half, top, -half], [half, top, half], [-half, top, half]], project);

    // Cardinal compass markers around the rim.
    ctx.font = '700 11px var(--mono, monospace)';
    ctx.textAlign = 'center';
    ctx.textBaseline = 'middle';
    for (const [label, x, z, color] of [
      ['N', 0, -half - 3, '#fa9549'],
      ['S', 0, half + 3, '#94a3b8'],
      ['E', half + 3, 0, '#cbd5e1'],
      ['W', -half - 3, 0, '#cbd5e1'],
    ] as [string, number, number, string][]) {
      const [sx, sy] = project(x, top, z);
      ctx.fillStyle = color;
      ctx.fillText(label, sx, sy);
    }
    ctx.restore();
  }

  private drawTerrainRoof(
    ctx: CanvasRenderingContext2D,
    v: VolumeData,
    half: number,
    midY: number,
    project: (x: number, y: number, z: number) => [number, number, number],
  ): void {
    const step = 4;
    ctx.save();
    ctx.strokeStyle = 'rgba(203, 213, 225, 0.22)';
    ctx.lineWidth = 1;
    for (let lz = 0; lz < v.size; lz += step) {
      ctx.beginPath();
      for (let lx = 0; lx < v.size; lx += step) {
        const y = v.surface[lz * v.size + lx] - midY;
        const [sx, sy] = project(lx - half, y, lz - half);
        if (lx === 0) ctx.moveTo(sx, sy); else ctx.lineTo(sx, sy);
      }
      ctx.stroke();
    }
    for (let lx = 0; lx < v.size; lx += step) {
      ctx.beginPath();
      for (let lz = 0; lz < v.size; lz += step) {
        const y = v.surface[lz * v.size + lx] - midY;
        const [sx, sy] = project(lx - half, y, lz - half);
        if (lz === 0) ctx.moveTo(sx, sy); else ctx.lineTo(sx, sy);
      }
      ctx.stroke();
    }
    ctx.restore();
  }

  private drawSlicePlane(
    ctx: CanvasRenderingContext2D,
    v: VolumeData,
    half: number,
    midY: number,
    dir: number,
    perp: number,
    project: (x: number, y: number, z: number) => [number, number, number],
  ): void {
    const clamped = Math.max(-half, Math.min(half, perp));
    const [ax, az] = dirAxis(dir);
    const [px, pz] = dirAxis(dir + 1);
    const corners: [number, number, number][] = [
      [px * clamped - ax * half, -midY, pz * clamped - az * half],
      [px * clamped + ax * half, -midY, pz * clamped + az * half],
      [px * clamped + ax * half, v.topY - midY, pz * clamped + az * half],
      [px * clamped - ax * half, v.topY - midY, pz * clamped - az * half],
    ];
    ctx.save();
    ctx.beginPath();
    corners.forEach(([x, y, z], i) => {
      const [sx, sy] = project(x, y, z);
      if (i === 0) ctx.moveTo(sx, sy); else ctx.lineTo(sx, sy);
    });
    ctx.closePath();
    ctx.fillStyle = 'rgba(250, 149, 73, 0.08)';
    ctx.fill();
    ctx.strokeStyle = 'rgba(250, 149, 73, 0.58)';
    ctx.lineWidth = 1.3;
    ctx.stroke();
    ctx.restore();
  }

  private drawVoxels(
    ctx: CanvasRenderingContext2D,
    v: VolumeData,
    half: number,
    midY: number,
    scale: number,
    st: VolumeState,
    project: (x: number, y: number, z: number) => [number, number, number],
  ): VolumePick | null {
    const arr = this.sorted;
    arr.length = 0;
    const voxelScale = v.tier === 0 ? 1.72 : 1.05;
    const waveR = st.waveProgress < 1 ? st.waveProgress * Math.hypot(half * 1.5, v.topY * 1.1) : 1e9;
    const surfC = v.surface[(v.size >> 1) * v.size + (v.size >> 1)];
    const vx = v.voxels;

    for (let i = 0; i < vx.length; i += 6) {
      const lx = vx[i], y = vx[i + 1], lz = vx[i + 2], imp = vx[i + 3], cls = vx[i + 4], det = vx[i + 5];
      if (y > st.peelY) continue;
      if (cls >= CLASS_ORE_UNKNOWN && !st.filters.ores) continue;
      if ((cls === CLASS_TUNNEL || cls === CLASS_STRUCTURE) && !st.filters.tunnels) continue;
      if (cls === CLASS_CAVE && !st.filters.caves) continue;
      if (cls === CLASS_WATER && !st.filters.water) continue;
      const dx = lx - half, dz = lz - half;
      if (st.waveProgress < 1 && Math.hypot(dx, y - surfC, dz) > waveR) continue;
      const [sx, sy, depth] = project(dx, y - midY, dz);
      const sz = Math.max(2.2, scale * voxelScale * (175 / Math.max(45, 175 - depth * 0.42)));
      arr.push({ sx, sy, depth, size: sz, idx: i, cls, imp, det, lx, y, lz });
    }

    // Painter's sort: back-to-front (lowest depth first).
    arr.sort((a, b) => a.depth - b.depth);

    let bestPick: VolumePick | null = null;
    const [mx, my] = st.pointer ?? [-1e6, -1e6];

    for (let i = 0; i < arr.length; i++) {
      const p = arr[i];
      const [r, g, b] = colorForCell('volume', p.imp, p.cls, p.det, st.oreColors);
      const alpha = p.cls === CLASS_CAVE ? 0.34 : p.cls === CLASS_WATER ? 0.62 : 0.92;
      const s = p.size * 0.56;
      const sh = s * 0.55;

      // Crisp 3-face shaded voxel prism (top bright, left mid, right dark).
      ctx.fillStyle = `rgba(${Math.min(255, r + 26)},${Math.min(255, g + 26)},${Math.min(255, b + 20)},${alpha})`;
      ctx.beginPath();
      ctx.moveTo(p.sx, p.sy - s);
      ctx.lineTo(p.sx + s, p.sy - sh);
      ctx.lineTo(p.sx, p.sy);
      ctx.lineTo(p.sx - s, p.sy - sh);
      ctx.closePath();
      ctx.fill();

      ctx.fillStyle = `rgba(${Math.round(r * 0.84)},${Math.round(g * 0.84)},${Math.round(b * 0.88)},${alpha})`;
      ctx.beginPath();
      ctx.moveTo(p.sx - s, p.sy - sh);
      ctx.lineTo(p.sx, p.sy);
      ctx.lineTo(p.sx, p.sy + s);
      ctx.lineTo(p.sx - s, p.sy + sh);
      ctx.closePath();
      ctx.fill();

      ctx.fillStyle = `rgba(${Math.round(r * 0.62)},${Math.round(g * 0.62)},${Math.round(b * 0.68)},${alpha})`;
      ctx.beginPath();
      ctx.moveTo(p.sx, p.sy);
      ctx.lineTo(p.sx + s, p.sy - sh);
      ctx.lineTo(p.sx + s, p.sy + sh);
      ctx.lineTo(p.sx, p.sy + s);
      ctx.closePath();
      ctx.fill();

      if (st.pointer) {
        const hitR = Math.max(7, s * 1.35);
        if ((p.sx - mx) ** 2 + (p.sy - my) ** 2 <= hitR * hitR) {
          bestPick = {
            lx: p.lx, y: p.y, lz: p.lz,
            wx: v.anchorX + p.lx - (v.size >> 1), wy: p.y, wz: v.anchorZ + p.lz - (v.size >> 1),
            imp: p.imp, cls: p.cls, detail: p.det,
          };
        }
      }
    }
    return bestPick;
  }

  private drawMarkers(
    ctx: CanvasRenderingContext2D,
    v: VolumeData,
    half: number,
    midY: number,
    st: VolumeState,
    picked: VolumePick | null,
    project: (x: number, y: number, z: number) => [number, number, number],
  ): void {
    ctx.save();
    // Thumper beacon on surface at (0, anchorY, 0).
    const [tx, ty] = project(0, v.anchorY - midY, 0);
    ctx.fillStyle = '#fa9549';
    ctx.beginPath();
    ctx.moveTo(tx, ty + 4);
    ctx.lineTo(tx - 6, ty - 7);
    ctx.lineTo(tx + 6, ty - 7);
    ctx.closePath();
    ctx.fill();
    ctx.strokeStyle = '#070a11';
    ctx.lineWidth = 1.3;
    ctx.stroke();

    // Player live marker inside the 48×48 volume.
    if (Math.abs(v.playerDx) <= half && Math.abs(v.playerDz) <= half) {
      const [px, py] = project(v.playerDx, v.playerY - midY, v.playerDz);
      ctx.fillStyle = '#f8fafc';
      ctx.beginPath();
      ctx.arc(px, py, 4.5, 0, Math.PI * 2);
      ctx.fill();
      ctx.strokeStyle = '#fa9549';
      ctx.lineWidth = 1.8;
      ctx.stroke();
    }

    // Highlight ring on hovered or focused voxel.
    const mark = picked ?? st.focus;
    if (mark) {
      const [fx, fy] = project(mark.lx - half, mark.y - midY, mark.lz - half);
      ctx.strokeStyle = '#fde047';
      ctx.lineWidth = 1.8;
      ctx.beginPath();
      ctx.arc(fx, fy, 8, 0, Math.PI * 2);
      ctx.stroke();
    }
    ctx.restore();
  }

  private drawWaveRing(
    ctx: CanvasRenderingContext2D,
    v: VolumeData,
    half: number,
    midY: number,
    progress: number,
    project: (x: number, y: number, z: number) => [number, number, number],
  ): void {
    const r = progress * half * 1.25;
    const y = v.anchorY - progress * v.anchorY - midY;
    ctx.save();
    ctx.strokeStyle = `rgba(250, 165, 85, ${(1 - progress) * 0.85})`;
    ctx.lineWidth = 2;
    ctx.beginPath();
    for (let i = 0; i <= 32; i++) {
      const a = (i / 32) * Math.PI * 2;
      const [sx, sy] = project(Math.cos(a) * r, y, Math.sin(a) * r);
      if (i === 0) ctx.moveTo(sx, sy); else ctx.lineTo(sx, sy);
    }
    ctx.stroke();
    ctx.restore();
  }

  private strokeLoop(
    ctx: CanvasRenderingContext2D,
    pts: [number, number, number][],
    project: (x: number, y: number, z: number) => [number, number, number],
  ): void {
    ctx.beginPath();
    pts.forEach(([x, y, z], i) => {
      const [sx, sy] = project(x, y, z);
      if (i === 0) ctx.moveTo(sx, sy); else ctx.lineTo(sx, sy);
    });
    ctx.closePath();
    ctx.stroke();
  }
}
