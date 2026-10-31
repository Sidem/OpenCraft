// Colour wheel (the sign editor's, ui/sign.ts): a disc with the hue round it and the saturation out from its white middle,
// at the brightness the slider under it sets, and a ring marking the colour. Drag on the disc or the slider; `onChange`
// gets each new colour (0xRRGGBB). The maths is ui/colour.ts.

import './colour-wheel.css';
import { hex, hsvToRgb, rgbToHsv, wheelPick, wheelPoint } from './colour';
import { h } from './dom';

/** The disc's size in canvas pixels. */
const SIZE = 168;

export class ColourWheel {
  readonly el = h('div', 'cw');
  onChange: (rgb: number) => void = () => {};

  private readonly disc = h('canvas', 'cw-disc');
  private readonly mark = h('div', 'cw-mark');
  private readonly value = h('input', 'cw-value');
  private hsv: [number, number, number] = [0, 0, 1];
  private drawnAt = -1; // the brightness the disc was last painted at

  constructor() {
    this.disc.width = this.disc.height = SIZE;
    this.value.type = 'range';
    this.value.min = '0';
    this.value.max = '100';
    this.value.setAttribute('aria-label', 'Brightness');
    const wrap = h('div', 'cw-wrap');
    wrap.append(this.disc, this.mark);
    this.el.append(wrap, this.value);
    const pick = (e: PointerEvent) => {
      const box = this.disc.getBoundingClientRect(), r = box.width / 2;
      const [hue, sat] = wheelPick(e.clientX - box.left - r, e.clientY - box.top - r, r);
      this.hsv = [hue, sat, this.hsv[2]];
      this.changed();
    };
    this.disc.addEventListener('pointerdown', (e) => {
      this.disc.setPointerCapture(e.pointerId);
      pick(e);
    });
    this.disc.addEventListener('pointermove', (e) => {
      if (this.disc.hasPointerCapture(e.pointerId)) pick(e);
    });
    this.value.addEventListener('input', () => {
      this.hsv[2] = Number(this.value.value) / 100;
      this.changed();
    });
  }

  /** Shows `rgb` without calling `onChange`. */
  set(rgb: number): void {
    this.hsv = rgbToHsv(rgb);
    this.draw();
  }

  private changed(): void {
    this.draw();
    this.onChange(hsvToRgb(...this.hsv));
  }

  private draw(): void {
    const [hue, sat, v] = this.hsv;
    if (v !== this.drawnAt) this.paintDisc(v);
    this.value.value = String(Math.round(v * 100));
    this.value.style.setProperty('--full', hex(hsvToRgb(hue, sat, 1)));
    const [x, y] = wheelPoint(hue, sat, 50);
    this.mark.style.left = `${50 + x}%`;
    this.mark.style.top = `${50 + y}%`;
    this.mark.style.background = hex(hsvToRgb(...this.hsv));
  }

  /** Paints the disc at brightness `v`, its rim smoothed over a pixel. */
  private paintDisc(v: number): void {
    const ctx = this.disc.getContext('2d')!;
    const img = ctx.createImageData(SIZE, SIZE), r = SIZE / 2;
    for (let y = 0; y < SIZE; y++) {
      for (let x = 0; x < SIZE; x++) {
        const dx = x + 0.5 - r, dy = y + 0.5 - r, i = (y * SIZE + x) * 4;
        const rgb = hsvToRgb(...wheelPick(dx, dy, r), v);
        img.data[i] = rgb >> 16;
        img.data[i + 1] = (rgb >> 8) & 255;
        img.data[i + 2] = rgb & 255;
        img.data[i + 3] = 255 * Math.max(0, Math.min(1, r - Math.hypot(dx, dy)));
      }
    }
    ctx.putImageData(img, 0, 0);
    this.drawnAt = v;
  }
}
