// A rotary dial for values 0–100. Drag up/right to turn it up (Shift for fine control), use the
// mouse wheel or arrow keys to nudge it, double-click to reset. A small tick on the rim marks the
// default, and the words under the dial say what each end means.

import './knob.css';
import type { DialInfo } from '../audio/settings';

const SWEEP = 270; // degrees of travel, centred on 12 o'clock
const RIM = 26; // arc radius in the 64×64 viewBox

const angle = (v: number) => -SWEEP / 2 + (v / 100) * SWEEP;

function polar(deg: number, r: number): [number, number] {
  const a = ((deg - 90) * Math.PI) / 180;
  return [32 + r * Math.cos(a), 32 + r * Math.sin(a)];
}

function arc(from: number, to: number, r: number): string {
  if (to - from < 0.01) return '';
  const [x0, y0] = polar(from, r), [x1, y1] = polar(to, r);
  return `M${x0.toFixed(2)} ${y0.toFixed(2)}A${r} ${r} 0 ${to - from > 180 ? 1 : 0} 1 ${x1.toFixed(2)} ${y1.toFixed(2)}`;
}

export interface KnobOptions extends DialInfo {
  value: number;
  defaultValue: number;
  small?: boolean;
  onInput: (value: number) => void;
  /** Receives the dial's explanation while it is hovered, focused or dragged, and null after. */
  onHelp: (text: string | null) => void;
}

export class Knob {
  readonly el: HTMLDivElement;
  private value: number;
  private readonly dial: HTMLDivElement;
  private readonly valueArc: SVGPathElement;
  private readonly pointer: SVGLineElement;
  private readonly readout: HTMLSpanElement;
  private drag: { id: number; x: number; y: number; v: number } | null = null;

  constructor(private readonly opts: KnobOptions) {
    this.value = opts.value;
    this.el = document.createElement('div');
    this.el.className = opts.small ? 'knob small' : 'knob';
    const [tx0, ty0] = polar(angle(opts.defaultValue), RIM - 5);
    const [tx1, ty1] = polar(angle(opts.defaultValue), RIM + 5);
    this.el.innerHTML = `
      <div class="knob-label"></div>
      <div class="knob-dial" role="slider" tabindex="0" aria-valuemin="0" aria-valuemax="100">
        <svg viewBox="0 0 64 64" aria-hidden="true">
          <path class="knob-track" d="${arc(angle(0), angle(100), RIM)}" />
          <path class="knob-value" />
          <line class="knob-default" x1="${tx0}" y1="${ty0}" x2="${tx1}" y2="${ty1}" />
          <circle class="knob-body" cx="32" cy="32" r="19" />
          <line class="knob-pointer" />
        </svg>
        <span class="knob-readout"></span>
      </div>
      <div class="knob-ends"><span></span><span></span></div>`;
    const q = <T extends Element>(sel: string) => this.el.querySelector(sel) as T;
    q<HTMLDivElement>('.knob-label').textContent = opts.label;
    const [low, high] = this.el.querySelectorAll('.knob-ends span');
    low.textContent = opts.low;
    high.textContent = opts.high;
    this.dial = q('.knob-dial');
    this.dial.setAttribute('aria-label', opts.label);
    this.valueArc = q('.knob-value');
    this.pointer = q('.knob-pointer');
    this.readout = q('.knob-readout');
    this.el.title = `${opts.label}: ${opts.low} ↔ ${opts.high}`;
    this.bind();
    this.render();
  }

  private bind(): void {
    const d = this.dial;
    const help = () => this.opts.onHelp(`${this.opts.label} (${this.opts.low} ↔ ${this.opts.high}): ${this.opts.help}`);
    const unhelp = () => {
      if (!this.drag && document.activeElement !== d) this.opts.onHelp(null);
    };
    d.addEventListener('pointerenter', help);
    d.addEventListener('focus', help);
    d.addEventListener('pointerleave', unhelp);
    d.addEventListener('blur', () => this.opts.onHelp(null));

    d.addEventListener('pointerdown', (e) => {
      if (e.button !== 0) return;
      e.preventDefault();
      d.focus();
      d.setPointerCapture(e.pointerId);
      this.drag = { id: e.pointerId, x: e.clientX, y: e.clientY, v: this.value };
      this.el.classList.add('dragging');
    });
    d.addEventListener('pointermove', (e) => {
      const drag = this.drag;
      if (!drag || e.pointerId !== drag.id) return;
      // Up and right both turn it up; 200 px covers the full range, 1000 px with Shift.
      const moved = drag.y - e.clientY + (e.clientX - drag.x);
      drag.v += moved * (e.shiftKey ? 0.1 : 0.5);
      drag.x = e.clientX;
      drag.y = e.clientY;
      this.change(drag.v);
    });
    const end = (e: PointerEvent) => {
      if (this.drag?.id !== e.pointerId) return;
      this.drag = null;
      this.el.classList.remove('dragging');
    };
    d.addEventListener('pointerup', end);
    d.addEventListener('pointercancel', end);
    d.addEventListener('dblclick', () => this.change(this.opts.defaultValue));
    d.addEventListener(
      'wheel',
      (e) => {
        e.preventDefault();
        const dir = Math.sign(-(e.deltaY || e.deltaX));
        this.change(this.value + dir * (e.shiftKey ? 1 : 2));
      },
      { passive: false },
    );
    d.addEventListener('keydown', (e) => {
      const step = e.shiftKey ? 10 : 1;
      const keys: Record<string, number> = {
        ArrowUp: this.value + step,
        ArrowRight: this.value + step,
        ArrowDown: this.value - step,
        ArrowLeft: this.value - step,
        PageUp: this.value + 10,
        PageDown: this.value - 10,
        Home: 0,
        End: 100,
      };
      if (e.key in keys) {
        e.preventDefault();
        this.change(keys[e.key]);
      }
    });
  }

  private change(raw: number): void {
    const v = Math.round(Math.min(100, Math.max(0, raw)));
    if (v === this.value) return;
    this.value = v;
    this.render();
    this.opts.onInput(v);
  }

  private render(): void {
    const v = this.value;
    const a = angle(v);
    this.valueArc.setAttribute('d', arc(angle(0), a, RIM));
    // A short pointer near the body's edge, leaving the centre free for the number.
    const [x0, y0] = polar(a, 13);
    const [x1, y1] = polar(a, 18);
    this.pointer.setAttribute('x1', x0.toFixed(2));
    this.pointer.setAttribute('y1', y0.toFixed(2));
    this.pointer.setAttribute('x2', x1.toFixed(2));
    this.pointer.setAttribute('y2', y1.toFixed(2));
    this.readout.textContent = String(v);
    const side = v === 50 ? 'middle' : v < 50 ? `toward ${this.opts.low}` : `toward ${this.opts.high}`;
    this.dial.setAttribute('aria-valuenow', String(v));
    this.dial.setAttribute('aria-valuetext', `${v}, ${side}`);
    this.el.classList.toggle('changed', v !== this.opts.defaultValue);
  }
}
