// Sound controls: the master volume/mute control (pause menu and designer footer) and the sound
// designer, where every material and action is tuned with plain-language dials while listening.

import type { SoundSystem } from '../audio/sound';
import {
  ACTIONS,
  ACTION_DIAL_INFO,
  ACTION_INFO,
  DEFAULT_DESIGN,
  MATERIALS,
  MATERIAL_DIAL_INFO,
  MATERIAL_GROUPS,
  MATERIAL_LABELS,
  PRESETS,
  defaultDesign,
  exportDesign,
  mergeDesign,
  type ActionName,
} from '../audio/settings';
import { Knob } from './knob';

const SPEAKER_ON =
  '<svg viewBox="0 0 24 24" aria-hidden="true"><path fill="currentColor" d="M3 9v6h4l5 4V5L7 9z"/><path fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" d="M15.5 8.5a5 5 0 0 1 0 7M18.5 5.5a9 9 0 0 1 0 13"/></svg>';
const SPEAKER_OFF =
  '<svg viewBox="0 0 24 24" aria-hidden="true"><path fill="currentColor" d="M3 9v6h4l5 4V5L7 9z"/><path fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" d="M16 9.5l5 5M21 9.5l-5 5"/></svg>';

const TIP =
  'Drag a dial up or down to turn it (hold Shift for fine control), or scroll over it. Double-click a dial to reset it; the small mark on its rim shows the default, and a dot next to its name means it was changed.';

/** Actions the Listen buttons offer for a material. */
const LISTEN: ActionName[] = ['dig', 'break', 'place', 'step', 'land'];
/** Minimum gap between automatic previews while a dial is being turned. */
const AUDITION_GAP_MS = 280;

function h<K extends keyof HTMLElementTagNameMap>(tag: K, className = '', text = ''): HTMLElementTagNameMap[K] {
  const e = document.createElement(tag);
  if (className) e.className = className;
  if (text) e.textContent = text;
  return e;
}

function button(className: string, text: string, onClick: (b: HTMLButtonElement) => void): HTMLButtonElement {
  const b = h('button', className, text);
  b.type = 'button';
  b.addEventListener('click', () => onClick(b));
  return b;
}

/** Mute button, volume slider and level readout, kept in sync with the sound system. */
export class VolumeControl {
  readonly el = h('div', 'volume-control');
  private readonly toggle: HTMLButtonElement;
  private readonly range = h('input');
  private readonly readout = h('span', 'volume-value');

  constructor(private readonly sound: SoundSystem) {
    this.toggle = button('mute-toggle', '', () => {
      sound.unlock();
      sound.toggleMute();
    });
    this.range.type = 'range';
    this.range.min = '0';
    this.range.max = '100';
    this.range.setAttribute('aria-label', 'Volume');
    this.range.addEventListener('input', () => {
      sound.unlock();
      if (sound.muted) sound.setMuted(false);
      sound.setVolume(Number(this.range.value));
    });
    // A sample on release, so the new level can be judged.
    this.range.addEventListener('change', () => sound.preview('place', 0));
    this.el.append(this.toggle, this.range, this.readout);
    sound.subscribe(() => this.sync());
    this.sync();
  }

  private sync(): void {
    const { volume, muted } = this.sound.settings;
    this.range.value = String(volume);
    this.readout.textContent = muted ? 'Muted' : `${volume}%`;
    this.toggle.innerHTML = muted || volume === 0 ? SPEAKER_OFF : SPEAKER_ON;
    this.toggle.title = muted ? 'Unmute (M)' : 'Mute (M)';
    this.toggle.setAttribute('aria-label', muted ? 'Unmute' : 'Mute');
    this.el.classList.toggle('muted', muted);
  }
}

export interface SoundBlock {
  id: number;
  name: string;
  /** Sound material index. */
  material: number;
}

type Loop = { kind: 'walk' | 'mine'; timer: number; tick: number };

