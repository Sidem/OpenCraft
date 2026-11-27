// Lush 3D grass blades (`LUSH-GRASS`): one camera-centered instanced draw call of solid tapered 3D blades (zero
// alpha discard, keeping early-Z) over nearby lit, open turf cells polled from `game.grass_cells()`
// (engine `surroundings/grass.rs`). Blade height, density and colour follow the seasons (`u_seasonGround`), bend
// smoothly along the real wind (`weather_here()` wind speed and direction, phase-integrated), part gently around the
// player's feet, frost under snow (`u_ground`) and shrink seamlessly into `GRASS_TOP` across 18..28 blocks.
// Presentation only: zero core state, zero chunk remeshes. To tune: `BLADES`, `FADE_START`, `FADE_END`, `VERT`.

import { createProgram, uniforms } from './gl';
import { uploadHeldLights } from './held-light';
import { FOG, HELD_LIGHT, LIGHT, TERRAIN_TINT_PERIOD } from './shaders';
import type { Sky } from './sky';

/** Blades per turf block by quality setting (0 Off, 1 Low, 2 Full). */
const BLADES = [0, 8, 16] as const;
const MAX_CELLS = 4096; // 64 × 64 R32UI texture (matches engine `surroundings/grass.rs`)
const POLL_MS = 320;
const MOVE_POLL_SQ = 2.25; // re-poll immediately after moving 1.5 blocks
const FADE_START = 18.0;
const FADE_END = 28.0;

/** 7-vertex, 5-triangle curved tapered grass blade: (side -1..1, height 0..1). */
const BLADE_VERTS = new Float32Array([
  -1.0, 0.0, 1.0, 0.0,
  -0.82, 0.33, 0.82, 0.33,
  -0.46, 0.67, 0.46, 0.67,
  0.0, 1.0,
]);
const BLADE_INDICES = new Uint16Array([0, 1, 2, 1, 3, 2, 2, 3, 4, 3, 5, 4, 4, 5, 6]);

