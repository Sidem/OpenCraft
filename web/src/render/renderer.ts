// WebGL2 world renderer: the sky (sky.ts: day, night and weather), chunk meshes (opaque then cutout pass,
// frustum-culled, front to back with fog in the horizon colour), instanced boxes (boxes.ts), water (water.ts:
// blended, last, and the underwater fog), then the overlays (outlines.ts: mining cracks, the target outline, a dragged
// belt line, placement boxes).
// Chunk meshes are uploaded from wasm memory as they arrive and share one quad index buffer.

import { BoxPipeline } from './boxes';
import { createProgram, uniforms } from './gl';
import { PrecipPass } from './precip';
import { setHeldLights, uploadHeldLights } from './held-light';
import { boxInFrustum, frustumPlanes, multiply, perspective, viewRotation } from './mat4';
import { Overlays } from './outlines';
import * as S from './shaders';
import { skyAt, SkyPass } from './sky';
import { skyFrame } from './sky/astro';
import { DEFAULTS as SKY_DEFAULTS, type SkySettings } from './sky/settings';
import { WeatherSky } from './sky/weather';
import { fogFor, WaterPass } from './water';

const CHUNK = 32;

interface ChunkMesh {
  x: number;
  y: number;
  z: number;
  vao: WebGLVertexArrayObject;
  vbo: WebGLBuffer;
  capacity: number;
  opaque: number;
  cutout: number;
  liquid: number;
  dist: number;
}

export interface FrameInfo {
  eye: [number, number, number];
  yaw: number;
  pitch: number;
  target: [number, number, number] | null;
  /** Where the target's model sits in its cell (lowest corner, size) so the outline hugs a pole or belt; empty: whole cell. */
  targetFit: number[];
  /** The selected belt item's box in the world (lowest corner, then size), or empty. */
  selectedItem: number[];
  mineProgress: number;
  /** The engine's time of day (0 midnight, 0.5 noon). */
  time: number;
  /** Days since the world began as a fraction (0 at the first midnight); moves the stars and the moon's phase. */
  days: number;
  /** The world's seed and the biome the player stands in (engine order): what the weather is a function of (sky/weather.ts). */
  seed: number;
  biome: number;
  /** Box instances (dropped items, belt items, machine parts), `INSTANCE_FLOATS` each. */
  boxes: Float32Array;
  boxCount: number;
  /** The round bodies among them, copied out (`round_ptr`), drawn with the cylinder mesh. */
  round: Float32Array;
  roundCount: number;
  /** A belt line being dragged out: x, y, z and 1 (will be built) or 0 (past the belts in hand) per cell. */
  lineCells: Int32Array;
  /** Quarry boxes to outline, as lowest and highest cells (6 numbers each). */
  quarryBoxes: Int32Array;
  /** Blocks quarries are digging: x, y, z and progress in thousandths each. */
  cracks: Int32Array;
  underwater: boolean; // the engine's `eye_in_water`
  /** Satellites in orbit as the sky draws them (engine `sky_satellites`: x, y, z, kind each). */
  satellites: Float32Array;
  /** Lights held in hands this frame: camera-relative x, y, z and reach in blocks each (held-light.ts). */
  held: Float32Array;
}

export interface RenderStats {
  meshes: number;
  visible: number;
  drawCalls: number;
  quads: number;
}

const chunkKey = (x: number, y: number, z: number) => ((x + 1048576) * 2097152 + (z + 1048576)) * 16 + y;

export class Renderer {
  readonly gl: WebGL2RenderingContext;
  readonly stats: RenderStats = { meshes: 0, visible: 0, drawCalls: 0, quads: 0 };

  private readonly meshes = new Map<number, ChunkMesh>();
  private readonly visible: ChunkMesh[] = [];
  private readonly quadIndex: WebGLBuffer;
  private quadCapacity = 0;
  private texture: WebGLTexture | null = null;
  private glowTexture: WebGLTexture | null = null;
  private texSize = 0;

  private readonly opaque;
  private readonly cutout;

  private readonly boxes: BoxPipeline;
  private readonly sky: SkyPass;
  private readonly water: WaterPass;
  private readonly overlays: Overlays;

  private readonly proj = new Float32Array(16);
  private readonly view = new Float32Array(16);
  private readonly viewProj = new Float32Array(16);
  private readonly planes = new Float32Array(24);

