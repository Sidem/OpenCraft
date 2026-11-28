// Instanced boxes: dropped items, belt items and machine parts. The engine writes the instance records each frame
// (`factory/render.rs`); this uploads them into growing GPU buffers and draws them with the lit cutout shader: the
// daylight, lamps and shade of the cell each box stands in (the engine packs that light into each record). Two
// instanced draw calls: the boxes over the unit cube, then the round bodies (a list of their own) over the unit
// cylinder (`round-mesh.ts`). `drawGhost` draws the engine's see-through records the same way, tinted and blended
// (`u_ghost`).
// Instance layout: 8 words (`game.instance_floats()`) packed by the engine's `box_pack`: the centre as three floats,
// then five words of halves and bit fields that `boxVert` (box-glsl.ts) decodes.

import { createProgram, uniforms } from './gl';
import { uploadHeldLights } from './held-light';
import { boxInFrustum } from './mat4';
import { roundMesh } from './round-mesh';
import { boxVert } from './box-glsl';
import * as S from './shaders';
import type { Sky } from './sky';

const CHUNK = 32;
/** Static box chunks are packed into 4×8×4-chunk (128×256×128) GPU regions so `drawStatic` makes only a handful of draw calls. */
const REGION_SHIFT = 2;
const REGION_CHUNKS = 1 << REGION_SHIFT;
const REGION = CHUNK * REGION_CHUNKS;
const ZERO_SCROLL: readonly [number, number, number, number] = [0, 0, 0, 0];
const regionKey = (rx: number, rz: number) => (rx + 1048576) * 2097152 + (rz + 1048576);

/** `u_ghost` (tint, opacity) for solid boxes, a ghost, and a ghost in the way of something. */
const SOLID = [0, 0, 0, 0] as const;
const GHOST = [0.55, 0.85, 1.0, 0.4] as const;
const GHOST_BLOCKED = [1.0, 0.3, 0.25, 0.4] as const;
/** The X-ray goggles' tint and opacity: cyan, strong enough to read a belt's turns through rock. */
const XRAY = [0.3, 0.9, 1.0, 0.62] as const;

/** One mesh drawn once an instance: its vertex array, instance buffer and how many indices (or vertices) it has. */
interface Pass { vao: WebGLVertexArrayObject; instances: WebGLBuffer; capacity: number; count: number; indexed: boolean; }
type Fog = { color: [number, number, number]; range: [number, number] };
type Vec4 = readonly [number, number, number, number];

/** One cached 32³ chunk of static boxes, already shifted into its parent region's local coordinates. */
interface StaticChunk {
  rk: number;
  minX: number; minY: number; minZ: number; maxX: number; maxY: number; maxZ: number;
  cubes: Float32Array; cubeCount: number; rounds: Float32Array; roundCount: number;
}

/** One GPU-resident 128×256×128 region combining its constituent static box chunks into 1 cube + 1 round VBO. */
interface StaticRegion {
  rk: number; rx: number; rz: number;
  minX: number; minY: number; minZ: number; maxX: number; maxY: number; maxZ: number;
  chunks: Map<number, StaticChunk>;
  cubeVao: WebGLVertexArrayObject | null; cubeVbo: WebGLBuffer | null; cubeCount: number; cubeCap: number;
  roundVao: WebGLVertexArrayObject | null; roundVbo: WebGLBuffer | null; roundCount: number; roundCap: number;
}

export class BoxPipeline {
  private readonly prog: WebGLProgram;
  private readonly u;
  private readonly cubeVerts: WebGLBuffer;
  private readonly roundVerts: WebGLBuffer;
  private readonly roundIdx: WebGLBuffer;
  private readonly cube: Pass;
  private readonly round: Pass;
  private readonly staticChunks = new Map<number, StaticChunk>();
  private readonly staticRegions = new Map<number, StaticRegion>();
  private readonly dirtyRegions = new Set<StaticRegion>();
  private readonly visibleStatic: StaticRegion[] = [];
  private packScratch = new Float32Array(0);

