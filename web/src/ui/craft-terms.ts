// Search words for the build menu beyond item and material names: each row lists words a player might type
// (synonyms and related ideas) and the recipes they should find. A recipe matches a row when its output's name
// contains one of the row's `items` (so "belt" covers Belt Mk3 too); the row's words are then searchable on it.
// To teach the search a new word, add it to a row; to cover a new machine, add its name to the rows it belongs to.
// `hasWord` is how a typed word matches a tile's words, `nameScore` how results are ranked.

/** [words typed, item names (parts of the output's lower-case name)], both separated by `|`. */
const ROWS: [string, string][] = [
  [
    'belt|belts|conveyor|transport|logistics|route|sort|split|merge|move',
    'belt|splitter|filter|underpass|lift|sensor',
  ],
  [
    'water|fluid|fluids|liquid|pipe|pipes|pump|pumping|plumbing',
    'pump|pipe|outlet|boiler|water wheel|steam turbine|washer|cooling tower|electrolyser|canister|silo',
  ],
  ['steam|boil|heat', 'boiler|steam turbine|cooling tower'],
  [
    'power|electric|electricity|energy|grid|wire|wires|cable|battery|generator|generate',
    'generator|pole|cable|boiler|turbine|solar|accumulator|water wheel|reactor|sensor|winch',
  ],
  ['fuel|burn|coal|diesel', 'generator|boiler|jetpack|smelter|blast furnace'],
  ['solar|sun|daylight|panel', 'solar|accumulator'],
  ['storage|store|chest|container|inventory|bulk|stockpile', 'storage box|silo|hauler pack|canister'],
  [
    'mine|mining|miner|dig|drill|ore|excavate|extract|quarry|stone|rock',
    'miner|quarry|pickaxe|shovel|core drill|scanner|mining rig|pumpjack|drone port|planner|crusher|washer',
  ],
  ['scan|scanner|prospect|find|survey|detect', 'scanner|core drill'],
  [
    'smelt|smelting|melt|furnace|metal|ingot|steel|alloy|plate',
    'smelter|blast furnace|arc furnace|electrolytic|iron plate|centrifuge',
  ],
  ['craft|make|build|assemble|assembly|factory|manufacture|produce', 'constructor|assembler|recycler|chip fab'],
  ['research|science|lab|laboratory|tech|study|knowledge', 'research|science pack|ai lab'],
  [
    'oil|petroleum|chemical|chemistry|plastic|refine|refining|gas',
    'pumpjack|refinery|cracker|chemical plant|diesel|canister|electrolyser|centrifuge',
  ],
  [
    'compute|computer|data|ai|chip|network|processor|brain|intelligence',
    'chip fab|fibre node|datacenter|cooling tower|ai lab|optimizer|data receiver|swarm hub|laser',
  ],
  ['laser|beam|light|wireless|mirror|optic|photon|photonics', 'laser|data receiver|lamp|swarm hub'],
  ['light|lamp|lighting|torch|dark|night|illuminate', 'lamp|torch'],
  ['rail|rails|train|trains|track|railway|locomotive|cargo', 'rail|locomotive|wagon|signal|dock'],
  ['drone|drones|fly|flying|flight|air|robot|terraform', 'drone|jetpack|hover pack|planner'],
  [
    'gear|equipment|wear|wearable|armour|armor|suit|boots|backpack|pack|carry|gloves',
    'hauler pack|boots|exo frame|mining rig|hover pack|gloves|jetpack',
  ],
  ['tool|tools|pick|pickaxe|axe|shovel|spade|chop|wood|tree|dig', 'pickaxe|axe|shovel'],
  ['wood|tree|log|lumber|timber|stick|plank|planks', 'planks|stick|ladder|torch|axe'],
  ['climb|ladder|ladders|stairs', 'ladder|hoist|lift'],
  ['elevator|hoist|shaft|winch|lift|vertical', 'hoist|lift'],
  ['nuclear|uranium|radiation|atom|atomic|fission', 'reactor|centrifuge'],
  ['recycle|recycling|waste|scrap|reuse', 'recycler'],
  ['space|rocket|orbit|satellite|launch', 'launch site'],
  ['kit|kits|upgrade|upgrades|tier|mk|improve', 'kit|upgrader'],
  ['logic|switch|sensor|automation|control|threshold', 'sensor|filter'],
];

const TABLE = ROWS.map(([words, items]) => ({ words: words.split('|'), items: items.split('|') }));

/** The extra search words for a recipe whose output is called `name`. */
export function relatedWords(name: string): string {
  const lower = name.toLowerCase();
  const found: string[] = [];
  for (const row of TABLE) if (row.items.some((i) => lower.includes(i))) found.push(...row.words);
  return found.join(' ');
}

/** How well an item's `name` answers the search `words` (lower case, as typed): 0 it is the name, 1 the name starts
 * with them, 2 each word starts a word of the name, 3 none of these (it matched on its materials or related words).
 * The build menu shows lower scores first, so typing "concrete" puts Concrete before what is made from it. */
export function nameScore(name: string, words: string[]): number {
  const lower = name.toLowerCase(), query = words.join(' ');
  if (words.length === 0) return 3;
  if (lower === query) return 0;
  if (lower.startsWith(query)) return 1;
  return words.every((w) => hasWord(lower, w)) ? 2 : 3;
}

/** Whether `text` has a word that starts with `w` (a plural also finds the singular: "belts" finds "belt"); a
 * very short `w` must be a whole word, so "ai" finds the AI Lab but not "air". */
export function hasWord(text: string, w: string): boolean {
  const padded = ` ${text} `;
  if (w.length <= 2) return padded.includes(` ${w} `);
  return padded.includes(` ${w}`) || (w.length > 3 && w.endsWith('s') && padded.includes(` ${w.slice(0, -1)}`));
}
