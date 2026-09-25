// WebGL2 world renderer: chunk meshes (opaque then cutout pass, frustum-culled, front to back with
// fog), instanced boxes (boxes.ts), and the target outline plus mining-crack overlay.
// Chunk meshes are uploaded from wasm memory as they arrive and share one quad index buffer.

import { BoxPipeline } from './boxes';
import { createProgram, uniforms } from './gl';
import { boxInFrustum, frustumPlanes, multiply, perspective, viewRotation } from './mat4';
import * as S from './shaders';

const CHUNK = 32;
export const SKY_COLOR: [number, number, number] = [0.62, 0.79, 0.96];
const FOV_Y = (72 * Math.PI) / 180;

interface ChunkMesh {
  x: number;
  y: number;
  z: number;
  vao: WebGLVertexArrayObject;
  vbo: WebGLBuffer;
  capacity: number;
  opaque: number;
  cutout: number;
  dist: number;
}

export interface FrameInfo {
  eye: [number, number, number];
  yaw: number;
  pitch: number;
  target: [number, number, number] | null;
  mineProgress: number;
  /** Box instances (dropped items, belt items, machine parts), `INSTANCE_FLOATS` each. */
  boxes: Float32Array;
  boxCount: number;
}

export interface RenderStats {
  meshes: number;
  visible: number;
  drawCalls: number;
  quads: number;
}

const chunkKey = (x: number, y: number, z: number) => ((x + 1048576) * 2097152 + (z + 1048576)) * 16 + y;

/**
 * Unit cube centred on the origin: 36 vertices of (pos.xyz, u, v, face kind, shade).
 * Face order and u/v axes match the engine mesher so textures line up with world blocks.
 */
function cubeVertices(): Float32Array {
  const faces: [number[], number[], number[], number, number, number[]][] = [
    // normal, u axis, v axis, face kind (0 top, 1 side, 2 bottom), shade, texture right axis
    [[1, 0, 0], [0, 1, 0], [0, 0, 1], 1, 0.72, [0, 0, -1]],
    [[-1, 0, 0], [0, 0, 1], [0, 1, 0], 1, 0.72, [0, 0, 1]],
    [[0, 1, 0], [0, 0, 1], [1, 0, 0], 0, 1.0, [1, 0, 0]],
    [[0, -1, 0], [1, 0, 0], [0, 0, 1], 2, 0.52, [1, 0, 0]],
    [[0, 0, 1], [1, 0, 0], [0, 1, 0], 1, 0.86, [1, 0, 0]],
    [[0, 0, -1], [0, 1, 0], [1, 0, 0], 1, 0.86, [-1, 0, 0]],
  ];
  const out: number[] = [];
  for (const [n, u, v, kind, shade, right] of faces) {
    const corner = (cu: number, cv: number) => {
      const p = [0, 1, 2].map((i) => n[i] * 0.5 + (cu - 0.5) * u[i] + (cv - 0.5) * v[i]);
      const s = p[0] * right[0] + p[1] * right[1] + p[2] * right[2] + 0.5;
      const t = kind === 1 ? 0.5 - p[1] : p[2] + 0.5;
      return [...p, s, t, kind, shade];
    };
    const c = [corner(0, 0), corner(1, 0), corner(1, 1), corner(0, 1)];
    for (const i of [0, 1, 2, 0, 2, 3]) out.push(...c[i]);
  }
  return new Float32Array(out);
}

function boxEdges(): Float32Array {
  const e: number[] = [];
  for (let a = 0; a < 3; a++) {
    for (const i of [0, 1]) {
      for (const j of [0, 1]) {
        const p0 = [0, 0, 0], p1 = [0, 0, 0];
        p1[a] = 1;
        p0[(a + 1) % 3] = p1[(a + 1) % 3] = i;
        p0[(a + 2) % 3] = p1[(a + 2) % 3] = j;
        e.push(...p0, ...p1);
      }
    }
  }
  return new Float32Array(e);
}

export class Renderer {
  readonly gl: WebGL2RenderingContext;
  readonly stats: RenderStats = { meshes: 0, visible: 0, drawCalls: 0, quads: 0 };

  private readonly meshes = new Map<number, ChunkMesh>();
  private readonly visible: ChunkMesh[] = [];
  private readonly quadIndex: WebGLBuffer;
  private quadCapacity = 0;
  private texture: WebGLTexture | null = null;

  private readonly opaque;
  private readonly cutout;
  private readonly line;
  private readonly crack;