const VERT = /* glsl */ `#version 300 es
precision highp float;
precision highp int;
precision highp usampler2D;

layout(location = 0) in vec2 a_blade; // .x = across (-1..1), .y = height along blade (0..1)

uniform mat4 u_viewProj;
uniform usampler2D u_cells;
uniform ivec3 u_origin;     // world coords of cell offset (0, 0, 0)
uniform vec3 u_eye;         // camera world position
uniform vec3 u_skyLight;
uniform vec4 u_seasonGround[3]; // [0] grass tint, [2].z = height scale, [2].w = density scale
uniform vec2 u_ground;      // wet, snow (0..1)
uniform vec4 u_wind;        // (dirX, dirZ, strength 0..1+, integrated phase)
uniform int u_blades;       // blades per turf cell (8 or 16)

out vec3 v_col;
out float v_light;
out vec3 v_sun;
out vec3 v_tint;
out vec3 v_rel;
out vec3 v_sub;
out float v_h;
out float v_open;
out float v_field;
${LIGHT}
uint hash3(uvec3 p) {
  uint h = p.x * 374761393u + p.y * 668265263u + p.z * 2147483647u;
  h = (h ^ (h >> 13u)) * 1274126177u;
  return h ^ (h >> 16u);
}
float terrainHash(vec2 p) {
  return fract(sin(dot(p, vec2(127.1, 311.7))) * 43758.5453);
}
float terrainField(vec2 p, float cells) {
  vec2 c = floor(p), f = fract(p);
  f = f * f * (3.0 - 2.0 * f);
  return mix(
    mix(terrainHash(mod(c, cells)), terrainHash(mod(c + vec2(1, 0), cells)), f.x),
    mix(terrainHash(mod(c + vec2(0, 1), cells)), terrainHash(mod(c + vec2(1, 1), cells)), f.x),
    f.y
  );
}

// Tip palettes per tone class (0 plains, 1 lush, 2 savanna, 3..6 ore stains, 7 alpine).
const vec3 TONES[8] = vec3[8](
  vec3(0.32, 0.48, 0.16), vec3(0.26, 0.45, 0.17), vec3(0.42, 0.44, 0.17), vec3(0.36, 0.44, 0.14),
  vec3(0.25, 0.37, 0.14), vec3(0.27, 0.47, 0.21), vec3(0.37, 0.50, 0.20), vec3(0.24, 0.43, 0.19)
);

void main() {
  int cellIdx = gl_InstanceID / u_blades;
  int bladeIdx = gl_InstanceID - cellIdx * u_blades;
  uint packed = texelFetch(u_cells, ivec2(cellIdx & 63, cellIdx >> 6), 0).r;
  ivec3 wcell = u_origin + ivec3(int(packed & 127u), int((packed >> 7u) & 127u), int((packed >> 14u) & 127u));
  uint light = (packed >> 21u) & 255u;
  uint tone = packed >> 29u;

  // Stratified 4x4 sub-grid inside the 1x1 block so blades carpet evenly without bald spots or clumping.
  int slot = u_blades == 8 ? (bladeIdx * 2 + (bladeIdx & 1)) : bladeIdx;
  uint h1 = hash3(uvec3(wcell + 4096) ^ uvec3(uint(slot) * 17u, uint(slot) * 31u, uint(slot) * 53u));
  uint h2 = hash3(uvec3(h1, uint(slot) + 19u, uint(wcell.y + 4096)));
  float r0 = float(h1 & 255u) / 255.0;
  float r1 = float((h1 >> 8u) & 255u) / 255.0;
  float r2 = float((h1 >> 16u) & 255u) / 255.0;
  float r3 = float((h1 >> 24u) & 255u) / 255.0;
  float r4 = float(h2 & 255u) / 255.0;

  // Seasonal density thinning (winter leaves only sparse clumps; deep snow buries blades).
  float density = u_seasonGround[2].w * (1.0 - u_ground.y * 0.55);
  vec2 sub = (vec2(float(slot & 3), float(slot >> 2)) + 0.15 + 0.70 * vec2(r0, r1)) * 0.25;
  vec3 rootWorld = vec3(float(wcell.x) + sub.x, float(wcell.y + 1), float(wcell.z) + sub.y);
  vec3 rootRel = rootWorld - u_eye;
  float dist = length(rootRel.xz);
  float distFade = 1.0 - smoothstep(${FADE_START.toFixed(1)}, ${FADE_END.toFixed(1)}, dist);
  if (r4 >= density || distFade <= 0.01) {
    gl_Position = vec4(2.0, 2.0, 2.0, 1.0);
    return;
  }

  float h = a_blade.y;
  float hScale = u_seasonGround[2].z * (1.0 - u_ground.y * 0.45);
  float bladeH = mix(0.18, 0.36, r2) * hScale * distFade;
  float bladeW = (u_blades == 8 ? mix(0.048, 0.072, r3) : mix(0.036, 0.056, r3)) * (0.65 + 0.35 * distFade);
  float yaw = r3 * 6.2831853;
  vec2 dir = vec2(cos(yaw), sin(yaw));
  vec2 side = vec2(-dir.y, dir.x);

  // Natural arch + rolling two-frequency wind waves along u_wind.xy + player foot parting.
  vec2 wdir = u_wind.xy;
  vec2 wperp = vec2(-wdir.y, wdir.x);
  float gust = sin(dot(rootWorld.xz, wdir) * 0.28 - u_wind.w) * 0.65
             + sin(dot(rootWorld.xz, wdir * 0.7 + wperp * 0.7) * 0.54 - u_wind.w * 1.45 + r4 * 1.4) * 0.35;
  float wAmp = u_wind.z * (0.55 + 0.55 * gust) * (0.75 + 0.5 * r2);
  vec2 bend = dir * (0.04 + 0.11 * r4) + wdir * (0.28 * wAmp) + wperp * (0.06 * gust * u_wind.z);
  if (abs(rootRel.y + 1.62) < 1.35) {
    float footDist = length(rootRel.xz);
    float push = smoothstep(0.90, 0.15, footDist) * 0.32;
    bend += (rootRel.xz / max(footDist, 0.05)) * push;
  }
  vec2 dispXZ = bend * (h * h) * (0.55 + 0.65 * bladeH / 0.3);
  float bendLen = length(dispXZ);
  float y = h * bladeH * sqrt(max(0.28, 1.0 - min(0.68, bendLen * bendLen * 2.2)));
  vec3 posRel = rootRel + vec3(side.x * a_blade.x * bladeW + dispXZ.x, y, side.y * a_blade.x * bladeW + dispXZ.y);

  // Upward-biased normal with gentle cross-blade curvature so meadows light smoothly without harsh backfaces.
  vec3 n = normalize(vec3(dispXZ.x * 1.1 + side.x * a_blade.x * 0.22, 1.0, dispXZ.y * 1.1 + side.y * a_blade.x * 0.22));
  vec2 groundMod = mod(rootWorld.xz, ${TERRAIN_TINT_PERIOD}.0);
  float field = terrainField(groundMod / 16.0, ${TERRAIN_TINT_PERIOD / 16}.0) * 0.65
              + terrainField(groundMod / 64.0, ${TERRAIN_TINT_PERIOD / 64}.0) * 0.35;

  // Root-to-tip colour & AO ramp matched to GRASS_TOP, shifted by season and warm dry-straw blades in autumn.
  vec3 tipCol = TONES[tone] * mix(0.86, 1.14, r3);
  float straw = smoothstep(0.76, 0.96, r4) * clamp(u_seasonGround[0].r - 1.0, 0.0, 0.25) * 3.5;
  tipCol = mix(tipCol, vec3(0.46, 0.39, 0.18), straw);
  vec3 rootCol = tipCol * vec3(0.48, 0.54, 0.44);
  vec3 base = mix(rootCol, tipCol * 1.24, smoothstep(0.0, 0.95, h));
  base *= mix(vec3(0.93, 0.96, 1.0), vec3(1.05, 1.03, 0.95), field);
  v_col = clamp(base * u_seasonGround[0].rgb, 0.0, 1.0);

  float ao = mix(0.62, 1.0, h);
  v_light = mix(0.84, 1.0, n.y) * AMBIENT * ao;
  v_sun = sunLight(n, light) * ao;
  v_tint = lightTint(light, u_skyLight);
  float sky = pow(0.8, 15.0 - float(light & 15u));
  float backLit = pow(max(dot(normalize(posRel), u_sunDir), 0.0), 3.0);
  v_sub = u_direct * (backLit * h * sky * 0.42);
  v_rel = posRel;
  v_h = h;
  v_open = clamp(float(light & 15u) - 12.0, 0.0, 3.0) / 3.0;
  v_field = field;
  gl_Position = u_viewProj * vec4(posRel, 1.0);
}
`;

