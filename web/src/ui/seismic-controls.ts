// Sub-toolbar, telemetry strip, detected-horizon chips and legend for the Seismic Prospecting Console (`seismic.ts`).
// Keeps DOM view helpers separated from the canvas/wasm coordinator so every UI module stays compact.

import { button, h } from './dom';
import {
  compassEnds, describeCell, legendEntries, offsetLabel, perpLabel,
  type DisplayMode, type SliceData, type VolumeFilters,
} from './seismic-data';

export interface ControlCallbacks {
  onPerpChange: (perp: number) => void;
  onFlipAxis: () => void;
  onPeelChange: (peelY: number) => void;
  onFilterToggle: (key: keyof VolumeFilters) => void;
  onResetView: () => void;
}

const FILTER_DEFS: [keyof VolumeFilters, string][] = [
  ['ores', 'Ores'], ['tunnels', 'Tunnels'], ['caves', 'Caves'],
  ['water', 'Water'], ['surface', 'Surface'], ['slicePlane', 'Slice'],
];

export class SeismicControls {
  readonly root = h('div', 'seis-subbar');
  private readonly sliceRow = h('div', 'seis-subrow');
  private readonly volRow = h('div', 'seis-subrow hidden');
  private readonly sliceSlider = document.createElement('input');
  private readonly sliceVal = h('span', 'seis-subval', 'Center (0m)');
  private readonly centerBtn: HTMLButtonElement;
  private readonly rotateBtn: HTMLButtonElement;
  private readonly peelSlider = document.createElement('input');
  private readonly peelVal = h('span', 'seis-subval', 'All');
  private readonly filterBtns = new Map<keyof VolumeFilters, HTMLButtonElement>();

  constructor(cb: ControlCallbacks) {
    const prevBtn = button('secondary-btn seis-step-btn', '◀', () => cb.onPerpChange(Number(this.sliceSlider.value) - 1));
    prevBtn.title = 'Step slice −1m (Left / Down / Scroll)';
    const nextBtn = button('secondary-btn seis-step-btn', '▶', () => cb.onPerpChange(Number(this.sliceSlider.value) + 1));
    nextBtn.title = 'Step slice +1m (Right / Up / Scroll)';
    this.sliceSlider.type = 'range';
    this.sliceSlider.className = 'seis-slider';
    this.sliceSlider.min = '-32';
    this.sliceSlider.max = '31';
    this.sliceSlider.value = '0';
    this.sliceSlider.addEventListener('input', () => cb.onPerpChange(Number(this.sliceSlider.value)));

    this.centerBtn = button('secondary-btn seis-tool-btn', 'Center', () => cb.onPerpChange(0));
    this.centerBtn.title = 'Snap slice back to thumper center (0m)';
    this.rotateBtn = button('secondary-btn seis-tool-btn', 'Rotate 90°', () => cb.onFlipAxis());
    this.rotateBtn.title = 'Rotate vertical cross-section 90° between W–E and N–S (R)';

    const sliceLeft = h('div', 'seis-sub-group');
    sliceLeft.append(
      h('span', 'seis-tele-k', 'Slice'), prevBtn, this.sliceSlider, nextBtn,
      this.sliceVal, this.centerBtn, this.rotateBtn,
    );
    this.sliceRow.append(sliceLeft, h('span', 'seis-hint', 'Wheel / Arrows scrub · R rotate · Drag measure'));

    this.peelSlider.type = 'range';
    this.peelSlider.className = 'seis-slider seis-peel-slider';
    this.peelSlider.min = '1';
    this.peelSlider.max = '96';
    this.peelSlider.value = '96';
    this.peelSlider.addEventListener('input', () => cb.onPeelChange(Number(this.peelSlider.value)));

    const peelGroup = h('div', 'seis-sub-group');
    peelGroup.append(h('span', 'seis-tele-k', 'Peel Y'), this.peelSlider, this.peelVal);

    const filtersGroup = h('div', 'seis-sub-group seis-filters');
    for (const [key, label] of FILTER_DEFS) {
      const btn = button('seis-filter-btn active', label, () => cb.onFilterToggle(key));
      this.filterBtns.set(key, btn);
      filtersGroup.append(btn);
    }
    const resetBtn = button('secondary-btn seis-tool-btn', 'Reset View', () => cb.onResetView());
    resetBtn.title = 'Reset 3D camera orbit, zoom and peel depth (Double-click)';
    filtersGroup.append(resetBtn);

    this.volRow.append(peelGroup, filtersGroup);
    this.root.append(this.sliceRow, this.volRow);
  }

  setMode(mode: DisplayMode): void {
    const isVol = mode === 'volume';
    this.sliceRow.classList.toggle('hidden', isVol);
    this.volRow.classList.toggle('hidden', !isVol);
  }