  private readonly cubeVao: WebGLVertexArrayObject;
  private readonly boxes: BoxPipeline;
  private readonly lineVao: WebGLVertexArrayObject;

  private readonly proj = new Float32Array(16);
  private readonly view = new Float32Array(16);
  private readonly viewProj = new Float32Array(16);
  private readonly planes = new Float32Array(24);

  constructor(private readonly canvas: HTMLCanvasElement, private viewRadius: number) {
    const gl = canvas.getContext('webgl2', {
      antialias: true,
      alpha: false,
      depth: true,
      stencil: false,
      powerPreference: 'high-performance',
    });
    if (!gl) throw new Error('WebGL2 is not available in this browser.');
    this.gl = gl;

    const lit = ['u_viewProj', 'u_offset', 'u_tex', 'u_fogColor', 'u_fog'] as const;
    const opaqueProg = createProgram(gl, S.chunkVert, S.litFrag);
    const cutoutProg = createProgram(gl, S.chunkVert, S.litFrag, ['CUTOUT']);
    const lineProg = createProgram(gl, S.lineVert, S.lineFrag);
    const crackProg = createProgram(gl, S.crackVert, S.crackFrag);
    this.opaque = { prog: opaqueProg, u: uniforms(gl, opaqueProg, lit) };
    this.cutout = { prog: cutoutProg, u: uniforms(gl, cutoutProg, lit) };
    this.line = { prog: lineProg, u: uniforms(gl, lineProg, ['u_viewProj', 'u_offset', 'u_color'] as const) };
    this.crack = { prog: crackProg, u: uniforms(gl, crackProg, ['u_viewProj', 'u_offset', 'u_progress'] as const) };

    this.quadIndex = gl.createBuffer()!;
    this.ensureQuadCapacity(1 << 15);

    // Cube geometry for the mining-crack overlay.
    const cube = gl.createBuffer()!;
    gl.bindBuffer(gl.ARRAY_BUFFER, cube);
    gl.bufferData(gl.ARRAY_BUFFER, cubeVertices(), gl.STATIC_DRAW);
    this.cubeVao = gl.createVertexArray()!;
    gl.bindVertexArray(this.cubeVao);
    gl.enableVertexAttribArray(0);
    gl.vertexAttribPointer(0, 3, gl.FLOAT, false, 28, 0);
    gl.enableVertexAttribArray(1);
    gl.vertexAttribPointer(1, 4, gl.FLOAT, false, 28, 12);

    this.boxes = new BoxPipeline(gl);

    this.lineVao = gl.createVertexArray()!;
    gl.bindVertexArray(this.lineVao);
    const lines = gl.createBuffer()!;
    gl.bindBuffer(gl.ARRAY_BUFFER, lines);
    gl.bufferData(gl.ARRAY_BUFFER, boxEdges(), gl.STATIC_DRAW);
    gl.enableVertexAttribArray(0);
    gl.vertexAttribPointer(0, 3, gl.FLOAT, false, 0, 0);
    gl.bindVertexArray(null);
  }

  setViewRadius(r: number): void {
    this.viewRadius = r;
  }

  /** Uploads all block textures as one TEXTURE_2D_ARRAY (one layer per texture, no atlas bleeding). */
  setTextures(pixels: Uint8Array, size: number, layers: number): void {
    const gl = this.gl;
    const tex = gl.createTexture()!;
    gl.bindTexture(gl.TEXTURE_2D_ARRAY, tex);
    gl.texImage3D(gl.TEXTURE_2D_ARRAY, 0, gl.RGBA8, size, size, layers, 0, gl.RGBA, gl.UNSIGNED_BYTE, pixels);
    gl.generateMipmap(gl.TEXTURE_2D_ARRAY);
    gl.texParameteri(gl.TEXTURE_2D_ARRAY, gl.TEXTURE_MIN_FILTER, gl.NEAREST_MIPMAP_LINEAR);
    gl.texParameteri(gl.TEXTURE_2D_ARRAY, gl.TEXTURE_MAG_FILTER, gl.NEAREST);
    gl.texParameteri(gl.TEXTURE_2D_ARRAY, gl.TEXTURE_WRAP_S, gl.REPEAT);
    gl.texParameteri(gl.TEXTURE_2D_ARRAY, gl.TEXTURE_WRAP_T, gl.REPEAT);
    const aniso = gl.getExtension('EXT_texture_filter_anisotropic');
    if (aniso) {
      const max = gl.getParameter(aniso.MAX_TEXTURE_MAX_ANISOTROPY_EXT) as number;
      gl.texParameterf(gl.TEXTURE_2D_ARRAY, aniso.TEXTURE_MAX_ANISOTROPY_EXT, Math.min(8, max));
    }
    this.texture = tex;
  }