/** The sound designer panel. */
export class SoundLab {
  onClose: () => void = () => {};

  private readonly backdrop = h('div', 'lab-backdrop hidden');
  private readonly dialog = h('div', 'lab');
  private readonly tabBar = h('div', 'lab-tabs');
  private readonly tabs: HTMLButtonElement[] = [];
  private readonly note = h('div', 'lab-note hidden');
  private readonly body = h('div', 'lab-scroll');
  private readonly help = h('p', 'lab-help', TIP);
  private readonly paste = h('div', 'lab-paste hidden');
  private readonly pasteText = h('textarea');
  private readonly pasteStatus = h('span', 'lab-paste-status');
  private readonly loopButtons = new Map<Loop['kind'], HTMLButtonElement>();
  private readonly listenButtons = new Map<ActionName, HTMLButtonElement>();

  /** Material index, or MATERIALS.length for the Actions tab. */
  private tab = 0;
  /** Material the Actions tab (and loops started there) play on. */
  private hearOn = 0;
  /** The action replayed when a material dial turns. */
  private listen: ActionName = 'dig';
  private playOnChange = true;
  private loop: Loop | null = null;
  private lastAudition = 0;
  private auditionTimer = 0;

  constructor(
    private readonly sound: SoundSystem,
    private readonly blocks: SoundBlock[],
    private readonly icon: (block: number) => HTMLCanvasElement,
  ) {
    this.dialog.setAttribute('role', 'dialog');
    this.dialog.setAttribute('aria-modal', 'true');
    this.dialog.setAttribute('aria-labelledby', 'lab-title');
    this.dialog.tabIndex = -1;

    const head = h('div', 'lab-head');
    const title = h('h2', '', 'Sound designer');
    title.id = 'lab-title';
    head.append(
      title,
      h(
        'p',
        'lab-intro',
        'Pick a material, press a Listen button, then turn the dials until it sounds right. Changes apply in the game straight away and are saved in this browser.',
      ),
      button('lab-close', '×', () => this.close()),
    );
    head.lastElementChild!.setAttribute('aria-label', 'Close sound designer');

    this.tabBar.setAttribute('role', 'tablist');
    MATERIALS.forEach((name, m) => {
      const t = button('lab-tab', '', () => this.select(m));
      const rep = blocks.find((b) => b.material === m);
      if (rep) t.append(this.iconCanvas(rep.id));
      t.append(MATERIAL_LABELS[name]);
      this.tabs.push(t);
    });
    this.tabs.push(button('lab-tab lab-tab-actions', 'Actions', () => this.select(MATERIALS.length)));
    for (const t of this.tabs) t.setAttribute('role', 'tab');
    this.tabBar.append(...this.tabs);

    this.note.append('Sound is muted, so you will not hear anything. ', button('secondary-btn', 'Unmute', () => {
      this.sound.unlock();
      this.sound.setMuted(false);
      if (this.sound.settings.volume === 0) this.sound.setVolume(80);
    }));

    this.dialog.append(head, this.tabBar, this.note, this.body, this.help, this.buildPaste(), this.buildFooter());
    this.backdrop.append(this.dialog);
    this.backdrop.addEventListener('keydown', (e) => {
      if (e.key === 'Escape') {
        e.preventDefault();
        this.close();
      }
    });
    this.backdrop.addEventListener('pointerdown', (e) => {
      if (e.target === this.backdrop) this.close();
    });
    document.body.append(this.backdrop);
    sound.subscribe(() => this.syncNote());
  }

  get isOpen(): boolean {
    return !this.backdrop.classList.contains('hidden');
  }

  /** Opens on a material tab (the one you were just hearing, ideally). */
  open(material = this.tab): void {
    this.backdrop.classList.remove('hidden');
    this.select(Math.min(MATERIALS.length, Math.max(0, material)));
    this.syncNote();
    this.dialog.focus();
  }

