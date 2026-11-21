// Seismic Prospecting Console (F-SEIS): opens when right-clicking a Seismic Thumper (`main.ts` via
// `game.take_panel_request()`). Supports 2D MRI-scrubbable cross-sections (Tomography and Seismogram wiggle-trace)
// with 90° axis rotation and an interactive 48×48×H 3D subsurface cutaway volume.

import './machine.css';
import './seismic.css';
import type { Game } from '../wasm/engine.js';
import { button, h } from './dom';
import type { Pins } from './pins';
import { SeismicControls, SeismicFooter } from './seismic-controls';
import {
  canvasToCell, CLASS_CAVE, CLASS_ORE_UNKNOWN, CLASS_TUNNEL, CLASS_WATER, compassEnds, describeCell,
  offsetLabel, plotRect, worldCoord,
  type DisplayMode, type GridPoint, type PlotRect, type SliceData, type VolumeData, type VolumeFilters, type VolumePick,
} from './seismic-data';
import { drawSeismoStrip, SeismicRenderer } from './seismic-render';
import { SeismicVolumeRenderer } from './seismic-volume';

const WAVE_DURATION_SEC = 0.65;
const TIER_NAMES = ['Mk1', 'Mk2', 'Mk3'];
const TIER_CLASSES = ['mk1', 'mk2', 'mk3'];
const MODES: [DisplayMode, string, string][] = [
  ['tomo', 'Tomography', '1'], ['wiggle', 'Seismogram', '2'], ['volume', '3D Volume', '3'],
];

export class SeismicPanel {
  onClose: (resume: boolean) => void = () => {};
  onPulse: (() => void) | null = null;

  private readonly backdrop = h('div', 'mp-backdrop hidden');
  private readonly dialog = h('div', 'mp seis-dialog');
  private readonly tierBadge = h('span', 'seis-tier');
  private readonly powerBadge = h('span', 'seis-pill');
  private readonly snrBadge = h('span', 'seis-pill');
  private readonly axisBadge = h('span', 'seis-pill');
  private readonly upgradeBtn: HTMLButtonElement;
  private readonly powerBtn: HTMLButtonElement;
  private readonly powerText = h('span', 'mp-switch-label', 'On');
  private readonly pulseBtn: HTMLButtonElement;
  private readonly pinBtn: HTMLButtonElement;
  private readonly clearBtn: HTMLButtonElement;
  private readonly modeBtns = new Map<DisplayMode, HTMLButtonElement>();
  private readonly controls: SeismicControls;
  private readonly footer = new SeismicFooter();
  private readonly stripCanvas = h('canvas', 'seis-strip') as HTMLCanvasElement;
  private readonly stripCtx = this.stripCanvas.getContext('2d')!;
  private readonly canvasWrap = h('div', 'seis-viewport');
  private readonly canvas = h('canvas', 'seis-canvas') as HTMLCanvasElement;
  private readonly ctx = this.canvas.getContext('2d')!;
  private readonly offlineBanner = h('div', 'seis-offline hidden');
  private readonly renderer = new SeismicRenderer();
  private readonly volRenderer = new SeismicVolumeRenderer();

  private readonly oreColors = new Map<number, [number, number, number]>();
  private readonly oreNames = new Map<number, string>();
  private readonly oreBlocks = new Map<number, number>();
  private at: [number, number, number] = [0, 0, 0];
  private slice: SliceData | null = null;
  private volume: VolumeData | null = null;
  private plot: PlotRect = { x: 0, y: 0, w: 1, h: 1 };
  private mode: DisplayMode = 'tomo';
  private perpOffset = 0;
  private flipAxis = false;
  private pulseSeed = 1;
  private pulseTime = -10;
  private waveProgress = 1;
  private hover: GridPoint | null = null;
  private focus: GridPoint | null = null;
  private caliperA: GridPoint | null = null;
  private caliperB: GridPoint | null = null;
  private dragging = false;
  private volYaw = 0.65;
  private volPitch = 0.48;
  private volZoom = 1.0;
  private peelY = 96;
  private volFilters: VolumeFilters = { ores: true, tunnels: true, caves: true, water: true, surface: true, slicePlane: true };
  private volPointer: [number, number] | null = null;
  private volHover: VolumePick | null = null;
  private volFocus: VolumePick | null = null;
  private lastHoverKey = '';
  private orbiting = false;
  private orbitMoved = false;
  private lastPtr: [number, number] = [0, 0];
  private raf = 0;

