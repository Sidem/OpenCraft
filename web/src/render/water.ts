// The water pass and the underwater look. Liquid quads are stored after each chunk's opaque and cutout
// quads (engine `mesher.rs`); this draws them last, blended, without depth writes, chunks back to front,
// from both sides (the surface shows from below). With the camera in water the fog turns blue-green and
// short and the sky is not drawn. Presentation only. To change the look: the constants below and the
// `WATER` variant in shaders.ts.

import { createProgram, uniforms } from './gl';
import * as S from './shaders';
import type { Sky } from './sky';

type Rgb = [number, number, number];

/** Underwater fog: its colour in full daylight, and where it starts and ends (blocks from the camera). */
const UNDERWATER_FOG: Rgb = [0.09, 0.3, 0.36];
const UNDERWATER_RANGE: [number, number] = [-6, 20];

/** What the pass reads from a chunk mesh (renderer.ts `ChunkMesh`). */
interface WaterMesh {
  x: number;
  y: number;
  z: number;
  vao: WebGLVertexArrayObject;
  opaque: number;
  cutout: number;
  liquid: number;
}

/** Fog colour and range for this frame: the horizon's, or underwater's (dimmed by the daylight). */
export function fogFor(sky: Sky, underwater: boolean, range: [number, number]): { color: Rgb; range: [number, number] } {
  if (!underwater) return { color: sky.horizon, range };
  const color = UNDERWATER_FOG.map((c, i) => c * (0.25 + 0.75 * sky.light[i])) as Rgb;
  return { color, range: UNDERWATER_RANGE };
}

export class WaterPass {
  private readonly prog: WebGLProgram;
  private readonly u;

  constructor(private readonly gl: WebGL2RenderingContext) {
    this.prog = createProgram(gl, S.chunkVert, S.litFrag, ['WATER']);
    this.u = uniforms(gl, this.prog, [
      'u_viewProj', 'u_offset', 'u_worldOrigin', 'u_tex', 'u_fogColor', 'u_fog', 'u_skyLight', 'u_time',
    ] as const);
  }

  /** Draws the liquid quads of `vis` (sorted front to back); returns [draw calls, quads]. */
  draw(vis: readonly WaterMesh[], viewProj: Float32Array, eye: Rgb, sky: Sky, fog: { color: Rgb; range: [number, number] }): [number, number] {
    const gl = this.gl, u = this.u;
    gl.useProgram(this.prog);
    gl.uniformMatrix4fv(u.u_viewProj, false, viewProj);
    gl.uniform1i(u.u_tex, 0);
    gl.uniform3f(u.u_fogColor, ...fog.color);
    gl.uniform3f(u.u_skyLight, ...sky.light);
    gl.uniform2f(u.u_fog, ...fog.range);
    gl.uniform1f(u.u_time, (performance.now() / 1000) % 1000);
    gl.enable(gl.BLEND);
    gl.blendFunc(gl.SRC_ALPHA, gl.ONE_MINUS_SRC_ALPHA);
    gl.depthMask(false);
    gl.disable(gl.CULL_FACE);
    let calls = 0, quads = 0;
    for (let i = vis.length - 1; i >= 0; i--) {
      const m = vis[i];
      if (m.liquid === 0) continue;
      gl.uniform3f(u.u_offset, m.x * 32 - eye[0], m.y * 32 - eye[1], m.z * 32 - eye[2]);
      gl.bindVertexArray(m.vao);
      gl.drawElements(gl.TRIANGLES, m.liquid * 6, gl.UNSIGNED_INT, (m.opaque + m.cutout) * 24);
      calls++;
      quads += m.liquid;
    }
    gl.enable(gl.CULL_FACE);
    gl.depthMask(true);
    gl.disable(gl.BLEND);
    return [calls, quads];
  }
}
