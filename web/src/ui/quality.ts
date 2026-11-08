// "Picture quality" on the menu's settings page, and what Auto does while playing: it drives `AutoQuality` with played frames,
// applies its steps, says so, and puts a suggestion on the launcher when it can do no more. It reuses the Comfort section's row styles
// (comfort.css). Applying a setting to the renderer and engine is `apply`. To add a control: a row in the constructor.

import './quality.css';
import type { Game } from '../wasm/engine.js';
import { AutoQuality, type AutoStep } from '../render/auto-quality';
import type { FrameTimes } from '../render/frame-stats';
import { type QualityStore, RADII, SCALES } from '../render/quality';
import type { Renderer } from '../render/renderer';
import { button, h } from './dom';

export interface QualitySources {
  store: QualityStore;
  game: Game;
  renderer: Renderer;
  frames: FrameTimes;
  /** Shows a short line over the game. */
  notice: (text: string) => void;
}

export class QualityPanel {
  readonly el = h('div', 'comfort');
  /** The suggestion on the launcher, when Auto ran out of things it may change. */
  readonly suggestion = h('div', 'quality-suggestion hidden');
  private readonly sync: (() => void)[] = [];
  private readonly auto: AutoQuality;

  constructor(private readonly src: QualitySources) {
    const { store } = src;
    this.auto = new AutoQuality(SCALES.indexOf(store.settings.scale as (typeof SCALES)[number]), SCALES.length - 1);
    this.el.append(
      this.choices('Sharpness', SCALES.map((s) => `${s}%`), () => SCALES.indexOf(store.settings.scale as (typeof SCALES)[number]), (i) => store.choose('scale', SCALES[i])),
      h('p', 'comfort-help', 'Draws fewer pixels and stretches them to fit. The first thing to lower on a slow graphics card.'),
      this.choices('Render distance', RADII.map(String), () => RADII.indexOf(store.settings.radius as (typeof RADII)[number]), (i) => store.choose('radius', RADII[i])),
      h('p', 'comfort-help', 'How many chunks away the world is shown. Shorter is easier on both the processor and the graphics card.'),
      this.choices('Smooth edges', ['Off', 'On'], () => Number(store.settings.antialias), (i) => store.choose('antialias', i === 1)),
      h('p', 'comfort-help', 'Antialiasing. A change here applies the next time you load the game.'),
      this.choices('Automatic', ['Off', 'On'], () => Number(store.settings.auto), (i) => store.set('auto', i === 1)),
      h('p', 'comfort-help', 'Lowers the sharpness by itself when the game runs slowly, and tells you when it does. Picking any setting above turns it off.'),
      button('secondary-btn', 'Reset to defaults', () => store.reset()),
    );
    store.subscribe(() => {
      this.auto.level = SCALES.indexOf(store.settings.scale as (typeof SCALES)[number]);
      this.apply();
      this.sync.forEach((fn) => fn());
    });
  }

  /** Once per drawn frame. `playing`: nothing covers the game, so the frame says something about play. */
  update(now: number, scriptMs: number, playing: boolean): void {
    const { store, frames } = this.src;
    if (!playing || !store.settings.auto || frames.lastMs === 0) return;
    this.act(this.auto.frame(now, frames.lastMs, scriptMs));
  }

  private act(step: AutoStep): void {
    const { store, notice } = this.src;
    if (step === null) return;
    if (step === 'suggest-drawing' || step === 'suggest-script') {
      if (this.suggest(step === 'suggest-drawing')) notice('The game is still running slowly. Press Esc for a suggestion.');
      return;
    }
    const scale = SCALES[this.auto.level];
    store.set('scale', scale);
    if (step === 'lower') notice(`Running slowly, so the picture is now ${scale}% sharp. You can change this in Settings.`);
    else notice(`Picture sharpness is back to ${scale}%.`);
  }

  /** Offered and never just done, because the player would see the difference. What is offered follows the measurements
   *  (docs/history/CHANGELOG_R.md, "Performance check"): when drawing is the limit smooth edges cost more than distance does;
   *  when the script is the limit only a shorter distance helps. False when there is nothing to offer. */
  private suggest(drawing: boolean): boolean {
    const { store } = this.src;
    const shorter = [...RADII].reverse().find((r) => r < store.settings.radius);
    if (drawing && store.settings.antialias) {
      this.offer('Turning off smooth edges would ease the load, but edges will look more jagged. It applies the next time you load the game.', 'Turn them off', () => store.set('antialias', false));
      return true;
    }
    if (shorter === undefined) return false;
    this.offer(`A shorter render distance (${shorter} instead of ${store.settings.radius}) would ease the load, but you will see less of the world.`, `Use ${shorter}`, () => store.set('radius', shorter));
    return true;
  }

  private offer(text: string, yes: string, accept: () => void): void {
    const close = () => this.suggestion.classList.add('hidden');
    this.suggestion.classList.remove('hidden');
    this.suggestion.replaceChildren(
      h('strong', '', 'The game is running slowly'),
      h('p', '', text),
      button('secondary-btn', yes, () => {
        accept();
        close();
      }),
      button('secondary-btn', 'No thanks', close),
    );
  }

  private apply(): void {
    const { store, game, renderer } = this.src;
    renderer.setScale(store.settings.scale / 100);
    renderer.setViewRadius(store.settings.radius);
    game.set_view_radius(store.settings.radius);
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
