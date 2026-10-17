// How loud the ambient beds may be together (used by ambience.ts, tested in web/test/ambience-mix.test.mjs). Each bed
// has a peak gain, and a loudness weight (how loud one unit of its gain sounds next to the wind's: running water is
// bright and carries far more than the same gain of wind or surf, a cave's rumble is felt more than heard). The beds
// are uncorrelated noise, so their loudness adds as the root of the sum of squares; when that passes `CEILING` every
// bed is turned down by the same factor, so piling wind, surf and a river on top of each other is never louder than
// the loudest of them alone. Pure numbers, no Web Audio. To add a bed: a row in `BEDS`.

export type BedName = 'wind' | 'surf' | 'river' | 'rain' | 'hum';

/** `peak` is the gain at a bed's level 1; `loud` its loudness per unit of gain next to the wind's (measured with an
 * offline render, A-weighted: river 0.17 sounds as loud as wind 0.3). */
export const BEDS: Record<BedName, { peak: number; loud: number }> = {
  wind: { peak: 0.4, loud: 1 },
  surf: { peak: 0.5, loud: 0.5 },
  river: { peak: 0.17, loud: 1.8 },
  rain: { peak: 0.3, loud: 1.2 },
  hum: { peak: 0.5, loud: 0.5 },
};

/** The most the beds may add up to, in the wind's units: one full wind bed. */
export const CEILING = 0.4;

/** The gain of each bed for target `levels` (0 to 1 each), turned down together when they would add up past `CEILING`. */
export function bedGains(levels: Record<BedName, number>): Record<BedName, number> {
  const out = { wind: 0, surf: 0, river: 0, rain: 0, hum: 0 };
  let sum = 0;
  for (const name of Object.keys(BEDS) as BedName[]) {
    const { peak, loud } = BEDS[name];
    out[name] = Math.min(1, Math.max(0, levels[name])) * peak;
    sum += (out[name] * loud) ** 2;
  }
  const scale = Math.min(1, CEILING / Math.sqrt(sum || 1));
  for (const name of Object.keys(out) as BedName[]) out[name] *= scale;
  return out;
}
