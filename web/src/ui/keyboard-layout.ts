// What is printed on each key, for the player's keyboard layout. The game reads keys by their place (`KeyboardEvent.code`:
// `KeyW` is wherever W sits on a US keyboard), so WASD stays a cluster on every layout, but the caps differ: on a French
// keyboard that place says Z. `KeyLabels.label(code)` answers with, in order: the layout the player chose; the browser's
// own layout map (`navigator.keyboard.getLayoutMap`, Chromium only, and sometimes empty); labels learned from keys pressed
// (`e.key`), which also tell which built-in layout it is; and QWERTY. Digits always read as their numeral, the named keys as
// words. Pure apart from `start()`; tested in web/test/controls.test.mjs. To add a layout: a row in `PRESETS`.

export type PresetId = 'qwerty' | 'qwertz' | 'azerty' | 'dvorak' | 'colemak';
export type LayoutChoice = 'auto' | PresetId;
export type Source = 'chosen' | 'map' | 'learned' | 'assumed';

/** The places the three letter rows have, left to right; a preset gives one printed character for each. */
const ROW1 = ['KeyQ', 'KeyW', 'KeyE', 'KeyR', 'KeyT', 'KeyY', 'KeyU', 'KeyI', 'KeyO', 'KeyP', 'BracketLeft', 'BracketRight'];
const ROW2 = ['KeyA', 'KeyS', 'KeyD', 'KeyF', 'KeyG', 'KeyH', 'KeyJ', 'KeyK', 'KeyL', 'Semicolon', 'Quote'];
const ROW3 = ['KeyZ', 'KeyX', 'KeyC', 'KeyV', 'KeyB', 'KeyN', 'KeyM', 'Comma', 'Period', 'Slash'];

interface Preset {
  name: string;
  row1: string;
  row2: string;
  row3: string;
  backquote: string;
  /** The places left of the letters that these keep from QWERTY (others are left blank). */
  keepsPunctuation?: boolean;
}

export const PRESETS: Record<PresetId, Preset> = {
  qwerty: { name: 'QWERTY', row1: 'qwertyuiop[]', row2: "asdfghjkl;'", row3: 'zxcvbnm,./', backquote: '`', keepsPunctuation: true },
  qwertz: { name: 'QWERTZ (German)', row1: 'qwertzuiopü+', row2: 'asdfghjklöä', row3: 'yxcvbnm,.-', backquote: '^' },
  azerty: { name: 'AZERTY (French)', row1: 'azertyuiop^$', row2: 'qsdfghjklmù', row3: 'wxcvbn,;:!', backquote: '²' },
  dvorak: { name: 'Dvorak', row1: "',.pyfgcrl/=", row2: 'aoeuidhtns-', row3: ';qjkxbmwvz', backquote: '`' },
  colemak: { name: 'Colemak', row1: 'qwfpgjluy;[]', row2: "arstdhneio'", row3: 'zxcvbkm,./', backquote: '`' },
};

export const CHOICES: { id: LayoutChoice; name: string }[] = [
  { id: 'auto', name: 'Automatic' },
  ...(Object.keys(PRESETS) as PresetId[]).map((id) => ({ id, name: PRESETS[id].name })),
];

const QWERTY_PUNCTUATION: Record<string, string> = { Minus: '-', Equal: '=', Backslash: '\\' };

const NAMED: Record<string, string> = {
  Space: 'Space', Tab: 'Tab', Enter: 'Enter', Escape: 'Esc', Home: 'Home', Backspace: '⌫', CapsLock: 'Caps',
  ShiftLeft: 'Shift', ShiftRight: 'Shift', ControlLeft: 'Ctrl', ControlRight: 'Ctrl', AltLeft: 'Alt', AltRight: 'Alt',
  MetaLeft: 'Win', MetaRight: 'Win', ContextMenu: 'Menu', ArrowUp: '↑', ArrowDown: '↓', ArrowLeft: '←', ArrowRight: '→',
};
for (let i = 1; i <= 12; i++) NAMED[`F${i}`] = `F${i}`;

/** Places whose printed character depends on the layout. */
const PRINTED = /^(Key[A-Z]|Backquote|Minus|Equal|Bracket(Left|Right)|Backslash|Semicolon|Quote|Comma|Period|Slash)$/;

/** A key's label that no layout changes: a word, or a digit's numeral. */
function fixedLabel(code: string): string | undefined {
  return NAMED[code] ?? /^Digit(\d)$/.exec(code)?.[1];
}

const cap = (text: string): string => {
  const upper = text.toUpperCase();
  return upper.length === text.length ? upper : text;
};

/** What `id` prints at `code`, or undefined for a place it does not describe. */
export function presetLabel(id: PresetId, code: string): string | undefined {
  const p = PRESETS[id];
  const rows: [string[], string][] = [[ROW1, p.row1], [ROW2, p.row2], [ROW3, p.row3]];
  for (const [places, printed] of rows) {
    const at = places.indexOf(code);
    if (at >= 0) return cap([...printed][at]);
  }
  if (code === 'Backquote') return p.backquote;
  return p.keepsPunctuation ? QWERTY_PUNCTUATION[code] : undefined;
}

