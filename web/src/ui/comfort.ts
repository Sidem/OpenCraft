// "Comfort" section of the pause menu: sliders and choices for the settings that help against motion
// sickness (comfort/settings.ts): field of view, mouse sensitivity, the movement vignette (ui/vignette.ts),
// third-person view and the crosshair, which this file also draws from the settings. While the section is open the menu
// steps aside (`previewing`: no dimming, panel to the side) so changes show on the game behind it.
// To add a control: a row in `SLIDERS` or a choice in the constructor.

import './comfort.css';
import { type ComfortSettings, type ComfortStore, CROSSHAIR_STYLES, RANGES } from '../comfort/settings';
import { button, h } from './dom';
import type { Vignette } from './vignette';

type NumberKey = keyof typeof RANGES;

interface Slider {
  key: Exclude<NumberKey, 'vignette'>;
  label: string;
  help: string;
  unit: string;
}

const SLIDERS: Slider[] = [
  { key: 'fov', label: 'Field of view', help: 'Try 60 to 90. Too wide stretches the edges; too narrow feels like a tunnel.', unit: '°' },
  { key: 'sensitivity', label: 'Mouse sensitivity', help: 'Slower turning is easier on the stomach.', unit: '%' },
  { key: 'thirdDistance', label: 'Camera distance', help: 'How far behind the player the camera sits in third person.', unit: ' blocks' },
  { key: 'crosshairSize', label: 'Crosshair size', help: 'A fixed point to rest your eyes on.', unit: ' px' },
  { key: 'crosshairThickness', label: 'Crosshair thickness', help: '', unit: ' px' },
  { key: 'crosshairOpacity', label: 'Crosshair opacity', help: '', unit: '%' },
];
const VIGNETTE_LABELS = ['Off', 'Low', 'Medium', 'High'];
const STYLE_LABELS = { cross: 'Cross', dot: 'Dot', both: 'Both' };

export class ComfortPanel {
  readonly el = h('details', 'comfort');
  private readonly sync: (() => void)[] = [];

  constructor(private readonly comfort: ComfortStore, vignette: Vignette, menu: HTMLElement) {
    const crosshair = document.getElementById('crosshair')!;
    this.el.append(h('summary', '', 'Comfort settings (motion sickness)'));
    this.el.append(h('p', 'comfort-note', 'Open this to see changes live. Everything is saved in this browser.'));

    const [view, third, aim] = [SLIDERS.slice(0, 2), SLIDERS[2], SLIDERS.slice(3)];
    this.el.append(
      ...view.map((s) => this.slider(s)),
      this.slider(third),
      h('p', 'comfort-help', 'Third person shows your character, a fixed shape on screen to anchor your eyes. The crosshair still marks where you aim.'),
      this.choices('Movement vignette', VIGNETTE_LABELS, () => comfort.settings.vignette, (i) => comfort.set('vignette', i)),
      h('p', 'comfort-help', 'Darkens the screen edges while you turn or move fast, which narrows the motion your eyes see.'),
      this.choices('Crosshair style', CROSSHAIR_STYLES.map((k) => STYLE_LABELS[k]), () => CROSSHAIR_STYLES.indexOf(comfort.settings.crosshairStyle), (i) =>
        comfort.set('crosshairStyle', CROSSHAIR_STYLES[i]),
      ),
      ...aim.map((s) => this.slider(s)),
      button('secondary-btn', 'Reset to defaults', () => comfort.reset()),
      h('p', 'comfort-help', "Also helps: a lit room, sitting an arm's length from the screen, a steady frame rate (F3 shows it), short sessions, and stopping at the first queasiness."),
    );

    comfort.subscribe(() => {
      this.applyCrosshair(crosshair, comfort.settings);
      for (const fn of this.sync) fn();
    });
    vignette.previewing = () => this.el.open && !menu.classList.contains('hidden');
    this.el.addEventListener('toggle', () => menu.classList.toggle('previewing', this.el.open));
  }

  private applyCrosshair(el: HTMLElement, s: ComfortSettings): void {
    el.style.setProperty('--ch-size', `${s.crosshairSize}px`);
    el.style.setProperty('--ch-thick', `${s.crosshairThickness}px`);
    el.style.setProperty('--ch-opacity', String(s.crosshairOpacity / 100));
    el.classList.remove('ch-cross', 'ch-dot', 'ch-both');
    el.classList.add(`ch-${s.crosshairStyle}`);
  }

  private slider(s: Slider): HTMLElement {
    const [min, max, step] = RANGES[s.key];
    const range = h('input');
    range.type = 'range';
    range.min = String(min);
    range.max = String(max);
    range.step = String(step);
    range.setAttribute('aria-label', s.label);
    const value = h('span', 'comfort-value');
    range.addEventListener('input', () => this.comfort.set(s.key, Number(range.value)));
    this.sync.push(() => {
      range.value = String(this.comfort.settings[s.key]);
      value.textContent = `${range.value}${s.unit}`;
    });
    const row = h('label', 'comfort-row');
    row.append(h('span', 'comfort-label', s.label), range, value);
    row.title = s.help;
    return row;
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
