// Instanced boxes: dropped items, belt items and machine parts. The engine writes the instance records each frame
// (`factory/render.rs`); this uploads them into growing GPU buffers and draws them with the lit cutout shader: the
// daylight, lamps and shade of the cell each box stands in (the engine packs that light into the mode float). Two
// instanced draw calls: every record over the unit cube, which skips the round bodies, then the round bodies (copied
// out by the engine, `split_round`) over the unit cylinder (`round-mesh.ts`).
// Instance layout (INSTANCE_FLOATS floats): centre xyz, yaw, size xyz, uv scroll, texture layers
// top/side/bottom, mode bits + 4 × light, pitch, roll, taper, glow. Four vec4s (see `S.boxVert`).

import { createProgram, uniforms } from './gl';
import { uploadHeldLights } from './held-light';
import { roundMesh } from './round-mesh';
import * as S from './shaders';
import type { Sky } from './sky';

/** One mesh drawn once an instance: its vertex array, instance buffer and how many indices (or vertices) it has. */
interface Pass {
  vao: WebGLVertexArrayObject;
  instances: WebGLBuffer;
  capacity: number;
  count: number;
  indexed: boolean;
}

export class BoxPipeline {
  private readonly prog: WebGLProgram;
  private readonly u;
  private readonly cube: Pass;
  private readonly round: Pass;

  constructor(private readonly gl: WebGL2RenderingContext, private readonly instanceFloats: number, roundBit: number) {
    this.prog = createProgram(gl, S.boxVert, S.litFrag, ['CUTOUT', 'BOXES', `ROUND_BIT ${roundBit}u`]);
    this.u = uniforms(gl, this.prog, ['u_viewProj', 'u_offset', 'u_tex', 'u_glow', 'u_fogColor', 'u_fog', 'u_skyLight', 'u_sunDir', 'u_direct', 'u_held', 'u_round'] as const);
    this.cube = this.pass(boxCorners(), null);
    const { corners, indices } = roundMesh();
    this.round = this.pass(corners, indices);
  }

  /** A vertex array over `corners` (and `indices`, if any) with per-instance centre/size/rotation/textures. */
  private pass(corners: Float32Array, indices: Uint16Array | null): Pass {
    const gl = this.gl;
    const vao = gl.createVertexArray()!;
    gl.bindVertexArray(vao);
    gl.bindBuffer(gl.ARRAY_BUFFER, gl.createBuffer()!);
    gl.bufferData(gl.ARRAY_BUFFER, corners, gl.STATIC_DRAW);
    gl.enableVertexAttribArray(0);
    gl.vertexAttribPointer(0, 4, gl.FLOAT, false, 16, 0);
    if (indices) {
      gl.bindBuffer(gl.ELEMENT_ARRAY_BUFFER, gl.createBuffer()!);
      gl.bufferData(gl.ELEMENT_ARRAY_BUFFER, indices, gl.STATIC_DRAW);
    }
    const instances = gl.createBuffer()!;
    gl.bindBuffer(gl.ARRAY_BUFFER, instances);
    for (const [loc, offset] of [[1, 0], [2, 16], [3, 32], [4, 48]]) {
      gl.enableVertexAttribArray(loc);
      gl.vertexAttribPointer(loc, 4, gl.FLOAT, false, this.instanceFloats * 4, offset);
      gl.vertexAttribDivisor(loc, 1);
    }
    gl.bindVertexArray(null);
    return { vao, instances, capacity: 0, count: indices ? indices.length : corners.length / 4, indexed: !!indices };
  }

  /**
   * Draws `count` instances from `boxes` and `roundCount` from `round` (views into wasm memory, or any arrays).
   * Expects the block texture array bound to unit 0. Returns the number of draw calls made.
   */
  draw(
    viewProj: Float32Array,
    sky: Sky,
    fog: { color: [number, number, number]; range: [number, number] },
    boxes: Float32Array,
    count: number,
    round: Float32Array,
    roundCount: number,
  ): number {
    if (count === 0) return 0;
    const gl = this.gl;
    gl.useProgram(this.prog);
    gl.uniformMatrix4fv(this.u.u_viewProj, false, viewProj);
    gl.uniform1i(this.u.u_tex, 0);
    gl.uniform1i(this.u.u_glow, S.GLOW_UNIT);
    gl.uniform3f(this.u.u_fogColor, ...fog.color);
    gl.uniform3f(this.u.u_skyLight, ...sky.light);
    gl.uniform3f(this.u.u_sunDir, ...sky.lightDir);
    gl.uniform3f(this.u.u_direct, ...sky.direct);
    gl.uniform2f(this.u.u_fog, ...fog.range);
    uploadHeldLights(gl, this.u.u_held);
    gl.uniform1i(this.u.u_round, 0);
    this.drawPass(this.cube, boxes, count);
    if (roundCount === 0) return 1;
    gl.uniform1i(this.u.u_round, 1);
    this.drawPass(this.round, round, roundCount);
    return 2;
  }

  private drawPass(pass: Pass, records: Float32Array, count: number): void {
    const gl = this.gl;
    gl.bindBuffer(gl.ARRAY_BUFFER, pass.instances);
    const bytes = count * this.instanceFloats * 4;
    if (bytes > pass.capacity) {
      pass.capacity = Math.max(bytes, pass.capacity * 2);
      gl.bufferData(gl.ARRAY_BUFFER, pass.capacity, gl.DYNAMIC_DRAW);
    }
    gl.bufferSubData(gl.ARRAY_BUFFER, 0, records, 0, count * this.instanceFloats);
    gl.bindVertexArray(pass.vao);
    if (pass.indexed) gl.drawElementsInstanced(gl.TRIANGLES, pass.count, gl.UNSIGNED_SHORT, 0, count);
    else gl.drawArraysInstanced(gl.TRIANGLES, 0, pass.count, count);
    gl.bindVertexArray(null);
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