  constructor(private readonly game: Game, private readonly pins: Pins) {
    this.loadOrePalette();
    this.dialog.setAttribute('role', 'dialog');
    this.dialog.setAttribute('aria-modal', 'true');
    this.dialog.tabIndex = -1;

    const head = h('div', 'mp-head seis-head');
    const titleGroup = h('div', 'seis-title-group');
    titleGroup.append(h('h2', 'seis-title', 'Seismic Thumper'), this.tierBadge, this.axisBadge, this.powerBadge, this.snrBadge);
    this.upgradeBtn = button('secondary-btn seis-upgrade hidden', '', () => {
      this.game.upgrade_machine(...this.at);
      window.setTimeout(() => { this.volume = null; this.refreshHeaderAndPulse(true); }, 70);
    });
    this.powerBtn = button('secondary-btn mp-switch', '', () => {
      this.game.set_machine_enabled(...this.at, !this.game.machine_enabled(...this.at));
      window.setTimeout(() => this.refreshHeaderAndPulse(false), 60);
    });
    this.powerBtn.append(h('span', 'mp-switch-dot'), this.powerText);
    const close = button('close-btn', '×', () => this.close(true));
    close.setAttribute('aria-label', 'Close');
    const headRight = h('div', 'seis-head-right');
    headRight.append(this.upgradeBtn, this.powerBtn);
    head.append(titleGroup, headRight, close);

    const bar = h('div', 'seis-toolbar');
    const modes = h('div', 'seis-modes');
    for (const [m, label, key] of MODES) {
      const btn = button('seis-mode-btn', label, () => this.setMode(m));
      btn.title = `${label} (${key})`;
      this.modeBtns.set(m, btn);
      modes.append(btn);
    }
    this.pulseBtn = button('seis-pulse-btn', 'Send Pulse', () => this.sendPulse(true));
    this.pulseBtn.title = 'Fire seismic drop-weight (Space)';
    this.pinBtn = button('secondary-btn seis-tool-btn', 'Pin Spot', () => this.pinSelected());
    this.pinBtn.title = 'Drop a map pin at the selected anomaly or hover point (P)';
    this.clearBtn = button('secondary-btn seis-tool-btn hidden', 'Clear Measure', () => this.clearCaliper());
    this.clearBtn.title = 'Clear measurement caliper (C)';
    const tools = h('div', 'seis-tools');
    tools.append(this.pulseBtn, this.stripCanvas, this.pinBtn, this.clearBtn);
    bar.append(modes, tools);

    this.controls = new SeismicControls({
      onPerpChange: (perp) => this.setPerp(perp),
      onFlipAxis: () => this.flipSliceAxis(),
      onPeelChange: (y) => { this.peelY = y; if (this.volume) this.controls.syncVolume(this.volume.topY, y, this.volFilters); },
      onFilterToggle: (k) => { this.volFilters[k] = !this.volFilters[k]; if (this.volume) this.controls.syncVolume(this.volume.topY, this.peelY, this.volFilters); },
      onResetView: () => this.resetVolumeView(),
    });

    this.canvasWrap.append(this.canvas, this.offlineBanner);
    this.dialog.append(head, bar, this.controls.root, this.canvasWrap, this.footer.root);
    this.backdrop.append(this.dialog);
    document.body.append(this.backdrop);
    this.bindEvents();
    this.setMode('tomo');
  }

  get isOpen(): boolean { return !this.backdrop.classList.contains('hidden'); }