  fovY = (72 * Math.PI) / 180; // vertical field of view in radians (the comfort setting)
  skySettings: SkySettings = SKY_DEFAULTS; // how the sky is drawn (sky/settings.ts, set from the menu)
  readonly weather = new WeatherSky(); // the weather as drawn (`.ease.value`); the rain and snow pass reads it
  readonly precip: PrecipPass; // rain, snow and the ground's wet or snowy look (precip.ts)
  constructor(private readonly canvas: HTMLCanvasElement, private viewRadius: number, instanceFloats: number, roundBit: number) {
    const gl = canvas.getContext('webgl2', {
      antialias: true,
      alpha: false,
      depth: true,
      stencil: false,
      powerPreference: 'high-performance',
    });
    if (!gl) throw new Error('WebGL2 is not available in this browser.');
    this.gl = gl;

    const lit = ['u_viewProj', 'u_offset', 'u_worldOrigin', 'u_tex', 'u_glow', 'u_fogColor', 'u_fog', 'u_skyLight', 'u_sunDir', 'u_direct', 'u_ground', 'u_turns', 'u_held'] as const;
    const opaqueProg = createProgram(gl, S.chunkVert, S.litFrag, ['TERRAIN']);
    const cutoutProg = createProgram(gl, S.chunkVert, S.litFrag, ['CUTOUT', 'TERRAIN']);
    this.opaque = { prog: opaqueProg, u: uniforms(gl, opaqueProg, lit) };
    this.cutout = { prog: cutoutProg, u: uniforms(gl, cutoutProg, lit) };

    this.quadIndex = gl.createBuffer()!;
    this.ensureQuadCapacity(1 << 15);

    this.boxes = new BoxPipeline(gl, instanceFloats, roundBit);
    this.sky = new SkyPass(gl);
    this.water = new WaterPass(gl);
    this.precip = new PrecipPass(gl);

    this.overlays = new Overlays(gl);
  }

  setViewRadius(r: number): void {
    this.viewRadius = r;
  }

  /** Uploads all block textures as one TEXTURE_2D_ARRAY (one layer per texture, no atlas bleeding). */
  /** The texture layer of water (`game.water_layer()`; liquid vertices don't carry one). */
  setWaterLayer(layer: number): void { this.water.layer = layer; }

  setTextures(pixels: Uint8Array, size: number, layers: number, glow: Uint8Array | null = null): void {
    const gl = this.gl;
    const upload = (format: number, base: number, data: Uint8Array, w: number, depth: number) => {
      const tex = gl.createTexture()!;
      gl.bindTexture(gl.TEXTURE_2D_ARRAY, tex);
      gl.texImage3D(gl.TEXTURE_2D_ARRAY, 0, format, w, w, depth, 0, base, gl.UNSIGNED_BYTE, data);
      gl.generateMipmap(gl.TEXTURE_2D_ARRAY);
      gl.texParameteri(gl.TEXTURE_2D_ARRAY, gl.TEXTURE_MIN_FILTER, gl.NEAREST_MIPMAP_LINEAR);
      gl.texParameteri(gl.TEXTURE_2D_ARRAY, gl.TEXTURE_MAG_FILTER, gl.NEAREST);
      gl.texParameteri(gl.TEXTURE_2D_ARRAY, gl.TEXTURE_WRAP_S, gl.REPEAT);
      gl.texParameteri(gl.TEXTURE_2D_ARRAY, gl.TEXTURE_WRAP_T, gl.REPEAT);
      return tex;
    };
    this.texture = upload(gl.RGBA8, gl.RGBA, pixels, size, layers);
    this.texSize = size;
    const aniso = gl.getExtension('EXT_texture_filter_anisotropic');
    if (aniso) {
      const max = gl.getParameter(aniso.MAX_TEXTURE_MAX_ANISOTROPY_EXT) as number;
      gl.texParameterf(gl.TEXTURE_2D_ARRAY, aniso.TEXTURE_MAX_ANISOTROPY_EXT, Math.min(8, max));
    }
    // Self-light, one byte a texel on its own unit; without any, a single black layer (array layers clamp).
    this.glowTexture = glow ? upload(gl.R8, gl.RED, glow, size, layers) : upload(gl.R8, gl.RED, new Uint8Array(1), 1, 1);
  }

