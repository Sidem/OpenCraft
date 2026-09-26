// Keeps a co-op game going when its frame loop stops. Browsers stop
// `requestAnimationFrame` in hidden tabs and minimised or covered windows, and slow their timers
// (Chrome: to once a second, and to once a minute after five minutes), which would freeze a host and
// with it everyone. A dedicated worker's timer isn't slowed, so a tiny one posts 60 messages a second,
// and whenever no frame has run for `STALL_MS` each message runs `step` (a frame without drawing).
// The page's visibility isn't trusted: a page can say it is visible while getting no frames.

/** Worker ticks per second. */
const TICK_HZ = 60;
/** The frame loop counts as stopped after this long without a frame. */
const STALL_MS = 250;

/** Calls `step` 60 times a second while the frame loop, which last ran at `lastFrame()`
 * (`performance.now()` time), is stopped. Returns a function that stops it. */
export function tickWhenStalled(lastFrame: () => number, step: () => void): () => void {
  const source = `setInterval(() => postMessage(0), ${1000 / TICK_HZ});`;
  const url = URL.createObjectURL(new Blob([source], { type: 'text/javascript' }));
  const worker = new Worker(url);
  URL.revokeObjectURL(url);
  worker.onmessage = () => {
    if (performance.now() - lastFrame() > STALL_MS) step();
  };
  return () => worker.terminate();
}
