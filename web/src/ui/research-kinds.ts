// What kind of research a tech is, for its icon and colour on the research tree (research-node.ts): logistics, power,
// upgrades, materials, science, electronics, fluids, drones, tools, personal gear, building, survey, space and the
// endless bonuses. Pure data, no DOM: the engine has no category for a tech, so the kinds are listed here by tech name
// (`BY_NAME`), and a tech not listed is guessed from words in its name (`GUESS`), else it is "general". To give a new
// tech its kind: add its name to `BY_NAME`. To add a kind: a row in `KINDS` (colour and a 24×24 line drawing).

export type Kind =
  | 'logistics'
  | 'machines'
  | 'upgrade'
  | 'materials'
  | 'power'
  | 'science'
  | 'chip'
  | 'fluid'
  | 'oil'
  | 'drone'
  | 'tools'
  | 'player'
  | 'build'
  | 'nature'
  | 'survey'
  | 'space'
  | 'endless'
  | 'general';

interface KindStyle {
  /** What it stands for, in words (the icon's tooltip). */
  label: string;
  /** Colour as "r, g, b". */
  colour: string;
  /** SVG path data, drawn as lines in a 24×24 box (several subpaths allowed). */
  path: string;
}

export const KINDS: Record<Kind, KindStyle> = {
  logistics: {
    label: 'Logistics',
    colour: '251, 191, 36',
    path: 'M7 7.5h10a4.5 4.5 0 0 1 0 9H7a4.5 4.5 0 0 1 0-9zM10.2 10l2.2 2-2.2 2M13.6 10l2.2 2-2.2 2M3 20.5h18',
  },
  machines: {
    label: 'Machines',
    colour: '251, 146, 60',
    path: 'M9.5 5.5L10.4 5.2L10.8 2.6H13.2L13.6 5.2L14.8 5.6L15.7 6L17.8 4.5L19.5 6.2L18 8.3L18.5 9.5L18.8 10.4L21.4 10.8V13.2L18.8 13.6L18.4 14.8L18 15.7L19.5 17.8L17.8 19.5L15.7 18L14.5 18.5L13.6 18.8L13.2 21.4H10.8L10.4 18.8L9.2 18.4L8.3 18L6.2 19.5L4.5 17.8L6 15.7L5.5 14.5L5.2 13.6L2.6 13.2V10.8L5.2 10.4L5.6 9.2L6 8.3L4.5 6.2L6.2 4.5L8.3 6zM15 12a3 3 0 1 0-6 0 3 3 0 1 0 6 0z',
  },
  upgrade: {
    label: 'Upgrades',
    colour: '163, 230, 53',
    path: 'M6 11l6-6 6 6M6 18l6-6 6 6',
  },
  materials: {
    label: 'Materials',
    colour: '148, 163, 184',
    path: 'M3 5.5h18v12H3zM3 9.5h18M3 13.5h18M9 5.5v4M15 5.5v4M6 9.5v4M12 9.5v4M18 9.5v4M9 13.5v4M15 13.5v4',
  },
  power: {
    label: 'Power',
    colour: '253, 224, 71',
    path: 'M13.5 2.5L5.5 13.5h6l-1 8 8-11h-6z',
  },
  science: {
    label: 'Science',
    colour: '129, 140, 248',
    path: 'M9.5 3h5M10 3v6.2L4.8 18a2 2 0 0 0 1.7 3h11a2 2 0 0 0 1.7-3L14 9.2V3M7.6 15h8.8',
  },
  chip: {
    label: 'Electronics and compute',
    colour: '45, 212, 191',
    path: 'M7.5 7.5h9v9h-9zM10.5 10.5h3v3h-3zM10 3v4.5M14 3v4.5M10 16.5V21M14 16.5V21M3 10h4.5M3 14h4.5M16.5 10H21M16.5 14H21',
  },
  fluid: {
    label: 'Water and fluids',
    colour: '96, 165, 250',
    path: 'M12 3c3 4.4 6 7.6 6 11a6 6 0 0 1-12 0c0-3.4 3-6.6 6-11zM9.2 14.6a3 3 0 0 0 2.6 2.6',
  },
  oil: {
    label: 'Oil and chemicals',
    colour: '205, 125, 62',
    path: 'M3.5 21h17M8.5 21l3.5-10.5 3.5 10.5M3 11.5l15-4M3 11.5v4.2M18 7.5v3.5M16.2 12.8a2.3 2.3 0 1 0 4 0 2.3 2.3 0 0 0-4 0zM12 10.5v-2',
  },
  drone: {
    label: 'Drones and automation',
    colour: '192, 132, 252',
    path: 'M9.5 10h5v4h-5zM9.5 11L6.4 8.2M14.5 11l3.1-2.8M9.5 13l-3.1 2.8M14.5 13l3.1 2.8M3.4 6.5a2.2 2.2 0 1 0 4.4 0 2.2 2.2 0 0 0-4.4 0zM16.2 6.5a2.2 2.2 0 1 0 4.4 0 2.2 2.2 0 0 0-4.4 0zM3.4 17.5a2.2 2.2 0 1 0 4.4 0 2.2 2.2 0 0 0-4.4 0zM16.2 17.5a2.2 2.2 0 1 0 4.4 0 2.2 2.2 0 0 0-4.4 0z',
  },
  tools: {
    label: 'Tools and crafting',
    colour: '214, 195, 165',
    path: 'M8 4.5Q18.5 3.5 19.5 15M4 20L15 9',
  },
  player: {
    label: 'Personal gear',
    colour: '244, 114, 182',
    path: 'M9.5 6.2V5a2.5 2.5 0 0 1 5 0v1.2M7 21a2 2 0 0 1-2-2V10a4.5 4.5 0 0 1 4.5-4.5h5A4.5 4.5 0 0 1 19 10v9a2 2 0 0 1-2 2zM5 11.5h14M8.5 21v-5.5h7V21M12 13v2.5',
  },
  build: {
    label: 'Building',
    colour: '253, 164, 105',
    path: 'M3.5 11.5L12 4l8.5 7.5M6 10v10h12V10M10 20v-5h4v5',
  },
  nature: {
    label: 'Nature',
    colour: '132, 204, 22',
    path: 'M12 3l5.5 7.5H14l4.5 6h-13l4.5-6H6.5zM12 16.5V21',
  },
  survey: {
    label: 'Exploration',
    colour: '34, 211, 238',
    path: 'M12 4.5a7.5 7.5 0 1 0 0 15 7.5 7.5 0 0 0 0-15zM12 2v4M12 18v4M2 12h4M18 12h4M12 11a1 1 0 1 0 0 2 1 1 0 0 0 0-2z',
  },
  space: {
    label: 'Space',
    colour: '167, 139, 250',
    path: 'M12 2.5c3.4 2.4 5 5.8 5 9.5l-2.2 3.5H9.2L7 12c0-3.7 1.6-7.1 5-9.5zM12 8.2a1.6 1.6 0 1 0 0 3.2 1.6 1.6 0 0 0 0-3.2zM7 12l-2.5 3.5L9 15M17 12l2.5 3.5L15 15M10 18.5c0 1.5.8 2.6 2 3.5 1.2-.9 2-2 2-3.5',
  },
  endless: {
    label: 'Endless bonus',
    colour: '251, 113, 133',
    path: 'M12 12c2-2.6 3.6-4.5 5.6-4.5a4.5 4.5 0 1 1 0 9c-2 0-3.6-1.9-5.6-4.5zM12 12c-2 2.6-3.6 4.5-5.6 4.5a4.5 4.5 0 1 1 0-9c2 0 3.6 1.9 5.6 4.5z',
  },
  general: {
    label: 'Research',
    colour: '160, 170, 190',
    path: 'M12 3l8 9-8 9-8-9z',
  },
};