const FRAG = /* glsl */ `#version 300 es
precision highp float;
precision highp int;
${FOG}
${HELD_LIGHT}
uniform vec2 u_ground;
in vec3 v_col;
in float v_light;
in vec3 v_sun;
in vec3 v_tint;
in vec3 v_rel;
in vec3 v_sub;
in float v_h;
in float v_open;
in float v_field;
out vec4 o_color;

void main() {
  vec3 c = v_col;
  c *= 1.0 - 0.3 * u_ground.x * v_open;
  float snow = smoothstep(v_field * 0.7, v_field * 0.7 + 0.3, u_ground.y) * v_open;
  c = mix(c, vec3(0.9, 0.93, 0.98), snow * (0.45 + 0.50 * v_h));
  vec3 tint = max(v_tint, heldGlow(v_rel));
  vec3 lit = c * (v_light * tint + v_sun) + c * v_sub;
  o_color = vec4(applyFog(lit, v_rel), 1.0);
}
`;

const NAMES = [
  'u_viewProj', 'u_cells', 'u_origin', 'u_eye', 'u_skyLight', 'u_sunDir', 'u_direct',
  'u_fogColor', 'u_fog', 'u_held', 'u_seasonGround', 'u_ground', 'u_wind', 'u_blades',
] as const;

export class GrassPass {
  /** Supplies nearby packed lit turf cells from the engine (`game.grass_cells()`). */
  cellSource: (() => Uint32Array) | undefined;

  private readonly prog: WebGLProgram;
  private readonly u;
  private readonly vao: WebGLVertexArrayObject;
  private readonly tex: WebGLTexture;
  private readonly texBuf = new Uint32Array(MAX_CELLS);
  private origin: [number, number, number] = [0, 0, 0];
  private cellCount = 0;
  private pollEye: [number, number, number] = [1e9, 1e9, 1e9];
  private polled = -1e9;
  private lastNow = 0;
  private windX = 1;
  private windZ = 0;
  private windStr = 0.2;
  private windPhase = 0;

