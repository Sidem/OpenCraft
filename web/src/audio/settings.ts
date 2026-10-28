// The sound mix: a Volume dial (0 to 100, 50 = normal) per sound material and per action; the sounds themselves are
// files (effects.ts). `SoundDesign` is also the copy/paste format of the sound designer, so a mix tuned in the browser
// can be pasted back here as the new DEFAULT_DESIGN. Older saved or pasted designs had more dials; only Volume is read.

/** Sound materials in engine order; must match `block::sound` in the engine. */
export const MATERIALS = ['stone', 'dirt', 'grass', 'sand', 'wood', 'leaves', 'metal'] as const;
export type MaterialName = (typeof MATERIALS)[number];

export const ACTIONS = ['dig', 'break', 'place', 'step', 'land', 'pickup', 'drop', 'click', 'scan', 'splash', 'powerdown', 'powerup', 'launch', 'podup', 'poddown', 'denied'] as const;
export type ActionName = (typeof ACTIONS)[number];

/** The dials, the same for materials and actions. */
const DIALS = ['volume'] as const;
export type Params = Record<(typeof DIALS)[number], number>;

export interface SoundDesign {
  materials: Record<MaterialName, Params>;
  actions: Record<ActionName, Params>;
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

export const MATERIAL_VOLUME_INFO: DialInfo = {
  label: 'Volume',
  low: 'quiet',
  high: 'loud',
  help: 'How loud this material is compared to the others. Its default is the level it was mixed at.',
};

export const ACTION_VOLUME_INFO: DialInfo = { label: 'Volume', low: 'quiet', high: 'loud', help: 'How loud this action is. 50 is normal.' };

export const ACTION_INFO: Record<ActionName, { label: string; help: string; material: boolean }> = {
  dig: { label: 'Mining', help: 'Repeated hits while you hold the mouse on a block.', material: true },
  break: { label: 'Break', help: 'The block crumbling when mining finishes.', material: true },
  place: { label: 'Place', help: 'Setting a block down.', material: true },
  step: { label: 'Footstep', help: 'Walking. Sneaking is quieter, sprinting louder.', material: true },
  land: { label: 'Landing', help: 'Hitting the ground after a jump or fall. Longer falls are louder.', material: true },
  pickup: { label: 'Item pickup', help: 'The pop when an item flies into your hotbar. The same for every material.', material: false },
  drop: { label: 'Throw item', help: 'The whoosh when you drop an item with Q. The same for every material.', material: false },
  click: { label: 'Hotbar click', help: 'The tick when you change hotbar slots.', material: false },
  scan: { label: 'Scanner', help: 'The ping when the scanner or core drill reports what it found.', material: false },
  splash: { label: 'Splash', help: 'Falling or jumping into water. Faster falls are louder. The same for every material.', material: false },
  powerdown: { label: 'Power down', help: 'The hover pack losing its charging connection: an electric whine falling away.', material: false },
  powerup: { label: 'Power up', help: 'The hover pack starting to charge: an electric whine rising to a thump.', material: false },
  launch: { label: 'Rocket launch', help: 'A rocket leaving its launch site: a heavy blast, heard from far away.', material: false },
  podup: { label: 'Pod launch', help: 'A drop pod fired from its launcher: a rushing whoosh.', material: false },
  poddown: { label: 'Pod landing', help: 'A drop pod coming down on its pad: a retro-burn hiss, then a heavy thud.', material: false },
  denied: {
    label: 'Cannot place',
    help: 'A low double knock when a block or machine cannot go where you aim: you are standing in the way, or something already is.',
    material: false,
  },
};

// Every entry gets its own object: structuredClone keeps shared references shared, so aliasing one
// object here would make a dial move several sounds at once.
const vol = (volume: number): Params => ({ volume });

export const DEFAULT_DESIGN: SoundDesign = {
  // The material levels the hand-tuned mix had; the effect files are levelled to match it (effect-levels.ts).
  materials: { stone: vol(15), dirt: vol(15), grass: vol(10), sand: vol(11), wood: vol(21), leaves: vol(10), metal: vol(14) },
  actions: {
    dig: vol(50),
    break: vol(50),
    place: vol(50),
    step: vol(50),
    land: vol(50),
    pickup: vol(50),
    drop: vol(50),
    click: vol(50),
    scan: vol(50),
    splash: vol(50),
    powerdown: vol(50),
    powerup: vol(50),
    launch: vol(50),
    podup: vol(50),
    poddown: vol(50),
    denied: vol(50),
  },
};

export const defaultDesign = (): SoundDesign => structuredClone(DEFAULT_DESIGN);

export interface SoundSettings {
  /** Master volume, 0–100. */
  volume: number;
  muted: boolean;
  /** Sound effects are silent while the tab or window is not in focus. */
  muteInBackground: boolean;
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
  if (isRecord(materials)) for (const m of MATERIALS) copy(into.materials[m], materials[m], DIALS);
  if (isRecord(actions)) for (const a of ACTIONS) copy(into.actions[a], actions[a], DIALS);
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
    const d = pick(design.materials[m], DEFAULT_DESIGN.materials[m], DIALS);
    if (Object.keys(d).length) materials[m] = d;
  }
  const actions: Record<string, Record<string, number>> = {};
  for (const a of ACTIONS) {
    const d = pick(design.actions[a], DEFAULT_DESIGN.actions[a], DIALS);
    if (Object.keys(d).length) actions[a] = d;
  }
  return { materials, actions };
}

export function loadSettings(): SoundSettings {
  const settings: SoundSettings = { volume: 80, muted: false, muteInBackground: false, design: defaultDesign() };
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (raw) {
      const saved: unknown = JSON.parse(raw);
      if (isRecord(saved)) {
        if (typeof saved.volume === 'number' && Number.isFinite(saved.volume)) settings.volume = clampDial(saved.volume);
        settings.muted = saved.muted === true;
        settings.muteInBackground = saved.muteInBackground === true;
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
    const { volume, muted, muteInBackground } = s;
    localStorage.setItem(STORAGE_KEY, JSON.stringify({ volume, muted, muteInBackground, design: changedDials(s.design) }));
  } catch {
    // Not persisted; the settings still apply for this session.
  }
}

/** The full design as readable JSON, one material or action per line (the copy/paste format). */
export function exportDesign(design: SoundDesign): string {
  const line = (key: string, values: Record<string, number>, keys: readonly string[]) =>
    `    "${key}": { ${keys.map((k) => `"${k}": ${values[k]}`).join(', ')} }`;
  const materials = MATERIALS.map((m) => line(m, design.materials[m], DIALS));
  const actions = ACTIONS.map((a) => line(a, design.actions[a], DIALS));
  return `{\n  "materials": {\n${materials.join(',\n')}\n  },\n  "actions": {\n${actions.join(',\n')}\n  }\n}\n`;
}