  close(): void {
    if (!this.isOpen) return;
    this.stopLoop();
    this.backdrop.classList.add('hidden');
    this.onClose();
  }

  private select(tab: number): void {
    this.tab = tab;
    if (tab < MATERIALS.length) this.hearOn = tab;
    else this.stopLoop(); // loop buttons live on the material tabs
    this.tabs.forEach((t, i) => {
      t.setAttribute('aria-selected', String(i === tab));
      t.tabIndex = i === tab ? 0 : -1;
    });
    this.render();
  }

  private render(): void {
    this.body.replaceChildren();
    this.listenButtons.clear();
    this.loopButtons.clear();
    if (this.tab < MATERIALS.length) this.renderMaterial(this.tab);
    else this.renderActions();
    this.body.scrollTop = 0;
  }

  private renderMaterial(m: number): void {
    const name = MATERIALS[m];
    const users = this.blocks.filter((b) => b.material === m);

    const head = h('div', 'lab-mat-head');
    if (users[0]) head.append(this.iconCanvas(users[0].id));
    const titles = h('div');
    titles.append(h('h3', '', MATERIAL_LABELS[name]), h('p', 'lab-used-by', `Used by: ${users.map((b) => b.name).join(', ') || 'no blocks'}`));
    const tools = h('div', 'lab-mat-tools');
    const preset = h('select', 'lab-select');
    preset.setAttribute('aria-label', 'Start from a preset');
    preset.append(new Option('Start from a preset…', ''));
    for (const p of Object.keys(PRESETS)) preset.append(new Option(p, p));
    preset.addEventListener('change', () => {
      const p = PRESETS[preset.value];
      if (!p) return;
      this.sound.setMaterial(m, { ...p });
      this.render();
      this.audition(this.listen, m);
    });
    tools.append(
      preset,
      button('secondary-btn', `Reset ${MATERIAL_LABELS[name]}`, () => {
        this.sound.setMaterial(m, { ...DEFAULT_DESIGN.materials[name] });
        this.render();
        this.audition(this.listen, m);
      }),
    );
    head.append(titles, tools);

    const listen = h('div', 'lab-listen');
    listen.append(h('span', 'lab-listen-label', 'Listen'));
    for (const a of LISTEN) {
      const b = button('secondary-btn listen-btn', `▶ ${ACTION_INFO[a].label}`, () => {
        this.listen = a;
        this.syncListen();
        this.sound.unlock();
        this.sound.preview(a, m);
      });
      b.title = `${ACTION_INFO[a].help} Also replayed when you turn a dial.`;
      this.listenButtons.set(a, b);
      listen.append(b);
    }
    listen.append(h('span', 'sep'));
    for (const [kind, label, help] of [
      ['walk', 'Walk loop', 'Keeps playing footsteps so you can turn dials while listening.'],
      ['mine', 'Mining loop', 'Keeps mining and breaking a block so you can turn dials while listening.'],
    ] as const) {
      const b = button('secondary-btn loop-btn', label, () => this.toggleLoop(kind));
      b.title = help;
      this.loopButtons.set(kind, b);
      listen.append(b);
    }
    this.syncListen();
    this.syncLoops();

    const groups = h('div', 'lab-groups');
    const params = this.sound.design.materials[name];
    for (const g of MATERIAL_GROUPS) {
      const box = h('div', 'lab-group');
      const knobs = h('div', 'lab-knobs');
      for (const dial of g.dials) {
        const knob = new Knob({
          ...MATERIAL_DIAL_INFO[dial],
          value: params[dial],
          defaultValue: DEFAULT_DESIGN.materials[name][dial],
          onInput: (v) => {
            this.sound.setMaterialDial(m, dial, v);
            this.dialTurned(this.listen, m);
          },
          onHelp: (text) => this.setHelp(text),
        });
        knobs.append(knob.el);
      }
      box.append(h('h4', '', g.title), h('p', '', g.blurb), knobs);
      groups.append(box);
    }

    this.body.append(head, listen, groups);
  }

