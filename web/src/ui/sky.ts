// "Sky" on the menu's settings page: how the night sky is drawn (render/sky/settings.ts). It reuses the Comfort section's
// row styles (comfort.css). To add a control: a row in the constructor; weather settings will join here.

import { QUALITY_NAMES, type SkyQuality, type SkyStore } from '../render/sky/settings';
import { button, h } from './dom';

export class SkyPanel {
  readonly el = h('div', 'comfort');
  private readonly sync: (() => void)[] = [];

  constructor(sky: SkyStore) {
    this.el.append(
      this.choices('Night sky', [...QUALITY_NAMES], () => sky.settings.quality, (i) => sky.set('quality', i as SkyQuality)),
      h('p', 'comfort-help', 'Classic is the simple sky. Stars draws the real stars and the moon in its phases; adding the Milky Way costs a little more.'),
      this.choices('Constellation lines', ['Off', 'On'], () => Number(sky.settings.lines), (i) => sky.set('lines', i === 1)),
      this.choices('Twinkling', ['Off', 'On'], () => Number(sky.settings.twinkle), (i) => sky.set('twinkle', i === 1)),
      button('secondary-btn', 'Reset to defaults', () => sky.reset()),
    );
    sky.subscribe(() => this.sync.forEach((fn) => fn()));
  }

  private choices(label: string, names: string[], current: () => number, pick: (i: number) => void): HTMLElement {
    const row = h('div', 'comfort-row');
    const buttons = names.map((n, i) => button('secondary-btn', n, () => pick(i)));
    this.sync.push(() => buttons.forEach((b, i) => b.classList.toggle('active', i === current())));
    row.append(h('span', 'comfort-label', label), h('div', 'comfort-choices'));
    row.lastElementChild!.append(...buttons);
    return row;
  }
}
