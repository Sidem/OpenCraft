// Volumetric wind-reactive smoke pipeline (engine `factory/smoke.rs`, GLSL in `smoke-glsl.ts`): one instanced WebGL2
// draw call expands each 8-float emitter into up to 16 billowing cloudlets on the GPU with smooth distance LOD, using
// a single precomputed 128×128 tileable 4-channel FBM/Worley noise texture (64 KB VRAM) and phase-integrated wind.

import { createProgram, uniforms } from './gl';
import type { Sky } from './sky';
import { SMOKE_FRAG, SMOKE_VERT } from './smoke-glsl';

/** Puffs expanded per emitter in the static quad-batch VBO (must match `SLOTS` length in `smoke-glsl.ts`). */
const PUFFS = 16;
/** Side length of the precomputed tileable 4-channel smoke noise texture (64 KB RGBA8). */
const NOISE_SIZE = 128;
/** Texture unit bound for the smoke noise texture (TEXTURE0 = atlas, 1 = roof, 2 = clouds, 4 = glow). */
const NOISE_UNIT = 3;

/** Deterministic tileable 2D value hash on an integer lattice of period `p`. */
function tileHash(ix: number, iy: number, p: number, seed: number): number {
  const x = ((ix % p) + p) % p;
  const y = ((iy % p) + p) % p;
  let h = (x * 374761393 + y * 668265263 + seed * 1442695041) | 0;
  h = Math.imul(h ^ (h >>> 13), 1274126177);
  return ((h ^ (h >>> 16)) >>> 0) / 4294967296;
}

/** Smooth tileable value noise at normalized `(u, v)` with `cells` periods across 0..1. */
function tileValue(u: number, v: number, cells: number, seed: number): number {
  const x = u * cells, y = v * cells;
  const ix = Math.floor(x), iy = Math.floor(y);
  let fx = x - ix, fy = y - iy;
  fx = fx * fx * (3 - 2 * fx);
  fy = fy * fy * (3 - 2 * fy);
  const a = tileHash(ix, iy, cells, seed);
  const b = tileHash(ix + 1, iy, cells, seed);
  const c = tileHash(ix, iy + 1, cells, seed);
  const d = tileHash(ix + 1, iy + 1, cells, seed);
  return (a + (b - a) * fx) + ((c + (d - c) * fx) - (a + (b - a) * fx)) * fy;
}

/** Tileable inverted Worley cellular billows (1 at soft puff crests, 0 in crevices). */
function tileBillow(u: number, v: number, cells: number, seed: number): number {
  const x = u * cells, y = v * cells;
  const ix = Math.floor(x), iy = Math.floor(y);
  let minD2 = 4.0;
  for (let dy = -1; dy <= 1; dy++) {
    for (let dx = -1; dx <= 1; dx++) {
      const cx = ix + dx, cy = iy + dy;
      const px = cx + 0.15 + 0.7 * tileHash(cx, cy, cells, seed);
      const py = cy + 0.15 + 0.7 * tileHash(cx, cy, cells, seed + 37);
      const ddx = px - x, ddy = py - y;
      const d2 = ddx * ddx + ddy * ddy;
      if (d2 < minD2) minD2 = d2;
    }
  }
  return Math.max(0, 1 - Math.sqrt(minD2) * 0.92);
}

