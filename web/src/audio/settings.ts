// The sound design: every sound in the game is described by dials with a plain-language meaning,
// each a number from 0 to 100. synth.ts turns dial positions into physical units (Hz, seconds).
// `SoundDesign` is also the copy/paste format of the sound designer, so a design tuned in the
// browser can be pasted back here as the new DEFAULT_DESIGN.

/** Sound materials in engine order; must match `block::sound` in the engine. */
export const MATERIALS = ['stone', 'dirt', 'grass', 'sand', 'wood', 'leaves', 'metal'] as const;
export type MaterialName = (typeof MATERIALS)[number];

export const MATERIAL_DIALS = ['pitch', 'tone', 'length', 'crunch', 'scatter', 'snap', 'thud', 'ring', 'volume', 'variation'] as const;
export type MaterialDial = (typeof MATERIAL_DIALS)[number];
export type MaterialParams = Record<MaterialDial, number>;

export const ACTIONS = ['dig', 'break', 'place', 'step', 'land', 'pickup', 'drop', 'click'] as const;
export type ActionName = (typeof ACTIONS)[number];

export const ACTION_DIALS = ['volume', 'pitch', 'length', 'weight', 'debris'] as const;
export type ActionDial = (typeof ACTION_DIALS)[number];
export type ActionParams = Record<ActionDial, number>;

export interface SoundDesign {
  materials: Record<MaterialName, MaterialParams>;
  actions: Record<ActionName, ActionParams>;
}

export interface DialInfo {
  label: string;
  /** What the dial means turned fully down. */
  low: string;
  /** What the dial means turned fully up. */
  high: string;
  help: string;
}

export const MATERIAL_LABELS: Record<MaterialName, string> = {
  stone: 'Stone',
  dirt: 'Dirt',
  grass: 'Grass',
  sand: 'Sand',
  wood: 'Wood',
  leaves: 'Leaves',
  metal: 'Metal',
};

export const MATERIAL_DIAL_INFO: Record<MaterialDial, DialInfo> = {
  pitch: {
    label: 'Pitch',
    low: 'deep',
    high: 'high',
    help: 'How high or low the whole sound is. Big, heavy things sound deep; small, light things sound high.',
  },
  tone: {
    label: 'Tone',
    low: 'muffled',
    high: 'bright',
    help: 'Muffled is dull, as if heard through a wall. Bright is crisp and airy. Soft ground like dirt is muffled; sand and leaves are bright.',
  },
  length: {
    label: 'Length',
    low: 'short',
    high: 'long',
    help: 'How long each hit lasts. Hard materials stop almost at once; loose ones like sand or leaves trail off.',
  },
  crunch: {
    label: 'Crunch',
    low: 'smooth',
    high: 'gritty',
    help: 'Smooth is a soft "shh". Gritty is made of tiny crackles, like gravel, dry leaves or crumbling rock.',
  },
  scatter: {
    label: 'Scatter',
    low: 'one hit',
    high: 'many bits',
    help: 'One solid knock, or lots of little bits moving at once? Turn it up for grass, sand, gravel and leaves.',
  },
  snap: {
    label: 'Snap',
    low: 'soft',
    high: 'sharp',
    help: 'How suddenly the sound starts. Sharp is a hard click, like stone or wood. Soft fades in gently, like brushing through leaves.',
  },
  thud: {
    label: 'Thud',
    low: 'none',
    high: 'heavy',
    help: 'A deep thump under the sound: the weight you feel from packed earth or a heavy block. Too much sounds boomy.',
  },
  ring: {
    label: 'Ring',
    low: 'dead',
    high: 'ringing',
    help: 'A musical note that rings after the hit. Wood gives a short, hollow knock; metal rings for a long time. Soft materials have none.',
  },
  volume: {
    label: 'Volume',
    low: 'quiet',
    high: 'loud',
    help: 'How loud this material is compared to the others. 50 is normal.',
  },
  variation: {
    label: 'Variation',
    low: 'identical',
    high: 'varied',
    help: 'How different each hit is from the last. At zero every hit is the same and sounds robotic; high sounds natural, until it gets messy.',
  },
};

/** How the material dials are grouped in the sound designer. */
export const MATERIAL_GROUPS: { title: string; blurb: string; dials: MaterialDial[] }[] = [
  { title: 'Character', blurb: 'Pitch, brightness and length', dials: ['pitch', 'tone', 'length'] },
  { title: 'Texture', blurb: 'Smooth or gritty, one hit or many', dials: ['crunch', 'scatter', 'snap'] },
  { title: 'Body', blurb: 'Weight and resonance', dials: ['thud', 'ring'] },
  { title: 'Mix', blurb: 'Level and naturalness', dials: ['volume', 'variation'] },
];