  /** Replaces one layer of the block textures (`size` × `size` RGBA, the size `setTextures` was given): a sign's text. */
  paintLayer(layer: number, pixels: Uint8Array): void {
    const gl = this.gl;
    gl.bindTexture(gl.TEXTURE_2D_ARRAY, this.texture);
    gl.texSubImage3D(gl.TEXTURE_2D_ARRAY, 0, 0, 0, layer, this.texSize, this.texSize, 1, gl.RGBA, gl.UNSIGNED_BYTE, pixels);
  }

  /** Rebuilds the block textures' mipmaps after `paintLayer`. */
  refreshMipmaps(): void {
    this.gl.bindTexture(this.gl.TEXTURE_2D_ARRAY, this.texture);
    this.gl.generateMipmap(this.gl.TEXTURE_2D_ARRAY);
  }

  /** How each layer turns per block (`game.texture_turns()`: two bits a layer, 32 words); all zeros turns none. */
  setTurns(mask: Uint32Array): void {
    for (const pass of [this.opaque, this.cutout]) {
      this.gl.useProgram(pass.prog);
      this.gl.uniform1uiv(pass.u.u_turns, mask);
    }
  }

  /**
   * Creates or replaces a chunk mesh. `verts` may be a view into wasm memory; it is consumed
   * immediately and never retained. It holds the packed vertices, then one light byte per vertex
   * (`mesher.rs`), read as attribute 1. Quads come opaque, then cutout, then liquid.
   */
  upsertChunk(x: number, y: number, z: number, verts: Uint32Array, opaque: number, cutout: number, liquid: number): void {
    const key = chunkKey(x, y, z);
    let mesh = this.meshes.get(key);
    const total = opaque + cutout + liquid;
    if (total === 0) {
      if (mesh) this.deleteMesh(key, mesh);
      return;
    }
    const gl = this.gl;
    this.ensureQuadCapacity(total);
    if (!mesh) {
      const vao = gl.createVertexArray()!;
      const vbo = gl.createBuffer()!;
      gl.bindVertexArray(vao);
      gl.bindBuffer(gl.ARRAY_BUFFER, vbo);
      gl.enableVertexAttribArray(0);
      gl.vertexAttribIPointer(0, 1, gl.UNSIGNED_INT, 4, 0);
      gl.enableVertexAttribArray(1);
      gl.bindBuffer(gl.ELEMENT_ARRAY_BUFFER, this.quadIndex);
      gl.bindVertexArray(null);
      mesh = { x, y, z, vao, vbo, capacity: 0, opaque: 0, cutout: 0, liquid: 0, dist: 0 };
      this.meshes.set(key, mesh);
    }
    gl.bindBuffer(gl.ARRAY_BUFFER, mesh.vbo);
    gl.bindVertexArray(mesh.vao);
    gl.vertexAttribIPointer(1, 1, gl.UNSIGNED_BYTE, 1, total * 16); // the light bytes follow the vertices
    gl.bindVertexArray(null);
    if (verts.byteLength > mesh.capacity) {
      gl.bufferData(gl.ARRAY_BUFFER, verts, gl.STATIC_DRAW);
      mesh.capacity = verts.byteLength;
    } else {
      gl.bufferSubData(gl.ARRAY_BUFFER, 0, verts);
    }
    mesh.opaque = opaque;
    mesh.cutout = cutout;
    mesh.liquid = liquid;
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
    const look = this.weather.look(this.skySettings, f.seed, f.days, f.biome, performance.now()); // undefined: the plain sky
    const sky = skyAt(f.time, look);
    const now = performance.now(), drawn = look ? this.weather.ease.value : undefined;
    const ground = this.precip.groundLook(drawn, this.skySettings.precip, now);
    const murk = 1 - 0.35 * (look?.gloom ?? 0); // gloom shortens the sight
    const fog = fogFor(sky, f.underwater, [this.viewRadius * CHUNK * 0.95 * 0.55 * murk, this.viewRadius * CHUNK * 0.95 * murk]);
    if (f.underwater) {
      gl.clearColor(...fog.color, 1);
      gl.clear(gl.COLOR_BUFFER_BIT | gl.DEPTH_BUFFER_BIT);
    } else {
      gl.clear(gl.DEPTH_BUFFER_BIT);
      const view = { settings: this.skySettings, frame: skyFrame(f.days), time: performance.now() / 1000, days: f.days, look };
      this.sky.draw(sky, f.yaw, f.pitch, this.fovY, this.canvas.width / this.canvas.height, f.satellites, view);
    }
    gl.enable(gl.DEPTH_TEST);
    gl.depthFunc(gl.LEQUAL);
    gl.depthMask(true);
    gl.enable(gl.CULL_FACE);
    gl.cullFace(gl.BACK);
    gl.disable(gl.BLEND);

    const far = (this.viewRadius + 3) * CHUNK * 1.8;
    perspective(this.proj, this.fovY, this.canvas.width / this.canvas.height, 0.05, far);
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

    const [fogStart, fogEnd] = fog.range;
    setHeldLights(f.held);
    gl.activeTexture(gl.TEXTURE0 + S.GLOW_UNIT);
    gl.bindTexture(gl.TEXTURE_2D_ARRAY, this.glowTexture);
    gl.activeTexture(gl.TEXTURE0);
    gl.bindTexture(gl.TEXTURE_2D_ARRAY, this.texture);

    let drawCalls = 0, quads = 0;
    for (const pass of [this.opaque, this.cutout]) {
      const isCutout = pass === this.cutout;
      gl.useProgram(pass.prog);
      gl.uniformMatrix4fv(pass.u.u_viewProj, false, this.viewProj);
      gl.uniform1i(pass.u.u_tex, 0);
      gl.uniform1i(pass.u.u_glow, S.GLOW_UNIT);
      gl.uniform3f(pass.u.u_fogColor, ...fog.color);
      gl.uniform3f(pass.u.u_skyLight, ...sky.light);
      gl.uniform3f(pass.u.u_sunDir, ...sky.lightDir);
      gl.uniform3f(pass.u.u_direct, ...sky.direct);
      gl.uniform2f(pass.u.u_fog, fogStart, fogEnd);
      gl.uniform2f(pass.u.u_ground, ground[0], ground[1]);
      uploadHeldLights(gl, pass.u.u_held);
      for (const m of vis) {
        const n = isCutout ? m.cutout : m.opaque;
        if (n === 0) continue;
        gl.uniform3f(pass.u.u_offset, m.x * CHUNK - ex, m.y * CHUNK - ey, m.z * CHUNK - ez);
        const period = S.TERRAIN_TINT_PERIOD;
        gl.uniform3f(pass.u.u_worldOrigin, (m.x * CHUNK) % period, (m.y * CHUNK) % period, (m.z * CHUNK) % period);
        gl.bindVertexArray(m.vao);
        // Cutout quads are stored after the opaque ones; the shared index buffer is offset to match.
        gl.drawElements(gl.TRIANGLES, n * 6, gl.UNSIGNED_INT, isCutout ? m.opaque * 24 : 0);
        drawCalls++;
        quads += n;
      }
    }

    drawCalls += this.boxes.draw(this.viewProj, sky, fog, f.boxes, f.boxCount, f.round, f.roundCount);
    const [waterCalls, waterQuads] = this.water.draw(vis, this.viewProj, [ex, ey, ez], sky, fog);
    drawCalls += waterCalls;
    quads += waterQuads;
    if (!f.underwater) drawCalls += this.precip.draw(this.viewProj, f.eye, sky, drawn, this.skySettings.precip, now);

    drawCalls += this.overlays.draw(this.viewProj, f, [ex, ey, ez]);
    gl.bindVertexArray(null);

    this.stats.meshes = this.meshes.size;
    this.stats.visible = vis.length;
    this.stats.drawCalls = drawCalls;
    this.stats.quads = quads;
  }

  /** Where a camera-relative point shows on the canvas in the last frame, in CSS pixels, or null if it
   * is behind the camera. */
  project(x: number, y: number, z: number): [number, number] | null {
    const m = this.viewProj;
    const w = m[3] * x + m[7] * y + m[11] * z + m[15];
    if (w < 0.05) return null;
    const cx = (m[0] * x + m[4] * y + m[8] * z + m[12]) / w;
    const cy = (m[1] * x + m[5] * y + m[9] * z + m[13]) / w;
    return [(cx * 0.5 + 0.5) * this.canvas.clientWidth, (0.5 - cy * 0.5) * this.canvas.clientHeight];
  }
}
