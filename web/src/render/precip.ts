// Rain and snow: one instanced draw of camera-facing streaks (rain) or flakes (snow) in a box around the camera, and the wet or
// snowy look of open ground. Presentation only. The drops have no vertex data at all: the vertex shader makes every drop's
// place from its instance number, falls it with the clock and wraps it round the camera, so the drops stay put in the world
// as the camera moves. Roofs: the engine's `rain_roof` (surroundings/roof.rs) gives the height each coarse column of the grid
// round the camera catches a drop at; a drop below that height is not drawn, so none fall in houses and caves. The grid is
// polled a few times a second while it falls and kept as a small float texture. Wet ground and snow cover are shader
// uniforms (`ground()` → `u_ground` of the terrain programs, shaders.ts): they show only where the vertex's sky light is
// full. To tune: the constants below and the two looks in `SHADER`.

import { createProgram, uniforms } from './gl';
import { Ground } from './sky/ground';
import type { Weather } from './sky/weather';
import type { Sky } from './sky';

/** The engine's grid (surroundings/roof.rs `SIDE`, `STRIDE`): cells along a side and blocks a cell. */
const ROOF_SIDE = 17;
const ROOF_STRIDE = 2;
/** The box the drops fall in, round the camera: its side, its height, and how much of it is above the eye. */
const BOX = 24;
const BOX_HEIGHT = 36;
const BOX_ABOVE = 20;
/** Drops at full strength by quality (Low, Full) and how often the roof grid is read (ms). */
const DROPS = [0, 3500, 8000];
const POLL_MS = 200;

const SHADER = /* glsl */ `#version 300 es
precision highp float;
precision highp int;
precision highp sampler2D;
uniform mat4 u_viewProj;
uniform sampler2D u_roof;
uniform vec2 u_gridRel;  // the roof grid's corner minus the eye (x, z)
uniform vec3 u_eyeMod;   // the eye modulo the box (x, y, z): drops stay put in the world
uniform float u_eyeY;
uniform float u_time;
out vec2 v_uv;
out float v_fade;
#ifdef SNOW
#define NEAR_FADE 1.4, 3.2, length(rel)
#else
#define NEAR_FADE 0.6, 1.8, length(rel)
#endif

vec3 hash3(uint n) {
  uvec3 h = uvec3(n * 747796405u + 2891336453u, n * 2654435761u + 40503u, n * 1597334677u + 3812015801u);
  h = ((h >> ((h >> 28u) + 4u)) ^ h) * 277803737u;
  h = (h >> 22u) ^ h;
  return vec3(h & 0xFFFFu) / 65536.0;
}

void main() {
  vec3 seed = hash3(uint(gl_InstanceID));
  vec2 corner = vec2(float(gl_VertexID & 1), float(gl_VertexID >> 1)); // (0..1, 0..1)
#ifdef SNOW
  float speed = 1.3 + seed.z * 0.9;
#else
  float speed = 17.0 + seed.z * 5.0;
#endif
  vec3 rel;
  rel.x = mod(seed.x * ${BOX}.0 - u_eyeMod.x + ${BOX / 2}.0, ${BOX}.0) - ${BOX / 2}.0;
  rel.z = mod(seed.y * ${BOX}.0 - u_eyeMod.z + ${BOX / 2}.0, ${BOX}.0) - ${BOX / 2}.0;
  rel.y = mod(fract(seed.x * 91.7 + seed.y * 37.3) * ${BOX_HEIGHT}.0 - speed * u_time - u_eyeMod.y + ${BOX_HEIGHT - BOX_ABOVE}.0, ${BOX_HEIGHT}.0) - ${BOX_HEIGHT - BOX_ABOVE}.0;
#ifdef SNOW
  rel.xz += vec2(sin(u_time * 0.8 + seed.x * 40.0), cos(u_time * 0.7 + seed.y * 40.0)) * 0.45;
#endif

  // Nothing falls below the height the roof grid says this column catches it at.
  ivec2 cell = clamp(ivec2(floor((rel.xz - u_gridRel) / ${ROOF_STRIDE}.0)), ivec2(0), ivec2(${ROOF_SIDE - 1}));
  float lands = texelFetch(u_roof, cell, 0).r;
  float hidden = step(rel.y + u_eyeY, lands);

  float d = length(rel.xz);
  v_fade = (1.0 - smoothstep(${BOX / 2 - 3}.0, ${BOX / 2}.0, d)) * smoothstep(NEAR_FADE)
         * (1.0 - smoothstep(${BOX_ABOVE - 6}.0, ${BOX_ABOVE}.0, rel.y)) * (1.0 - hidden);
  v_uv = corner;

  vec3 side, up;
#ifdef SNOW
  side = normalize(cross(vec3(0.0, 1.0, 0.0), rel)) * 0.065;
  up = vec3(0.0, 0.065, 0.0);
  vec3 p = rel + side * (corner.x - 0.5) * 2.0 + up * (corner.y - 0.5) * 2.0;
#else
  vec3 slant = normalize(vec3(0.07, -1.0, 0.04));
  side = normalize(cross(slant, rel)) * 0.022;
  vec3 p = rel + side * (corner.x - 0.5) * 2.0 - slant * (corner.y * 1.2);
#endif
  gl_Position = u_viewProj * vec4(p, 1.0);
}
`;