export const ACTION_DIAL_INFO: Record<ActionDial, DialInfo> = {
  volume: { label: 'Volume', low: 'quiet', high: 'loud', help: 'How loud this action is. 50 is normal.' },
  pitch: {
    label: 'Pitch',
    low: 'lower',
    high: 'higher',
    help: "Shifts this action higher or lower than the material's own pitch. 50 leaves it unchanged.",
  },
  length: {
    label: 'Length',
    low: 'shorter',
    high: 'longer',
    help: "Makes this action shorter or longer than the material's own length. 50 leaves it unchanged.",
  },
  weight: {
    label: 'Weight',
    low: 'lighter',
    high: 'heavier',
    help: 'Adds or removes low thump for this action only. 50 leaves it unchanged.',
  },
  debris: { label: 'Debris', low: 'few', high: 'many', help: 'How many pieces you hear clattering when a block breaks.' },
};

export const ACTION_INFO: Record<ActionName, { label: string; help: string; material: boolean; dials: ActionDial[] }> = {
  dig: { label: 'Mining', help: 'Repeated hits while you hold the mouse on a block.', material: true, dials: ['volume', 'pitch', 'length'] },
  break: { label: 'Break', help: 'The block crumbling when mining finishes.', material: true, dials: ['volume', 'pitch', 'length', 'debris'] },
  place: { label: 'Place', help: 'Setting a block down.', material: true, dials: ['volume', 'pitch', 'length', 'weight'] },
  step: { label: 'Footstep', help: 'Walking. Sneaking is quieter, sprinting louder.', material: true, dials: ['volume', 'pitch', 'length'] },
  land: {
    label: 'Landing',
    help: 'Hitting the ground after a jump or fall. Longer falls are louder.',
    material: true,
    dials: ['volume', 'pitch', 'length', 'weight'],
  },
  pickup: {
    label: 'Item pickup',
    help: 'The pop when an item flies into your hotbar. The same for every material.',
    material: false,
    dials: ['volume', 'pitch', 'length'],
  },
  drop: {
    label: 'Throw item',
    help: 'The whoosh when you drop an item with Q. The same for every material.',
    material: false,
    dials: ['volume', 'pitch', 'length'],
  },
  click: { label: 'Hotbar click', help: 'The tick when you change hotbar slots.', material: false, dials: ['volume', 'pitch'] },
};

// prettier-ignore
/** Starting points for a material. The first six are the built-in defaults; Metal's default is METAL_DEFAULT. */
export const PRESETS: Record<string, MaterialParams> = {
  Stone:  { pitch: 33, tone: 53, length: 16, crunch: 19, scatter: 2, snap: 70, thud: 19, ring: 7, volume: 15, variation: 35 },
  Dirt:   { pitch: 53, tone: 46, length: 28, crunch: 37, scatter: 84, snap: 42, thud: 16, ring: 0, volume: 15, variation: 24 },
  Grass:  { pitch: 40, tone: 30, length: 40, crunch: 55, scatter: 35, snap: 55, thud: 50, ring: 0, volume: 10, variation: 55 },
  Sand:   { pitch: 72, tone: 75, length: 64, crunch: 33, scatter: 100, snap: 20, thud: 11, ring: 0, volume: 11, variation: 50 },
  Wood:   { pitch: 15, tone: 28, length: 8, crunch: 33, scatter: 0, snap: 35, thud: 82, ring: 2, volume: 21, variation: 45 },
  Leaves: { pitch: 60, tone: 78, length: 55, crunch: 56, scatter: 84, snap: 35, thud: 9, ring: 0, volume: 10, variation: 60 },
  Gravel: { pitch: 50, tone: 50, length: 30, crunch: 85, scatter: 70, snap: 75, thud: 25, ring: 10, volume: 50, variation: 60 },
  Snow:   { pitch: 50, tone: 45, length: 50, crunch: 30, scatter: 60, snap: 20, thud: 10, ring: 0, volume: 45, variation: 50 },
  Metal:  { pitch: 60, tone: 70, length: 30, crunch: 5, scatter: 0, snap: 95, thud: 20, ring: 85, volume: 45, variation: 30 },
  Glass:  { pitch: 80, tone: 85, length: 15, crunch: 20, scatter: 15, snap: 100, thud: 0, ring: 60, volume: 45, variation: 40 },
  Wool:   { pitch: 35, tone: 20, length: 30, crunch: 10, scatter: 20, snap: 15, thud: 20, ring: 0, volume: 50, variation: 40 },
};

// Every entry gets its own object: structuredClone keeps shared references shared, so aliasing one
// object here would make a dial move several sounds at once.
const neutral = (): ActionParams => ({ volume: 50, pitch: 50, length: 50, weight: 50, debris: 50 });

/** Machines (belts, miners). Not tuned by ear yet; levelled to sit with the hand-tuned materials. */
const METAL_DEFAULT: MaterialParams = { ...PRESETS.Metal, ring: 55, thud: 30, volume: 14 };