const BY_NAME: Record<string, Kind> = {};
const LISTS: Record<Kind, string> = {
  logistics:
    'Belt Routing|Belt Lifts|Underpasses|Bulk Storage|Rails|Trains|Freight|Auto-Routing',
  machines: 'Mechanics|Assembly',
  upgrade: 'Belt Mk2|Mk3 Logistics|Mk3 Machines|Mk4 Logistics|Mk4 Machines|Mk5 Machines',
  materials: 'Masonry|Steelmaking|Ore Crushing|Ore Washing|Bauxite Processing|Recycling',
  power: 'Steam Power|Solar Power|Hydropower|Diesel Power|Nuclear Power|Long-Distance Power|Orbital Power',
  science:
    'Green Science|Blue Science|Violet Science|Gold Science|Research Center|AI Labs|AI Research',
  chip: 'Electronics|Logic|Processors|Wafers|Accelerators|Data Network|Datacenters|Cooling|Optimizer|Photonics',
  fluid: 'Fluid Handling|Electrolysis',
  oil: 'Oil Processing|Refining|Plastics|Acids and Lubricants',
  drone:
    'Robotics|Drone Power|Navigation|Construction Drones|Swarm Logistics|Personal Drone|Cargo Drones|Drone Swarms',
  tools: 'Steel Tools|Handcrafting I|Handcrafting II|Handcrafting III|Fabricator Gloves',
  player: 'Hauler Gear|Field Gear|Exosuit|Jetpack|Hover Pack|Hauler Mk3|Hauler Mk4|Hauler Mk5|X-ray Goggles',
  build: 'Earthworks|Hoists|Settlements|Architecture|LED Lamps',
  nature: 'Forestry',
  survey: 'Advanced Scanning|AI Survey',
  space: 'Rocketry|Orbital Comms|Orbital Science|Drop Pods',
  endless: 'Mining Productivity|Machine Speed|Drone Speed',
  general: '',
};
for (const [kind, names] of Object.entries(LISTS)) {
  for (const name of names.split('|')) if (name) BY_NAME[name] = kind as Kind;
}