  constructor(private readonly gl: WebGL2RenderingContext) {
    this.prog = createProgram(gl, VERT, FRAG, []);
    this.u = uniforms(gl, this.prog, NAMES);
    this.vao = gl.createVertexArray()!;
    const vbo = gl.createBuffer()!;
    const ebo = gl.createBuffer()!;
    gl.bindVertexArray(this.vao);
    gl.bindBuffer(gl.ARRAY_BUFFER, vbo);
    gl.bufferData(gl.ARRAY_BUFFER, BLADE_VERTS, gl.STATIC_DRAW);
    gl.enableVertexAttribArray(0);
    gl.vertexAttribPointer(0, 2, gl.FLOAT, false, 0, 0);
    gl.bindBuffer(gl.ELEMENT_ARRAY_BUFFER, ebo);
    gl.bufferData(gl.ELEMENT_ARRAY_BUFFER, BLADE_INDICES, gl.STATIC_DRAW);
    gl.bindVertexArray(null);

    this.tex = gl.createTexture()!;
    gl.bindTexture(gl.TEXTURE_2D, this.tex);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.NEAREST);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.NEAREST);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);
    gl.texImage2D(gl.TEXTURE_2D, 0, gl.R32UI, 64, 64, 0, gl.RED_INTEGER, gl.UNSIGNED_INT, this.texBuf);
  }

  /** Draws all nearby 3D grass blades in one instanced call; returns [drawCalls, triangles]. */
  draw(
    viewProj: Float32Array, eye: [number, number, number], sky: Sky, fog: { color: [number, number, number]; range: [number, number] },
    seasonGround: Float32Array, ground: [number, number], wind: [number, number, number], quality: 0 | 1 | 2, now: number,
  ): [number, number] {
    const blades = BLADES[quality] ?? 0;
    if (blades === 0 || !this.cellSource) return [0, 0];

    const dx = eye[0] - this.pollEye[0], dy = eye[1] - this.pollEye[1], dz = eye[2] - this.pollEye[2];
    if (now - this.polled > POLL_MS || dx * dx + dy * dy + dz * dz > MOVE_POLL_SQ) {
      this.polled = now;
      this.pollEye = [eye[0], eye[1], eye[2]];
      this.refresh();
    }
    if (this.cellCount === 0) return [0, 0];

    const dt = this.lastNow > 0 ? Math.min((now - this.lastNow) / 1000, 0.1) : 1 / 60;
    this.lastNow = now;
    const ease = 1 - Math.exp(-dt * 2.5);
    const targetStr = Math.min(1.5, 0.16 + wind[2] / 10.0);
    this.windX += (wind[0] - this.windX) * ease;
    this.windZ += (wind[1] - this.windZ) * ease;
    const len = Math.hypot(this.windX, this.windZ) || 1;
    this.windX /= len;
    this.windZ /= len;
    this.windStr += (targetStr - this.windStr) * ease;
    this.windPhase = (this.windPhase + dt * (1.2 + this.windStr * 2.4)) % (Math.PI * 200);

    const gl = this.gl, u = this.u;
    gl.useProgram(this.prog);
    gl.uniformMatrix4fv(u.u_viewProj, false, viewProj);
    gl.activeTexture(gl.TEXTURE3);
    gl.bindTexture(gl.TEXTURE_2D, this.tex);
    gl.uniform1i(u.u_cells, 3);
    gl.uniform3i(u.u_origin, this.origin[0], this.origin[1], this.origin[2]);
    gl.uniform3f(u.u_eye, eye[0], eye[1], eye[2]);
    gl.uniform3f(u.u_skyLight, ...sky.light);
    gl.uniform3f(u.u_sunDir, ...sky.lightDir);
    gl.uniform3f(u.u_direct, ...sky.direct);
    gl.uniform3f(u.u_fogColor, ...fog.color);
    gl.uniform2f(u.u_fog, ...fog.range);
    gl.uniform4fv(u.u_seasonGround, seasonGround);
    gl.uniform2f(u.u_ground, ground[0], ground[1]);
    gl.uniform4f(u.u_wind, this.windX, this.windZ, this.windStr, this.windPhase);
    gl.uniform1i(u.u_blades, blades);
    uploadHeldLights(gl, u.u_held);

    gl.disable(gl.CULL_FACE);
    gl.bindVertexArray(this.vao);
    const instances = this.cellCount * blades;
    gl.drawElementsInstanced(gl.TRIANGLES, BLADE_INDICES.length, gl.UNSIGNED_SHORT, 0, instances);
    gl.enable(gl.CULL_FACE);
    gl.activeTexture(gl.TEXTURE0);
    return [1, instances * 5];
  }

  private refresh(): void {
    const raw = this.cellSource!();
    if (raw.length < 3) {
      this.cellCount = 0;
      return;
    }
    this.origin = [raw[0] | 0, raw[1] | 0, raw[2] | 0];
    const count = Math.min(MAX_CELLS, raw.length - 3);
    this.cellCount = count;
    if (count === 0) return;
    this.texBuf.set(raw.subarray(3, 3 + count), 0);
    const rows = Math.ceil(count / 64);
    const gl = this.gl;
    gl.activeTexture(gl.TEXTURE3);
    gl.bindTexture(gl.TEXTURE_2D, this.tex);
    gl.texSubImage2D(gl.TEXTURE_2D, 0, 0, 0, 64, rows, gl.RED_INTEGER, gl.UNSIGNED_INT, this.texBuf, 0);
    gl.activeTexture(gl.TEXTURE0);
  }
}