  /**
   * Creates or replaces a chunk mesh. `verts` may be a view into wasm memory; it is consumed
   * immediately and never retained.
   */
  upsertChunk(x: number, y: number, z: number, verts: Uint32Array, opaque: number, cutout: number): void {
    const key = chunkKey(x, y, z);
    let mesh = this.meshes.get(key);
    if (opaque + cutout === 0) {
      if (mesh) this.deleteMesh(key, mesh);
      return;
    }
    const gl = this.gl;
    this.ensureQuadCapacity(opaque + cutout);
    if (!mesh) {
      const vao = gl.createVertexArray()!;
      const vbo = gl.createBuffer()!;
      gl.bindVertexArray(vao);
      gl.bindBuffer(gl.ARRAY_BUFFER, vbo);
      gl.enableVertexAttribArray(0);
      gl.vertexAttribIPointer(0, 1, gl.UNSIGNED_INT, 4, 0);
      gl.bindBuffer(gl.ELEMENT_ARRAY_BUFFER, this.quadIndex);
      gl.bindVertexArray(null);
      mesh = { x, y, z, vao, vbo, capacity: 0, opaque: 0, cutout: 0, dist: 0 };
      this.meshes.set(key, mesh);
    }
    gl.bindBuffer(gl.ARRAY_BUFFER, mesh.vbo);
    if (verts.byteLength > mesh.capacity) {
      gl.bufferData(gl.ARRAY_BUFFER, verts, gl.STATIC_DRAW);
      mesh.capacity = verts.byteLength;
    } else {
      gl.bufferSubData(gl.ARRAY_BUFFER, 0, verts);
    }
    mesh.opaque = opaque;
    mesh.cutout = cutout;
  }

  removeChunk(x: number, y: number, z: number): void {
    const key = chunkKey(x, y, z);
    const mesh = this.meshes.get(key);
    if (mesh) this.deleteMesh(key, mesh);
  }

  private deleteMesh(key: number, mesh: ChunkMesh): void {
    this.gl.deleteVertexArray(mesh.vao);
    this.gl.deleteBuffer(mesh.vbo);
    this.meshes.delete(key);
  }

  /** One shared index buffer (0,1,2, 0,2,3 per quad) serves every chunk. */
  private ensureQuadCapacity(quads: number): void {
    if (quads <= this.quadCapacity) return;
    let cap = Math.max(this.quadCapacity, 1024);
    while (cap < quads) cap *= 2;
    const idx = new Uint32Array(cap * 6);
    for (let q = 0, i = 0; q < cap; q++) {
      const v = q * 4;
      idx[i++] = v; idx[i++] = v + 1; idx[i++] = v + 2;
      idx[i++] = v; idx[i++] = v + 2; idx[i++] = v + 3;
    }
    const gl = this.gl;
    // Rebinding the same buffer object keeps every chunk VAO's element binding valid.
    gl.bindVertexArray(null);
    gl.bindBuffer(gl.ELEMENT_ARRAY_BUFFER, this.quadIndex);
    gl.bufferData(gl.ELEMENT_ARRAY_BUFFER, idx, gl.STATIC_DRAW);
    this.quadCapacity = cap;
  }

  private resize(): void {
    const dpr = Math.min(window.devicePixelRatio || 1, 2);
    const w = Math.max(1, Math.round(this.canvas.clientWidth * dpr));
    const h = Math.max(1, Math.round(this.canvas.clientHeight * dpr));
    if (this.canvas.width !== w || this.canvas.height !== h) {
      this.canvas.width = w;
      this.canvas.height = h;
    }
  }

