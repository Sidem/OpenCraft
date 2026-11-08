// The words of the Performance section (ui/perf-report.ts): the report a player pastes and how to turn graphics acceleration
// on. Pure, so web/test/perf.test.mjs can run it. The report holds no world name, seed or player name; it leaves the
// machine only if the player copies it. To report something more: a field in `ReportFacts` and a line in `formatReport`.

import type { FrameSummary } from '../render/frame-stats';
import type { GpuKind } from '../render/gpu-probe';

export interface ReportFacts {
  notes: string; // the patch notes' source: the newest date says which build this is
  gpu: string;
  kind: GpuKind;
  browser: string;
  window: [number, number];
  screen: [number, number];
  drawn: [number, number];
  ratio: number;
  cores: number;
  memoryGb: number | undefined;
  quality: object;
  sky: object;
  playing: FrameSummary;
  menu: FrameSummary;
  frameMs: number;
  workMs: number;
  meshes: number;
  visible: number;
  drawCalls: number;
  quads: number;
  chunks: [loaded: number, queued: number, unmeshed: number];
  factory: [belts: number, miners: number, boxes: number];
  items: number;
  session: string;
  lost: number;
}

const KIND_WORDS: Record<GpuKind, string> = { software: 'no', hardware: 'yes', unknown: "can't tell" };

export const kindWords = (kind: GpuKind) => KIND_WORDS[kind];

const size = ([w, h]: [number, number]) => `${w}x${h}`;
const settings = (o: object) => Object.entries(o).map(([k, v]) => `${k}=${String(v)}`).join(' ');

function frames(s: FrameSummary): string {
  if (s.frames === 0) return 'no frames yet';
  return `${(1000 / s.medianMs).toFixed(0)} fps typical, slowest 5% over ${s.p95Ms.toFixed(0)} ms, ${s.long} long of ${s.frames} frames`;
}

/** The newest `## date · title` line's date. */
export function buildDate(notes: string): string {
  return /^## (\S+)/m.exec(notes)?.[1] ?? 'unknown';
}

export function formatReport(f: ReportFacts): string {
  return [
    'OpenCraft performance report',
    `Build: ${buildDate(f.notes)}`,
    `Graphics: ${f.gpu || 'not named by the browser'}`,
    `Using the graphics card: ${KIND_WORDS[f.kind]}`,
    `Browser: ${f.browser}`,
    `Window: ${size(f.window)} on a ${size(f.screen)} screen, pixel ratio ${f.ratio}, drawing ${size(f.drawn)}`,
    `Computer: ${f.cores} cores, ${f.memoryGb === undefined ? 'memory not reported' : `at least ${f.memoryGb} GB`}`,
    `Quality: ${settings(f.quality)}`,
    `Sky: ${settings(f.sky)}`,
    `While playing: ${frames(f.playing)}`,
    `In the menu: ${frames(f.menu)}`,
    `Script per frame: ${f.frameMs.toFixed(1)} ms, of which world building ${f.workMs.toFixed(1)} ms`,
    `Drawn: ${f.visible} of ${f.meshes} meshes, ${f.drawCalls} draw calls, ${(f.quads / 1000).toFixed(0)}k quads`,
    `Chunks: ${f.chunks[0]} loaded, ${f.chunks[1]} queued, ${f.chunks[2]} awaiting mesh`,
    `Factory: ${f.factory[0]} belts, ${f.factory[1]} miners, ${f.factory[2]} boxes, ${f.items} loose items`,
    `Session: ${f.session}`,
    `Graphics lost: ${f.lost} times`,
  ].join('\n');
}

/** How to turn acceleration on in this browser. Pages can't link to `chrome://` addresses, so these are to type. */
export function accelerationSteps(userAgent: string): string {
  const switchOn = 'turn on graphics acceleration ("Use graphics acceleration when available"), then restart the browser.';
  if (/Edg\//.test(userAgent)) return `Type edge://settings/system in the address bar, ${switchOn}`;
  if (/OPR\//.test(userAgent)) return `Type opera://settings/system in the address bar, ${switchOn}`;
  if (/Firefox\//.test(userAgent)) {
    return 'Type about:preferences in the address bar. Under Performance, untick "Use recommended performance settings", tick "Use hardware acceleration when available", then restart Firefox.';
  }
  if (/Chrome\//.test(userAgent)) return `Type chrome://settings/system in the address bar, ${switchOn}`;
  if (/Safari\//.test(userAgent)) return 'Safari has no switch for this: update Safari and macOS.';
  return "Look for hardware or graphics acceleration in your browser's settings, turn it on, then restart the browser.";
}