  open(x: number, y: number, z: number): boolean {
    const info = this.game.seismic_info(x, y, z);
    if (info.length < 16) return false;
    this.at = [info[0], info[1], info[2]];
    this.perpOffset = 0;
    this.flipAxis = false;
    this.volume = null;
    this.hover = this.focus = this.volHover = this.volFocus = null;
    this.clearCaliper();
    this.backdrop.classList.remove('hidden');
    this.dialog.focus();
    this.refreshHeaderAndPulse(true);
    this.startLoop();
    return true;
  }

  close(resume: boolean): void {
    if (!this.isOpen) return;
    this.backdrop.classList.add('hidden');
    cancelAnimationFrame(this.raf);
    this.raf = 0;
    this.onClose(resume);
  }

  private loadOrePalette(): void {
    const byBlock = new Map<number, [number, number, number]>();
    for (const line of this.game.ore_guide().trim().split('\n')) {
      if (!line) continue;
      const [idStr, colStr] = line.split('\t');
      const c = Number(colStr);
      byBlock.set(Number(idStr), [(c >> 16) & 255, (c >> 8) & 255, c & 255]);
    }
    for (let detail = 1; detail <= 12; detail++) {
      const block = this.game.seismic_ore_block(detail);
      if (block === 0) break;
      this.oreBlocks.set(detail, block);
      this.oreNames.set(detail, this.game.ore_name(block));
      const rgb = byBlock.get(block);
      if (rgb) this.oreColors.set(detail, rgb);
    }
  }

  private bindEvents(): void {
    this.backdrop.addEventListener('pointerdown', (e) => { if (e.target === this.backdrop) this.close(true); });
    this.backdrop.addEventListener('contextmenu', (e) => e.preventDefault());
    window.addEventListener('keydown', (e) => {
      if (!this.isOpen) return;
      if (e.key === 'Escape') { e.preventDefault(); this.close(true); }
      else if (e.key === ' ' || e.code === 'Space') { e.preventDefault(); this.sendPulse(true); }
      else if (e.key === '1') this.setMode('tomo');
      else if (e.key === '2') this.setMode('wiggle');
      else if (e.key === '3') this.setMode('volume');
      else if (e.key === 'p' || e.key === 'P') this.pinSelected();
      else if (e.key === 'c' || e.key === 'C') this.clearCaliper();
      else if (e.key === 'r' || e.key === 'R') this.flipSliceAxis();
      else if (this.mode !== 'volume' && (e.key === 'ArrowLeft' || e.key === 'ArrowDown')) { e.preventDefault(); this.setPerp(this.perpOffset - (e.shiftKey ? 4 : 1)); }
      else if (this.mode !== 'volume' && (e.key === 'ArrowRight' || e.key === 'ArrowUp')) { e.preventDefault(); this.setPerp(this.perpOffset + (e.shiftKey ? 4 : 1)); }
    });
    const pick2d = (e: PointerEvent): GridPoint | null => {
      if (!this.slice) return null;
      const r = this.canvas.getBoundingClientRect();
      return canvasToCell(this.plot, this.slice, e.clientX - r.left, e.clientY - r.top);
    };
    this.canvas.addEventListener('wheel', (e) => {
      e.preventDefault();
      if (this.mode === 'volume') this.volZoom = Math.max(0.55, Math.min(2.5, this.volZoom * (e.deltaY > 0 ? 0.91 : 1.1)));
      else this.setPerp(this.perpOffset + (e.deltaY > 0 ? 1 : -1) * (e.shiftKey ? 4 : 1));
    }, { passive: false });
    this.canvas.addEventListener('pointermove', (e) => {
      const r = this.canvas.getBoundingClientRect();
      if (this.mode === 'volume') {
        this.volPointer = [e.clientX - r.left, e.clientY - r.top];
        if (this.orbiting) {
          const dx = e.clientX - this.lastPtr[0], dy = e.clientY - this.lastPtr[1];
          if (Math.hypot(dx, dy) > 1) this.orbitMoved = true;
          this.volYaw += dx * 0.0085;
          this.volPitch = Math.max(-1.05, Math.min(1.35, this.volPitch + dy * 0.007));
          this.lastPtr = [e.clientX, e.clientY];
        }
        return;
      }
      const pt = pick2d(e);
      this.hover = pt;
      if (this.dragging && pt) { this.caliperB = pt; this.clearBtn.classList.remove('hidden'); }
      this.updateTelemetry();
    });
    this.canvas.addEventListener('pointerleave', () => { this.hover = this.volPointer = this.volHover = null; this.updateTelemetry(); });
    this.canvas.addEventListener('pointerdown', (e) => {
      if (e.button !== 0) return;
      if (this.mode === 'volume') {
        this.canvas.setPointerCapture(e.pointerId);
        this.orbiting = true; this.orbitMoved = false; this.lastPtr = [e.clientX, e.clientY];
        this.canvas.classList.add('is-grabbing');
        return;
      }
      const pt = pick2d(e);
      if (!pt) return;
      this.canvas.setPointerCapture(e.pointerId);
      this.focus = this.caliperA = pt; this.caliperB = null; this.dragging = true;
      this.updateTelemetry();
    });
    this.canvas.addEventListener('pointerup', (e) => {
      if (this.mode === 'volume') {
        if (!this.orbiting) return;
        this.orbiting = false; this.canvas.classList.remove('is-grabbing');
        if (!this.orbitMoved) { this.volFocus = this.volHover; this.updateTelemetry(); }
        return;
      }
      if (!this.dragging) return;
      this.dragging = false;
      const pt = pick2d(e);
      if (pt && this.caliperA && (pt.s !== this.caliperA.s || pt.y !== this.caliperA.y)) { this.caliperB = pt; this.clearBtn.classList.remove('hidden'); }
      else this.clearCaliper();
      this.updateTelemetry();
    });
    this.canvas.addEventListener('dblclick', () => { if (this.mode === 'volume') this.resetVolumeView(); else this.clearCaliper(); });
  }