  constructor(private readonly gl: WebGL2RenderingContext, private readonly instanceFloats: number) {
    this.prog = createProgram(gl, boxVert, S.litFrag, ['CUTOUT', 'BOXES']);
    this.u = uniforms(gl, this.prog, ['u_viewProj', 'u_offset', 'u_beltScroll', 'u_tex', 'u_glow', 'u_fogColor', 'u_fog', 'u_skyLight', 'u_sunDir', 'u_direct', 'u_held', 'u_round', 'u_ghost', 'u_xray'] as const);
    const cubeCorners = boxCorners();
    this.cubeVerts = gl.createBuffer()!;
    gl.bindBuffer(gl.ARRAY_BUFFER, this.cubeVerts);
    gl.bufferData(gl.ARRAY_BUFFER, cubeCorners, gl.STATIC_DRAW);
    const { corners, indices } = roundMesh();
    this.roundVerts = gl.createBuffer()!;
    gl.bindBuffer(gl.ARRAY_BUFFER, this.roundVerts);
    gl.bufferData(gl.ARRAY_BUFFER, corners, gl.STATIC_DRAW);
    this.roundIdx = gl.createBuffer()!;
    gl.bindBuffer(gl.ELEMENT_ARRAY_BUFFER, this.roundIdx);
    gl.bufferData(gl.ELEMENT_ARRAY_BUFFER, indices, gl.STATIC_DRAW);
    this.cube = this.pass(this.cubeVerts, null, cubeCorners.length / 4);
    this.round = this.pass(this.roundVerts, this.roundIdx, indices.length);
  }

  /** A vertex array over shared unit geometry `verts` (and `idx`, if indexed) with per-instance attributes. */
  private makeVao(verts: WebGLBuffer, idx: WebGLBuffer | null, instances: WebGLBuffer): WebGLVertexArrayObject {
    const gl = this.gl;
    const vao = gl.createVertexArray()!;
    gl.bindVertexArray(vao);
    gl.bindBuffer(gl.ARRAY_BUFFER, verts);
    gl.enableVertexAttribArray(0);
    gl.vertexAttribPointer(0, 4, gl.FLOAT, false, 16, 0);
    if (idx) gl.bindBuffer(gl.ELEMENT_ARRAY_BUFFER, idx);
    gl.bindBuffer(gl.ARRAY_BUFFER, instances);
    const stride = this.instanceFloats * 4;
    gl.vertexAttribPointer(1, 3, gl.FLOAT, false, stride, 0);
    gl.vertexAttribIPointer(2, 4, gl.UNSIGNED_INT, stride, 12);
    gl.vertexAttribIPointer(3, 1, gl.UNSIGNED_INT, stride, 28);
    for (const loc of [1, 2, 3]) {
      gl.enableVertexAttribArray(loc);
      gl.vertexAttribDivisor(loc, 1);
    }
    gl.bindVertexArray(null);
    return vao;
  }

  private pass(verts: WebGLBuffer, idx: WebGLBuffer | null, count: number): Pass {
    const instances = this.gl.createBuffer()!;
    const vao = this.makeVao(verts, idx, instances);
    return { vao, instances, capacity: 0, count, indexed: idx !== null };
  }

