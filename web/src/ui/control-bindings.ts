// Every control the player can use, as data: the Controls page draws its keyboard and its list from this one table
// (ui/controls.ts), so they cannot disagree. A key is a `KeyboardEvent.code` (a place on the keyboard, as input.ts reads it:
// `KeyW` is where W sits on a US keyboard), and `{KeyW}` in a label or text is replaced by what is printed on that place for
// the player's layout (ui/keyboard-layout.ts). `web/test/controls.test.mjs` fails when this table and input.ts disagree.
// To add a control: a row below (and its key in input.ts); a new key place also needs a cap in keyboard-view.ts.

export type Category = 'move' | 'build' | 'screens' | 'plan' | 'camera' | 'other';

/** Category colours: started from the Okabe-Ito set and tuned (a search in the scratch, kept near its hues) until every pair
 *  stays apart for the common kinds of colour blindness; `web/test/controls.test.mjs` fails if an edit here loses that. */
export const CATEGORIES: readonly { id: Category; title: string; colour: string }[] = [
  { id: 'move', title: 'Getting around', colour: '#5dc3ff' },
  { id: 'build', title: 'Building', colour: '#e69f00' },
  { id: 'screens', title: 'Screens', colour: '#188e66' },
  { id: 'plan', title: 'Planning and helpers', colour: '#987099' },
  { id: 'camera', title: 'Camera', colour: '#ffecbb' },
  { id: 'other', title: 'Other', colour: '#e7fafe' },
];

export type MouseInput = 'move' | 'left' | 'right' | 'wheel';

export interface Binding {
  cat: Category;
  /** The key places this control uses (all of them light up, and hovering one lists this control). */
  keys: string[];
  /** The mouse inputs it uses. */
  mouse?: MouseInput[];
  /** What the player presses, in words; `{Code}` becomes that key's printed label. */
  label: string;
  /** What it does; `{Code}` works here too. */
  text: string;
}

const key = (cat: Category, keys: string[], label: string, text: string): Binding => ({ cat, keys, label, text });
const mouse = (cat: Category, inputs: MouseInput[], label: string, text: string, keys: string[] = []): Binding => ({ cat, keys, mouse: inputs, label, text });

const WASD = ['KeyW', 'KeyA', 'KeyS', 'KeyD'];
const ARROWS = ['ArrowUp', 'ArrowLeft', 'ArrowDown', 'ArrowRight'];
const SHIFTS = ['ShiftLeft', 'ShiftRight'];
const CTRLS = ['ControlLeft', 'ControlRight'];
const DIGITS = ['Digit1', 'Digit2', 'Digit3', 'Digit4', 'Digit5', 'Digit6', 'Digit7', 'Digit8', 'Digit9'];

