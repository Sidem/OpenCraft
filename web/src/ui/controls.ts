// The menu's Controls page: a keyboard picture with every key the game uses lit in its category's colour, a mouse row, a line
// that says what the key under the pointer (or focus, or finger) does, and the same controls as a list below, grouped by
// category. Everything comes from one table (control-bindings.ts) and is printed with the player's key labels
// (keyboard-layout.ts), so it matches their keyboard. Hovering or focusing a key rings it and the rows that use it; hovering a row
// rings its keys; clicking a key keeps it shown; pressing a key on this page lights it; a category chip fades the rest.
// The page also keeps every `kbd[data-code]` on the menu printed with the current labels. To add a control: control-bindings.ts.

import './controls.css';
import { BINDINGS, CATEGORIES, type Binding, type MouseInput, categoryOf, fill, tokensOf } from './control-bindings';
import { button, h } from './dom';
import { CHOICES, type KeyLabels, type LayoutChoice, PRESETS, keyLabels } from './keyboard-layout';
import { KeyboardView } from './keyboard-view';

const MOUSE: { id: MouseInput; name: string }[] = [
  { id: 'move', name: 'Look' },
  { id: 'left', name: 'Left' },
  { id: 'right', name: 'Right' },
  { id: 'wheel', name: 'Wheel' },
];
const HINT = 'Point at a key to see what it does, or press one. The colours are the groups below.';
const colourOf = (cat: string) => CATEGORIES.find((c) => c.id === cat)!.colour;

export class ControlsPage {
  readonly el = h('div', 'controls-page');
  private readonly view: KeyboardView;
  private readonly detail = h('p', 'ctl-detail');
  private readonly status = h('span', 'ctl-status');
  private readonly select = h('select');
  private readonly rows: { b: Binding; row: HTMLElement; name: HTMLElement; text: HTMLElement }[] = [];
  private readonly pills = new Map<MouseInput, HTMLButtonElement>();
  private readonly chips: HTMLButtonElement[] = [];
  private pointed: string | null = null;
  private pinned: string | null = null;
  private category: string | null = null;

  constructor(private readonly labels: KeyLabels = keyLabels) {
    this.view = new KeyboardView(labels, {
      point: (code) => this.point(code),
      pick: (code) => this.pin(this.pinned === code ? null : code),
      describe: (code) => this.bindingsOf(code).map((b) => fill(b.text, (c) => labels.label(c))).join('; '),
    });
    this.detail.setAttribute('aria-live', 'polite');
    this.el.append(this.top(), this.view.el, this.mouseRow(), this.detail, this.list());
    labels.subscribe(() => this.sync());
    // A key pressed while this page shows lights its cap (and says what it does); the labels also learn from it elsewhere.
    window.addEventListener('keydown', (e) => this.onKey(e, true));
    window.addEventListener('keyup', (e) => this.onKey(e, false));
    this.sync();
  }

  // ---- building

  private top(): HTMLElement {
    const bar = h('div', 'ctl-top');
    const legend = h('div', 'ctl-legend');
    for (const c of CATEGORIES) {
      const chip = button('ctl-chip', c.title, () => this.filter(this.category === c.id ? null : c.id));
      chip.style.setProperty('--cat', c.colour);
      chip.dataset.cat = c.id;
      chip.setAttribute('aria-pressed', 'false');
      this.chips.push(chip);
      legend.append(chip);
    }
    const pick = h('label', 'ctl-layout');
    for (const c of CHOICES) this.select.append(new Option(c.name, c.id));
    this.select.addEventListener('change', () => this.labels.setChoice(this.select.value as LayoutChoice));
    pick.append(h('span', '', 'Keyboard layout'), this.select);
    bar.append(legend, pick, this.status);
    return bar;
  }

  private mouseRow(): HTMLElement {
    const row = h('div', 'ctl-mouse');
    row.append(h('span', 'ctl-mouse-name', 'Mouse'));
    for (const m of MOUSE) {
      const cat = BINDINGS.find((b) => b.mouse?.includes(m.id))?.cat;
      const pill = button('ctl-pill', m.name, () => this.pin(this.pinned === `mouse:${m.id}` ? null : `mouse:${m.id}`));
      pill.style.setProperty('--cat', colourOf(cat ?? 'other'));
      pill.dataset.cat = cat ?? 'other';
      pill.addEventListener('pointerenter', () => this.point(`mouse:${m.id}`));
      pill.addEventListener('pointerleave', () => this.point(null));
      pill.addEventListener('focus', () => this.point(`mouse:${m.id}`));
      pill.addEventListener('blur', () => this.point(null));
      this.pills.set(m.id, pill);
      row.append(pill);
    }
    return row;
  }

