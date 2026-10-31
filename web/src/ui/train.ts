// Train panel: opens when the player right-clicks a train car (the engine asks with `take_panel_request` and the car's
// cargo address, api/trains.rs). It shows inside the inventory screen beside the open wagon's slots: the wagon is a
// box to that screen (`box_slots`, `click_box` at its address), so items move in and out like a chest. The panel says
// what the train is doing, has a bar per wagon (click one to open it) and lists the schedule, whose stops can be taken
// off. Everything is read from the engine each frame; buttons queue engine actions.

import './train.css';
import type { Game } from '../wasm/engine.js';
import { button, h } from './dom';
import type { Beside, InventoryPanel } from './inventory';

const ADD_STOP = 'To add a stop: hold a locomotive, click a rail node beside the train, then click a dock.';

export class TrainPanel implements Beside {
  readonly el = h('section', 'tp-panel');
  title = 'Train';
  private readonly state = h('p', 'tp-state');
  private readonly cars = h('div', 'tp-cars');
  private readonly stops = h('ol', 'tp-stops');
  private readonly clear: HTMLButtonElement;
  /** The cargo address of the car shown: [train, the engine's off-world y, car]. */
  private addr: [number, number, number] = [0, 0, 0];
  private drawn = '';
  private shown = false;
  /** The inventory screen's body while the panel is in it (it is already detached when `closed` runs). */
  private host: Element | null = null;

  constructor(
    private readonly game: Game,
    private readonly inventory: InventoryPanel,
  ) {
    this.clear = button('secondary-btn tp-clear', 'Clear schedule', () => this.game.clear_stops(...this.addr));
    this.el.append(
      this.state,
      h('h3', '', 'Wagons'),
      this.cars,
      h('h3', '', 'Schedule'),
      this.stops,
      this.clear,
      h('p', 'tp-hint', ADD_STOP),
    );
  }

  /** Opens the panel if `x, y, z` is a train car's cargo address (a locomotive opens its first wagon); false if not. */
  open(x: number, y: number, z: number): boolean {
    const data = this.game.train_panel(x, y, z);
    if (data.length === 0) return false;
    this.addr = [x, y, z];
    this.shown = true;
    this.show(z === 0 ? Math.min(1, data[0]) : z);
    return true;
  }

  get isOpen(): boolean {
    return this.shown;
  }

  /** Shows wagon `car` (0: none, for a locomotive without wagons) in the screen's box slots. */
  private show(car: number): void {
    const a = this.game.train_car(...this.addr, car);
    this.addr = [a[0], a[1], a[2]];
    this.drawn = '';
    this.inventory.open(car > 0 ? this.addr : null, this);
    this.host = this.el.parentElement;
    this.host?.classList.add('tp-host');
    this.boxHeading('Wagon ' + car);
  }

  /** Names the screen's box slots (the inventory screen calls them a storage box). */
  private boxHeading(text: string): void {
    const head = this.host?.querySelector('.inv-box > .inv-box-head h3');
    if (head) head.textContent = text;
  }

  closed(): void {
    this.boxHeading('Storage box');
    this.host?.classList.remove('tp-host');
    this.host = null;
    this.shown = false;
  }

  shiftSlot(): boolean {
    return false; // with a wagon open, Shift-click stores into it as into a box
  }

  wants(): boolean {
    return false;
  }

  update(): boolean {
    const g = this.game;
    const data = g.train_panel(...this.addr);
    if (data.length === 0) return false;
    const state = g.train_state(...this.addr);
    const key = `${data.join(',')}|${state}`;
    if (key === this.drawn) return true;
    this.drawn = key;
    const [cars, next, count] = data;
    this.state.textContent = state;

    const first = g.train_car(...this.addr, 1);
    const per = cars > 0 ? g.box_slots(first[0], first[1], first[2]).length / 2 : 0;
    this.cars.replaceChildren();
    if (cars === 0) this.cars.append(h('p', 'tp-hint', 'No wagons yet: hold a wagon and click the track just behind the train.'));
    for (let c = 1; c <= cars; c++) {
      const [used, items] = data.slice(3 + count * 3 + (c - 1) * 2);
      const b = button('tp-car', '', () => this.show(c));
      b.classList.toggle('open', c === this.addr[2]);
      const bar = h('span', 'tp-gauge');
      const fill = h('span', 'tp-fill');
      fill.style.width = `${per > 0 ? (100 * used) / per : 0}%`;
      bar.append(fill);
      b.append(h('span', 'tp-name', `Wagon ${c}`), bar, h('span', 'tp-count', `${used} of ${per} slots · ${items} items`));
      this.cars.append(b);
    }

    this.stops.replaceChildren();
    if (count === 0) this.stops.append(h('li', 'tp-hint', 'None: it stops at every dock it reaches.'));
    for (let i = 0; i < count; i++) {
      const [x, y, z] = data.slice(3 + i * 3, 6 + i * 3);
      const li = h('li', i === next ? 'next' : '');
      li.append(
        h('span', 'tp-stop', `${g.machine_name(x, y, z) || 'A dock that is gone'} at ${x}, ${z}`),
        h('span', 'tp-next', i === next ? 'next' : ''),
        button('close-btn tp-drop', '×', () => this.game.drop_stop(...this.addr, i)),
      );
      this.stops.append(li);
    }
    this.clear.classList.toggle('hidden', count === 0);
    return true;
  }
}
