// Periodic table layout and metadata for all 118 chemical elements, plus form labels/colours for
// CodexPanel (`ui/codex.ts`). Pure data with no DOM imports so `web/test/codex.test.mjs` verifies it.

export type ElementCategory =
  | 'nonmetal'
  | 'noble'
  | 'alkali'
  | 'alkaline'
  | 'metalloid'
  | 'halogen'
  | 'metal'
  | 'transition'
  | 'lanthanide'
  | 'actinide';

export interface PeriodicElement {
  z: number;
  symbol: string;
  name: string;
  col: number; // 1..18
  row: number; // 1..7, plus 9 (lanthanides) and 10 (actinides)
  cat: ElementCategory;
}

export const CATEGORY_LABELS: Record<ElementCategory, string> = {
  nonmetal: 'Reactive nonmetal',
  noble: 'Noble gas',
  alkali: 'Alkali metal',
  alkaline: 'Alkaline earth metal',
  metalloid: 'Metalloid',
  halogen: 'Halogen',
  metal: 'Post-transition metal',
  transition: 'Transition metal',
  lanthanide: 'Lanthanide',
  actinide: 'Actinide',
};

/** Form labels indexed by `codex::Form as u8` (0..7). */
export const FORM_LABELS = ['Ore', 'Oxide', 'Carbonate', 'Pure', 'Alloy', 'Compound', 'Gas', 'Liquid'] as const;

type Row = [number, string, string, number, number, ElementCategory];