  /** Creates or updates one cached static box chunk (packed into its 128×256×128 GPU region on draw). */
  upsertStaticChunk(
    key: number, x: number, y: number, z: number, bounds: ArrayLike<number>,
    cubes: Float32Array, cubeCount: number, rounds: Float32Array, roundCount: number,
  ): void {
    if (cubeCount === 0 && roundCount === 0) {
      this.removeStaticChunk(key);
      return;
    }
    const rx = x >> REGION_SHIFT, rz = z >> REGION_SHIFT, rk = regionKey(rx, rz);
    const dx = (x - rx * REGION_CHUNKS) * CHUNK, dy = y * CHUNK, dz = (z - rz * REGION_CHUNKS) * CHUNK;
    const stride = this.instanceFloats;
    const shiftCopy = (src: Float32Array, count: number): Float32Array => {
      if (count === 0) return new Float32Array(0);
      const dst = src.slice(0, count * stride);
      for (let i = 0; i < dst.length; i += stride) {
        dst[i] += dx; dst[i + 1] += dy; dst[i + 2] += dz;
      }
      return dst;
    };
    const chunk: StaticChunk = {
      rk,
      minX: bounds[0] + dx, minY: bounds[1] + dy, minZ: bounds[2] + dz,
      maxX: bounds[3] + dx, maxY: bounds[4] + dy, maxZ: bounds[5] + dz,
      cubes: shiftCopy(cubes, cubeCount), cubeCount,
      rounds: shiftCopy(rounds, roundCount), roundCount,
    };
    let region = this.staticRegions.get(rk);
    if (!region) {
      region = {
        rk, rx, rz, minX: 0, minY: 0, minZ: 0, maxX: 0, maxY: 0, maxZ: 0,
        chunks: new Map(), cubeVao: null, cubeVbo: null, cubeCount: 0, cubeCap: 0,
        roundVao: null, roundVbo: null, roundCount: 0, roundCap: 0,
      };
      this.staticRegions.set(rk, region);
    }
    this.staticChunks.set(key, chunk);
    region.chunks.set(key, chunk);
    this.dirtyRegions.add(region);
  }

  /** Removes a static box chunk and marks its parent GPU region dirty. */
  removeStaticChunk(key: number): void {
    const c = this.staticChunks.get(key);
    if (!c) return;
    this.staticChunks.delete(key);
    const region = this.staticRegions.get(c.rk);
    if (!region) return;
    region.chunks.delete(key);
    this.dirtyRegions.add(region);
  }

  /** Repacks and uploads only regions touched by chunk upserts/removals since the previous frame. */
  private flushStaticRegions(): void {
    const gl = this.gl, stride = this.instanceFloats;
    const upload = (
      vao: WebGLVertexArrayObject | null, vbo: WebGLBuffer | null, cap: number,
      total: number, chunks: Map<number, StaticChunk>, pick: (c: StaticChunk) => Float32Array,
      verts: WebGLBuffer, idx: WebGLBuffer | null,
    ): [WebGLVertexArrayObject | null, WebGLBuffer | null, number] => {
      if (total === 0) return [vao, vbo, cap];
      const floats = total * stride, bytes = floats * 4;
      if (this.packScratch.length < floats) this.packScratch = new Float32Array(Math.max(floats, this.packScratch.length * 2));
      let off = 0;
      for (const c of chunks.values()) {
        const a = pick(c);
        if (a.length === 0) continue;
        this.packScratch.set(a, off);
        off += a.length;
      }
      if (!vbo) {
        vbo = gl.createBuffer()!;
        vao = this.makeVao(verts, idx, vbo);
      }
      gl.bindBuffer(gl.ARRAY_BUFFER, vbo);
      if (bytes > cap) {
        gl.bufferData(gl.ARRAY_BUFFER, this.packScratch.subarray(0, floats), gl.STATIC_DRAW);
        cap = bytes;
      } else {
        gl.bufferSubData(gl.ARRAY_BUFFER, 0, this.packScratch, 0, floats);
      }
      return [vao, vbo, cap];
    };
    for (const r of this.dirtyRegions) {
      if (r.chunks.size === 0) {
        if (r.cubeVao) gl.deleteVertexArray(r.cubeVao);
        if (r.cubeVbo) gl.deleteBuffer(r.cubeVbo);
        if (r.roundVao) gl.deleteVertexArray(r.roundVao);
        if (r.roundVbo) gl.deleteBuffer(r.roundVbo);
        this.staticRegions.delete(r.rk);
        continue;
      }
      let cubeCount = 0, roundCount = 0;
      let minX = Infinity, minY = Infinity, minZ = Infinity, maxX = -Infinity, maxY = -Infinity, maxZ = -Infinity;
      for (const c of r.chunks.values()) {
        cubeCount += c.cubeCount; roundCount += c.roundCount;
        if (c.minX < minX) minX = c.minX;
        if (c.minY < minY) minY = c.minY;
        if (c.minZ < minZ) minZ = c.minZ;
        if (c.maxX > maxX) maxX = c.maxX;
        if (c.maxY > maxY) maxY = c.maxY;
        if (c.maxZ > maxZ) maxZ = c.maxZ;
      }
      r.minX = minX; r.minY = minY; r.minZ = minZ; r.maxX = maxX; r.maxY = maxY; r.maxZ = maxZ;
      [r.cubeVao, r.cubeVbo, r.cubeCap] = upload(r.cubeVao, r.cubeVbo, r.cubeCap, cubeCount, r.chunks, (c) => c.cubes, this.cubeVerts, null);
      r.cubeCount = cubeCount;
      [r.roundVao, r.roundVbo, r.roundCap] = upload(r.roundVao, r.roundVbo, r.roundCap, roundCount, r.chunks, (c) => c.rounds, this.roundVerts, this.roundIdx);
      r.roundCount = roundCount;
    }
    this.dirtyRegions.clear();
  }