/** Builds the 128×128 tileable 4-channel smoke noise atlas once on startup: R FBM, G Worley billows, B wisps, A warp. */
function buildNoiseTexture(gl: WebGL2RenderingContext): WebGLTexture {
  const data = new Uint8Array(NOISE_SIZE * NOISE_SIZE * 4);
  for (let y = 0; y < NOISE_SIZE; y++) {
    const v = y / NOISE_SIZE;
    for (let x = 0; x < NOISE_SIZE; x++) {
      const u = x / NOISE_SIZE;
      const r = 0.56 * tileValue(u, v, 4, 11) + 0.30 * tileValue(u, v, 8, 23) + 0.14 * tileValue(u, v, 16, 41);
      const g = 0.62 * tileBillow(u, v, 4, 59) + 0.38 * tileBillow(u, v, 8, 83);
      const b = 0.50 * tileValue(u, v, 8, 101) + 0.32 * tileValue(u, v, 16, 131) + 0.18 * tileValue(u, v, 32, 167);
      const a = 0.58 * tileValue(u, v, 4, 199) + 0.28 * tileValue(u, v, 8, 227) + 0.14 * tileValue(u, v, 16, 251);
      const i = (y * NOISE_SIZE + x) * 4;
      data[i] = Math.min(255, Math.max(0, Math.round(r * 255)));
      data[i + 1] = Math.min(255, Math.max(0, Math.round(g * 255)));
      data[i + 2] = Math.min(255, Math.max(0, Math.round(b * 255)));
      data[i + 3] = Math.min(255, Math.max(0, Math.round(a * 255)));
    }
  }
  const tex = gl.createTexture()!;
  gl.activeTexture(gl.TEXTURE0 + NOISE_UNIT);
  gl.bindTexture(gl.TEXTURE_2D, tex);
  gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA8, NOISE_SIZE, NOISE_SIZE, 0, gl.RGBA, gl.UNSIGNED_BYTE, data);
  gl.generateMipmap(gl.TEXTURE_2D);
  gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.LINEAR_MIPMAP_LINEAR);
  gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.LINEAR);
  gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.REPEAT);
  gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.REPEAT);
  gl.activeTexture(gl.TEXTURE0);
  return tex;
}

/** Static vertex buffer holding `PUFFS` quads (6 vertices each): `(cornerX, cornerY, puffIndex)`. */
function buildPuffBatch(): Float32Array {
  const verts = new Float32Array(PUFFS * 6 * 3);
  const corners: [number, number][] = [[-1, -1], [1, -1], [-1, 1], [1, -1], [1, 1], [-1, 1]];
  let o = 0;
  for (let p = 0; p < PUFFS; p++) {
    for (const [cx, cy] of corners) {
      verts[o++] = cx;
      verts[o++] = cy;
      verts[o++] = p;
    }
  }
  return verts;
}

export class SmokePipeline {
  private readonly prog: WebGLProgram;
  private readonly u;
  private readonly vao: WebGLVertexArrayObject;
  private readonly instances: WebGLBuffer;
  private readonly noiseTex: WebGLTexture;
  private capacity = 0;
  private windSpeed = 2.0;
  private turbPhase = 0;
  private locoPhase = 0;
  private gustPhase = 0;

  constructor(private readonly gl: WebGL2RenderingContext) {
    this.prog = createProgram(gl, SMOKE_VERT, SMOKE_FRAG);
    this.u = uniforms(gl, this.prog, [
      'u_view', 'u_proj', 'u_eyeMod', 'u_wind', 'u_phase', 'u_skyLight', 'u_sunDir', 'u_sunView', 'u_direct',
      'u_fogColor', 'u_fog', 'u_noise',
    ] as const);
    this.noiseTex = buildNoiseTexture(gl);
    this.vao = gl.createVertexArray()!;
    gl.bindVertexArray(this.vao);
    gl.bindBuffer(gl.ARRAY_BUFFER, gl.createBuffer()!);
    gl.bufferData(gl.ARRAY_BUFFER, buildPuffBatch(), gl.STATIC_DRAW);
    gl.enableVertexAttribArray(0);
    gl.vertexAttribPointer(0, 3, gl.FLOAT, false, 12, 0);
    this.instances = gl.createBuffer()!;
    for (const loc of [1, 2, 3]) {
      gl.enableVertexAttribArray(loc);
      gl.vertexAttribDivisor(loc, 1);
    }
    gl.bindVertexArray(null);
  }

