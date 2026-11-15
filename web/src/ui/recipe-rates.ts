// How fast a machine runs a recipe, in the units a factory is planned in: items a minute and items a second. Pure, so it is
// tested (`web/test/recipe-rates.test.mjs`). The recipe cards (`machine-recipes.ts`) print these for every ingredient and
// product, so the ratio between two machines reads straight off the panels.

/** "4 s", "1.5 s", "1 min 30 s". */
export function formatSeconds(s: number): string {
  if (s < 60) return `${Number.isInteger(s) ? s : s.toFixed(1)} s`;
  const m = Math.floor(s / 60);
  const rest = Math.round(s - m * 60);
  return rest === 0 ? `${m} min` : `${m} min ${rest} s`;
}

/** A rate as a short number: "120", "37.5", "0.67", "<0.01"; never more digits than a glance needs. */
export function formatRate(n: number): string {
  if (n >= 100) return String(Math.round(n));
  if (n >= 0.01) {
    const text = n.toFixed(n >= 10 ? 1 : 2);
    return text.replace(/\.?0+$/, '');
  }
  return n > 0 ? '<0.01' : '0';
}

/** "40/min", for `count` items every `seconds` seconds. */
export function perMinute(count: number, seconds: number): string {
  return `${formatRate((count * 60) / Math.max(seconds, 1e-9))}/min`;
}

/** "0.67/s", for `count` items every `seconds` seconds. */
export function perSecond(count: number, seconds: number): string {
  return `${formatRate(count / Math.max(seconds, 1e-9))}/s`;
}

/** "40/min · 0.67/s". */
export function rateWords(count: number, seconds: number): string {
  return `${perMinute(count, seconds)} · ${perSecond(count, seconds)}`;
}