/** In the order shown. A key takes the colour of the first row that uses it. */
export const BINDINGS: readonly Binding[] = [
  mouse('move', ['move'], 'Mouse', 'Look around'),
  key('move', [...WASD, ...ARROWS], '{KeyW} {KeyA} {KeyS} {KeyD}', 'Walk (the arrow keys work too)'),
  key('move', ['Space'], '{Space}', 'Jump; in water, swim up'),
  key('move', SHIFTS, '{ShiftLeft}', 'Sprint'),
  key('move', ['KeyC', ...CTRLS], '{KeyC} / {ControlLeft}', "Crouch (you won't walk off edges)"),
  key('move', ['KeyF'], '{KeyF}', 'Fly on or off, in creative worlds ({Space} and {KeyC} rise and sink)'),

  mouse('build', ['left'], 'Hold left mouse', 'Mine the block you point at'),
  mouse('build', ['right'], 'Right mouse', 'Place the selected block (belts run the way you face; drag to lay a line)'),
  mouse('build', ['right'], 'Right mouse on a box or machine', 'Open it (hold {KeyC} to place against it instead)'),
  key('build', ['KeyR'], '{KeyR}', 'Turn the belt or machine you point at'),
  key('build', [...SHIFTS, 'KeyR'], '{ShiftLeft} + {KeyR}', 'Bend the belt you point at to take items in from its left or right, so a machine beside it can feed it (with nothing pointed at: the next belt you lay)'),
  key('build', ['KeyI'], '{KeyI}', 'Pick up the item on a belt you point at (or the one you clicked to select)'),
  mouse('build', ['wheel'], '{Digit1}–{Digit9} / wheel', 'Pick a hotbar slot', DIGITS),
  key('build', ['KeyJ'], '{KeyJ}', 'Swap the selected slot with your left hand (a torch there still lights the way; a tool there mines too)'),
  key('build', ['KeyQ'], '{KeyQ}', 'Drop one item'),

  key('screens', ['KeyE'], '{KeyE}', 'Inventory and build menu'),
  key('screens', ['KeyT'], '{KeyT}', 'Research: choose what labs work on'),
  key('screens', ['Slash', 'Backquote'], '{Slash} or {Backquote}', 'Command line for testing: weather, time of day, teleport, items (type help)'),
  key('screens', ['KeyM'], '{KeyM}', "Map: where you've been, your pins, where each ore lies"),
  key('screens', ['KeyN'], '{KeyN}', 'Minimap on or off'),
  key('screens', ['KeyP'], '{KeyP}', 'Power and production graphs'),
  key('screens', ['KeyL'], '{KeyL}', 'Blueprints'),
  key('screens', ['Tab'], 'Hold {Tab}', 'Who is playing with you'),

  key('plan', ['KeyB'], '{KeyB}', 'Ghost mode: right-click plants a free ghost to build later'),
  key('plan', ['KeyZ', 'Enter'], '{KeyZ}, {Enter}', 'In ghost mode: mark a box, then copy its machines as a blueprint'),
  key('plan', ['KeyX'], '{KeyX}', 'Build the ghost you aim at, taking its item from anywhere in your pack (in strategy view: stop walking)'),
  key('plan', ['KeyY'], '{KeyY}', 'Personal drone: fetch, put away, store, or fetch the parts and craft'),
  key('plan', [...SHIFTS, 'KeyY'], '{ShiftLeft} + {KeyY}', 'Your personal drone fetches a stack more of what you hold'),
  key('plan', ['KeyU'], '{KeyU}', 'Point the scanner at the next deposit'),

  key('camera', ['KeyV'], '{KeyV}', 'First person, third person or strategy view'),
  key('camera', ['KeyG', 'Home'], '{KeyG} / {Home}', 'Free or follow camera; {Home} returns to your character'),
  mouse('camera', ['wheel', 'right'], 'Strategy view', '{KeyW}{KeyA}{KeyS}{KeyD} pans, wheel zooms, right-click walks there, {KeyX} stops', ['KeyX']),
  mouse('camera', ['right'], '{ControlLeft} + right mouse', 'Use or place in strategy view (within reach)', CTRLS),

  key('other', ['KeyH'], '{KeyH}', 'Skip the tip on screen'),
  key('other', ['KeyK'], '{KeyK}', 'Mute or unmute all sound'),
  key('other', ['KeyO'], '{KeyO}', 'Sound designer'),
  key('other', ['F3'], '{F3}', 'Debug stats'),
  key('other', ['Escape'], '{Escape}', 'Close a screen, free the mouse, then this menu; on the menu, play on'),
];

const TOKEN = /\{(\w+)\}/g;

/** The key places named in `{...}` in `text`. */
export function tokensOf(text: string): string[] {
  return [...text.matchAll(TOKEN)].map((m) => m[1]);
}

/** `text` with every `{Code}` replaced by `name(Code)`. */
export function fill(text: string, name: (code: string) => string): string {
  return text.replace(TOKEN, (_, code: string) => name(code));
}

/** Every key place the table uses, for the drift test and for lighting. */
export function boundKeys(): Set<string> {
  return new Set(BINDINGS.flatMap((b) => b.keys));
}

/** The place and colour a key takes: its first row's category. */
export function categoryOf(code: string): Category | null {
  return BINDINGS.find((b) => b.keys.includes(code))?.cat ?? null;
}