  syncSlice(dir: number, perpOffset: number): void {
    this.sliceSlider.value = String(perpOffset);
    this.sliceVal.textContent = perpLabel(dir, perpOffset);
    this.centerBtn.disabled = perpOffset === 0;
    const [neg, pos] = compassEnds(dir);
    this.rotateBtn.textContent = `${neg}–${pos} ↻ 90°`;
  }

  syncVolume(topY: number, peelY: number, filters: VolumeFilters): void {
    this.peelSlider.max = String(Math.max(1, topY));
    this.peelSlider.value = String(Math.min(topY, peelY));
    this.peelVal.textContent = peelY >= topY ? `All (Y ${topY})` : `≤ Y ${peelY}`;
    for (const [key, btn] of this.filterBtns) btn.classList.toggle('active', filters[key]);
  }
}

export class SeismicFooter {
  readonly root = h('div', 'seis-footer');
  private readonly teleCoords = h('span', 'seis-tele-val', '—');
  private readonly teleDepth = h('span', 'seis-tele-val', '—');
  private readonly teleClass = h('span', 'seis-tele-class', 'Hover cross-section to inspect');
  private readonly teleBarFill = h('div', 'seis-density-fill');
  private readonly telePct = h('span', 'seis-tele-val', '—');
  private readonly anomaliesEl = h('div', 'seis-anomalies');
  private readonly legendEl = h('div', 'seis-legend');

  constructor() {
    const tele = h('div', 'seis-telemetry');
    const densityBar = h('div', 'seis-density-bar');
    densityBar.append(this.teleBarFill);
    const densityWrap = h('div', 'seis-density-wrap');
    densityWrap.append(h('span', 'seis-tele-k', 'Impedance'), densityBar, this.telePct);
    const coordWrap = h('div', 'seis-tele-pair');
    coordWrap.append(h('span', 'seis-tele-k', 'Coords'), this.teleCoords, h('span', 'seis-tele-k', 'Depth'), this.teleDepth);
    tele.append(this.teleClass, coordWrap, densityWrap);
    const deck = h('div', 'seis-deck');
    deck.append(this.anomaliesEl, this.legendEl);
    this.root.append(tele, deck);
  }

  clearAnomalies(): void { this.anomaliesEl.replaceChildren(); }

  renderAnomalies(
    d: SliceData | null,
    stride: number,
    oreName: (det: number) => string,
    onPick: (s: number, y: number, imp: number, cls: number, detail: number) => void,
  ): void {
    this.anomaliesEl.replaceChildren();
    if (!d || d.anomalies.length === 0) {
      this.anomaliesEl.append(h('span', 'seis-empty', 'No major anomalies in this slice — scrub or switch to 3D Volume'));
      return;
    }
    for (let i = 0; i < d.anomalies.length; i += stride) {
      const [cls, s, y, strength, detail] = d.anomalies.subarray(i, i + stride);
      const imp = Math.round((strength / 100) * 255);
      const desc = describeCell(cls, detail, imp, oreName);
      const chip = button('seis-chip', '', () => onPick(s, y, imp, cls, detail));
      const dot = h('span', 'seis-chip-dot');
      dot.style.background = desc.color;
      chip.append(dot, h('span', 'seis-chip-name', desc.label), h('span', 'seis-chip-pos', `−${Math.max(0, d.surfaceAtThumper - y)}m · ${offsetLabel(d, s)}`));
      this.anomaliesEl.append(chip);
    }
  }

  renderLegend(tier: number): void {
    this.legendEl.replaceChildren();
    for (const [color, label] of legendEntries(tier)) {
      const item = h('span', 'seis-leg-item');
      const sw = h('span', 'seis-leg-sw');
      sw.style.background = color;
      item.append(sw, label);
      this.legendEl.append(item);
    }
  }

  setEmpty(msg: string): void {
    this.teleCoords.textContent = this.teleDepth.textContent = this.telePct.textContent = '—';
    this.teleBarFill.style.transform = 'scaleX(0)';
    this.teleClass.textContent = msg;
    this.teleClass.style.color = 'var(--muted)';
  }

  setReading(
    wx: number, wy: number, wz: number, suffix: string, depth: number,
    imp: number, cls: number, detail: number, oreName: (det: number) => string,
  ): void {
    const desc = describeCell(cls, detail, imp, oreName);
    this.teleCoords.textContent = `${wx}, ${wy}, ${wz}${suffix}`;
    this.teleDepth.textContent = depth >= 0 ? `−${depth}m` : `+${-depth}m`;
    this.telePct.textContent = `${Math.round((imp / 255) * 100)}%`;
    this.teleBarFill.style.transform = `scaleX(${imp / 255})`;
    this.teleBarFill.style.background = desc.color;
    this.teleClass.textContent = desc.label;
    this.teleClass.style.color = desc.color;
  }
}