/** Words in a tech's name that hint at its kind, for a tech not in `BY_NAME`; the first row that matches wins. */
const GUESS: [RegExp, Kind][] = [
  [/mk\d/i, 'upgrade'],
  [/science|research|lab\b/i, 'science'],
  [/drone|robot|swarm/i, 'drone'],
  [/rocket|orbit|satellite|launch/i, 'space'],
  [/power|solar|steam|nuclear|reactor|diesel|energy/i, 'power'],
  [/oil|plastic|acid|lubric|refin|chemi|diesel|fuel/i, 'oil'],
  [/fluid|water|pipe|pump/i, 'fluid'],
  [/belt|rail|train|storage|freight|logistic|lift/i, 'logistics'],
  [/chip|circuit|logic|data|compute|wafer/i, 'chip'],
  [/scan|survey|map/i, 'survey'],
  [/tool|craft|glove/i, 'tools'],
  [/gear|pack|suit/i, 'player'],
  [/build|house|town|earth|arch/i, 'build'],
  [/steel|ore|ingot|bauxite|brick|crush|wash/i, 'materials'],
];

/** The kind of the tech called `name`. */
export function kindOf(name: string): Kind {
  const known = BY_NAME[name];
  if (known) return known;
  for (const [pattern, kind] of GUESS) if (pattern.test(name)) return kind;
  return 'general';
}

/** The kind's icon as an SVG element's markup: a line drawing in the current colour. */
export function glyphSvg(kind: Kind): string {
  return `<svg viewBox="0 0 24 24" aria-hidden="true" fill="none" stroke="currentColor" stroke-width="1.7" stroke-linecap="round" stroke-linejoin="round"><path d="${KINDS[kind].path}"/></svg>`;
}
