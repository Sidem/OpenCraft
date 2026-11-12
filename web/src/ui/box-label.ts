// The storage box screen's label picker: a row of chips under the box's title, one per label the engine knows
// (`box_label_names`, `box_label_icons`: engine `factory/box_label.rs`). "Contents" makes the box's sides show what it
// holds most of; a category shows its icon. A click sends `label_box` (an action, so co-op players see the same
// label); the pressed chip is read back from the engine each frame. Hidden for a wagon, which has no label.

import './box-label.css';
import type { Game } from '../wasm/engine.js';
import { h } from './dom';

const CHIP_ICON_PX = 20;

export class BoxLabel {
  readonly el = h('div', 'box-label');
  private readonly chips: HTMLButtonElement[] = [];
  private box: [number, number, number] | null = null;
  private shown = -2;

  constructor(private readonly game: Game, icon: (item: number) => HTMLCanvasElement) {
    this.el.append(h('span', 'box-label-title', 'Label'));
    const icons = game.box_label_icons();
    game.box_label_names().split('\n').forEach((name, i) => {
      const chip = h('button', 'box-label-chip') as HTMLButtonElement;
      chip.type = 'button';
      chip.title = icons[i] ? `The box's sides show ${name.toLowerCase()}` : 'The box shows what it holds most of';
      if (icons[i]) {
        const c = h('canvas');
        c.width = c.height = CHIP_ICON_PX;
        c.getContext('2d')!.drawImage(icon(icons[i]), 0, 0, CHIP_ICON_PX, CHIP_ICON_PX);
        chip.append(c);
      }
      chip.append(name);
      chip.addEventListener('click', () => this.box && game.label_box(...this.box, i));
      this.chips.push(chip);
      this.el.append(chip);
    });
  }

  /** Shows the label of the box at `box` (null: none open). */
  open(box: [number, number, number] | null): void {
    this.box = box;
    this.shown = -2;
    this.update();
  }

  /** Marks the box's label pressed; cheap when it is unchanged. */
  update(): void {
    const label = this.box ? this.game.box_label(...this.box) : -1;
    if (label === this.shown) return;
    this.shown = label;
    this.el.classList.toggle('hidden', label < 0);
    this.chips.forEach((c, i) => c.setAttribute('aria-pressed', String(i === label)));
  }
}
