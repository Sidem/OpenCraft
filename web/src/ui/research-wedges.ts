// The twelve disciplines (wedges) of the research grid's rows (`docs/TECH_TREE.md` section 10). Pure data, no DOM:
// testable in Node.
// Order of `WEDGES` matches engine `research::rings::Wedge` (0 Logistics .. 11 Building, top row first).

import { KINDS, type Kind } from './research-kinds.ts';

export const WEDGE_COUNT = 12;

export interface WedgeInfo {
  /** Engine `Wedge` index (0..11), clockwise from the top. */
  id: number;
  name: string;
  /** Compact label for narrow pills and the radar rim. */
  short: string;
  kind: Kind;
  /** RGB triplet matching `KINDS[kind].colour`. */
  colour: string;
  /** One-line plain-language summary for the wedge lane header. */
  blurb: string;
}

const wedge = (id: number, name: string, short: string, kind: Kind, blurb: string): WedgeInfo => ({
  id,
  name,
  short,
  kind,
  colour: KINDS[kind].colour,
  blurb,
});

export const WEDGES: readonly WedgeInfo[] = [
  wedge(0, 'Logistics', 'Logistics', 'logistics', 'Belts, lifts, storage, trains and orbital drop pods.'),
  wedge(1, 'Workshop', 'Workshop', 'machines', 'Assembly machines, upgrade kits, recycling and labs.'),
  wedge(2, 'Ores & Metals', 'Metals', 'materials', 'Smelting, steel, crushing, washing and rare ores.'),
  wedge(3, 'Chemistry', 'Chemistry', 'oil', 'Oil, gas, plastics, acids, pigments and brines.'),
  wedge(4, 'Power', 'Power', 'power', 'Water, steam, solar, diesel, nuclear and orbital power.'),
  wedge(5, 'Computing', 'Computing', 'chip', 'Circuits, logic, processors, datacenters and AI labs.'),
  wedge(6, 'Robots & Drones', 'Drones', 'drone', 'Construction drones, earthworks and cargo swarms.'),
  wedge(7, 'Your Gear', 'Gear', 'player', 'Packs, gloves, tools, suits, flight and X-ray goggles.'),
  wedge(8, 'Survey', 'Survey', 'survey', 'Ore scanning, seismics, rocketry and deep survey.'),
  wedge(9, 'Sea', 'Sea', 'fluid', 'Salt pans, boats, fishing grounds, harbours and nodules.'),
  wedge(10, 'Towns & Life', 'Towns', 'nature', 'Forestry, settlements, farming and livestock.'),
  wedge(11, 'Building', 'Building', 'build', 'Masonry, glass, metalwork, paint, kilns and lighting.'),
];
