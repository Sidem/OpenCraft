// "Performance" on the menu's settings page: which graphics the game draws on and a report to copy and paste, plus a banner
// on the launcher when the browser draws without the graphics card (`banner`, dismissed for good in this browser). Uses the
// Comfort section's row styles (comfort.css); the words are in perf-text.ts. To report something more: perf-text.ts, then
// the matching field in `facts`.

import './perf-report.css';
import source from '../patch-notes.md?raw';
import type { Game } from '../wasm/engine.js';
import type { Coop } from '../net/coop';
import type { FrameTimes } from '../render/frame-stats';
import { type GpuFacts, probeGpu } from '../render/gpu-probe';
import type { QualityStore } from '../render/quality';
import type { Renderer } from '../render/renderer';
import type { SkyStore } from '../render/sky/settings';
import { button, h } from './dom';
import { accelerationSteps, formatReport, kindWords, type ReportFacts } from './perf-text';

const DISMISSED_KEY = 'opencraft.softwareNoticeDismissed';
const NO_CARD = "Your browser isn't using your graphics card";
const WHY = 'That makes the game, and even this menu, very slow.';
const DRIVER = 'If it is already on, updating your graphics driver may help.';

export interface PerfSources {
  game: Game;
  renderer: Renderer;
  frames: FrameTimes;
  sky: SkyStore;
  quality: QualityStore;
  canvas: HTMLCanvasElement;
  coop: () => Coop | null;
  timing: () => { frameMs: number; workMs: number };
}

export class PerfPanel {
  readonly el = h('div', 'comfort');
  readonly banner = h('div', 'perf-banner hidden');
  readonly gpu: GpuFacts;
  private readonly result = h('p', 'comfort-help');
  private readonly box = h('textarea', 'perf-box hidden');
  private lost = 0;

  constructor(private readonly src: PerfSources) {
    this.gpu = probeGpu(src.renderer.gl);
    src.canvas.addEventListener('webglcontextlost', () => this.lost++);
    this.box.readOnly = true;
    this.box.rows = 8;
    this.el.append(this.row('Graphics', this.gpu.name || 'Not named by your browser'), this.row('Using the graphics card', kindWords(this.gpu.kind)));
    const steps = `${accelerationSteps(navigator.userAgent)} ${DRIVER}`;
    if (this.gpu.kind === 'software') this.el.append(h('p', 'comfort-help', `${WHY} ${steps}`));
    this.el.append(
      button('secondary-btn', 'Copy report', () => void this.copy()),
      this.result,
      this.box,
    );
    if (this.gpu.kind === 'software' && !dismissed()) this.showBanner(steps);
  }

  /** The report as it stands now. */
  report(): string {
    return formatReport(this.facts());
  }

  private row(label: string, value: string): HTMLElement {
    const row = h('div', 'comfort-row');
    row.append(h('span', 'comfort-label', label), h('span', 'perf-value', value));
    return row;
  }

  private showBanner(steps: string): void {
    this.banner.classList.remove('hidden');
    this.banner.append(
      h('strong', '', NO_CARD),
      h('p', '', `${WHY} ${steps}`),
      button('secondary-btn', 'Got it', () => {
        this.banner.classList.add('hidden');
        remember();
      }),
    );
  }

  /** Copying is refused in some embeds (an itch.io frame, for one): then the text is shown to copy by hand. */
  private async copy(): Promise<void> {
    const text = this.report();
    this.box.classList.add('hidden');
    try {
      await navigator.clipboard.writeText(text);
      this.result.textContent = 'Copied. Paste it wherever you told us about the problem.';
    } catch {
      this.box.value = text;
      this.box.classList.remove('hidden');
      this.box.select();
      this.result.textContent = 'Your browser would not let the game copy this. Select the text below and copy it.';
    }
  }

  private facts(): ReportFacts {
    const { game, renderer, frames, sky, quality, canvas, coop, timing } = this.src;
    const session = coop();
    return {
      notes: source,
      gpu: this.gpu.name,
      kind: this.gpu.kind,
      browser: navigator.userAgent,
      window: [canvas.clientWidth, canvas.clientHeight],
      screen: [screen.width, screen.height],
      drawn: [canvas.width, canvas.height],
      ratio: window.devicePixelRatio || 1,
      cores: navigator.hardwareConcurrency,
      memoryGb: (navigator as Navigator & { deviceMemory?: number }).deviceMemory,
      quality: quality.settings,
      sky: sky.settings,
      playing: frames.playing.summary(),
      menu: frames.menu.summary(),
      ...timing(),
      ...renderer.stats,
      chunks: [game.chunks_loaded(), game.chunks_pending(), game.chunks_dirty()],
      factory: [game.belts(), game.miners(), game.boxes()],
      items: game.item_entities(),
      session: session ? `co-op ${game.is_client() ? 'guest' : 'host'}, ${session.players().length} players` : 'solo',
      lost: this.lost,
    };
  }
}

// Storage can be unavailable (private windows, blocked site data): the banner then comes back each visit.
function dismissed(): boolean {
  try {
    return localStorage.getItem(DISMISSED_KEY) !== null;
  } catch {
    return false;
  }
}

function remember(): void {
  try {
    localStorage.setItem(DISMISSED_KEY, '1');
  } catch {
    // Not remembered; see dismissed.
  }
}
