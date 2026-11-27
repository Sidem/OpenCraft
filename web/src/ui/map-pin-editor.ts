// The world map's pin editor (worldmap.ts): over the view, the kind buttons, a short note and Save (or Add pin),
// Remove and Cancel; Enter saves, Escape cancels. Pins themselves live in pins.ts.

import './map-pin-editor.css';
import { button, h } from './dom';
import type { Pin, Pins } from './pins';

export class PinEditor {
  readonly el = h('div', 'wm-editor hidden');
  private editing: { pin: Pin | null; x: number; z: number; kind: string } | null = null;

  constructor(private readonly pins: Pins) {}

  get isOpen(): boolean {
    return this.editing !== null;
  }

  /** Opens the editor for `pin`, or for a new pin at (x, z). */
  open(x: number, z: number, pin: Pin | null): void {
    const kind = pin?.kind ?? this.editing?.kind ?? this.pins.kinds[1].id;
    this.editing = { pin, x: pin?.x ?? Math.floor(x), z: pin?.z ?? Math.floor(z), kind };
    const e = this.el;
    e.replaceChildren();
    const kinds = h('div', 'wm-kinds');
    for (const k of this.pins.kinds) {
      const b = button('wm-kind', k.label, () => {
        this.editing!.kind = k.id;
        kinds.querySelectorAll('.wm-kind').forEach((el) => el.classList.toggle('on', el === b));
      });
      b.style.setProperty('--kind', k.color);
      b.classList.toggle('on', k.id === kind);
      kinds.append(b);
    }
    const note = h('input', 'wm-note');
    note.placeholder = 'A few words (optional)';
    note.maxLength = 40;
    note.value = pin?.note ?? '';
    const save = () => {
      const ed = this.editing!;
      if (ed.pin) this.pins.update(ed.pin, ed.kind, note.value.trim());
      else this.pins.add({ x: ed.x, z: ed.z, kind: ed.kind, note: note.value.trim() });
      this.close();
    };
    note.addEventListener('keydown', (ev) => {
      if (ev.key === 'Enter') save();
      else if (ev.key === 'Escape') {
        ev.preventDefault();
        this.close();
      }
    });
    const buttons = h('div', 'wm-editor-buttons');
    buttons.append(button('secondary-btn active', pin ? 'Save' : 'Add pin', save));
    if (pin) {
      buttons.append(button('secondary-btn danger', 'Remove', () => {
        this.pins.remove(pin);
        this.close();
      }));
    }
    buttons.append(button('secondary-btn', 'Cancel', () => this.close()));
    e.append(h('h4', '', `${pin ? 'Pin' : 'New pin'} at x ${this.editing.x}, z ${this.editing.z}`), kinds, note, buttons);
    e.classList.remove('hidden');
    note.focus();
  }

  close(): void {
    this.editing = null;
    this.el.classList.add('hidden');
  }
}