  render(f: FrameInfo): void {
    const gl = this.gl;
    this.resize();
    gl.viewport(0, 0, this.canvas.width, this.canvas.height);
    gl.clearColor(SKY_COLOR[0], SKY_COLOR[1], SKY_COLOR[2], 1);
    gl.clear(gl.COLOR_BUFFER_BIT | gl.DEPTH_BUFFER_BIT);
    gl.enable(gl.DEPTH_TEST);
    gl.depthFunc(gl.LEQUAL);
    gl.depthMask(true);
    gl.enable(gl.CULL_FACE);
    gl.cullFace(gl.BACK);
    gl.disable(gl.BLEND);

    const far = (this.viewRadius + 3) * CHUNK * 1.8;
    perspective(this.proj, FOV_Y, this.canvas.width / this.canvas.height, 0.05, far);
    viewRotation(this.view, f.yaw, f.pitch);
    multiply(this.viewProj, this.proj, this.view);
    frustumPlanes(this.planes, this.viewProj);

    // Frustum-cull chunks, then draw front to back so early-z rejects hidden fragments.
    const [ex, ey, ez] = f.eye;
    const vis = this.visible;
    vis.length = 0;
    for (const m of this.meshes.values()) {
      const ox = m.x * CHUNK - ex, oy = m.y * CHUNK - ey, oz = m.z * CHUNK - ez;
      if (!boxInFrustum(this.planes, ox, oy, oz, ox + CHUNK, oy + CHUNK, oz + CHUNK)) continue;
      const cx = ox + 16, cy = oy + 16, cz = oz + 16;
      m.dist = cx * cx + cy * cy + cz * cz;
      vis.push(m);
    }
    vis.sort((a, b) => a.dist - b.dist);

    const fogEnd = this.viewRadius * CHUNK * 0.95;
    const fogStart = fogEnd * 0.55;
    gl.activeTexture(gl.TEXTURE0);
    gl.bindTexture(gl.TEXTURE_2D_ARRAY, this.texture);

    let drawCalls = 0, quads = 0;
    for (const pass of [this.opaque, this.cutout]) {
      const isCutout = pass === this.cutout;
      gl.useProgram(pass.prog);
      gl.uniformMatrix4fv(pass.u.u_viewProj, false, this.viewProj);
      gl.uniform1i(pass.u.u_tex, 0);
      gl.uniform3f(pass.u.u_fogColor, SKY_COLOR[0], SKY_COLOR[1], SKY_COLOR[2]);
      gl.uniform2f(pass.u.u_fog, fogStart, fogEnd);
      for (const m of vis) {
        const n = isCutout ? m.cutout : m.opaque;
        if (n === 0) continue;
        gl.uniform3f(pass.u.u_offset, m.x * CHUNK - ex, m.y * CHUNK - ey, m.z * CHUNK - ez);
        gl.bindVertexArray(m.vao);
        // Cutout quads are stored after the opaque ones; the shared index buffer is offset to match.
        gl.drawElements(gl.TRIANGLES, n * 6, gl.UNSIGNED_INT, isCutout ? m.opaque * 24 : 0);
        drawCalls++;
        quads += n;
      }
    }

    drawCalls += this.boxes.draw(this.viewProj, SKY_COLOR, [fogStart, fogEnd], f.boxes, f.boxCount);

    if (f.target) {
      const [tx, ty, tz] = f.target;
      const ox = tx - ex, oy = ty - ey, oz = tz - ez;
      gl.enable(gl.BLEND);
      gl.blendFunc(gl.SRC_ALPHA, gl.ONE_MINUS_SRC_ALPHA);
      gl.depthMask(false);
      if (f.mineProgress > 0) {
        gl.enable(gl.POLYGON_OFFSET_FILL);
        gl.polygonOffset(-1, -1);
        gl.useProgram(this.crack.prog);
        gl.uniformMatrix4fv(this.crack.u.u_viewProj, false, this.viewProj);
        gl.uniform3f(this.crack.u.u_offset, ox, oy, oz);
        gl.uniform1f(this.crack.u.u_progress, f.mineProgress);
        gl.bindVertexArray(this.cubeVao);
        gl.drawArrays(gl.TRIANGLES, 0, 36);
        gl.disable(gl.POLYGON_OFFSET_FILL);
        drawCalls++;
      }
      gl.useProgram(this.line.prog);
      gl.uniformMatrix4fv(this.line.u.u_viewProj, false, this.viewProj);
      gl.uniform3f(this.line.u.u_offset, ox, oy, oz);
      gl.uniform4f(this.line.u.u_color, 0.05, 0.05, 0.08, 0.6);
      gl.bindVertexArray(this.lineVao);
      gl.drawArrays(gl.LINES, 0, 24);
      gl.depthMask(true);
      gl.disable(gl.BLEND);
      drawCalls++;
    }
    gl.bindVertexArray(null);

    this.stats.meshes = this.meshes.size;
    this.stats.visible = vis.length;
    this.stats.drawCalls = drawCalls;
    this.stats.quads = quads;
  }
}
