// The equipment band of the inventory screen (GEAR-BODY): the gear slots stand in two rows either side of an outline of
// the player's robot seen from behind, upper-body gear in the top row and the rest below, each on the side where it
// shows on the avatar (engine `avatars/worn.rs`). A marker on the outline lights where something is worn, and pointing
// at a slot rings its marker. An empty slot shows a faint sketch of the gear it takes (`gear_slot_example`) instead of
// a written name; the name is in its tooltip. Slots come in the engine's order (`equipment::SLOT_NAMES`), the left hand
// last. To place a new slot: a row in PLACES and its area in gear-figure.css.

import './gear-figure.css';
import { h } from './dom';
import { iconEpoch } from './hud';
import { ICON_PX } from './slot-view';

/** Per engine slot (Back, Boots, Torso, Tool belt, Flight pack, Gloves, Head), then the left hand: its grid area and
 * where its marker sits on the 100×160 outline. */
const PLACES: [string, number, number][] = [
  ['back', 50, 60], ['boots', 41, 147], ['torso', 63, 44], ['belt', 50, 83],
  ['jet', 50, 42], ['gloves', 80, 98], ['head', 50, 16], ['hand', 20, 98],
];

const OUTLINE = [
  '<rect x="36" y="5" width="28" height="23" rx="7"/>', '<rect x="45" y="28" width="10" height="5"/>',
  '<rect x="30" y="33" width="40" height="44" rx="6"/>', '<rect x="32" y="78" width="36" height="10" rx="3"/>',
  '<rect x="14" y="36" width="12" height="56" rx="6"/>', '<rect x="74" y="36" width="12" height="56" rx="6"/>',
  '<circle cx="20" cy="98" r="6"/>', '<circle cx="80" cy="98" r="6"/>',
  '<rect x="34" y="90" width="13" height="52" rx="5"/>', '<rect x="53" y="90" width="13" height="52" rx="5"/>',
  '<rect x="32" y="142" width="17" height="10" rx="3"/>', '<rect x="51" y="142" width="17" height="10" rx="3"/>',
].join('');

export class GearFigure {
  readonly el = h('div', 'gear-figure');
  private readonly marks: SVGCircleElement[] = [];
  private readonly sketches: HTMLCanvasElement[] = [];
  private sketched = -1;

  /** `slots` are the slot elements in PLACES order; `examples` the item sketched in each while it is empty. */
  constructor(
    slots: HTMLElement[],
    private readonly examples: number[],
    private readonly icon: (item: number) => HTMLCanvasElement,
  ) {
    const ns = 'http://www.w3.org/2000/svg';
    const svg = document.createElementNS(ns, 'svg');
    svg.setAttribute('viewBox', '0 0 100 160');
    svg.classList.add('gear-figure-svg');
    svg.innerHTML = `<g class="gear-outline">${OUTLINE}</g>`;
    slots.forEach((slot, i) => {
      const [area, x, y] = PLACES[i];
      slot.style.gridArea = area;
      slot.classList.add('gear-slot');
      const sketch = h('canvas', 'gear-sketch');
      sketch.width = sketch.height = ICON_PX;
      slot.prepend(sketch);
      this.sketches.push(sketch);
      const mark = document.createElementNS(ns, 'circle');
      mark.setAttribute('cx', String(x));
      mark.setAttribute('cy', String(y));
      mark.setAttribute('r', '4.5');
      mark.classList.add('gear-mark');
      svg.append(mark);
      this.marks.push(mark);
      // Pointing at a slot points at where it shows.
      slot.addEventListener('mouseenter', () => mark.classList.add('hover'));
      slot.addEventListener('mouseleave', () => mark.classList.remove('hover'));
    });
    this.el.append(svg, ...slots);
  }

  /** Lights the markers of the slots that hold something (PLACES order); sketches the empty slots' gear once per icon
   * palette (icons are drawn lazily, so not in the constructor). */
  update(worn: boolean[]): void {
    worn.forEach((on, i) => this.marks[i]?.classList.toggle('on', on));
    if (this.sketched === iconEpoch.n) return;
    this.sketched = iconEpoch.n;
    this.sketches.forEach((c, i) => {
      const ctx = c.getContext('2d')!;
      ctx.clearRect(0, 0, ICON_PX, ICON_PX);
      ctx.drawImage(this.icon(this.examples[i]), 0, 0);
    });
  }
}