  private renderActions(): void {
    const intro = h('div', 'lab-actions-intro');
    const pick = h('select', 'lab-select');
    pick.setAttribute('aria-label', 'Material to hear actions on');
    MATERIALS.forEach((name, m) => pick.append(new Option(MATERIAL_LABELS[name], String(m), false, m === this.hearOn)));
    pick.addEventListener('change', () => (this.hearOn = Number(pick.value)));
    const label = h('label');
    label.append('Hear them on ', pick);
    intro.append(
      h('p', '', "Actions start from the material's sound and adjust it. 50 on these dials leaves the material's sound as it is."),
      label,
    );
    this.body.append(intro);

    for (const action of ACTIONS) {
      const info = ACTION_INFO[action];
      const row = h('div', 'lab-action-row');
      const play = button('play-btn', '▶', () => {
        this.sound.unlock();
        this.sound.preview(action, this.hearOn);
      });
      play.setAttribute('aria-label', `Play ${info.label}`);
      const text = h('div', 'lab-action-text');
      text.append(h('h4', '', info.label), h('p', '', info.help));
      const knobs = h('div', 'lab-knobs');
      const params = this.sound.design.actions[action];
      for (const dial of info.dials) {
        const knob = new Knob({
          ...ACTION_DIAL_INFO[dial],
          value: params[dial],
          defaultValue: DEFAULT_DESIGN.actions[action][dial],
          small: true,
          onInput: (v) => {
            this.sound.setActionDial(action, dial, v);
            this.dialTurned(action, this.hearOn);
          },
          onHelp: (t) => this.setHelp(t),
        });
        knobs.append(knob.el);
      }
      const reset = button('secondary-btn', 'Reset', () => {
        this.sound.setAction(action, { ...DEFAULT_DESIGN.actions[action] });
        this.render();
        this.audition(action, this.hearOn);
      });
      reset.setAttribute('aria-label', `Reset ${info.label}`);
      row.append(play, text, knobs, reset);
      this.body.append(row);
    }
  }

  private buildFooter(): HTMLElement {
    const foot = h('div', 'lab-foot');
    const check = h('label', 'lab-check');
    const box = h('input');
    box.type = 'checkbox';
    box.checked = this.playOnChange;
    box.addEventListener('change', () => (this.playOnChange = box.checked));
    check.append(box, 'Play the sound when I turn a dial');

    const copy = button('secondary-btn', 'Copy settings', (b) => void this.copy(b));
    copy.title = 'Copies every sound setting as text, to keep, share, or make the new defaults.';
    const paste = button('secondary-btn', 'Paste settings', () => this.showPaste(''));
    paste.title = 'Load settings copied earlier.';
    let armed = 0;
    const resetAll = button('secondary-btn danger', 'Reset everything', (b) => {
      if (!armed) {
        b.textContent = 'Click again to reset all';
        armed = window.setTimeout(() => {
          armed = 0;
          b.textContent = 'Reset everything';
        }, 3000);
        return;
      }
      clearTimeout(armed);
      armed = 0;
      b.textContent = 'Reset everything';
      this.sound.setDesign(defaultDesign());
      this.render();
    });
    foot.append(new VolumeControl(this.sound).el, check, h('span', 'spacer'), copy, paste, resetAll);
    return foot;
  }

  private buildPaste(): HTMLElement {
    this.pasteText.spellcheck = false;
    this.pasteText.setAttribute('aria-label', 'Sound settings text');
    const row = h('div', 'lab-paste-row');
    row.append(
      button('secondary-btn', 'Apply', () => this.applyPaste()),
      button('secondary-btn', 'Close', () => this.paste.classList.add('hidden')),
      this.pasteStatus,
    );
    this.paste.append(this.pasteText, row);
    return this.paste;
  }