  private bindShared(
    viewProj: Float32Array, sky: Sky, fog: { color: [number, number, number]; range: [number, number] },
    ghost: readonly [number, number, number, number], beltScroll: readonly [number, number, number, number],
  ): void {
    const gl = this.gl;
    gl.useProgram(this.prog);
    gl.uniform4f(this.u.u_ghost, ...ghost);
    gl.uniformMatrix4fv(this.u.u_viewProj, false, viewProj);
    gl.uniform4f(this.u.u_beltScroll, ...beltScroll);
    gl.uniform1i(this.u.u_tex, 0);
    gl.uniform1i(this.u.u_glow, S.GLOW_UNIT);
    gl.uniform3f(this.u.u_fogColor, ...fog.color);
    gl.uniform3f(this.u.u_skyLight, ...sky.light);
    gl.uniform3f(this.u.u_sunDir, ...sky.lightDir);
    gl.uniform3f(this.u.u_direct, ...sky.direct);
    gl.uniform2f(this.u.u_fog, ...fog.range);
    uploadHeldLights(gl, this.u.u_held);
  }

  /** Draws all frustum-visible static box regions with zero per-frame GPU upload. */
  drawStatic(
    viewProj: Float32Array, planes: Float32Array, eye: readonly [number, number, number],
    sky: Sky, fog: { color: [number, number, number]; range: [number, number] },
    beltScroll: readonly [number, number, number, number],
  ): number {
    if (this.dirtyRegions.size > 0) this.flushStaticRegions();
    if (this.staticRegions.size === 0) return 0;
    const [ex, ey, ez] = eye;
    const vis = this.visibleStatic;
    vis.length = 0;
    for (const r of this.staticRegions.values()) {
      const ox = r.rx * REGION - ex, oy = -ey, oz = r.rz * REGION - ez;
      if (boxInFrustum(planes, ox + r.minX, oy + r.minY, oz + r.minZ, ox + r.maxX, oy + r.maxY, oz + r.maxZ)) vis.push(r);
    }
    if (vis.length === 0) return 0;
    const gl = this.gl;
    this.bindShared(viewProj, sky, fog, SOLID, beltScroll);
    let calls = 0;
    gl.uniform1i(this.u.u_round, 0);
    for (const r of vis) {
      if (r.cubeCount === 0 || !r.cubeVao) continue;
      gl.uniform3f(this.u.u_offset, r.rx * REGION - ex, -ey, r.rz * REGION - ez);
      gl.bindVertexArray(r.cubeVao);
      gl.drawArraysInstanced(gl.TRIANGLES, 0, this.cube.count, r.cubeCount);
      calls++;
    }
    gl.uniform1i(this.u.u_round, 1);
    for (const r of vis) {
      if (r.roundCount === 0 || !r.roundVao) continue;
      gl.uniform3f(this.u.u_offset, r.rx * REGION - ex, -ey, r.rz * REGION - ez);
      gl.bindVertexArray(r.roundVao);
      gl.drawElementsInstanced(gl.TRIANGLES, this.round.count, gl.UNSIGNED_SHORT, 0, r.roundCount);
      calls++;
    }
    gl.bindVertexArray(null);
    return calls;
  }

