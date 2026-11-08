// Decides when the game should lower its picture quality by itself. Pure: frames in, a step out (ui/quality.ts acts on it), so
// web/test/auto-quality.test.mjs replays recorded frame traces through it. Levels index `SCALES` in quality.ts, 0 being the best.
// Rules: judge the average frame of a few seconds without its slowest one (a median hid stutter: frames alternating 11 and 149 ms
// read as 11; the slowest goes because the first frame after a resize is long); "too slow" is a fixed frame time, not the
// display's refresh, so 100 fps on a 144 Hz screen is not lag; a step that doesn't help is undone and Auto stops, which covers a
// browser's frame-rate lock (battery saver); raise once at most, so it can't flip back and forth.
// To tune: the constants below; `scripts/frame-bench.mjs` measures what each step buys.

/** Frames before this after the world is ready say nothing: shaders and first uploads. */
const WARM_MS = 5000;
const WINDOW_MS = 3000;
/** At least this many frames make a judgement, however slow they are. */
const MIN_FRAMES = 3;
/** A typical frame longer than this is too slow (under 40 fps). */
const SLOW_MS = 25;
/** A typical frame at or under this is comfortable, at a 60 Hz display's cap or better. */
const GOOD_MS = 18;
const CONFIRM_WINDOWS = 2;
/** Each lowering must cut the typical frame by at least this share, or it is undone. A frame-rate lock gains nothing at all;
 *  Firefox drawing in software gained 17% at best, and one noisy window was enough to throw that away. */
const MIN_GAIN = 0.05;
/** Windows averaged to judge a lowering. */
const VERIFY_WINDOWS = 2;
/** Slower than this after an undo is worth a suggestion; a 30 fps lock isn't. */
const VERY_SLOW_MS = 40;
/** Picture changes at most this often: enough to reach the lowest level inside half a minute. */
const CHANGE_GAP_MS = 12000;
const RAISE_AFTER_MS = 60000;
/** Script taking this share of the frame means drawing isn't the limit: fewer pixels won't help. */
const CPU_SHARE = 0.6;
/** Tolerance when noticing frames that weren't fed (a menu, a panel, a hidden tab). */
const SKIP_MS = 200;

/** lower or raise one level; undo the last lowering; or, out of levels, suggest what would help with this limit. */
export type AutoStep = 'lower' | 'raise' | 'undo' | 'suggest-drawing' | 'suggest-script' | null;

export class AutoQuality {
  level: number;
  private readonly lowest: number;
  private first = Number.NaN;
  private lastNow = Number.NaN;
  private windowStart = Number.NaN;
  private samples: number[] = [];
  private scriptSum = 0;
  private slowWindows = 0;
  private lastChange = Number.NEGATIVE_INFINITY;
  private goodSince = Number.NaN;
  private before = Number.NaN; // the typical frame just before a lowering still to be judged
  private checks = 0;
  private previous = Number.NaN; // the last window's typical frame
  private stopped = false;
  private raised = false;
  private suggested = false;

  constructor(level: number, lowest: number) {
    this.level = level;
    this.lowest = lowest;
  }

  /** One played frame. Don't call it while a menu or panel is open: gaps are noticed and restart the window. */
  frame(now: number, intervalMs: number, scriptMs: number): AutoStep {
    if (Number.isNaN(this.first)) this.first = now;
    const skipped = now - this.lastNow > intervalMs + SKIP_MS;
    this.lastNow = now;
    if (now - this.first < WARM_MS) return null;
    if (skipped || Number.isNaN(this.windowStart)) this.restart(now);
    this.samples.push(intervalMs);
    this.scriptSum += scriptMs;
    if (now - this.windowStart < WINDOW_MS || this.samples.length < MIN_FRAMES) return null;
    const sum = this.samples.reduce((a, b) => a + b, 0);
    const typical = (sum - Math.max(...this.samples)) / (this.samples.length - 1);
    const script = this.scriptSum / this.samples.length;
    this.restart(now);
    return this.judge(now, typical, script);
  }

  private restart(now: number): void {
    this.windowStart = now;
    this.samples = [];
    this.scriptSum = 0;
  }

  private judge(now: number, typical: number, script: number): AutoStep {
    const previous = this.previous;
    this.previous = typical;
    if (!Number.isNaN(this.before) && ++this.checks === VERIFY_WINDOWS) {
      const helped = (this.before - (previous + typical) / 2) / this.before >= MIN_GAIN;
      this.before = Number.NaN;
      if (!helped) {
        this.level--;
        this.stopped = true;
        this.lastChange = now;
        this.slowWindows = 0;
        return 'undo';
      }
    }
    this.goodSince = typical > GOOD_MS ? Number.NaN : Number.isNaN(this.goodSince) ? now : this.goodSince;
    this.slowWindows = typical > SLOW_MS ? this.slowWindows + 1 : 0;
    if (!Number.isNaN(this.before)) return null;
    if (this.level > 0 && !this.raised && !this.stopped && now - this.goodSince >= RAISE_AFTER_MS) {
      this.level--;
      this.raised = true;
      this.lastChange = now;
      this.goodSince = Number.NaN;
      return 'raise';
    }
    if (this.slowWindows < CONFIRM_WINDOWS || now - this.lastChange < CHANGE_GAP_MS) return null;
    const drawingIsNotTheLimit = script >= CPU_SHARE * typical;
    if (!drawingIsNotTheLimit && !this.stopped && this.level < this.lowest) {
      this.level++;
      this.before = Number.isNaN(previous) ? typical : (previous + typical) / 2;
      this.checks = 0;
      this.lastChange = now;
      this.slowWindows = 0;
      return 'lower';
    }
    if (this.suggested || (this.stopped && !drawingIsNotTheLimit && typical <= VERY_SLOW_MS)) return null;
    this.suggested = true;
    return drawingIsNotTheLimit ? 'suggest-script' : 'suggest-drawing';
  }
}
