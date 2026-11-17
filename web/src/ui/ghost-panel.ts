// The ghost-mode controls panel: a small card on the right edge that says "you are in ghost mode" and shows the
// keys that work right now, as key caps and one or two words each (with the player's own keyboard layout:
// keyboard-layout.ts). B, the way out, is always the first row. The rows follow the blueprint keys' state
// (`ghost_stage`: 0 nothing marked, 1 one corner marked, 2 a box selected, 3 a blueprint in hand), so the panel
// always says what the next Z does. To add a control: a row in `ROWS` (and the key in `input.ts`).

import './ghost-panel.css';
import type { Game } from '../wasm/engine.js';
import { h } from './dom';
import { keyLabels } from './keyboard-layout';

/** A mouse button, or a key place (`KeyboardEvent.code`), printed with the player's layout. */
type Icon = { mouse: 'left' | 'right' } | { key: string };

interface Row {
  icons: Icon[];
  word: string;
  /** The longer answer, as a tooltip. */
  tip: string;
}

const RIGHT: Icon = { mouse: 'right' };
const LEFT: Icon = { mouse: 'left' };
const Z: Icon = { key: 'KeyZ' };
const R: Icon = { key: 'KeyR' };
const ENTER: Icon = { key: 'Enter' };

/** The way out, first in every stage. */
const EXIT: Row = { icons: [{ key: 'KeyB' }], word: 'Exit', tip: 'Leave ghost mode (your ghosts stay)' };

const PLAN: Row[] = [
  { icons: [RIGHT], word: 'Plant / remove', tip: 'Plant a ghost of the block or machine in hand, or remove the ghost you aim at' },
  { icons: [LEFT], word: 'Tear down', tip: 'Mark the aimed block for tear-down (drones break marked blocks); click again to unmark' },
  { icons: [R], word: 'Turn', tip: 'Turn what you will plant a quarter turn' },
];

/** Works whatever is in hand, so it is shown in the stages where you are not busy marking a box. */
const BUILD: Row = {
  icons: [{ key: 'KeyX' }],
  word: 'Build ghost',
  tip: 'Build the ghost you aim at now, taking its item from anywhere in your pack (selected or not)',
};

/** By `ghost_stage`. */
const ROWS: Row[][] = [
  [...PLAN, BUILD, { icons: [Z], word: 'Mark a box', tip: 'Aim at one corner of a group of machines and press Z, then at the opposite corner' }],
  [
    { icons: [Z], word: '2nd corner', tip: 'Aim at the opposite corner and press Z' },
    { icons: [Z], word: 'Cancel (aim at sky)', tip: 'Press Z while aiming at the sky to drop the corner' },
  ],
  [
    { icons: [ENTER], word: 'Copy', tip: 'Copy the machines in the gold box as a blueprint' },
    { icons: [Z], word: 'New box', tip: 'Aim at a block and press Z to start a different box' },
    { icons: [Z], word: 'Deselect (aim at sky)', tip: 'Press Z while aiming at the sky to clear the box' },
  ],
  [
    { icons: [RIGHT], word: 'Stamp', tip: 'Stamp the blueprint as ghosts where the green box stands' },
    { icons: [R], word: 'Turn', tip: 'Turn the blueprint a quarter turn' },
    BUILD,
    { icons: [Z], word: 'Put away', tip: 'Put the blueprint away' },
  ],
];

export class GhostPanel {
  private readonly el = h('div', 'ghost-panel hidden');
  private drawn = -1;

  constructor(private readonly game: Game) {
    document.getElementById('hud')!.append(this.el);
    keyLabels.subscribe(() => (this.drawn = -1)); // redrawn with the new caps
  }

  /** Call every frame: shown in ghost mode, redrawn only when the stage changes. */
  update(): void {
    const on = this.game.ghost_mode();
    this.el.classList.toggle('hidden', !on);
    if (!on) {
      this.drawn = -1;
      return;
    }
    const stage = this.game.ghost_stage();
    if (stage !== this.drawn) this.draw(stage);
  }

  private draw(stage: number): void {
    this.drawn = stage;
    const head = h('div', 'ghost-head');
    head.append(h('span', 'ghost-title', 'Ghost mode'));
    this.el.replaceChildren(head, this.row(EXIT, 'ghost-exit'));
    for (const row of ROWS[stage] ?? []) this.el.append(this.row(row));
  }

  private row(row: Row, extra = ''): HTMLElement {
    const el = h('div', `ghost-row ${extra}`.trim());
    el.title = row.tip;
    const icons = h('span', 'ghost-icons');
    for (const icon of row.icons) icons.append(this.icon(icon));
    el.append(icons, h('span', 'ghost-word', row.word));
    return el;
  }

  private icon(icon: Icon): HTMLElement {
    if ('key' in icon) return h('span', 'ghost-cap', keyLabels.label(icon.key));
    const mouse = h('span', `ghost-mouse lit-${icon.mouse}`);
    mouse.append(h('i', 'left'), h('i', 'right'));
    return mouse;
  }
}
