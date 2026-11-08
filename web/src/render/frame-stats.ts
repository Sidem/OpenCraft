// How long frames really take: the time between one drawn frame and the next, kept apart for playing and for the menu so a
// report taken from the settings page still describes play. Measures the interval, which includes the wait for the GPU;
// `frameMs` in main.ts is JavaScript time only. A long interval with a small `frameMs` means the GPU is the limit.
// No DOM or GL here: web/test/perf.test.mjs runs it in node. To add a figure: a field in `FrameSummary`, set in `summary`.

/** About ten seconds at 60 fps; a slow machine's window is longer, which suits it. */
const SAMPLES = 600;
/** Longer than this between frames is a hidden tab or a sleeping laptop, not a frame. */
const GAP_MS = 5000;
/** A frame this long is a visible hitch. */
const LONG_MS = 50;
const SMOOTHING = 0.1;

export interface FrameSummary {
  frames: number;
  medianMs: number;
  /** One frame in twenty takes longer than this. */
  p95Ms: number;
  /** Frames of `LONG_MS` or more among `frames`. */
  long: number;
}

/** The latest frame intervals. `summary` sorts a copy, so call it for a readout, not every frame. */
export class FrameStats {
  private readonly ms = new Float32Array(SAMPLES);
  private count = 0;
  private next = 0;

  add(ms: number): void {
    this.ms[this.next] = ms;
    this.next = (this.next + 1) % SAMPLES;
    this.count = Math.min(this.count + 1, SAMPLES);
  }

  summary(): FrameSummary {
    const sorted = this.ms.slice(0, this.count).sort();
    const at = (q: number) => (sorted.length ? sorted[Math.min(sorted.length - 1, Math.ceil(q * sorted.length) - 1)] : 0);
    return { frames: sorted.length, medianMs: at(0.5), p95Ms: at(0.95), long: sorted.filter((v) => v >= LONG_MS).length };
  }
}

/** The frame loop's clock: `tick` once per drawn frame with the frame's timestamp. */
export class FrameTimes {
  readonly playing = new FrameStats();
  readonly menu = new FrameStats();
  /** The window the latest frame went into. */
  current = this.playing;
  /** The latest frame's interval; 0 when it was a gap, not a frame. */
  lastMs = 0;
  private last = Number.NaN;
  private smoothedMs = 0;

  tick(now: number, inMenu: boolean): void {
    const ms = now - this.last;
    this.last = now;
    this.lastMs = ms > 0 && ms < GAP_MS ? ms : 0;
    if (this.lastMs === 0) return;
    this.current = inMenu ? this.menu : this.playing;
    this.current.add(ms);
    this.smoothedMs = this.smoothedMs === 0 ? ms : this.smoothedMs + (ms - this.smoothedMs) * SMOOTHING;
  }

  /** Frames a second, smoothed; unlike a capped time step it keeps falling however slow the frames get. */
  get fps(): number {
    return this.smoothedMs > 0 ? 1000 / this.smoothedMs : 0;
  }
}