export const DEFAULT_DESIGN: SoundDesign = {
  materials: {
    stone: { ...PRESETS.Stone },
    dirt: { ...PRESETS.Dirt },
    grass: { ...PRESETS.Grass },
    sand: { ...PRESETS.Sand },
    wood: { ...PRESETS.Wood },
    leaves: { ...PRESETS.Leaves },
    metal: { ...METAL_DEFAULT },
  },
  actions: {
    dig: neutral(),
    break: neutral(),
    place: neutral(),
    step: neutral(),
    land: neutral(),
    pickup: neutral(),
    drop: neutral(),
    click: neutral(),
  },
};

export const defaultDesign = (): SoundDesign => structuredClone(DEFAULT_DESIGN);

export interface SoundSettings {
  /** Master volume, 0–100. */
  volume: number;
  muted: boolean;
  design: SoundDesign;
}

const STORAGE_KEY = 'opencraft.sound';
const LEGACY_MUTE_KEY = 'opencraft.muted';

const isRecord = (v: unknown): v is Record<string, unknown> => typeof v === 'object' && v !== null;
export const clampDial = (v: number) => Math.round(Math.min(100, Math.max(0, v)));

/**
 * Copies every valid dial value from `src` into `into` and returns how many were applied. `src` is
 * untrusted (saved or pasted text), so anything that isn't a known dial with a finite number is ignored.
 */
export function mergeDesign(into: SoundDesign, src: unknown): number {
  if (!isRecord(src)) return 0;
  let applied = 0;
  const copy = <K extends string>(target: Record<K, number>, from: unknown, keys: readonly K[]) => {
    if (!isRecord(from)) return;
    for (const k of keys) {
      const v = from[k];
      if (typeof v === 'number' && Number.isFinite(v)) {
        target[k] = clampDial(v);
        applied++;
      }
    }
  };
  const { materials, actions } = src;
  if (isRecord(materials)) for (const m of MATERIALS) copy(into.materials[m], materials[m], MATERIAL_DIALS);
  if (isRecord(actions)) for (const a of ACTIONS) copy(into.actions[a], actions[a], ACTION_DIALS);
  return applied;
}

/**
 * Only dials that differ from the defaults are stored, so improving a default later still reaches
 * players who never touched that dial.
 */
function changedDials(design: SoundDesign): Partial<Record<string, Record<string, Record<string, number>>>> {
  const pick = <K extends string>(cur: Record<K, number>, def: Record<K, number>, keys: readonly K[]) => {
    const out: Record<string, number> = {};
    for (const k of keys) if (cur[k] !== def[k]) out[k] = cur[k];
    return out;
  };
  const materials: Record<string, Record<string, number>> = {};
  for (const m of MATERIALS) {
    const d = pick(design.materials[m], DEFAULT_DESIGN.materials[m], MATERIAL_DIALS);
    if (Object.keys(d).length) materials[m] = d;
  }
  const actions: Record<string, Record<string, number>> = {};
  for (const a of ACTIONS) {
    const d = pick(design.actions[a], DEFAULT_DESIGN.actions[a], ACTION_DIALS);
    if (Object.keys(d).length) actions[a] = d;
  }
  return { materials, actions };
}

export function loadSettings(): SoundSettings {
  const settings: SoundSettings = { volume: 80, muted: false, design: defaultDesign() };
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (raw) {
      const saved: unknown = JSON.parse(raw);
      if (isRecord(saved)) {
        if (typeof saved.volume === 'number' && Number.isFinite(saved.volume)) settings.volume = clampDial(saved.volume);
        settings.muted = saved.muted === true;
        mergeDesign(settings.design, saved.design);
      }
    } else {
      settings.muted = localStorage.getItem(LEGACY_MUTE_KEY) === '1';
    }
  } catch {
    // Storage unavailable (private mode, blocked site data) or corrupt: use the defaults.
  }
  return settings;
}

export function saveSettings(s: SoundSettings): void {
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify({ volume: s.volume, muted: s.muted, design: changedDials(s.design) }));
  } catch {
    // Not persisted; the settings still apply for this session.
  }
}

/** The full design as readable JSON, one material or action per line (the copy/paste format). */
export function exportDesign(design: SoundDesign): string {
  const line = (key: string, values: Record<string, number>, keys: readonly string[]) =>
    `    "${key}": { ${keys.map((k) => `"${k}": ${values[k]}`).join(', ')} }`;
  const materials = MATERIALS.map((m) => line(m, design.materials[m], MATERIAL_DIALS));
  const actions = ACTIONS.map((a) => line(a, design.actions[a], ACTION_INFO[a].dials));
  return `{\n  "materials": {\n${materials.join(',\n')}\n  },\n  "actions": {\n${actions.join(',\n')}\n  }\n}\n`;
}
