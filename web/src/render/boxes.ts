// Instanced boxes: dropped items, belt items and machine parts, drawn as one instanced draw call of
// a unit cube. The engine writes the instance records each frame (`factory/render.rs`); this uploads
// them into a growing GPU buffer and draws them with the lit cutout shader.
// Instance layout (INSTANCE_FLOATS floats): centre xyz, yaw, size xyz, uv scroll, texture layers
// top/side/bottom, uv mode. Attribute locations 1–3 read it as three vec4s (see `S.boxVert`).

import { createProgram, uniforms } from './gl';
import * as S from './shaders';

/** Floats per box instance; must match `factory::INSTANCE_FLOATS` in the engine. */
export const INSTANCE_FLOATS = 12;

export class BoxPipeline {
  private readonly prog: WebGLProgram;
  private readonly u;
  private readonly vao: WebGLVertexArrayObject;
  private readonly instances: WebGLBuffer;
  private capacity = 0;

  constructor(private readonly gl: WebGL2RenderingContext) {
    this.prog = createProgram(gl, S.boxVert, S.litFrag, ['CUTOUT']);
    this.u = uniforms(gl, this.prog, ['u_viewProj', 'u_offset', 'u_tex', 'u_fogColor', 'u_fog'] as const);

    // One unit cube, per-instance centre/size/rotation/textures.
    this.vao = gl.createVertexArray()!;
    gl.bindVertexArray(this.vao);
    const corners = gl.createBuffer()!;
    gl.bindBuffer(gl.ARRAY_BUFFER, corners);
    gl.bufferData(gl.ARRAY_BUFFER, boxCorners(), gl.STATIC_DRAW);
    gl.enableVertexAttribArray(0);
    gl.vertexAttribPointer(0, 4, gl.FLOAT, false, 16, 0);
    this.instances = gl.createBuffer()!;
    gl.bindBuffer(gl.ARRAY_BUFFER, this.instances);
    for (const [loc, offset] of [[1, 0], [2, 16], [3, 32]]) {
      gl.enableVertexAttribArray(loc);
      gl.vertexAttribPointer(loc, 4, gl.FLOAT, false, INSTANCE_FLOATS * 4, offset);
      gl.vertexAttribDivisor(loc, 1);
    }
    gl.bindVertexArray(null);
  }

  /**
   * Draws `count` instances from `boxes` (which may be a view into wasm memory). Expects the block
   * texture array bound to unit 0. Returns the number of draw calls made.
   */
  draw(viewProj: Float32Array, sky: readonly number[], fog: [number, number], boxes: Float32Array, count: number): number {
    if (count === 0) return 0;
    const gl = this.gl;
    gl.useProgram(this.prog);
    gl.uniformMatrix4fv(this.u.u_viewProj, false, viewProj);
    gl.uniform1i(this.u.u_tex, 0);
    gl.uniform3f(this.u.u_fogColor, sky[0], sky[1], sky[2]);
    gl.uniform2f(this.u.u_fog, fog[0], fog[1]);
    gl.bindBuffer(gl.ARRAY_BUFFER, this.instances);
    const bytes = count * INSTANCE_FLOATS * 4;
    if (bytes > this.capacity) {
      this.capacity = Math.max(bytes, this.capacity * 2);
      gl.bufferData(gl.ARRAY_BUFFER, this.capacity, gl.DYNAMIC_DRAW);
    }
    gl.bufferSubData(gl.ARRAY_BUFFER, 0, boxes, 0, count * INSTANCE_FLOATS);
    gl.bindVertexArray(this.vao);
    gl.drawArraysInstanced(gl.TRIANGLES, 0, 36, count);
    return 1;
  }
}

/**
 * Unit cube for instanced boxes: 36 vertices of (corner.xyz, face index). Faces follow the mesher's
 * order and axes (u × v = normal), so triangles wind counter-clockwise seen from outside.
 */
function boxCorners(): Float32Array {
  const faces: [number[], number[], number[]][] = [
    [[1, 0, 0], [0, 1, 0], [0, 0, 1]],
    [[-1, 0, 0], [0, 0, 1], [0, 1, 0]],
    [[0, 1, 0], [0, 0, 1], [1, 0, 0]],
    [[0, -1, 0], [1, 0, 0], [0, 0, 1]],
    [[0, 0, 1], [1, 0, 0], [0, 1, 0]],
    [[0, 0, -1], [0, 1, 0], [1, 0, 0]],
  ];
  const out: number[] = [];
  faces.forEach(([n, u, v], face) => {
    const corner = (cu: number, cv: number) => [0, 1, 2].map((i) => n[i] * 0.5 + (cu - 0.5) * u[i] + (cv - 0.5) * v[i]);
    const c = [corner(0, 0), corner(1, 0), corner(1, 1), corner(0, 1)];
    for (const i of [0, 1, 2, 0, 2, 3]) out.push(...c[i], face);
  });
  return new Float32Array(out);
}
