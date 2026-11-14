// Trees coming down. The engine fells a tree whole (`sim/felling.rs`) and hands it over meshed (`game.take_fallen()`,
// layout in `api/felling.rs`): up to two parts, each a shaped list (shaped.ts) with a pivot at the foot and a direction.
// Each part is drawn by the shaped passes with `u_pivot` and `u_turn` set: it tips about a horizontal axis through the
// pivot, perpendicular to the fall, easing in like gravity, and rests on its limbs a little short of flat. A moment
// after the engine's logs land it is dropped and its buffers deleted. Purely visual: the engine already decided.
// To change the look: the constants below. No camera motion is added.

import type { Game } from '../wasm/engine.js';
import { deleteShaped, uploadShaped, type ShapedPart } from './shaped';
import type { Uniforms } from './gl';

/** Seconds from the first tip to the ground (the engine's `FALL_TICKS` at 60 Hz). */
const FALL = 1.6;
/** Seconds the tree lies before it is gone. */
const LIE = 0.5;
/** How far it ends up tipped: it rests on its limbs, a little short of flat. */
const REST = (84 * Math.PI) / 180;
const HEADER = 10; // words before each part's mesh

interface Part {
  shaped: ShapedPart;
  start: number; // performance.now() ms
  origin: [number, number, number]; // the part's origin, world
  pivot: [number, number, number]; // world
  angle: number; // fall direction, radians from +x towards +z
}

type Pass = Uniforms<'u_offset' | 'u_pivot' | 'u_turn'>;

export class Falling {
  private parts: Part[] = [];
  private readonly turn = new Float32Array(9);

  /** `reserve` makes the shared quad index buffer long enough for a mesh of that many quads. */
  constructor(private readonly gl: WebGL2RenderingContext, private readonly quadIndex: WebGLBuffer,
    private readonly reserve: (quads: number) => void) {}

  /** Takes the trees the engine felled since the last call and uploads them; call once a frame. */
  take(game: Game, now: number): void {
    const words = game.take_fallen();
    const f32 = new Float32Array(words.buffer, words.byteOffset, words.length);
    const i32 = new Int32Array(words.buffer, words.byteOffset, words.length);
    let at = 1;
    for (let n = words[0]; n > 0; n--) {
      const opaque = words[at + 7], cutout = words[at + 8], len = words[at + 9];
      this.reserve(opaque + cutout);
      const shaped = uploadShaped(this.gl, null, this.quadIndex, words.subarray(at + HEADER, at + HEADER + len), opaque, cutout);
      if (shaped) {
        this.parts.push({
          shaped, start: now, angle: f32[at + 3],
          origin: [i32[at + 4], i32[at + 5], i32[at + 6]],
          pivot: [f32[at], f32[at + 1], f32[at + 2]],
        });
      }
      at += HEADER + len;
    }
    this.parts = this.parts.filter((p) => {
      if (now - p.start < (FALL + LIE) * 1000) return true;
      deleteShaped(this.gl, p.shaped);
      return false;
    });
  }

  /** Draws the falling parts of one shaped pass (`cutout`: the leaf cards, else the wood); returns the draw calls. */
  draw(u: Pass, cutout: boolean, now: number, eye: [number, number, number]): number {
    const gl = this.gl;
    let calls = 0;
    for (const p of this.parts) {
      const first = cutout ? p.shaped.opaque : 0, n = cutout ? p.shaped.cutout : p.shaped.opaque;
      if (n === 0) continue;
      const t = Math.min(1, Math.max(0, (now - p.start) / 1000 / FALL));
      tip(this.turn, p.angle, REST * t * t);
      gl.uniform3f(u.u_offset, p.origin[0] - eye[0], p.origin[1] - eye[1], p.origin[2] - eye[2]);
      gl.uniform3f(u.u_pivot, p.pivot[0] - p.origin[0], p.pivot[1] - p.origin[1], p.pivot[2] - p.origin[2]);
      gl.uniformMatrix3fv(u.u_turn, false, this.turn);
      gl.bindVertexArray(p.shaped.vao);
      gl.drawElements(gl.TRIANGLES, n * 6, gl.UNSIGNED_INT, first * 24);
      calls++;
    }
    if (calls > 0) {
      gl.uniform3f(u.u_pivot, 0, 0, 0);
      gl.uniformMatrix3fv(u.u_turn, false, IDENTITY);
    }
    return calls;
  }
}

const IDENTITY = new Float32Array([1, 0, 0, 0, 1, 0, 0, 0, 1]);

/** Writes (column-major) the turn of `theta` that tips "up" towards the direction (cos a, 0, sin a): about k = up × d. */
function tip(m: Float32Array, a: number, theta: number): void {
  const kx = Math.sin(a), kz = -Math.cos(a), c = Math.cos(theta), s = Math.sin(theta), v = 1 - c;
  // R = c I + s [k]x + (1 - c) k kT with k = (kx, 0, kz); element (row i, column j) is m[j * 3 + i].
  m[0] = c + v * kx * kx; m[1] = s * kz;           m[2] = v * kx * kz;
  m[3] = -s * kz;         m[4] = c;                m[5] = s * kx;
  m[6] = v * kx * kz;     m[7] = -s * kx;          m[8] = c + v * kz * kz;
}