  private setMode(mode: DisplayMode): void {
    this.mode = mode;
    for (const [m, btn] of this.modeBtns) btn.classList.toggle('active', m === mode);
    this.controls.setMode(mode);
    this.canvas.classList.toggle('is-3d', mode === 'volume');
    if (mode === 'volume') {
      this.clearBtn.classList.add('hidden');
      if (!this.volume && this.slice?.status === 1) this.scanVolume();
      if (this.volume) this.axisBadge.textContent = `${this.volume.size}×${this.volume.size}×${this.volume.topY + 1}m · 3D`;
    } else {
      if (this.caliperA && this.caliperB) this.clearBtn.classList.remove('hidden');
      if (this.slice) {
        const [neg, pos] = compassEnds(this.slice.dir);
        this.axisBadge.textContent = `${neg}–${pos} · ${this.slice.width}m × ${this.slice.height}m`;
      }
    }
    this.footer.renderLegend(this.slice?.tier ?? 0);
    this.updateTelemetry();
  }

  private setPerp(next: number): void {
    const clamped = Math.max(-32, Math.min(31, Math.round(next)));
    if (clamped === this.perpOffset) return;
    this.perpOffset = clamped; this.focus = null; this.clearCaliper();
    this.scanSlice(false, true);
  }

  private flipSliceAxis(): void {
    this.flipAxis = !this.flipAxis; this.focus = null; this.clearCaliper();
    this.scanSlice(false, true);
  }

  private resetVolumeView(): void {
    this.volYaw = 0.65; this.volPitch = 0.48; this.volZoom = 1.0;
    if (this.volume) { this.peelY = this.volume.topY; this.controls.syncVolume(this.volume.topY, this.peelY, this.volFilters); }
  }

  private clearCaliper(): void { this.caliperA = this.caliperB = null; this.clearBtn.classList.add('hidden'); }

