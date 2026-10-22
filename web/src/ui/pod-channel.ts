// The pod launcher's and landing pad's part of the machine panel (ui/machine.ts): one button per channel, each a
// colour swatch and its name. A pod goes only to a pad of its launcher's colour. Read from the engine each frame
// (`machine_channel`, -1 for any other machine); a button queues `set_machine_channel`. Styles come from machine.css.
// A new channel needs no change here: the count, names and colours come from the engine.

import type { Game } from '../wasm/engine.js';
import { button, h } from './dom';

type Pos = [number, number, number];

export class ChannelSection {
  readonly el = h('section', 'mp-channel hidden');
  private readonly buttons: HTMLButtonElement[];
  private pos: Pos = [0, 0, 0];
  /** The channel last shown, or -1 for none. */
  private shown = -1;

  constructor(private readonly game: Game) {
    this.buttons = Array.from({ length: game.channel_count() }, (_, channel) => {
      const swatch = h('span', 'mp-swatch');
      swatch.style.background = `#${game.channel_colour(channel).toString(16).padStart(6, '0')}`;
      const b = button('secondary-btn mp-insert', '', () => this.choose(channel));
      b.append(swatch, game.channel_name(channel));
      return b;
    });
    const list = h('div', 'mp-inserts');
    list.append(...this.buttons);
    this.el.append(h('h3', '', 'Channel'), list, h('p', 'mp-note', 'Pods go to the nearest pad on the same channel.'));
  }

  /** Shows the channel choice for the launcher or pad at `pos`, or hides the section for any other machine. Returns
   * the channel it is on, or -1. */
  update(pos: Pos): number {
    this.pos = [...pos];
    const channel = this.game.machine_channel(...pos);
    if (channel !== this.shown) {
      this.shown = channel;
      this.el.classList.toggle('hidden', channel < 0);
      this.buttons.forEach((b, i) => b.classList.toggle('active', i === channel));
    }
    return channel;
  }

  /** The panel shows the new choice at once, before the next tick applies it. */
  private choose(channel: number): void {
    this.game.set_machine_channel(...this.pos, channel);
    this.buttons.forEach((b, i) => b.classList.toggle('active', i === channel));
  }
}
