// Where every key sits on the Controls page's keyboard picture, in key widths (1 = a letter key): a full-size ANSI board
// without the numeric pad. Pure data, so a test can check that every key the game reads has a cap (keyboard-view.ts draws it).
// Places are `KeyboardEvent.code` values. To add a key the picture lacks: a row below.

export interface Place {
  code: string;
  x: number;
  y: number;
  w: number;
}

export const KB_WIDTH = 18.25;
export const KB_HEIGHT = 6.5;

/** A row of keys from `x`, each `[code, width]`. */
function row(y: number, x: number, keys: [string, number?][]): Place[] {
  const out: Place[] = [];
  for (const [code, w = 1] of keys) {
    out.push({ code, x, y, w });
    x += w;
  }
  return out;
}

const letters = (codes: string): [string][] => [...codes].map((c) => [`Key${c}`]);
const digits = [1, 2, 3, 4, 5, 6, 7, 8, 9, 0].map((d): [string] => [`Digit${d}`]);

export const KEY_PLACES: readonly Place[] = [
  ...row(0, 0, [['Escape']]),
  ...row(0, 2, [['F1'], ['F2'], ['F3'], ['F4']]),
  ...row(0, 6.5, [['F5'], ['F6'], ['F7'], ['F8']]),
  ...row(0, 11, [['F9'], ['F10'], ['F11'], ['F12']]),
  ...row(1.5, 0, [['Backquote'], ...digits, ['Minus'], ['Equal'], ['Backspace', 2]]),
  ...row(2.5, 0, [['Tab', 1.5], ...letters('QWERTYUIOP'), ['BracketLeft'], ['BracketRight'], ['Backslash', 1.5]]),
  ...row(3.5, 0, [['CapsLock', 1.75], ...letters('ASDFGHJKL'), ['Semicolon'], ['Quote'], ['Enter', 2.25]]),
  ...row(4.5, 0, [['ShiftLeft', 2.25], ...letters('ZXCVBNM'), ['Comma'], ['Period'], ['Slash'], ['ShiftRight', 2.75]]),
  ...row(5.5, 0, [
    ['ControlLeft', 1.25], ['MetaLeft', 1.25], ['AltLeft', 1.25], ['Space', 6.25],
    ['AltRight', 1.25], ['MetaRight', 1.25], ['ContextMenu', 1.25], ['ControlRight', 1.25],
  ]),
  ...row(1.5, 15.25, [['Home']]),
  ...row(4.5, 16.25, [['ArrowUp']]),
  ...row(5.5, 15.25, [['ArrowLeft'], ['ArrowDown'], ['ArrowRight']]),
];