  private list(): HTMLElement {
    const list = h('div', 'ctl-list');
    for (const c of CATEGORIES) {
      const section = h('section', 'ctl-section');
      section.dataset.cat = c.id;
      section.style.setProperty('--cat', c.colour);
      section.append(h('h3', '', c.title));
      for (const b of BINDINGS.filter((x) => x.cat === c.id)) {
        const row = h('div', 'ctl-row');
        const name = h('div', 'ctl-name');
        const text = h('div', 'ctl-text');
        row.append(name, text);
        row.addEventListener('pointerenter', () => this.point(`row:${BINDINGS.indexOf(b)}`));
        row.addEventListener('pointerleave', () => this.point(null));
        this.rows.push({ b, row, name, text });
        section.append(row);
      }
      list.append(section);
    }
    return list;
  }

  // ---- what a pointer, a focus or a click means

  private point(what: string | null): void {
    this.pointed = what;
    this.show();
  }

  private pin(what: string | null): void {
    this.pinned = what;
    this.show();
  }

  private filter(category: string | null): void {
    this.category = category;
    this.view.filter(category);
    for (const chip of this.chips) chip.setAttribute('aria-pressed', String(chip.dataset.cat === category));
    for (const s of this.el.querySelectorAll<HTMLElement>('.ctl-section, .ctl-pill')) s.classList.toggle('ctl-dim', category !== null && s.dataset.cat !== category);
  }

  /** The rows that use a key (`code`), a mouse input (`mouse:left`) or are one row (`row:3`). */
  private bindingsFor(what: string): Binding[] {
    if (what.startsWith('row:')) return [BINDINGS[Number(what.slice(4))]];
    if (what.startsWith('mouse:')) return BINDINGS.filter((b) => b.mouse?.includes(what.slice(6) as MouseInput));
    return this.bindingsOf(what);
  }

  private bindingsOf(code: string): Binding[] {
    return BINDINGS.filter((b) => b.keys.includes(code));
  }

  /** Draws the detail line and the rings for what is pointed at (else what is kept). */
  private show(): void {
    const what = this.pointed ?? this.pinned;
    const found = what ? this.bindingsFor(what) : [];
    const lit = new Set<string>();
    if (what && !what.startsWith('mouse:')) {
      if (what.startsWith('row:')) for (const b of found) [...b.keys, ...tokensOf(b.text)].forEach((c) => lit.add(c));
      else lit.add(what);
    }
    this.view.light(lit);
    const litRows = new Set(found);
    for (const r of this.rows) r.row.classList.toggle('ctl-hot', litRows.has(r.b));
    for (const [id, pill] of this.pills) pill.classList.toggle('ctl-hot', what === `mouse:${id}` || (what?.startsWith('row:') === true && found.some((b) => b.mouse?.includes(id))));
    this.detail.replaceChildren(...this.describe(what, found));
  }

  private describe(what: string | null, found: Binding[]): (Node | string)[] {
    if (!what) return [HINT];
    const name = (text: string) => fill(text, (c) => this.labels.label(c));
    const lines = found.map((b) => {
      const line = h('span', 'ctl-line');
      const dot = h('i', 'ctl-dot');
      dot.style.background = colourOf(b.cat);
      line.append(dot, h('b', '', name(b.label)), ` ${name(b.text)}`);
      return line;
    });
    if (what.startsWith('row:')) return lines;
    const title = what.startsWith('mouse:') ? `Mouse ${MOUSE.find((m) => `mouse:${m.id}` === what)!.name.toLowerCase()}` : this.labels.label(what);
    const head = h('kbd', '', title);
    return lines.length ? [head, ...lines] : [head, ' is not used by the game.'];
  }

  // ---- keeping everything in step

  /** Reprints every label, after the layout or what was learned changed. */
  private sync(): void {
    this.view.sync();
    this.select.value = this.labels.choice;
    for (const r of this.rows) {
      r.name.textContent = fill(r.b.label, (c) => this.labels.label(c));
      r.text.textContent = fill(r.b.text, (c) => this.labels.label(c));
    }
    const { source, looksLike } = this.labels;
    const layout = PRESETS[looksLike].name;
    this.status.textContent =
      source === 'chosen' ? `Showing the ${layout} layout you picked.`
      : source === 'map' ? 'Read from your browser.'
      : source === 'learned' ? `Learned from your key presses: it looks like ${layout}. Not right? Pick yours.`
      : 'Assuming QWERTY. Press a few keys, or pick your layout.';
    for (const kbd of document.querySelectorAll<HTMLElement>('kbd[data-code]')) kbd.textContent = this.labels.label(kbd.dataset.code!);
    this.show();
  }

  private onKey(e: KeyboardEvent, down: boolean): void {
    if (!this.el.offsetParent || !this.view.has(e.code)) return;
    // Keys that move around or press the focused control are for that control, not for the picture.
    if (e.target instanceof Element && e.target.closest('button, select, input, textarea') && /^(Tab|Enter|Space|Arrow\w+)$/.test(e.code)) return;
    this.view.press(e.code, down);
    if (down && !e.repeat && categoryOf(e.code)) this.pin(e.code);
  }
}