  private buildSlice(info: Int32Array, res: Int32Array, surface: Int32Array, cells: Uint8Array, anomalies: Int32Array): SliceData {
    return {
      anchorX: info[0], anchorY: info[1], anchorZ: info[2], dir: res[1], perpOffset: res[2], tier: info[4],
      powerShare: info[5], kw: info[6], width: info[7], height: res[0], surfaceAtThumper: res[3], status: info[10],
      seaLevel: info[11], playerS: res[4], playerY: info[13], playerPerp: res[5], snr: info[15], surface, cells, anomalies,
    };
  }

  private refreshHeaderAndPulse(autoPulse: boolean): void {
    const info = this.game.seismic_info(...this.at);
    if (info.length < 16) return this.close(true);
    const [ax, ay, az, dir, tier, powerShare, kw, width, topY, surf, status, , pS, , pP, snr] = info;
    this.at = [ax, ay, az];
    const effDir = (dir + (this.flipAxis ? 1 : 0)) & 3;
    const [neg, pos] = compassEnds(effDir);
    this.tierBadge.textContent = TIER_NAMES[tier] ?? `Mk${tier + 1}`;
    this.tierBadge.className = `seis-tier ${TIER_CLASSES[tier] ?? 'mk1'}`;
    this.axisBadge.textContent = `${neg}–${pos} · ${width}m × ${topY + 1}m`;
    this.controls.syncSlice(effDir, this.perpOffset);
    const on = status !== 2;
    this.powerBtn.dataset.on = String(on);
    this.powerText.textContent = on ? 'On' : 'Off';
    const powered = status === 1 && powerShare > 0;
    this.powerBadge.textContent = powered ? `${Math.round((kw * powerShare) / 1000)} / ${kw} kW` : status === 2 ? 'Switched off' : `No power (${kw} kW)`;
    this.powerBadge.dataset.state = powered ? (powerShare < 1000 ? 'warn' : 'ok') : 'off';
    this.snrBadge.textContent = powered ? `SNR ${snr}%` : 'SNR 0%';
    this.snrBadge.dataset.state = snr >= 85 ? 'ok' : snr >= 55 ? 'warn' : 'off';
    this.pulseBtn.disabled = !powered;

    const up = this.game.machine_upgrade(...this.at);
    const showUp = up.length >= 5 && up[4] !== 0;
    this.upgradeBtn.classList.toggle('hidden', !showUp);
    if (showUp) {
      this.upgradeBtn.disabled = up[3] < up[2];
      this.upgradeBtn.textContent = `Upgrade Mk${up[0] + 1} (${up[3]}/${up[2]} ${this.game.item_name(up[1])})`;
    }
    this.offlineBanner.classList.toggle('hidden', powered);
    if (!powered) {
      this.offlineBanner.textContent = status === 2 ? 'Thumper is switched off — toggle On to send seismic pulses.' : 'No power — wire this thumper to a power pole in reach.';
      this.slice = this.buildSlice(info, new Int32Array([topY + 1, effDir, this.perpOffset, surf, pS, pP]), new Int32Array(width), new Uint8Array(width * (topY + 1) * 4), new Int32Array(0));
      this.volume = null;
      this.footer.clearAnomalies();
      this.footer.renderLegend(tier);
      return;
    }
    if (autoPulse || !this.slice || this.slice.tier !== tier) this.sendPulse(autoPulse);
  }

  private sendPulse(playAudio: boolean): void {
    this.pulseSeed = (this.pulseSeed + 1) >>> 0;
    this.volume = null;
    this.scanSlice(playAudio, false);
    if (this.mode === 'volume') this.scanVolume();
  }