  /** Smoothly eases real wind speed (m/s) and integrates turbulent eddy phases so wind shifts never cause puff jumps. */
  stepWind(dt: number, targetSpeedMs: number, enabled: boolean): void {
    const ease = 1 - Math.exp(-dt * 2.2);
    const target = enabled ? Math.max(0, targetSpeedMs) : 0;
    this.windSpeed += (target - this.windSpeed) * ease;
    const s = this.windSpeed;
    this.turbPhase = (this.turbPhase + dt * (0.9 + 0.24 * s)) % (Math.PI * 200);
    this.locoPhase = (this.locoPhase + dt) % (Math.PI * 200);
    this.gustPhase = (this.gustPhase + dt * (0.55 + 0.16 * s)) % (Math.PI * 200);
  }

  /** Draws `count` packed smoke emitters (`floats` 32-bit words each) from `data` (a view into wasm memory). Returns draw calls. */
  draw(
    view: Float32Array,
    proj: Float32Array,
    eye: readonly [number, number, number],
    sky: Sky,
    fog: { color: [number, number, number]; range: [number, number] },
    windDir: readonly [number, number],
    data: Uint32Array,
    count: number,
    floats: number,
    plumePhase: number,
  ): number {
    if (count === 0) return 0;
    const gl = this.gl;
    gl.bindVertexArray(this.vao);
    gl.bindBuffer(gl.ARRAY_BUFFER, this.instances);
    if (count * floats > this.capacity) {
      this.capacity = Math.max(count * floats, this.capacity * 2, 512);
      gl.bufferData(gl.ARRAY_BUFFER, this.capacity * 4, gl.DYNAMIC_DRAW);
    }
    gl.bufferSubData(gl.ARRAY_BUFFER, 0, data.subarray(0, count * floats));
    gl.vertexAttribPointer(1, 4, gl.FLOAT, false, floats * 4, 0);
    gl.vertexAttribPointer(2, 3, gl.FLOAT, false, floats * 4, 16);
    gl.vertexAttribIPointer(3, 1, gl.UNSIGNED_INT, floats * 4, 28);

    gl.useProgram(this.prog);
    gl.activeTexture(gl.TEXTURE0 + NOISE_UNIT);
    gl.bindTexture(gl.TEXTURE_2D, this.noiseTex);
    gl.uniform1i(this.u.u_noise, NOISE_UNIT);
    gl.uniformMatrix4fv(this.u.u_view, false, view);
    gl.uniformMatrix4fv(this.u.u_proj, false, proj);
    const mod256 = (v: number) => ((v % 256) + 256) % 256;
    gl.uniform3f(this.u.u_eyeMod, mod256(eye[0]), mod256(eye[1]), mod256(eye[2]));
    gl.uniform4f(this.u.u_wind, windDir[0], windDir[1], this.windSpeed, this.gustPhase);
    gl.uniform4f(this.u.u_phase, plumePhase, this.turbPhase, this.locoPhase, 0);
    gl.uniform3f(this.u.u_skyLight, ...sky.light);
    gl.uniform3f(this.u.u_sunDir, ...sky.lightDir);
    // Transform direct light direction into view space for 3D volumetric puff shading.
    const [lx, ly, lz] = sky.lightDir;
    const svx = view[0] * lx + view[4] * ly + view[8] * lz;
    const svy = view[1] * lx + view[5] * ly + view[9] * lz;
    const svz = view[2] * lx + view[6] * ly + view[10] * lz;
    gl.uniform3f(this.u.u_sunView, svx, svy, svz);
    gl.uniform3f(this.u.u_direct, ...sky.direct);
    gl.uniform3f(this.u.u_fogColor, ...fog.color);
    gl.uniform2f(this.u.u_fog, fog.range[0], fog.range[1]);

    gl.enable(gl.BLEND);
    gl.blendFunc(gl.SRC_ALPHA, gl.ONE_MINUS_SRC_ALPHA);
    gl.depthMask(false);
    gl.disable(gl.CULL_FACE);
    gl.drawArraysInstanced(gl.TRIANGLES, 0, PUFFS * 6, count);
    gl.bindVertexArray(null);
    gl.depthMask(true);
    gl.enable(gl.CULL_FACE);
    gl.disable(gl.BLEND);
    gl.activeTexture(gl.TEXTURE0);
    return 1;
  }
}