/** The preset that agrees with every label learned so far (QWERTY when several do, or none). */
export function inferPreset(learned: Readonly<Record<string, string>>): PresetId {
  const ids = Object.keys(PRESETS) as PresetId[];
  const fits = ids.filter((id) => Object.entries(learned).every(([code, label]) => presetLabel(id, code) === undefined || presetLabel(id, code) === label));
  return fits[0] ?? 'qwerty';
}

/** The parts of a keydown the labels learn from. */
export interface LearnEvent {
  code: string;
  key: string;
  shiftKey?: boolean;
  altKey?: boolean;
  ctrlKey?: boolean;
  metaKey?: boolean;
}

export interface Store {
  get(key: string): string | null;
  set(key: string, value: string): void;
}

const LEARNED_KEY = 'opencraft.keyLabels';
const CHOICE_KEY = 'opencraft.keyboardLayout';

function browserStore(): Store | null {
  try {
    const ls = globalThis.localStorage;
    return ls ? { get: (k) => ls.getItem(k), set: (k, v) => ls.setItem(k, v) } : null;
  } catch {
    return null;
  }
}

/** What `navigator.keyboard` may offer; each part can be missing. */
interface KeyboardApi {
  getLayoutMap?: () => Promise<Iterable<[string, string]>>;
  addEventListener?: (type: string, listener: () => void) => void;
}

export class KeyLabels {
  private map: Record<string, string> | null = null;
  private learned: Record<string, string> = {};
  private picked: LayoutChoice = 'auto';
  private readonly listeners = new Set<() => void>();
  private readonly store: Store | null;

  constructor(store: Store | null = browserStore()) {
    this.store = store;
    try {
      const saved: unknown = JSON.parse(store?.get(LEARNED_KEY) ?? 'null');
      if (typeof saved === 'object' && saved !== null) {
        for (const [code, label] of Object.entries(saved)) if (PRINTED.test(code) && typeof label === 'string' && label) this.learned[code] = label;
      }
    } catch {
      // Unreadable: start with nothing learned.
    }
    const choice = store?.get(CHOICE_KEY);
    if (CHOICES.some((c) => c.id === choice)) this.picked = choice as LayoutChoice;
  }

  /** Hooks the browser: reads its layout map (and its changes) and learns from every key pressed. Call once. */
  start(): void {
    if (typeof window === 'undefined') return;
    window.addEventListener('keydown', (e) => this.learn(e), true);
    const keyboard = (navigator as unknown as { keyboard?: KeyboardApi }).keyboard;
    // Browsers differ in what `navigator.keyboard` has (Firefox and Safari: nothing; one Chromium: no events), so ask for each part.
    const load = () => {
      if (typeof keyboard?.getLayoutMap !== 'function') return;
      keyboard.getLayoutMap().then((m) => this.setMap(m), () => undefined);
    };
    load();
    if (typeof keyboard?.addEventListener === 'function') keyboard.addEventListener('layoutchange', load);
  }

  get choice(): LayoutChoice {
    return this.picked;
  }

  setChoice(choice: LayoutChoice): void {
    this.picked = choice;
    this.store?.set(CHOICE_KEY, choice);
    this.notify();
  }

  /** The browser's layout map; an empty one (some browsers answer with that) counts as none. */
  setMap(entries: Iterable<[string, string]> | null): void {
    const map: Record<string, string> = {};
    for (const [code, value] of entries ?? []) if (value) map[code] = cap(value);
    this.map = Object.keys(map).length > 0 ? map : null;
    this.notify();
  }

  /** What is printed on the key at `code`. */
  label(code: string): string {
    const fixed = fixedLabel(code);
    if (fixed) return fixed;
    if (this.picked !== 'auto') return presetLabel(this.picked, code) ?? '';
    return this.map?.[code] ?? this.learned[code] ?? presetLabel(inferPreset(this.learned), code) ?? '';
  }

  /** Where the labels come from, for the page to say. */
  get source(): Source {
    if (this.picked !== 'auto') return 'chosen';
    if (this.map) return 'map';
    return Object.keys(this.learned).length > 0 ? 'learned' : 'assumed';
  }

  /** The built-in layout the labels look like (a guess, from the keys learned), for the page to say. */
  get looksLike(): PresetId {
    return this.picked === 'auto' ? inferPreset(this.learned) : this.picked;
  }

  /** Learns what a plain key press printed; true when that changed a label. */
  learn(e: LearnEvent): boolean {
    if (e.key.length !== 1 || e.key === ' ' || e.shiftKey || e.altKey || e.ctrlKey || e.metaKey || !PRINTED.test(e.code)) return false;
    const label = cap(e.key);
    if (this.learned[e.code] === label) return false;
    this.learned[e.code] = label;
    this.store?.set(LEARNED_KEY, JSON.stringify(this.learned));
    this.notify();
    return true;
  }

  subscribe(fn: () => void): void {
    this.listeners.add(fn);
  }

  private notify(): void {
    for (const fn of this.listeners) fn();
  }
}

/** The page's labels (one `game` per page, so one set of labels). */
export const keyLabels = new KeyLabels();