  private scanSlice(playAudio: boolean, instant: boolean): void {
    const info = this.game.seismic_info(...this.at);
    if (info.length < 16 || info[10] !== 1 || info[5] === 0) return;
    const res = this.game.seismic_pulse(info[0], info[1], info[2], this.pulseSeed, this.perpOffset, this.flipAxis);
    if (res.length < 6 || res[0] === 0) return;
    this.slice = this.buildSlice(
      info, res, new Int32Array(this.game.seismic_surface()),
      new Uint8Array(this.game.seismic_cells()), new Int32Array(this.game.seismic_anomalies()),
    );
    const [neg, pos] = compassEnds(res[1]);
    this.axisBadge.textContent = `${neg}–${pos} · ${info[7]}m × ${res[0]}m`;
    this.controls.syncSlice(res[1], res[2]);
    if (!instant) { this.pulseTime = performance.now() / 1000; this.waveProgress = 0; if (playAudio) this.onPulse?.(); }
    else { this.pulseTime = -10; this.waveProgress = 1; }
    this.footer.renderAnomalies(this.slice, this.game.seismic_anomaly_fields(), (det) => this.oreNames.get(det) ?? '', (s, y, imp, cls, detail) => {
      this.focus = { s, y };
      if (this.volume && this.slice) {
        const [wx, wy, wz] = worldCoord(this.slice, s, y);
        const half = this.volume.size >> 1;
        this.volFocus = { lx: wx - this.volume.anchorX + half, y: wy, lz: wz - this.volume.anchorZ + half, wx, wy, wz, imp, cls, detail };
      }
      this.updateTelemetry();
    });
    this.footer.renderLegend(this.slice.tier);
    this.updateTelemetry();
  }

  private scanVolume(): void {
    const meta = this.game.seismic_volume_scan(...this.at, this.pulseSeed);
    if (meta.length < 12) return;
    this.volume = {
      size: meta[0], topY: meta[1], anchorX: meta[2], anchorY: meta[3], anchorZ: meta[4],
      dir: meta[5], tier: meta[6], seaLevel: meta[7], playerDx: meta[8], playerY: meta[9], playerDz: meta[10],
      surface: new Int32Array(this.game.seismic_volume_surface()),
      voxels: new Uint8Array(this.game.seismic_volume_voxels()),
    };
    this.peelY = meta[1];
    this.controls.syncVolume(meta[1], this.peelY, this.volFilters);
  }

  private updateTelemetry(): void {
    const d = this.slice;
    const oreName = (det: number) => this.oreNames.get(det) ?? '';
    if (this.mode === 'volume') {
      const p = this.volHover ?? this.volFocus;
      if (!this.volume || !p || d?.status !== 1) {
        this.footer.setEmpty('Drag to orbit 3D volume · Wheel to zoom · Hover a voxel to inspect');
        this.pinBtn.disabled = true;
        return;
      }
      const lx = Math.max(0, Math.min(this.volume.size - 1, p.lx));
      const lz = Math.max(0, Math.min(this.volume.size - 1, p.lz));
      const surf = this.volume.surface[lz * this.volume.size + lx] ?? this.volume.anchorY;
      this.footer.setReading(p.wx, p.wy, p.wz, '', surf - p.wy, p.imp, p.cls, p.detail, oreName);
      this.pinBtn.disabled = false;
      return;
    }
    const pt = this.hover ?? this.focus;
    if (!d || !pt || d.status !== 1) {
      this.footer.setEmpty('Hover or drag to measure · Scroll wheel or arrows to scrub slices');
      this.pinBtn.disabled = true;
      return;
    }
    const idx = (pt.y * d.width + pt.s) * 4;
    const [wx, wy, wz] = worldCoord(d, pt.s, pt.y);
    this.footer.setReading(wx, wy, wz, ` (${offsetLabel(d, pt.s)})`, d.surfaceAtThumper - wy, d.cells[idx], d.cells[idx + 2], d.cells[idx + 3], oreName);
    this.pinBtn.disabled = false;
  }

