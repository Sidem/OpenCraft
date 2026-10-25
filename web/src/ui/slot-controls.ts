// The controls bar along the bottom of the inventory screen: instead of a sentence listing every click, it shows
// only what the player can do right now, for the slot under the pointer and what they hold, as key caps and a
// small mouse with the button to use lit (the input legend console games use). `slotControls` holds the rules;
// the inventory screen (`inventory.ts`) says what is hovered and held. To teach a new click: a line in
// `slotControls`, with its keys from `KEYS`.

import './slot-controls.css';
import { h } from './dom';

/** What the inventory screen knows about the moment. */
export interface SlotContext {
  /** The hovered slot's kind, or null when the pointer is on no slot. */
  kind: 'hotbar' | 'pack' | 'gear' | 'hand' | 'box' | 'machine' | null;
  /** The hovered slot's item, 0 when it is empty. */
  item: number;
  /** The item held on the cursor, 0 for none. */
  held: number;
  /** What Shift-click does with the hovered stack ("Move to the box", "Wear it"…). */
  shift: string;
}

/** Keys and mouse buttons a control is pressed with. */
type Key = 'Shift' | 'Ctrl' | 'Left' | 'Right' | 'E';
interface Control {
  keys: Key[];
  label: string;
}

/** The controls that apply to `c`, most used first. */
export function slotControls(c: SlotContext): Control[] {
  if (c.kind === 'machine') {
    if (c.held) return [{ keys: ['Left'], label: 'Put in' }];
    return c.item ? [{ keys: ['Left'], label: 'Take' }] : [];
  }
  if (c.kind === 'hand') {
    if (c.held) return [{ keys: ['Left'], label: c.item ? 'Swap' : 'Put in the left hand' }];
    return c.item ? [{ keys: ['Left'], label: 'Pick up' }] : [];
  }
  if (c.kind === 'gear') {
    if (c.held) return [{ keys: ['Left'], label: c.item ? 'Swap' : 'Wear' }];
    return c.item ? [{ keys: ['Left'], label: 'Pick up' }, { keys: ['Shift', 'Left'], label: 'Take off' }] : [];
  }
  if (c.kind) {
    if (c.held) {
      const place = c.item === 0 ? 'Put down' : c.item === c.held ? 'Add to it' : 'Swap';
      return [{ keys: ['Left'], label: place }, { keys: ['Ctrl', 'Right'], label: 'Put one' }];
    }
    if (!c.item) return [];
    return [
      { keys: ['Left'], label: 'Pick up' },
      { keys: ['Right'], label: 'Take half' },
      { keys: ['Ctrl', 'Left'], label: 'Take one' },
      { keys: ['Shift', 'Left'], label: c.shift },
      { keys: ['Shift', 'Right'], label: 'Move all like it' },
    ];
  }
  if (c.held) return [{ keys: ['Shift', 'Right'], label: 'Throw one (outside)' }];
  return [{ keys: ['Left'], label: 'Pick up a stack' }, { keys: ['Shift', 'Left'], label: 'Move a stack' }];
}

export class SlotControls {
  readonly el = h('div', 'sc');
  private readonly list = h('div', 'sc-list');
  private drawn = '';

  constructor() {
    const close = h('span', 'sc-item sc-close');
    close.append(keyView('E'), 'Close');
    this.el.append(this.list, close);
  }

  /** Shows the controls for `c`; redraws only when they change. */
  show(c: SlotContext): void {
    const controls = slotControls(c);
    const key = JSON.stringify(controls);
    if (key === this.drawn) return;
    this.drawn = key;
    this.list.replaceChildren(
      ...controls.map((ctl) => {
        const item = h('span', 'sc-item');
        ctl.keys.forEach((k, i) => item.append(...(i > 0 ? [h('span', 'sc-plus', '+')] : []), keyView(k)));
        item.append(ctl.label);
        return item;
      }),
    );
  }
}

/** A key cap, or a mouse with its left or right button lit. */
function keyView(k: Key): HTMLElement {
  if (k !== 'Left' && k !== 'Right') return h('kbd', 'sc-cap', k);
  const mouse = h('span', `sc-mouse lit-${k.toLowerCase()}`);
  mouse.title = `${k} mouse button`;
  mouse.append(h('span', 'sc-btn'), h('span', 'sc-btn'));
  return mouse;
}