  private showPaste(text: string, status = 'Paste settings text below, then press Apply.'): void {
    this.paste.classList.remove('hidden');
    this.pasteText.value = text;
    this.pasteStatus.textContent = status;
    this.pasteText.focus();
    if (text) this.pasteText.select();
  }

  private applyPaste(): void {
    let data: unknown;
    try {
      data = JSON.parse(this.pasteText.value);
    } catch {
      this.pasteStatus.textContent = "That doesn't look like copied sound settings.";
      return;
    }
    const next = structuredClone(this.sound.design);
    const applied = mergeDesign(next, data);
    if (!applied) {
      this.pasteStatus.textContent = 'No sound settings found in that text.';
      return;
    }
    this.sound.setDesign(next);
    this.render();
    this.pasteStatus.textContent = `Applied ${applied} settings.`;
  }

  private async copy(b: HTMLButtonElement): Promise<void> {
    const text = exportDesign(this.sound.design);
    try {
      await navigator.clipboard.writeText(text);
      b.textContent = 'Copied ✓';
      setTimeout(() => (b.textContent = 'Copy settings'), 1500);
    } catch {
      this.showPaste(text, 'Your browser blocked copying: press Ctrl+C to copy the selected text.');
    }
  }

  /** A dial moved: replay the relevant sound, unless a loop is already playing it. */
  private dialTurned(action: ActionName, material: number): void {
    if (this.playOnChange && !this.loop) this.audition(action, material);
  }

  /**
   * Plays at most one preview per AUDITION_GAP_MS while a dial is dragged, and always one after the
   * last change, so the final setting is the one you hear.
   */
  private audition(action: ActionName, material: number): void {
    clearTimeout(this.auditionTimer);
    const wait = this.lastAudition + AUDITION_GAP_MS - performance.now();
    if (wait > 0) {
      this.auditionTimer = window.setTimeout(() => this.audition(action, material), wait);
      return;
    }
    this.lastAudition = performance.now();
    this.sound.unlock();
    this.sound.preview(action, material);
  }

  private toggleLoop(kind: Loop['kind']): void {
    const same = this.loop?.kind === kind;
    this.stopLoop();
    if (same) return;
    this.sound.unlock();
    const loop: Loop = { kind, timer: 0, tick: 0 };
    const step = () => {
      const m = this.tab < MATERIALS.length ? this.tab : this.hearOn;
      if (kind === 'walk') this.sound.preview('step', m);
      else {
        // Five mining hits, the break, then a short pause.
        const phase = loop.tick % 8;
        if (phase < 5) this.sound.preview('dig', m);
        else if (phase === 5) this.sound.preview('break', m);
      }
      loop.tick++;
    };
    step();
    loop.timer = window.setInterval(step, kind === 'walk' ? 420 : 240);
    this.loop = loop;
    this.syncLoops();
  }

  private stopLoop(): void {
    if (!this.loop) return;
    clearInterval(this.loop.timer);
    this.loop = null;
    this.syncLoops();
  }

  private syncLoops(): void {
    for (const [kind, b] of this.loopButtons) {
      const on = this.loop?.kind === kind;
      b.classList.toggle('active', on);
      b.textContent = on ? '■ Stop' : kind === 'walk' ? '⟳ Walk loop' : '⟳ Mining loop';
    }
  }

  private syncListen(): void {
    for (const [a, b] of this.listenButtons) b.classList.toggle('active', a === this.listen);
  }

  private syncNote(): void {
    this.note.classList.toggle('hidden', !this.sound.muted && this.sound.settings.volume > 0);
  }

  private setHelp(text: string | null): void {
    this.help.textContent = text ?? TIP;
  }

  private iconCanvas(block: number): HTMLCanvasElement {
    const src = this.icon(block);
    const c = h('canvas', 'lab-icon');
    c.width = src.width;
    c.height = src.height;
    c.getContext('2d')!.drawImage(src, 0, 0);
    return c;
  }
}