  private pinSelected(): void {
    let wx: number, wy: number, wz: number, imp: number, cls: number, detail: number, surf: number;
    if (this.mode === 'volume') {
      const p = this.volFocus ?? this.volHover;
      if (!this.volume || !p) return;
      ({ wx, wy, wz, imp, cls, detail } = p);
      surf = this.volume.anchorY;
    } else {
      const d = this.slice, pt = this.focus ?? this.hover;
      if (!d || !pt) return;
      const idx = (pt.y * d.width + pt.s) * 4;
      [wx, wy, wz] = worldCoord(d, pt.s, pt.y);
      imp = d.cells[idx]; cls = d.cells[idx + 2]; detail = d.cells[idx + 3]; surf = d.surfaceAtThumper;
    }
    const desc = describeCell(cls, detail, imp, (det) => this.oreNames.get(det) ?? '');
    const oreBlock = detail > 0 ? this.oreBlocks.get(detail) : undefined;
    const tag = cls === CLASS_TUNNEL || cls === CLASS_CAVE || cls === CLASS_WATER || cls >= CLASS_ORE_UNKNOWN ? desc.label : 'Seismic mark';
    this.pins.add({ x: wx, y: wy, z: wz, kind: oreBlock ? `ore-${oreBlock}` : 'note', note: `${tag} (Y ${wy}, −${Math.max(0, surf - wy)}m)` });
    this.pinBtn.textContent = 'Pinned!';
    window.setTimeout(() => { this.pinBtn.textContent = 'Pin Spot'; }, 900);
  }

  private startLoop(): void {
    cancelAnimationFrame(this.raf);
    const step = () => { if (!this.isOpen) return; this.drawFrame(); this.raf = requestAnimationFrame(step); };
    this.raf = requestAnimationFrame(step);
  }

  private drawFrame(): void {
    const now = performance.now() / 1000;
    const pulseAge = now - this.pulseTime;
    this.waveProgress = Math.min(1, Math.max(0, pulseAge / WAVE_DURATION_SEC));

    const dpr = Math.min(2, window.devicePixelRatio || 1);
    const cw = Math.max(320, Math.floor(this.canvasWrap.clientWidth));
    const ch = Math.max(240, Math.floor(this.canvasWrap.clientHeight));
    if (this.canvas.width !== Math.floor(cw * dpr) || this.canvas.height !== Math.floor(ch * dpr)) {
      this.canvas.width = Math.floor(cw * dpr);
      this.canvas.height = Math.floor(ch * dpr);
    }
    this.ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
    if (this.mode === 'volume' && this.volume) {
      this.volHover = this.volRenderer.draw(this.ctx, cw, ch, this.volume, {
        yaw: this.volYaw, pitch: this.volPitch, zoom: this.volZoom, peelY: this.peelY,
        waveProgress: this.waveProgress, sliceDir: this.slice?.dir ?? this.volume.dir,
        slicePerp: this.perpOffset, filters: this.volFilters, oreColors: this.oreColors,
        pointer: this.volPointer, focus: this.volFocus,
      });
      const hk = this.volHover ? `${this.volHover.wx},${this.volHover.wy},${this.volHover.wz}` : '';
      if (hk !== this.lastHoverKey) { this.lastHoverKey = hk; this.updateTelemetry(); }
    } else {
      this.plot = this.slice
        ? this.renderer.draw(this.ctx, cw, ch, this.slice, {
            mode: this.mode, waveProgress: this.waveProgress, hover: this.hover, focus: this.focus,
            caliperA: this.caliperA, caliperB: this.caliperB, oreColors: this.oreColors,
          })
        : plotRect(cw, ch);
    }
    const sw = 128, sh = 30;
    if (this.stripCanvas.width !== Math.floor(sw * dpr) || this.stripCanvas.height !== Math.floor(sh * dpr)) {
      this.stripCanvas.width = Math.floor(sw * dpr);
      this.stripCanvas.height = Math.floor(sh * dpr);
    }
    this.stripCtx.setTransform(dpr, 0, 0, dpr, 0, 0);
    drawSeismoStrip(this.stripCtx, sw, sh, now, pulseAge, this.slice?.status === 1);
  }
}