const RAW: readonly Row[] = [
  [1, 'H', 'Hydrogen', 1, 1, 'nonmetal'],
  [2, 'He', 'Helium', 18, 1, 'noble'],
  [3, 'Li', 'Lithium', 1, 2, 'alkali'],
  [4, 'Be', 'Beryllium', 2, 2, 'alkaline'],
  [5, 'B', 'Boron', 13, 2, 'metalloid'],
  [6, 'C', 'Carbon', 14, 2, 'nonmetal'],
  [7, 'N', 'Nitrogen', 15, 2, 'nonmetal'],
  [8, 'O', 'Oxygen', 16, 2, 'nonmetal'],
  [9, 'F', 'Fluorine', 17, 2, 'halogen'],
  [10, 'Ne', 'Neon', 18, 2, 'noble'],
  [11, 'Na', 'Sodium', 1, 3, 'alkali'],
  [12, 'Mg', 'Magnesium', 2, 3, 'alkaline'],
  [13, 'Al', 'Aluminium', 13, 3, 'metal'],
  [14, 'Si', 'Silicon', 14, 3, 'metalloid'],
  [15, 'P', 'Phosphorus', 15, 3, 'nonmetal'],
  [16, 'S', 'Sulfur', 16, 3, 'nonmetal'],
  [17, 'Cl', 'Chlorine', 17, 3, 'halogen'],
  [18, 'Ar', 'Argon', 18, 3, 'noble'],
  [19, 'K', 'Potassium', 1, 4, 'alkali'],
  [20, 'Ca', 'Calcium', 2, 4, 'alkaline'],
  [21, 'Sc', 'Scandium', 3, 4, 'transition'],
  [22, 'Ti', 'Titanium', 4, 4, 'transition'],
  [23, 'V', 'Vanadium', 5, 4, 'transition'],
  [24, 'Cr', 'Chromium', 6, 4, 'transition'],
  [25, 'Mn', 'Manganese', 7, 4, 'transition'],
  [26, 'Fe', 'Iron', 8, 4, 'transition'],
  [27, 'Co', 'Cobalt', 9, 4, 'transition'],
  [28, 'Ni', 'Nickel', 10, 4, 'transition'],
  [29, 'Cu', 'Copper', 11, 4, 'transition'],
  [30, 'Zn', 'Zinc', 12, 4, 'transition'],
  [31, 'Ga', 'Gallium', 13, 4, 'metal'],
  [32, 'Ge', 'Germanium', 14, 4, 'metalloid'],
  [33, 'As', 'Arsenic', 15, 4, 'metalloid'],
  [34, 'Se', 'Selenium', 16, 4, 'nonmetal'],
  [35, 'Br', 'Bromine', 17, 4, 'halogen'],
  [36, 'Kr', 'Krypton', 18, 4, 'noble'],
  [37, 'Rb', 'Rubidium', 1, 5, 'alkali'],
  [38, 'Sr', 'Strontium', 2, 5, 'alkaline'],
  [39, 'Y', 'Yttrium', 3, 5, 'transition'],
  [40, 'Zr', 'Zirconium', 4, 5, 'transition'],
  [41, 'Nb', 'Niobium', 5, 5, 'transition'],
  [42, 'Mo', 'Molybdenum', 6, 5, 'transition'],
  [43, 'Tc', 'Technetium', 7, 5, 'transition'],
  [44, 'Ru', 'Ruthenium', 8, 5, 'transition'],
  [45, 'Rh', 'Rhodium', 9, 5, 'transition'],
  [46, 'Pd', 'Palladium', 10, 5, 'transition'],
  [47, 'Ag', 'Silver', 11, 5, 'transition'],
  [48, 'Cd', 'Cadmium', 12, 5, 'transition'],
  [49, 'In', 'Indium', 13, 5, 'metal'],
  [50, 'Sn', 'Tin', 14, 5, 'metal'],
  [51, 'Sb', 'Antimony', 15, 5, 'metalloid'],
  [52, 'Te', 'Tellurium', 16, 5, 'metalloid'],
  [53, 'I', 'Iodine', 17, 5, 'halogen'],
  [54, 'Xe', 'Xenon', 18, 5, 'noble'],
  [55, 'Cs', 'Caesium', 1, 6, 'alkali'],
  [56, 'Ba', 'Barium', 2, 6, 'alkaline'],
  [57, 'La', 'Lanthanum', 4, 9, 'lanthanide'],
  [58, 'Ce', 'Cerium', 5, 9, 'lanthanide'],
  [59, 'Pr', 'Praseodymium', 6, 9, 'lanthanide'],
  [60, 'Nd', 'Neodymium', 7, 9, 'lanthanide'],
  [61, 'Pm', 'Promethium', 8, 9, 'lanthanide'],
  [62, 'Sm', 'Samarium', 9, 9, 'lanthanide'],
  [63, 'Eu', 'Europium', 10, 9, 'lanthanide'],
  [64, 'Gd', 'Gadolinium', 11, 9, 'lanthanide'],
  [65, 'Tb', 'Terbium', 12, 9, 'lanthanide'],
  [66, 'Dy', 'Dysprosium', 13, 9, 'lanthanide'],
  [67, 'Ho', 'Holmium', 14, 9, 'lanthanide'],
  [68, 'Er', 'Erbium', 15, 9, 'lanthanide'],
  [69, 'Tm', 'Thulium', 16, 9, 'lanthanide'],
  [70, 'Yb', 'Ytterbium', 17, 9, 'lanthanide'],
  [71, 'Lu', 'Lutetium', 18, 9, 'lanthanide'],
  [72, 'Hf', 'Hafnium', 4, 6, 'transition'],
  [73, 'Ta', 'Tantalum', 5, 6, 'transition'],
  [74, 'W', 'Tungsten', 6, 6, 'transition'],
  [75, 'Re', 'Rhenium', 7, 6, 'transition'],
  [76, 'Os', 'Osmium', 8, 6, 'transition'],
  [77, 'Ir', 'Iridium', 9, 6, 'transition'],
  [78, 'Pt', 'Platinum', 10, 6, 'transition'],
  [79, 'Au', 'Gold', 11, 6, 'transition'],
  [80, 'Hg', 'Mercury', 12, 6, 'transition'],
  [81, 'Tl', 'Thallium', 13, 6, 'metal'],
  [82, 'Pb', 'Lead', 14, 6, 'metal'],
  [83, 'Bi', 'Bismuth', 15, 6, 'metal'],
  [84, 'Po', 'Polonium', 16, 6, 'metalloid'],
  [85, 'At', 'Astatine', 17, 6, 'halogen'],
  [86, 'Rn', 'Radon', 18, 6, 'noble'],
  [87, 'Fr', 'Francium', 1, 7, 'alkali'],
  [88, 'Ra', 'Radium', 2, 7, 'alkaline'],
  [89, 'Ac', 'Actinium', 4, 10, 'actinide'],
  [90, 'Th', 'Thorium', 5, 10, 'actinide'],
  [91, 'Pa', 'Protactinium', 6, 10, 'actinide'],
  [92, 'U', 'Uranium', 7, 10, 'actinide'],
  [93, 'Np', 'Neptunium', 8, 10, 'actinide'],
  [94, 'Pu', 'Plutonium', 9, 10, 'actinide'],
  [95, 'Am', 'Americium', 10, 10, 'actinide'],
  [96, 'Cm', 'Curium', 11, 10, 'actinide'],
  [97, 'Bk', 'Berkelium', 12, 10, 'actinide'],
  [98, 'Cf', 'Californium', 13, 10, 'actinide'],
  [99, 'Es', 'Einsteinium', 14, 10, 'actinide'],
  [100, 'Fm', 'Fermium', 15, 10, 'actinide'],
  [101, 'Md', 'Mendelevium', 16, 10, 'actinide'],
  [102, 'No', 'Nobelium', 17, 10, 'actinide'],
  [103, 'Lr', 'Lawrencium', 18, 10, 'actinide'],
  [104, 'Rf', 'Rutherfordium', 4, 7, 'transition'],
  [105, 'Db', 'Dubnium', 5, 7, 'transition'],
  [106, 'Sg', 'Seaborgium', 6, 7, 'transition'],
  [107, 'Bh', 'Bohrium', 7, 7, 'transition'],
  [108, 'Hs', 'Hassium', 8, 7, 'transition'],
  [109, 'Mt', 'Meitnerium', 9, 7, 'transition'],
  [110, 'Ds', 'Darmstadtium', 10, 7, 'transition'],
  [111, 'Rg', 'Roentgenium', 11, 7, 'transition'],
  [112, 'Cn', 'Copernicium', 12, 7, 'transition'],
  [113, 'Nh', 'Nihonium', 13, 7, 'metal'],
  [114, 'Fl', 'Flerovium', 14, 7, 'metal'],
  [115, 'Mc', 'Moscovium', 15, 7, 'metal'],
  [116, 'Lv', 'Livermorium', 16, 7, 'metal'],
  [117, 'Ts', 'Tennessine', 17, 7, 'halogen'],
  [118, 'Og', 'Oganesson', 18, 7, 'noble'],
];

export const PERIODIC_ELEMENTS: readonly PeriodicElement[] = RAW.map(([z, symbol, name, col, row, cat]) => ({
  z,
  symbol,
  name,
  col,
  row,
  cat,
}));

export const ELEMENT_BY_Z: ReadonlyMap<number, PeriodicElement> = new Map(
  PERIODIC_ELEMENTS.map((e) => [e.z, e]),
);
