// Note signs' text (engine `signs/view.rs`). Each frame this reads which sign each text slot shows (`game.sign_faces()`:
// x, y, z and a stamp per slot), and for a slot whose stamp changed paints that sign's lines in its colours on a canvas
// twice as wide as a texture layer, uploads the halves into the slot's two layers of the block texture array
// (`game.sign_face_layer()`, left half first) and rebuilds the mipmaps once. So painting happens only when a sign comes
// near or its words or colours change. Layout: render/sign-layout.ts.

import type { Game } from '../wasm/engine.js';
import { hex } from '../ui/colour';
import type { Renderer } from './renderer';
import { layoutSign } from './sign-layout';

const ROWS = 4; // the engine's `signs::LINES`
const FONT = 'system-ui, "Segoe UI", sans-serif';

export class SignFaces {
  private readonly shown: number[] = [];
  private readonly ctx: CanvasRenderingContext2D;
  private readonly first: number;

  /** `size`: the texture array's layer size in pixels. */
  constructor(private readonly game: Game, private readonly renderer: Renderer, private readonly size: number) {
    const canvas = document.createElement('canvas');
    canvas.width = size * 2;
    canvas.height = size;
    this.ctx = canvas.getContext('2d', { willReadFrequently: true })!;
    this.first = game.sign_face_layer();
  }

  update(): void {
    const faces = this.game.sign_faces();
    let painted = false;
    for (let i = 0; i * 4 < faces.length; i++) {
      const stamp = faces[i * 4 + 3];
      if (stamp === 0 || this.shown[i] === stamp) continue;
      this.paint(i, faces[i * 4], faces[i * 4 + 1], faces[i * 4 + 2]);
      this.shown[i] = stamp;
      painted = true;
    }
    if (painted) this.renderer.refreshMipmaps();
  }

  /** Paints the sign at (x, y, z) into slot `slot`'s two layers. */
  private paint(slot: number, x: number, y: number, z: number): void {
    const [back, ink] = this.game.sign_colours(x, y, z);
    const lines = this.game.sign_text(x, y, z).split('\n');
    const ctx = this.ctx, w = this.size * 2, h = this.size;
    ctx.fillStyle = hex(back ?? 0);
    ctx.fillRect(0, 0, w, h);
    const font = (size: number) => `600 ${size}px ${FONT}`;
    const layout = layoutSign(lines, ROWS, w, h, (text, size) => ((ctx.font = font(size)), ctx.measureText(text).width));
    ctx.font = font(layout.size);
    ctx.fillStyle = hex(ink ?? 0);
    ctx.textAlign = 'center';
    ctx.textBaseline = 'middle';
    layout.middles.forEach((middle, i) => ctx.fillText(lines[i], w / 2, middle));
    for (const half of [0, 1]) {
      const pixels = ctx.getImageData(half * this.size, 0, this.size, this.size).data;
      this.renderer.paintLayer(this.first + slot * 2 + half, new Uint8Array(pixels.buffer, pixels.byteOffset, pixels.length));
    }
  }
}