  /** Draws `count` instances from `boxes` and `roundCount` from `round` (views into wasm memory). Expects textures on unit 0. */
  draw(
    viewProj: Float32Array, sky: Sky, fog: Fog, boxes: Float32Array, count: number,
    round: Float32Array, roundCount: number, ghost: Vec4 = SOLID, beltScroll: Vec4 = ZERO_SCROLL,
  ): number {
    if (count === 0 && roundCount === 0) return 0;
    const gl = this.gl;
    this.bindShared(viewProj, sky, fog, ghost, beltScroll);
    gl.uniform3f(this.u.u_offset, 0, 0, 0);
    let calls = 0;
    if (count > 0) {
      gl.uniform1i(this.u.u_round, 0);
      this.drawPass(this.cube, boxes, count);
      calls++;
    }
    if (roundCount > 0) {
      gl.uniform1i(this.u.u_round, 1);
      this.drawPass(this.round, round, roundCount);
      calls++;
    }
    return calls;
  }

  /** Draws see-through boxes (a held machine's ghost, belt stubs at ports: `port_view.rs`), blended without depth write. */
  drawGhost(
    viewProj: Float32Array, sky: Sky, fog: Fog, boxes: Float32Array, count: number,
    round: Float32Array, roundCount: number, blocked: boolean, beltScroll: Vec4 = ZERO_SCROLL,
  ): number {
    if (count === 0 && roundCount === 0) return 0;
    const gl = this.gl;
    gl.enable(gl.BLEND);
    gl.blendFunc(gl.SRC_ALPHA, gl.ONE_MINUS_SRC_ALPHA);
    gl.depthMask(false);
    const calls = this.draw(viewProj, sky, fog, boxes, count, round, roundCount, blocked ? GHOST_BLOCKED : GHOST, beltScroll);
    gl.depthMask(true);
    gl.disable(gl.BLEND);
    return calls;
  }

  /** X-ray goggles' view (`xray.rs`): draws near factory boxes where terrain hides them (depth test GREATER, no depth write). */
  drawXray(
    viewProj: Float32Array, sky: Sky, fog: Fog, boxes: Float32Array, count: number,
    round: Float32Array, roundCount: number, radius: number, fadeFrom: number, beltScroll: Vec4 = ZERO_SCROLL,
  ): number {
    if (count === 0 && roundCount === 0) return 0;
    const gl = this.gl;
    gl.enable(gl.BLEND);
    gl.blendFunc(gl.SRC_ALPHA, gl.ONE_MINUS_SRC_ALPHA);
    gl.depthMask(false);
    gl.depthFunc(gl.GREATER);
    gl.useProgram(this.prog);
    gl.uniform2f(this.u.u_xray, radius, fadeFrom);
    const calls = this.draw(viewProj, sky, fog, boxes, count, round, roundCount, XRAY, beltScroll);
    gl.uniform2f(this.u.u_xray, 0, 0);
    gl.depthFunc(gl.LEQUAL);
    gl.depthMask(true);
    gl.disable(gl.BLEND);
    return calls;
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
