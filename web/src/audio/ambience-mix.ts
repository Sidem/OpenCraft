// How loud the ambient beds may be together (used by ambience.ts, tested in web/test/ambience-mix.test.mjs). Each bed
// has a peak gain, and a loudness weight (how loud one unit of its gain sounds next to the wind's: running water is
// bright and carries far more than the same gain of wind or surf, a cave's rumble is felt more than heard). The beds
// are uncorrelated noise, so their loudness adds as the root of the sum of squares. Two limits keep them in check:
// water (surf and river together) is held under `WATER_CEILING`, because its steady hiss tires the ear long before
// the same loudness of wind would, and the sum of all beds is held under `CEILING`; each limit turns the beds it
// covers down by one common factor, so their proportions survive. Pure numbers, no Web Audio. To add a bed: a row in
// `BEDS` (and in `WATER` if it is water).

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

/** The beds that are water, which share one smaller ceiling. */
export const WATER: readonly BedName[] = ['surf', 'river'];

/** The most the beds may add up to, in the wind's units: one full wind bed. */
export const CEILING = 0.4;

/** The most the water beds may add up to: a river or the sea at its loudest is about a third of a full wind bed. */
export const WATER_CEILING = 0.13;

const loudness = (gains: Record<BedName, number>, names: readonly BedName[]): number =>
  Math.sqrt(names.reduce((sum, name) => sum + (gains[name] * BEDS[name].loud) ** 2, 0));

/** The gain of each bed for target `levels` (0 to 1 each), turned down together when they would pass a ceiling. */
export function bedGains(levels: Record<BedName, number>): Record<BedName, number> {
  const out = { wind: 0, surf: 0, river: 0, rain: 0, hum: 0 };
  const all = Object.keys(BEDS) as BedName[];
  for (const name of all) out[name] = Math.min(1, Math.max(0, levels[name])) * BEDS[name].peak;
  const water = Math.min(1, WATER_CEILING / (loudness(out, WATER) || 1));
  for (const name of WATER) out[name] *= water;
  const scale = Math.min(1, CEILING / (loudness(out, all) || 1));
  for (const name of all) out[name] *= scale;
  return out;
}