const FRAGMENT = /* glsl */ `#version 300 es
precision mediump float;
uniform vec3 u_colour;
uniform float u_alpha;
in vec2 v_uv;
in float v_fade;
out vec4 o_color;
void main() {
  if (v_fade <= 0.0) discard;
#ifdef SNOW
  float a = smoothstep(1.0, 0.45, length(v_uv * 2.0 - 1.0));
#else
  float a = (1.0 - abs(v_uv.x * 2.0 - 1.0)) * (0.35 + 0.65 * sin(v_uv.y * 3.14159));
#endif
  o_color = vec4(u_colour, a * u_alpha * v_fade);
}
`;

type Look = { prog: WebGLProgram; u: ReturnType<typeof makeUniforms> };
const NAMES = ['u_viewProj', 'u_roof', 'u_gridRel', 'u_eyeMod', 'u_eyeY', 'u_time', 'u_colour', 'u_alpha'] as const;
const makeUniforms = (gl: WebGL2RenderingContext, prog: WebGLProgram) => uniforms(gl, prog, NAMES);

export class PrecipPass {
  /** Where rain stops, from the engine (set once by main.ts); without it the drops fall through everything. */
  roofSource: (() => Float32Array) | undefined;

  private readonly ground = new Ground();
  private readonly rain: Look;
  private readonly snow: Look;
  private readonly roof: WebGLTexture;
  private grid = [0, 0];
  private polled = -1e9;
  private lastNow = 0;

  constructor(private readonly gl: WebGL2RenderingContext) {
    const make = (defines: string[]): Look => {
      const prog = createProgram(gl, SHADER, FRAGMENT, defines);
      return { prog, u: makeUniforms(gl, prog) };
    };
    this.rain = make([]);
    this.snow = make(['SNOW']);
    this.roof = gl.createTexture()!;
    gl.bindTexture(gl.TEXTURE_2D, this.roof);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.NEAREST);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.NEAREST);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);
    this.upload(new Float32Array(ROOF_SIDE * ROOF_SIDE).fill(-1e4)); // no roof anywhere until the first reading
  }

  /** Moves the ground's wetness and snow towards the weather and returns them for the terrain shaders: [wet, snow cover]. */
  groundLook(w: Weather | undefined, quality: number, now: number): [number, number] {
    const dt = (now - this.lastNow) / 1000;
    this.lastNow = now;
    this.ground.update(quality > 0 ? w : undefined, dt);
    return [this.ground.wet, this.ground.snow];
  }

  /** Draws the falling drops for the weather `w` (eased) round `eye`; returns the draw calls made (0 or 1). */
  draw(viewProj: Float32Array, eye: [number, number, number], sky: Sky, w: Weather | undefined, quality: number, now: number): number {
    const count = w ? Math.floor(DROPS[quality] * Math.min(1, w.precip * 1.15)) : 0;
    if (count < 8) return 0;
    const gl = this.gl, look = w!.snow ? this.snow : this.rain, u = look.u;
    if (now - this.polled > POLL_MS && this.roofSource) {
      this.polled = now;
      const roof = this.roofSource();
      this.grid = [roof[0], roof[1]];
      this.upload(roof.subarray(2));
    }
    // Pale blue-grey rain and white flakes, lit by the daylight (a faint moonlit floor at night).
    const lum = Math.max(0.12, (sky.light[0] + sky.light[1] + sky.light[2]) / 3);
    const tint = w!.snow ? [1, 1, 1] : [0.72, 0.8, 0.92];

    gl.useProgram(look.prog);
    gl.uniformMatrix4fv(u.u_viewProj, false, viewProj);
    gl.activeTexture(gl.TEXTURE1);
    gl.bindTexture(gl.TEXTURE_2D, this.roof);
    gl.uniform1i(u.u_roof, 1);
    gl.uniform2f(u.u_gridRel, this.grid[0] - eye[0], this.grid[1] - eye[2]);
    const mod = (v: number, m: number) => ((v % m) + m) % m;
    gl.uniform3f(u.u_eyeMod, mod(eye[0], BOX), mod(eye[1], BOX_HEIGHT), mod(eye[2], BOX));
    gl.uniform1f(u.u_eyeY, eye[1]);
    gl.uniform1f(u.u_time, (now / 1000) % 600);
    gl.uniform3f(u.u_colour, tint[0] * lum, tint[1] * lum, tint[2] * lum);
    gl.uniform1f(u.u_alpha, w!.snow ? 0.85 : 0.32);
    gl.enable(gl.BLEND);
    gl.blendFunc(gl.SRC_ALPHA, gl.ONE_MINUS_SRC_ALPHA);
    gl.depthMask(false);
    gl.disable(gl.CULL_FACE);
    gl.bindVertexArray(null);
    gl.drawArraysInstanced(gl.TRIANGLE_STRIP, 0, 4, count);
    gl.enable(gl.CULL_FACE);
    gl.depthMask(true);
    gl.disable(gl.BLEND);
    gl.activeTexture(gl.TEXTURE0);
    return 1;
  }

  private upload(heights: Float32Array): void {
    const gl = this.gl;
    gl.activeTexture(gl.TEXTURE1);
    gl.bindTexture(gl.TEXTURE_2D, this.roof);
    gl.texImage2D(gl.TEXTURE_2D, 0, gl.R32F, ROOF_SIDE, ROOF_SIDE, 0, gl.RED, gl.FLOAT, heights);
    gl.activeTexture(gl.TEXTURE0);
  }
}
